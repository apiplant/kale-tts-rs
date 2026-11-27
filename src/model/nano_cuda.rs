use candle_core::{self as candle, DType, Layout, Result, Shape, Tensor};

#[derive(Clone, Copy)]
#[repr(u32)]
pub(super) enum Binary {
    Add,
    Sub,
    Mul,
    Div,
    LeakyRelu,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct ElementwiseLayout {
    dims: [u32; 4],
    left: [u32; 4],
    right: [u32; 4],
}

// Passed by value, with the same fixed-width layout on both sides of the launch.
unsafe impl candle::cuda::cudarc::driver::DeviceRepr for ElementwiseLayout {}

struct Elementwise {
    op: Binary,
    alpha: f32,
    shape: Shape,
}

impl candle::CustomOp2 for Elementwise {
    fn name(&self) -> &'static str {
        "kale-nano-elementwise"
    }

    fn cpu_fwd(
        &self,
        _: &candle::CpuStorage,
        _: &Layout,
        _: &candle::CpuStorage,
        _: &Layout,
    ) -> Result<(candle::CpuStorage, Shape)> {
        candle::bail!("Nano elementwise operation requires CUDA")
    }

    fn cuda_fwd(
        &self,
        left: &candle::CudaStorage,
        lhs: &Layout,
        right: &candle::CudaStorage,
        rhs: &Layout,
    ) -> Result<(candle::CudaStorage, Shape)> {
        use candle::cuda::cudarc::driver::{LaunchConfig, PushKernelArg};

        let count = u32::try_from(self.shape.elem_count()).map_err(candle::Error::wrap)?;
        let mut layout = ElementwiseLayout {
            dims: [1; 4],
            left: [0; 4],
            right: [0; 4],
        };
        for (out, &dim) in layout
            .dims
            .iter_mut()
            .rev()
            .zip(self.shape.dims().iter().rev())
        {
            *out = u32::try_from(dim).map_err(candle::Error::wrap)?;
        }
        for (out, input) in [(&mut layout.left, lhs), (&mut layout.right, rhs)] {
            for (stride, (&dim, &input_stride)) in out
                .iter_mut()
                .rev()
                .zip(input.dims().iter().zip(input.stride()).rev())
            {
                if dim != 1 {
                    *stride = u32::try_from(input_stride).map_err(candle::Error::wrap)?;
                }
            }
        }
        let device = &left.device;
        let lhs_slice = left.as_cuda_slice::<f32>()?.slice(lhs.start_offset()..);
        let rhs_slice = right.as_cuda_slice::<f32>()?.slice(rhs.start_offset()..);
        let mut output = unsafe { device.alloc::<f32>(count as usize) }?;
        let func = device.get_or_load_custom_func(
            "kale_elementwise_f32",
            "kale_tts_audio",
            include_str!(concat!(env!("OUT_DIR"), "/kale_audio.ptx")),
        )?;
        let op = self.op as u32;
        let contiguous = u32::from(
            lhs.is_contiguous()
                && rhs.is_contiguous()
                && lhs.shape() == &self.shape
                && rhs.shape() == &self.shape,
        );
        let mut launch = func.builder();
        launch
            .arg(&lhs_slice)
            .arg(&rhs_slice)
            .arg(&mut output)
            .arg(&count)
            .arg(&op)
            .arg(&self.alpha)
            .arg(&contiguous)
            .arg(&layout);
        unsafe { launch.launch(LaunchConfig::for_num_elems(count)) }
            .map_err(candle::Error::wrap)?;
        Ok((
            candle::CudaStorage::wrap_cuda_slice(output, device.clone()),
            self.shape.clone(),
        ))
    }
}

pub(super) fn try_elementwise(
    left: &Tensor,
    right: &Tensor,
    op: Binary,
    alpha: f32,
) -> Result<Option<Tensor>> {
    if left.dtype() != DType::F32
        || right.dtype() != DType::F32
        || left.rank().max(right.rank()) > 4
        || left.elem_count() == 0
        || right.elem_count() == 0
        || !left.device().is_cuda()
        || !left.device().same_device(right.device())
    {
        return Ok(None);
    }
    let shape = left
        .shape()
        .broadcast_shape_binary_op(right.shape(), "Nano elementwise")?;
    if shape.elem_count() > u32::MAX as usize {
        return Ok(None);
    }
    left.apply_op2_no_bwd(right, &Elementwise { op, alpha, shape })
        .map(Some)
}

struct Range {
    start: i64,
    step: i64,
    length: usize,
    dtype: DType,
}

impl candle::CustomOp1 for Range {
    fn name(&self) -> &'static str {
        "kale-nano-range"
    }

    fn cpu_fwd(&self, _: &candle::CpuStorage, _: &Layout) -> Result<(candle::CpuStorage, Shape)> {
        candle::bail!("Nano range requires CUDA")
    }

    fn cuda_fwd(
        &self,
        input: &candle::CudaStorage,
        _: &Layout,
    ) -> Result<(candle::CudaStorage, Shape)> {
        use candle::cuda::cudarc::driver::{DeviceRepr, LaunchConfig, PushKernelArg};

        fn launch<T: candle::cuda::CudaDType + DeviceRepr>(
            device: &candle::cuda::CudaDevice,
            count: u32,
            name: &'static str,
            start: i64,
            step: i64,
        ) -> Result<candle::CudaStorage> {
            let mut output = unsafe { device.alloc::<T>(count as usize) }?;
            let func = device.get_or_load_custom_func(
                name,
                "kale_tts_audio",
                include_str!(concat!(env!("OUT_DIR"), "/kale_audio.ptx")),
            )?;
            let mut args = func.builder();
            args.arg(&mut output).arg(&count).arg(&start).arg(&step);
            unsafe { args.launch(LaunchConfig::for_num_elems(count)) }
                .map_err(candle::Error::wrap)?;
            Ok(candle::CudaStorage::wrap_cuda_slice(output, device.clone()))
        }

        let count = u32::try_from(self.length).map_err(candle::Error::wrap)?;
        let output = match self.dtype {
            DType::I64 => launch::<i64>(
                &input.device,
                count,
                "kale_range_i64",
                self.start,
                self.step,
            )?,
            DType::F32 => launch::<f32>(
                &input.device,
                count,
                "kale_range_f32",
                self.start,
                self.step,
            )?,
            dtype => candle::bail!("unexpected Nano range type {dtype:?}"),
        };
        Ok((output, self.length.into()))
    }
}

pub(super) fn zero_range(source: &Tensor, length: usize, dtype: DType) -> Result<Tensor> {
    source.apply_op1_no_bwd(&Range {
        length,
        dtype,
        start: 0,
        step: 1,
    })
}

pub(super) struct ReflectPad {
    pub left: usize,
    pub right: usize,
}

impl candle::CustomOp1 for ReflectPad {
    fn name(&self) -> &'static str {
        "kale-nano-reflect-pad"
    }

    fn cpu_fwd(&self, _: &candle::CpuStorage, _: &Layout) -> Result<(candle::CpuStorage, Shape)> {
        candle::bail!("Nano reflect padding requires CUDA")
    }

    fn cuda_fwd(
        &self,
        input: &candle::CudaStorage,
        layout: &Layout,
    ) -> Result<(candle::CudaStorage, Shape)> {
        use candle::cuda::cudarc::driver::{LaunchConfig, PushKernelArg};

        let source_len = layout.dims()[2];
        let count = source_len + self.left + self.right;
        let mut output = unsafe { input.device.alloc::<f32>(count) }?;
        let func = input.device.get_or_load_custom_func(
            "kale_reflect_pad_f32",
            "kale_tts_audio",
            include_str!(concat!(env!("OUT_DIR"), "/kale_audio.ptx")),
        )?;
        let count_arg = u32::try_from(count).map_err(candle::Error::wrap)?;
        let source = input.as_cuda_slice::<f32>()?.slice(layout.start_offset()..);
        let mut args = func.builder();
        args.arg(&source)
            .arg(&mut output)
            .arg(&count_arg)
            .arg(&source_len)
            .arg(&self.left)
            .arg(&layout.stride()[2]);
        unsafe { args.launch(LaunchConfig::for_num_elems(count_arg)) }
            .map_err(candle::Error::wrap)?;
        Ok((
            candle::CudaStorage::wrap_cuda_slice(output, input.device.clone()),
            (1, 1, count).into(),
        ))
    }
}

pub(super) fn slice_indexes(source: &Tensor, start: i64, end: i64, step: i64) -> Result<Tensor> {
    let distance = (i128::from(end) - i128::from(start)) * i128::from(step.signum());
    let step_abs = i128::from(step).abs();
    let length = (distance.max(0) + step_abs - 1) / step_abs;
    if length == 0 || length > i128::from(u32::MAX) {
        return Tensor::arange_step(start, end, step, source.device());
    }
    source.apply_op1_no_bwd(&Range {
        length: length as usize,
        dtype: DType::I64,
        start,
        step,
    })
}
