use candle_core as candle;
use crate::onnx::attribute_proto::AttributeType;
use crate::onnx::tensor_proto::DataType;
use crate::onnx::{self, GraphProto};
use candle::Module;
#[cfg(feature = "cuda")]
use super::nano_cuda::{Binary, try_elementwise};
use candle::{bail, DType, Device, IndexOp, Result, Tensor};
use candle_nn::activation::PReLU;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicUsize, Ordering};

static PROFILE_EVAL_CALLS: AtomicUsize = AtomicUsize::new(0);

#[cfg(feature = "cuda")]
fn pointwise_fusion_enabled() -> bool {
    std::env::var_os("KALE_TTS_ENABLE_FUSED_POINTWISE_TAIL").is_some()
}

#[cfg(feature = "cuda")]
struct DequantizeI16Cuda {
    axis: usize,
}

#[cfg(feature = "cuda")]
impl candle::CustomOp3 for DequantizeI16Cuda {
    fn name(&self) -> &'static str {
        "kale-dequantize-i16"
    }

    fn cpu_fwd(
        &self,
        _quantized: &candle::CpuStorage,
        _quantized_layout: &candle::Layout,
        _scale: &candle::CpuStorage,
        _scale_layout: &candle::Layout,
        _zero: &candle::CpuStorage,
        _zero_layout: &candle::Layout,
    ) -> Result<(candle::CpuStorage, candle::Shape)> {
        bail!("the fused I16 dequantization op is CUDA-only")
    }

    fn cuda_fwd(
        &self,
        quantized: &candle::CudaStorage,
        quantized_layout: &candle::Layout,
        scale: &candle::CudaStorage,
        scale_layout: &candle::Layout,
        zero: &candle::CudaStorage,
        zero_layout: &candle::Layout,
    ) -> Result<(candle::CudaStorage, candle::Shape)> {
        use candle::cuda::cudarc::driver::{LaunchConfig, PushKernelArg};

        let device = quantized.device.clone();
        let dims = quantized_layout.dims();
        let element_count = dims.iter().product::<usize>();
        let scale_count = scale_layout.dims().iter().product::<usize>();
        let zero_count = zero_layout.dims().iter().product::<usize>();
        if self.axis >= dims.len() {
            bail!("dequantization axis {} exceeds tensor rank {}", self.axis, dims.len());
        }
        let axis_dim = dims[self.axis];
        let inner_size = dims[self.axis + 1..].iter().product::<usize>().max(1);
        if scale_count != 1 && scale_count != axis_dim {
            bail!("dequantization scale has {scale_count} elements for axis size {axis_dim}");
        }
        if zero_count != 1 && zero_count != scale_count {
            bail!("dequantization zero point has {zero_count} elements for {scale_count} scales");
        }
        if !quantized_layout.is_contiguous()
            || !scale_layout.is_contiguous()
            || !zero_layout.is_contiguous()
            || quantized_layout.start_offset() != 0
            || scale_layout.start_offset() != 0
            || zero_layout.start_offset() != 0
        {
            bail!("fused I16 dequantization requires contiguous tensors without offsets");
        }
        if element_count == 0 {
            let output = unsafe { quantized.device.alloc::<f32>(0) }?;
            return Ok((
                candle::CudaStorage::wrap_cuda_slice(output, quantized.device.clone()),
                quantized_layout.shape().clone(),
            ));
        }

        let quantized_slice = quantized.as_cuda_slice::<i16>()?;
        let scale = scale.as_cuda_slice::<f32>()?;
        let zero = zero.as_cuda_slice::<i16>()?;
        let mut output = unsafe { device.alloc::<f32>(element_count) }?;
        let count = i32::try_from(element_count).map_err(candle::Error::wrap)?;
        let axis_dim = i32::try_from(axis_dim).map_err(candle::Error::wrap)?;
        let inner_size = i32::try_from(inner_size).map_err(candle::Error::wrap)?;
        let scale_count = i32::try_from(scale_count).map_err(candle::Error::wrap)?;
        let function = device.get_or_load_custom_func(
            "kale_dequantize_i16",
            "kale_tts_audio",
            include_str!(concat!(env!("OUT_DIR"), "/kale_audio.ptx")),
        )?;
        let mut launch = function.builder();
        launch
            .arg(quantized_slice)
            .arg(scale)
            .arg(zero)
            .arg(&mut output)
            .arg(&count)
            .arg(&axis_dim)
            .arg(&inner_size)
            .arg(&scale_count);
        unsafe { launch.launch(LaunchConfig::for_num_elems(count as u32)) }
            .map_err(candle::Error::wrap)?;
        Ok((
            candle::CudaStorage::wrap_cuda_slice(output, device),
            quantized_layout.shape().clone(),
        ))
    }
}

#[cfg(feature = "cuda")]
struct ModI64Cuda {
    output_shape: candle::Shape,
    lhs_count: usize,
    rhs_count: usize,
}

#[cfg(feature = "cuda")]
impl candle::CustomOp2 for ModI64Cuda {
    fn name(&self) -> &'static str {
        "kale-mod-i64"
    }

    fn cpu_fwd(
        &self,
        _lhs: &candle::CpuStorage,
        _lhs_layout: &candle::Layout,
        _rhs: &candle::CpuStorage,
        _rhs_layout: &candle::Layout,
    ) -> Result<(candle::CpuStorage, candle::Shape)> {
        bail!("the fused I64 Mod op is CUDA-only")
    }

    fn cuda_fwd(
        &self,
        lhs: &candle::CudaStorage,
        lhs_layout: &candle::Layout,
        rhs: &candle::CudaStorage,
        rhs_layout: &candle::Layout,
    ) -> Result<(candle::CudaStorage, candle::Shape)> {
        use candle::cuda::cudarc::driver::{LaunchConfig, PushKernelArg};

        let count = self.output_shape.dims().iter().product::<usize>();
        if (self.lhs_count != count && self.lhs_count != 1)
            || (self.rhs_count != count && self.rhs_count != 1)
        {
            bail!("fused I64 Mod supports equal-sized or scalar broadcast inputs");
        }
        if !lhs_layout.is_contiguous()
            || !rhs_layout.is_contiguous()
            || lhs_layout.start_offset() != 0
            || rhs_layout.start_offset() != 0
        {
            bail!("fused I64 Mod requires contiguous inputs without offsets");
        }
        let device = lhs.device.clone();
        if count == 0 {
            let output = unsafe { device.alloc::<i64>(0) }?;
            return Ok((
                candle::CudaStorage::wrap_cuda_slice(output, device),
                self.output_shape.clone(),
            ));
        }
        let lhs_slice = lhs.as_cuda_slice::<i64>()?;
        let rhs_slice = rhs.as_cuda_slice::<i64>()?;
        let mut output = unsafe { device.alloc::<i64>(count) }?;
        let count = i32::try_from(count).map_err(candle::Error::wrap)?;
        let lhs_count = i32::try_from(self.lhs_count).map_err(candle::Error::wrap)?;
        let rhs_count = i32::try_from(self.rhs_count).map_err(candle::Error::wrap)?;
        let function = device.get_or_load_custom_func(
            "kale_mod_i64",
            "kale_tts_audio",
            include_str!(concat!(env!("OUT_DIR"), "/kale_audio.ptx")),
        )?;
        let mut launch = function.builder();
        launch
            .arg(lhs_slice)
            .arg(rhs_slice)
            .arg(&mut output)
            .arg(&count)
            .arg(&lhs_count)
            .arg(&rhs_count);
        unsafe { launch.launch(LaunchConfig::for_num_elems(count as u32)) }
            .map_err(candle::Error::wrap)?;
        Ok((
            candle::CudaStorage::wrap_cuda_slice(output, device),
            self.output_shape.clone(),
        ))
    }
}

#[cfg(feature = "cuda")]
struct UpsampleNearest1dF32Cuda {
    target_size: usize,
}

struct DepthwiseConv1dF32 {
    padding: usize,
    stride: usize,
    dilation: usize,
}

impl DepthwiseConv1dF32 {
    fn output_size(&self, source_size: usize, kernel_size: usize) -> Result<usize> {
        let effective_kernel = self
            .dilation
            .checked_mul(kernel_size.saturating_sub(1))
            .and_then(|size| size.checked_add(1))
            .ok_or_else(|| candle::Error::Msg("convolution kernel extent overflow".into()))?;
        let padded_size = source_size
            .checked_add(2 * self.padding)
            .ok_or_else(|| candle::Error::Msg("convolution padded size overflow".into()))?;
        if padded_size < effective_kernel || self.stride == 0 {
            bail!("invalid depthwise Conv1d input/kernel/stride dimensions");
        }
        Ok((padded_size - effective_kernel) / self.stride + 1)
    }
}

impl candle::CustomOp3 for DepthwiseConv1dF32 {
    fn name(&self) -> &'static str {
        "kale-depthwise-conv1d-f32"
    }

    fn cpu_fwd(
        &self,
        input: &candle::CpuStorage,
        input_layout: &candle::Layout,
        weights: &candle::CpuStorage,
        weights_layout: &candle::Layout,
        bias: &candle::CpuStorage,
        bias_layout: &candle::Layout,
    ) -> Result<(candle::CpuStorage, candle::Shape)> {
        let input = match input {
            candle::CpuStorage::F32(values) => values,
            _ => bail!("depthwise Conv1d fast path requires F32 input"),
        };
        let weights = match weights {
            candle::CpuStorage::F32(values) => values,
            _ => bail!("depthwise Conv1d fast path requires F32 weights"),
        };
        let bias = match bias {
            candle::CpuStorage::F32(values) => values,
            _ => bail!("depthwise Conv1d fast path requires F32 bias"),
        };
        let (batches, channels, source_size) = input_layout.shape().dims3()?;
        let (weight_channels, input_channels_per_group, kernel_size) =
            weights_layout.shape().dims3()?;
        if channels != weight_channels || input_channels_per_group != 1 {
            bail!("depthwise Conv1d fast path received non-depthwise weights");
        }
        let output_size = self.output_size(source_size, kernel_size)?;
        if bias_layout.dims().iter().product::<usize>() != channels {
            bail!("depthwise Conv1d bias must contain one value per channel");
        }
        let input_strides = input_layout.stride();
        let weight_strides = weights_layout.stride();
        let bias_stride = bias_layout.stride().first().copied().unwrap_or(0);
        let output_count = batches * channels * output_size;
        let mut output = vec![0.0f32; output_count];
        for batch in 0..batches {
            for channel in 0..channels {
                let bias_value = bias[bias_layout.start_offset() + channel * bias_stride];
                for out_sample in 0..output_size {
                    let mut sum = bias_value;
                    for k in 0..kernel_size {
                        let source_sample = out_sample * self.stride + k * self.dilation;
                        if source_sample >= self.padding {
                            let source_sample = source_sample - self.padding;
                            if source_sample < source_size {
                                let input_index = input_layout.start_offset()
                                    + batch * input_strides[0]
                                    + channel * input_strides[1]
                                    + source_sample * input_strides[2];
                                let weight_index = weights_layout.start_offset()
                                    + channel * weight_strides[0]
                                    + k * weight_strides[2];
                                sum = input[input_index].mul_add(weights[weight_index], sum);
                            }
                        }
                    }
                    output[(batch * channels + channel) * output_size + out_sample] = sum;
                }
            }
        }
        Ok((
            candle::CpuStorage::F32(output),
            (batches, channels, output_size).into(),
        ))
    }

    #[cfg(feature = "cuda")]
    fn cuda_fwd(
        &self,
        input: &candle::CudaStorage,
        input_layout: &candle::Layout,
        weights: &candle::CudaStorage,
        weights_layout: &candle::Layout,
        bias: &candle::CudaStorage,
        bias_layout: &candle::Layout,
    ) -> Result<(candle::CudaStorage, candle::Shape)> {
        use candle::cuda::cudarc::driver::{LaunchConfig, PushKernelArg};

        let (batches, channels, source_size) = input_layout.shape().dims3()?;
        let (weight_channels, input_channels_per_group, kernel_size) =
            weights_layout.shape().dims3()?;
        if channels != weight_channels || input_channels_per_group != 1 {
            bail!("depthwise Conv1d fast path received non-depthwise weights");
        }
        let output_size = self.output_size(source_size, kernel_size)?;
        if bias_layout.dims().iter().product::<usize>() != channels {
            bail!("depthwise Conv1d bias must contain one value per channel");
        }
        let count = batches
            .checked_mul(channels)
            .and_then(|n| n.checked_mul(output_size))
            .ok_or_else(|| candle::Error::Msg("depthwise Conv1d output size overflow".into()))?;
        let as_u64 = |value: usize| u64::try_from(value).map_err(candle::Error::wrap);
        let batches = as_u64(batches)?;
        let channels = as_u64(channels)?;
        let source_size = as_u64(source_size)?;
        let output_size = as_u64(output_size)?;
        let kernel_size = as_u64(kernel_size)?;
        let padding = as_u64(self.padding)?;
        let stride = as_u64(self.stride)?;
        let dilation = as_u64(self.dilation)?;
        let input_strides = input_layout.stride();
        let weight_strides = weights_layout.stride();
        let input_stride_batch = as_u64(input_strides[0])?;
        let input_stride_channel = as_u64(input_strides[1])?;
        let input_stride_sample = as_u64(input_strides[2])?;
        let input_offset = as_u64(input_layout.start_offset())?;
        let weight_stride_channel = as_u64(weight_strides[0])?;
        let weight_stride_sample = as_u64(weight_strides[2])?;
        let weight_offset = as_u64(weights_layout.start_offset())?;
        let bias_stride = as_u64(bias_layout.stride().first().copied().unwrap_or(0))?;
        let bias_offset = as_u64(bias_layout.start_offset())?;
        let count = u64::from(u32::try_from(count).map_err(candle::Error::wrap)?);
        let device = input.device.clone();
        let input_slice = input.as_cuda_slice::<f32>()?;
        let weight_slice = weights.as_cuda_slice::<f32>()?;
        let bias_slice = bias.as_cuda_slice::<f32>()?;
        let mut output = unsafe { device.alloc::<f32>(count as usize) }?;
        let function = device.get_or_load_custom_func(
            "kale_depthwise_conv1d_f32",
            "kale_tts_audio",
            include_str!(concat!(env!("OUT_DIR"), "/kale_audio.ptx")),
        )?;
        let mut launch = function.builder();
        launch
            .arg(input_slice)
            .arg(weight_slice)
            .arg(bias_slice)
            .arg(&mut output)
            .arg(&batches)
            .arg(&channels)
            .arg(&source_size)
            .arg(&output_size)
            .arg(&kernel_size)
            .arg(&padding)
            .arg(&stride)
            .arg(&dilation)
            .arg(&input_stride_batch)
            .arg(&input_stride_channel)
            .arg(&input_stride_sample)
            .arg(&input_offset)
            .arg(&weight_stride_channel)
            .arg(&weight_stride_sample)
            .arg(&weight_offset)
            .arg(&bias_stride)
            .arg(&bias_offset)
            .arg(&count);
        unsafe { launch.launch(LaunchConfig::for_num_elems(count as u32)) }
            .map_err(candle::Error::wrap)?;
        Ok((
            candle::CudaStorage::wrap_cuda_slice(output, device),
            (batches as usize, channels as usize, output_size as usize).into(),
        ))
    }
}

#[cfg(feature = "cuda")]
struct DepthwiseConvTranspose1dF32Cuda {
    padding: usize,
    stride: usize,
    dilation: usize,
    output_padding: usize,
}

#[cfg(feature = "cuda")]
impl candle::CustomOp3 for DepthwiseConvTranspose1dF32Cuda {
    fn name(&self) -> &'static str {
        "kale-depthwise-convtranspose1d-f32"
    }

    fn cpu_fwd(
        &self,
        _input: &candle::CpuStorage,
        _input_layout: &candle::Layout,
        _weights: &candle::CpuStorage,
        _weights_layout: &candle::Layout,
        _bias: &candle::CpuStorage,
        _bias_layout: &candle::Layout,
    ) -> Result<(candle::CpuStorage, candle::Shape)> {
        bail!("the fused depthwise ConvTranspose1d op is CUDA-only")
    }

    fn cuda_fwd(
        &self,
        input: &candle::CudaStorage,
        input_layout: &candle::Layout,
        weights: &candle::CudaStorage,
        weights_layout: &candle::Layout,
        bias: &candle::CudaStorage,
        bias_layout: &candle::Layout,
    ) -> Result<(candle::CudaStorage, candle::Shape)> {
        use candle::cuda::cudarc::driver::{LaunchConfig, PushKernelArg};

        let (batches, channels, source_size) = input_layout.shape().dims3()?;
        let (weight_channels, channels_per_group, kernel_size) = weights_layout.shape().dims3()?;
        if channels != weight_channels || channels_per_group != 1 || self.stride == 0 || source_size == 0 || kernel_size == 0 {
            bail!("depthwise ConvTranspose1d fast path received incompatible dimensions");
        }
        if bias_layout.dims().iter().product::<usize>() != channels {
            bail!("depthwise ConvTranspose1d bias must contain one value per channel");
        }
        let effective_kernel = self.dilation
            .checked_mul(kernel_size - 1)
            .ok_or_else(|| candle::Error::Msg("depthwise ConvTranspose1d kernel overflow".into()))?;
        let total_padding = 2usize.checked_mul(self.padding)
            .ok_or_else(|| candle::Error::Msg("depthwise ConvTranspose1d padding overflow".into()))?;
        let output_size = (source_size - 1)
            .checked_mul(self.stride)
            .and_then(|size| size.checked_add(self.output_padding))
            .and_then(|size| size.checked_add(effective_kernel))
            .and_then(|size| size.checked_add(1))
            .and_then(|size| size.checked_sub(total_padding))
            .ok_or_else(|| candle::Error::Msg("depthwise ConvTranspose1d output size overflow".into()))?;
        let count = batches
            .checked_mul(channels)
            .and_then(|n| n.checked_mul(output_size))
            .ok_or_else(|| candle::Error::Msg("depthwise ConvTranspose1d output overflow".into()))?;
        let as_u64 = |v: usize| u64::try_from(v).map_err(candle::Error::wrap);
        let batches_arg = as_u64(batches)?;
        let channels_arg = as_u64(channels)?;
        let source_size_arg = as_u64(source_size)?;
        let output_size_arg = as_u64(output_size)?;
        let kernel_size_arg = as_u64(kernel_size)?;
        let padding = as_u64(self.padding)?;
        let stride = as_u64(self.stride)?;
        let dilation = as_u64(self.dilation)?;
        let input_strides = input_layout.stride();
        let weight_strides = weights_layout.stride();
        let input_stride_batch = as_u64(input_strides[0])?;
        let input_stride_channel = as_u64(input_strides[1])?;
        let input_stride_sample = as_u64(input_strides[2])?;
        let input_offset = as_u64(input_layout.start_offset())?;
        let weight_stride_channel = as_u64(weight_strides[0])?;
        let weight_stride_sample = as_u64(weight_strides[2])?;
        let weight_offset = as_u64(weights_layout.start_offset())?;
        let bias_stride = as_u64(bias_layout.stride()[0])?;
        let bias_offset = as_u64(bias_layout.start_offset())?;
        let count = u64::from(u32::try_from(count).map_err(candle::Error::wrap)?);
        let device = input.device.clone();
        let input_slice = input.as_cuda_slice::<f32>()?;
        let weight_slice = weights.as_cuda_slice::<f32>()?;
        let bias_slice = bias.as_cuda_slice::<f32>()?;
        let mut output = unsafe { device.alloc::<f32>(count as usize) }?;
        let function = device.get_or_load_custom_func(
            "kale_depthwise_convtranspose1d_f32",
            "kale_tts_audio",
            include_str!(concat!(env!("OUT_DIR"), "/kale_audio.ptx")),
        )?;
        let mut launch = function.builder();
        launch
            .arg(input_slice)
            .arg(weight_slice)
            .arg(bias_slice)
            .arg(&mut output)
            .arg(&batches_arg)
            .arg(&channels_arg)
            .arg(&source_size_arg)
            .arg(&output_size_arg)
            .arg(&kernel_size_arg)
            .arg(&padding)
            .arg(&stride)
            .arg(&dilation)
            .arg(&input_stride_batch)
            .arg(&input_stride_channel)
            .arg(&input_stride_sample)
            .arg(&input_offset)
            .arg(&weight_stride_channel)
            .arg(&weight_stride_sample)
            .arg(&weight_offset)
            .arg(&bias_stride)
            .arg(&bias_offset)
            .arg(&count);
        unsafe { launch.launch(LaunchConfig::for_num_elems(count as u32)) }
            .map_err(candle::Error::wrap)?;
        Ok((
            candle::CudaStorage::wrap_cuda_slice(output, device),
            (batches, channels, output_size).into(),
        ))
    }
}

#[cfg(feature = "cuda")]
struct FusedLstmStepCuda {
    recurrent_weights: Tensor,
    recurrent_bias: Tensor,
    previous_hidden: Tensor,
    previous_cell: Tensor,
    hidden_size: usize,
    sequence_reverse: Option<bool>,
    cooperative: bool,
}

#[cfg(feature = "cuda")]
impl candle::CustomOp1 for FusedLstmStepCuda {
    fn name(&self) -> &'static str {
        "kale-lstm-step-f32"
    }

    fn cpu_fwd(
        &self,
        _input: &candle::CpuStorage,
        _layout: &candle::Layout,
    ) -> Result<(candle::CpuStorage, candle::Shape)> {
        bail!("the fused LSTM step is CUDA-only")
    }

    fn cuda_fwd(
        &self,
        input: &candle::CudaStorage,
        input_layout: &candle::Layout,
    ) -> Result<(candle::CudaStorage, candle::Shape)> {
        use candle::cuda::cudarc::driver::{LaunchConfig, PushKernelArg};

        fn cuda_storage<'a>(tensor: &'a Tensor) -> Result<(candle::StorageRef<'a>, &'a candle::Layout)> {
            Ok(tensor.storage_and_layout())
        }
        let (weight_storage, weight_layout) = cuda_storage(&self.recurrent_weights)?;
        let (bias_storage, bias_layout) = cuda_storage(&self.recurrent_bias)?;
        let (hidden_storage, hidden_layout) = cuda_storage(&self.previous_hidden)?;
        let (cell_storage, cell_layout) = cuda_storage(&self.previous_cell)?;
        let candle::Storage::Cuda(weights) = &*weight_storage else {
            bail!("fused LSTM recurrent weights are not on CUDA");
        };
        let candle::Storage::Cuda(bias) = &*bias_storage else {
            bail!("fused LSTM recurrent bias is not on CUDA");
        };
        let candle::Storage::Cuda(hidden) = &*hidden_storage else {
            bail!("fused LSTM hidden state is not on CUDA");
        };
        let candle::Storage::Cuda(cell) = &*cell_storage else {
            bail!("fused LSTM cell state is not on CUDA");
        };
        let (steps, batch_size, gate_size, step_stride, batch_stride, gate_stride) =
            if self.sequence_reverse.is_some() {
                let (steps, batch, gates) = input_layout.shape().dims3()?;
                let strides = input_layout.stride();
                (steps, batch, gates, strides[0], strides[1], strides[2])
            } else {
                let (batch, gates) = input_layout.shape().dims2()?;
                let strides = input_layout.stride();
                (1, batch, gates, 0, strides[0], strides[1])
            };
        if gate_size != 4 * self.hidden_size
            || weight_layout.dims() != [4 * self.hidden_size, self.hidden_size]
            || bias_layout.dims() != [4 * self.hidden_size]
            || hidden_layout.dims() != [batch_size, self.hidden_size]
            || cell_layout.dims() != [batch_size, self.hidden_size]
        {
            bail!("fused LSTM step received incompatible tensor shapes");
        }
        if self.hidden_size > 1024 || batch_size == 0 {
            bail!("fused LSTM step supports hidden sizes up to 1024 and non-empty batches");
        }
        let device = input.device.clone();
        let input_slice = input.as_cuda_slice::<f32>()?;
        let weight_slice = weights.as_cuda_slice::<f32>()?;
        let bias_slice = bias.as_cuda_slice::<f32>()?;
        let hidden_slice = hidden.as_cuda_slice::<f32>()?;
        let cell_slice = cell.as_cuda_slice::<f32>()?;
        let mut output = unsafe { device.alloc::<f32>((steps + 1) * batch_size * self.hidden_size) }?;
        let to_u64 = |v: usize| u64::try_from(v).map_err(candle::Error::wrap);
        let batch_size_arg = to_u64(batch_size)?;
        let hidden_size = to_u64(self.hidden_size)?;
        let gate_stride_batch = to_u64(batch_stride)?;
        let gate_stride_hidden = to_u64(gate_stride)?;
        let gate_offset = to_u64(input_layout.start_offset())?;
        let weight_stride_row = to_u64(weight_layout.stride()[0])?;
        let weight_stride_col = to_u64(weight_layout.stride()[1])?;
        let weight_offset = to_u64(weight_layout.start_offset())?;
        let bias_stride = to_u64(bias_layout.stride()[0])?;
        let bias_offset = to_u64(bias_layout.start_offset())?;
        let hidden_stride_batch = to_u64(hidden_layout.stride()[0])?;
        let hidden_stride_element = to_u64(hidden_layout.stride()[1])?;
        let hidden_offset = to_u64(hidden_layout.start_offset())?;
        let cell_stride_batch = to_u64(cell_layout.stride()[0])?;
        let cell_stride_element = to_u64(cell_layout.stride()[1])?;
        let cell_offset = to_u64(cell_layout.start_offset())?;
        use candle::cuda::cudarc::driver::sys::CUdevice_attribute as Attribute;
        let stream = device.cuda_stream();
        let context = stream.context();
        let cooperative_sms = if self.cooperative
            && context.attribute(Attribute::CU_DEVICE_ATTRIBUTE_COOPERATIVE_LAUNCH).unwrap_or(0) != 0
        {
            context.attribute(Attribute::CU_DEVICE_ATTRIBUTE_MULTIPROCESSOR_COUNT).unwrap_or(0) as usize
        } else { 0 };
        let cooperative = self.cooperative && batch_size == 1 && self.hidden_size == 96
            && self.hidden_size.div_ceil(8) <= cooperative_sms;
        let function = device.get_or_load_custom_func(
            if cooperative { "kale_lstm_cooperative_f32" } else if self.sequence_reverse.is_some() { "kale_lstm_sequence_f32" } else { "kale_lstm_step_f32" },
            "kale_tts_audio",
            include_str!(concat!(env!("OUT_DIR"), "/kale_audio.ptx")),
        )?;
        let mut launch = function.builder();
        launch
            .arg(input_slice)
            .arg(weight_slice)
            .arg(bias_slice)
            .arg(hidden_slice)
            .arg(cell_slice)
            .arg(&mut output)
            .arg(&batch_size_arg)
            .arg(&hidden_size)
            .arg(&gate_stride_batch)
            .arg(&gate_stride_hidden)
            .arg(&gate_offset)
            .arg(&weight_stride_row)
            .arg(&weight_stride_col)
            .arg(&weight_offset)
            .arg(&bias_stride)
            .arg(&bias_offset)
            .arg(&hidden_stride_batch)
            .arg(&hidden_stride_element)
            .arg(&hidden_offset)
            .arg(&cell_stride_batch)
            .arg(&cell_stride_element)
            .arg(&cell_offset);
        let step_count = to_u64(steps)?;
        let step_stride = to_u64(step_stride)?;
        let reverse = u64::from(self.sequence_reverse.unwrap_or(false));
        if self.sequence_reverse.is_some() {
            launch.arg(&step_count).arg(&step_stride).arg(&reverse);
        }
        let config = LaunchConfig {
            grid_dim: (u32::try_from(batch_size).map_err(candle::Error::wrap)?, 1, 1),
            // The recurrent matrix has 4*H independent output rows. 32 warps
            // let the common H=96 case cover all 384 gate rows in 12 rounds,
            // instead of 96 rounds with the old four-warp launch.
            block_dim: (1024, 1, 1),
            shared_mem_bytes: 0,
        };
        if cooperative {
            let config = LaunchConfig {
                grid_dim: ((self.hidden_size.div_ceil(8)) as u32, 1, 1),
                block_dim: (256, 1, 1),
                shared_mem_bytes: 0,
            };
            unsafe { launch.launch_cooperative(config) }.map_err(candle::Error::wrap)?;
        } else {
            unsafe { launch.launch(config) }.map_err(candle::Error::wrap)?;
        }
        Ok((
            candle::CudaStorage::wrap_cuda_slice(output, device),
            (steps + 1, batch_size, self.hidden_size).into(),
        ))
    }
}


#[cfg(feature = "cuda")]
impl candle::CustomOp1 for UpsampleNearest1dF32Cuda {
    fn name(&self) -> &'static str {
        "kale-upsample-nearest1d-f32"
    }

    fn cpu_fwd(
        &self,
        _input: &candle::CpuStorage,
        _layout: &candle::Layout,
    ) -> Result<(candle::CpuStorage, candle::Shape)> {
        bail!("the fused nearest-1d op is CUDA-only")
    }

    fn cuda_fwd(
        &self,
        input: &candle::CudaStorage,
        layout: &candle::Layout,
    ) -> Result<(candle::CudaStorage, candle::Shape)> {
        use candle::cuda::cudarc::driver::{LaunchConfig, PushKernelArg};

        let (batches, channels, source_size) = layout.shape().dims3()?;
        if source_size == 0 || self.target_size == 0 {
            bail!("nearest-1d CUDA upsampling requires non-empty source and target dimensions");
        }
        let count = batches
            .checked_mul(channels)
            .and_then(|n| n.checked_mul(self.target_size))
            .ok_or_else(|| candle::Error::Msg("nearest-1d output size overflow".to_owned()))?;
        let count = u64::from(u32::try_from(count).map_err(candle::Error::wrap)?);
        let batches = u64::try_from(batches).map_err(candle::Error::wrap)?;
        let channels = u64::try_from(channels).map_err(candle::Error::wrap)?;
        let source_size = u64::try_from(source_size).map_err(candle::Error::wrap)?;
        let target_size = u64::try_from(self.target_size).map_err(candle::Error::wrap)?;
        let strides = layout.stride();
        let stride_batch = u64::try_from(strides[0]).map_err(candle::Error::wrap)?;
        let stride_channel = u64::try_from(strides[1]).map_err(candle::Error::wrap)?;
        let stride_sample = u64::try_from(strides[2]).map_err(candle::Error::wrap)?;
        let start_offset = u64::try_from(layout.start_offset()).map_err(candle::Error::wrap)?;
        let device = input.device.clone();
        let input_slice = input.as_cuda_slice::<f32>()?;
        let mut output = unsafe { device.alloc::<f32>(count as usize) }?;
        let function = device.get_or_load_custom_func(
            "kale_upsample_nearest1d_f32",
            "kale_tts_audio",
            include_str!(concat!(env!("OUT_DIR"), "/kale_audio.ptx")),
        )?;
        let mut launch = function.builder();
        launch
            .arg(input_slice)
            .arg(&mut output)
            .arg(&batches)
            .arg(&channels)
            .arg(&source_size)
            .arg(&target_size)
            .arg(&stride_batch)
            .arg(&stride_channel)
            .arg(&stride_sample)
            .arg(&start_offset)
            .arg(&count);
        unsafe { launch.launch(LaunchConfig::for_num_elems(count as u32)) }
            .map_err(candle::Error::wrap)?;
        Ok((
            candle::CudaStorage::wrap_cuda_slice(output, device),
            (batches as usize, channels as usize, target_size as usize).into(),
        ))
    }
}

#[cfg(feature = "cuda")]
struct LayerNormLastF32Cuda {
    epsilon: f32,
    per_channel_affine: bool,
}

#[cfg(feature = "cuda")]
impl candle::CustomOp3 for LayerNormLastF32Cuda {
    fn name(&self) -> &'static str {
        "kale-layernorm-last-f32"
    }

    fn cpu_fwd(
        &self,
        _input: &candle::CpuStorage,
        _input_layout: &candle::Layout,
        _scale: &candle::CpuStorage,
        _scale_layout: &candle::Layout,
        _bias: &candle::CpuStorage,
        _bias_layout: &candle::Layout,
    ) -> Result<(candle::CpuStorage, candle::Shape)> {
        bail!("the fused last-axis LayerNormalization op is CUDA-only")
    }

    fn cuda_fwd(
        &self,
        input: &candle::CudaStorage,
        input_layout: &candle::Layout,
        scale: &candle::CudaStorage,
        scale_layout: &candle::Layout,
        bias: &candle::CudaStorage,
        bias_layout: &candle::Layout,
    ) -> Result<(candle::CudaStorage, candle::Shape)> {
        use candle::cuda::cudarc::driver::{LaunchConfig, PushKernelArg};

        let dims = input_layout.dims();
        if dims.len() != 3 || dims[2] == 0 || dims[2] > 1024 {
            bail!("fused LayerNormalization expects rank 3 and hidden size in 1..=1024");
        }
        let hidden = dims[2];
        let affine_size = if self.per_channel_affine { dims[1] } else { hidden };
        let vector_layout = |layout: &candle::Layout| {
            let shape = layout.dims();
            shape.last() == Some(&affine_size) && shape[..shape.len().saturating_sub(1)].iter().all(|&d| d == 1)
        };
        if !vector_layout(scale_layout) || !vector_layout(bias_layout) {
            bail!("fused LayerNormalization requires scale and bias vectors matching the affine axis");
        }
        let strides = input_layout.stride();
        let scale_stride = *scale_layout.stride().last().unwrap_or(&0);
        let bias_stride = *bias_layout.stride().last().unwrap_or(&0);
        let batch_count = dims[0];
        let rows = dims[1];
        let row_count = batch_count
            .checked_mul(rows)
            .ok_or_else(|| candle::Error::Msg("LayerNormalization row count overflow".into()))?;
        let element_count = input_layout.shape().elem_count();
        let device = input.device.clone();
        let input = input.as_cuda_slice::<f32>()?;
        let scale = scale.as_cuda_slice::<f32>()?;
        let bias = bias.as_cuda_slice::<f32>()?;
        let mut output = unsafe { device.alloc::<f32>(element_count) }?;
        let batches = u64::try_from(batch_count).map_err(candle::Error::wrap)?;
        let rows = u64::try_from(rows).map_err(candle::Error::wrap)?;
        let hidden = u64::try_from(hidden).map_err(candle::Error::wrap)?;
        let stride_batch = u64::try_from(strides[0]).map_err(candle::Error::wrap)?;
        let stride_row = u64::try_from(strides[1]).map_err(candle::Error::wrap)?;
        let stride_hidden = u64::try_from(strides[2]).map_err(candle::Error::wrap)?;
        let input_offset = u64::try_from(input_layout.start_offset()).map_err(candle::Error::wrap)?;
        let scale_offset = u64::try_from(scale_layout.start_offset()).map_err(candle::Error::wrap)?;
        let scale_stride = u64::try_from(scale_stride).map_err(candle::Error::wrap)?;
        let bias_offset = u64::try_from(bias_layout.start_offset()).map_err(candle::Error::wrap)?;
        let bias_stride = u64::try_from(bias_stride).map_err(candle::Error::wrap)?;
        let function = device.get_or_load_custom_func(
            "kale_layernorm_last_f32",
            "kale_tts_audio",
            include_str!(concat!(env!("OUT_DIR"), "/kale_audio.ptx")),
        )?;
        let mut launch = function.builder();
        let per_channel_affine = u64::from(self.per_channel_affine);
        let contiguous_mode = if !input_layout.is_contiguous() {
            0u64
        } else if input_layout.start_offset() % 4 == 0 {
            1
        } else {
            2
        };
        launch
            .arg(input)
            .arg(scale)
            .arg(bias)
            .arg(&mut output)
            .arg(&batches)
            .arg(&rows)
            .arg(&hidden)
            .arg(&stride_batch)
            .arg(&stride_row)
            .arg(&stride_hidden)
            .arg(&input_offset)
            .arg(&scale_offset)
            .arg(&scale_stride)
            .arg(&bias_offset)
            .arg(&bias_stride)
            .arg(&self.epsilon)
            .arg(&per_channel_affine)
            .arg(&contiguous_mode);
        let block_size = hidden
            .checked_next_power_of_two()
            .ok_or_else(|| candle::Error::Msg("LayerNormalization block size overflow".into()))?
            .max(32) as u32;
        let config = LaunchConfig {
            grid_dim: (u32::try_from(row_count).map_err(candle::Error::wrap)?, 1, 1),
            block_dim: (block_size, 1, 1),
            shared_mem_bytes: 0,
        };
        unsafe { launch.launch(config) }.map_err(candle::Error::wrap)?;
        Ok((
            candle::CudaStorage::wrap_cuda_slice(output, device),
            input_layout.shape().clone(),
        ))
    }
}

#[cfg(feature = "cuda")]
struct MulAddLeakyReluF32Cuda {
    alpha: f32,
}

#[cfg(feature = "cuda")]
impl candle::CustomOp3 for MulAddLeakyReluF32Cuda {
    fn name(&self) -> &'static str {
        "kale-mul-add-leakyrelu-f32"
    }

    fn cpu_fwd(
        &self,
        _lhs: &candle::CpuStorage,
        _lhs_layout: &candle::Layout,
        _rhs: &candle::CpuStorage,
        _rhs_layout: &candle::Layout,
        _bias: &candle::CpuStorage,
        _bias_layout: &candle::Layout,
    ) -> Result<(candle::CpuStorage, candle::Shape)> {
        bail!("the fused Mul-Add-LeakyRelu op is CUDA-only")
    }

    fn cuda_fwd(
        &self,
        lhs: &candle::CudaStorage,
        lhs_layout: &candle::Layout,
        rhs: &candle::CudaStorage,
        rhs_layout: &candle::Layout,
        bias: &candle::CudaStorage,
        bias_layout: &candle::Layout,
    ) -> Result<(candle::CudaStorage, candle::Shape)> {
        use candle::cuda::cudarc::driver::{LaunchConfig, PushKernelArg};

        if lhs_layout.dims().len() != 3
            || rhs_layout.dims().len() != 3
            || bias_layout.dims().len() != 3
        {
            bail!("fused Mul-Add-LeakyRelu requires rank-3 inputs");
        }
        let mul_shape = lhs_layout
            .shape()
            .broadcast_shape_binary_op(rhs_layout.shape(), "Mul")?;
        let output_shape = mul_shape.broadcast_shape_binary_op(bias_layout.shape(), "Add")?;
        if output_shape.dims().len() != 3 {
            bail!("fused Mul-Add-LeakyRelu output must be rank 3");
        }
        let dims = output_shape.dims();
        let compatible = |input_dims: &[usize]| {
            (0..3).all(|axis| input_dims[axis] == 1 || input_dims[axis] == dims[axis])
        };
        if !compatible(lhs_layout.dims())
            || !compatible(rhs_layout.dims())
            || !compatible(bias_layout.dims())
        {
            bail!("fused Mul-Add-LeakyRelu input dimensions do not broadcast to output");
        }

        let count = output_shape.elem_count();
        let device = lhs.device.clone();
        let lhs = lhs.as_cuda_slice::<f32>()?;
        let rhs = rhs.as_cuda_slice::<f32>()?;
        let bias = bias.as_cuda_slice::<f32>()?;
        let mut output = unsafe { device.alloc::<f32>(count) }?;
        let to_u64 = |v: usize| u64::try_from(v).map_err(candle::Error::wrap);
        let [dim0, dim1, dim2] = [dims[0], dims[1], dims[2]].map(to_u64);
        let dim0 = dim0?;
        let dim1 = dim1?;
        let dim2 = dim2?;
        let [lhs_dim0, lhs_dim1, lhs_dim2] = [
            lhs_layout.dims()[0],
            lhs_layout.dims()[1],
            lhs_layout.dims()[2],
        ]
        .map(to_u64);
        let lhs_dim0 = lhs_dim0?;
        let lhs_dim1 = lhs_dim1?;
        let lhs_dim2 = lhs_dim2?;
        let [lhs_stride0, lhs_stride1, lhs_stride2] = [
            lhs_layout.stride()[0],
            lhs_layout.stride()[1],
            lhs_layout.stride()[2],
        ]
        .map(to_u64);
        let lhs_stride0 = lhs_stride0?;
        let lhs_stride1 = lhs_stride1?;
        let lhs_stride2 = lhs_stride2?;
        let lhs_offset = to_u64(lhs_layout.start_offset())?;
        let [rhs_dim0, rhs_dim1, rhs_dim2] = [
            rhs_layout.dims()[0],
            rhs_layout.dims()[1],
            rhs_layout.dims()[2],
        ]
        .map(to_u64);
        let rhs_dim0 = rhs_dim0?;
        let rhs_dim1 = rhs_dim1?;
        let rhs_dim2 = rhs_dim2?;
        let [rhs_stride0, rhs_stride1, rhs_stride2] = [
            rhs_layout.stride()[0],
            rhs_layout.stride()[1],
            rhs_layout.stride()[2],
        ]
        .map(to_u64);
        let rhs_stride0 = rhs_stride0?;
        let rhs_stride1 = rhs_stride1?;
        let rhs_stride2 = rhs_stride2?;
        let rhs_offset = to_u64(rhs_layout.start_offset())?;
        let [bias_dim0, bias_dim1, bias_dim2] = [
            bias_layout.dims()[0],
            bias_layout.dims()[1],
            bias_layout.dims()[2],
        ]
        .map(to_u64);
        let bias_dim0 = bias_dim0?;
        let bias_dim1 = bias_dim1?;
        let bias_dim2 = bias_dim2?;
        let [bias_stride0, bias_stride1, bias_stride2] = [
            bias_layout.stride()[0],
            bias_layout.stride()[1],
            bias_layout.stride()[2],
        ]
        .map(to_u64);
        let bias_stride0 = bias_stride0?;
        let bias_stride1 = bias_stride1?;
        let bias_stride2 = bias_stride2?;
        let bias_offset = to_u64(bias_layout.start_offset())?;
        let count = u64::try_from(count).map_err(candle::Error::wrap)?;
        let function = device.get_or_load_custom_func(
            "kale_mul_add_leaky_relu_f32",
            "kale_tts_audio",
            include_str!(concat!(env!("OUT_DIR"), "/kale_audio.ptx")),
        )?;
        let mut launch = function.builder();
        launch
            .arg(lhs)
            .arg(rhs)
            .arg(bias)
            .arg(&mut output)
            .arg(&dim0)
            .arg(&dim1)
            .arg(&dim2)
            .arg(&lhs_dim0)
            .arg(&lhs_dim1)
            .arg(&lhs_dim2)
            .arg(&lhs_stride0)
            .arg(&lhs_stride1)
            .arg(&lhs_stride2)
            .arg(&lhs_offset)
            .arg(&rhs_dim0)
            .arg(&rhs_dim1)
            .arg(&rhs_dim2)
            .arg(&rhs_stride0)
            .arg(&rhs_stride1)
            .arg(&rhs_stride2)
            .arg(&rhs_offset)
            .arg(&bias_dim0)
            .arg(&bias_dim1)
            .arg(&bias_dim2)
            .arg(&bias_stride0)
            .arg(&bias_stride1)
            .arg(&bias_stride2)
            .arg(&bias_offset)
            .arg(&self.alpha)
            .arg(&count);
        let count_u32 = u32::try_from(count).map_err(candle::Error::wrap)?;
        unsafe { launch.launch(LaunchConfig::for_num_elems(count_u32)) }
            .map_err(candle::Error::wrap)?;
        Ok((
            candle::CudaStorage::wrap_cuda_slice(output, device),
            output_shape,
        ))
    }
}

pub type Value = Tensor;

/// Reorder static ONNX LSTM gates into Candle's `[i, f, c, o]` order once at
/// model load, avoiding per-request index arrays and gathers.
pub fn prepare_lstm_weights(
    model: &onnx::ModelProto,
    init: &mut HashMap<String, Tensor>,
    device: &Device,
) -> Result<()> {
    let graph = match model.graph.as_ref() {
        Some(graph) => graph,
        None => bail!("ONNX graph is empty"),
    };
    for node in graph.node.iter().filter(|node| node.op_type == "LSTM") {
        let marker = format!("__kale_lstm_gates_preordered/{}", node.name);
        if init.contains_key(&marker) {
            continue;
        }
        let Some(w_name) = node.input.get(1).filter(|name| !name.is_empty()) else { continue; };
        let Some(r_name) = node.input.get(2).filter(|name| !name.is_empty()) else { continue; };
        let (Some(w), Some(r)) = (init.get(w_name).cloned(), init.get(r_name).cloned()) else { continue; };
        if w.dtype() != DType::F32 || r.dtype() != DType::F32 || w.rank() != 3 || r.rank() != 3 {
            continue;
        }
        let directions = w.dim(0)?;
        let four_hidden = w.dim(1)?;
        if four_hidden % 4 != 0 || r.dim(0)? != directions || r.dim(1)? != four_hidden {
            continue;
        }
        let hidden = four_hidden / 4;
        let Some(b_name) = node.input.get(3).filter(|name| !name.is_empty()) else { continue; };
        let Some(b) = init.get(b_name).cloned() else { continue; };
        if b.dtype() != DType::F32 || b.rank() != 2 || b.dim(0)? != directions || b.dim(1)? != 8 * hidden {
            continue;
        }
        // Peephole weights are unsupported by the evaluator. Validate them once,
        // not on every inference call (which would read a CUDA tensor back to CPU).
        if let Some(p_name) = node.input.get(7).filter(|name| !name.is_empty()) {
            let Some(p) = init.get(p_name) else { continue; };
            if p.dtype() != DType::F32 || p.to_vec2::<f32>()?.iter().flatten().any(|v| *v != 0.0) {
                continue;
            }
        }

        let gate_order = (0..hidden)
            .chain((2 * hidden)..(3 * hidden))
            .chain((3 * hidden)..(4 * hidden))
            .chain(hidden..(2 * hidden))
            .map(|i| i as i64)
            .collect::<Vec<_>>();
        let gate_order = Tensor::from_vec(gate_order, (four_hidden,), device)?;
        let mut w_dirs = Vec::with_capacity(directions);
        let mut r_dirs = Vec::with_capacity(directions);
        let mut b_dirs = Vec::with_capacity(directions);
        for direction in 0..directions {
            w_dirs.push(w.get(direction)?.index_select(&gate_order, 0)?);
            r_dirs.push(r.get(direction)?.index_select(&gate_order, 0)?);
            let b_dir = b.get(direction)?;
            let wb = b_dir.narrow(0, 0, four_hidden)?.index_select(&gate_order, 0)?;
            let rb = b_dir.narrow(0, four_hidden, four_hidden)?.index_select(&gate_order, 0)?;
            b_dirs.push(Tensor::cat(&[&wb, &rb], 0)?);
        }
        init.insert(w_name.clone(), Tensor::stack(&w_dirs, 0)?);
        init.insert(r_name.clone(), Tensor::stack(&r_dirs, 0)?);
        init.insert(b_name.clone(), Tensor::stack(&b_dirs, 0)?);
        init.insert(marker, Tensor::zeros((), DType::U8, device)?);
    }
    Ok(())
}

pub fn dtype(dt: DataType) -> Option<DType> {
    match dt {
        DataType::Int8 => Some(DType::I16),
        DataType::Int16 => Some(DType::I16),
        DataType::Int32 => Some(DType::I32),
        DataType::Uint8 => Some(DType::U8),
        DataType::Uint32 => Some(DType::U32),
        DataType::Int64 => Some(DType::I64),
        DataType::Float16 => Some(DType::F16),
        DataType::Float => Some(DType::F32),
        DataType::Double => Some(DType::F64),
        DataType::Bool => Some(DType::U8),
        _ => None,
    }
}

/// Hoist Cast nodes whose input is a model constant so each synthesis does not
/// relaunch the same parameter conversions on the device.
pub fn prepare_constant_casts(
    model: &onnx::ModelProto,
    init: &mut HashMap<String, Tensor>,
) -> Result<usize> {
    let graph = model.graph.as_ref()
        .ok_or_else(|| candle::Error::Msg("ONNX graph is empty".into()))?;
    let mut prepared = 0;
    for node in graph.node.iter().filter(|node| node.op_type == "Cast") {
        let Some(output) = node.output.first() else { continue; };
        if init.contains_key(output) {
            continue;
        }
        let Some(input_name) = node.input.first() else { continue; };
        let Some(input) = init.get(input_name).cloned() else { continue; };
        let target: i64 = *get_attr(node, "to")?;
        let target = match DataType::try_from(target as i32) {
            Ok(DataType::Int32) => DType::I64,
            Ok(target) => dtype(target).ok_or_else(|| {
                candle::Error::Msg(format!("unsupported Cast target {target:?} in {}", node.name).into())
            })?,
            Err(_) => bail!("unsupported Cast target {target} in {}", node.name),
        };
        init.insert(output.clone(), input.to_dtype(target)?);
        prepared += 1;
    }
    Ok(prepared)
}

/// Resolve the model's quantized weights after their scale casts have been
/// prepared. A node with any request-dependent input stays in the evaluator.
pub fn prepare_quantized_weights(
    model: &onnx::ModelProto,
    initializers: &mut HashMap<String, Tensor>,
) -> Result<usize> {
    let graph = model.graph.as_ref()
        .ok_or_else(|| candle::Error::Msg("ONNX graph is empty".into()))?;
    let mut prepared = 0;
    for node in graph.node.iter().filter(|node| node.op_type == "DequantizeLinear") {
        let ([quantized_name, scale_name, zero_names @ ..], [output]) =
            (node.input.as_slice(), node.output.as_slice()) else { continue; };
        let (Some(quantized), Some(scale)) =
            (initializers.get(quantized_name), initializers.get(scale_name)) else { continue; };
        let zero = match zero_names {
            [] => None,
            [name] if name.is_empty() => None,
            [name] => {
                let Some(zero) = initializers.get(name) else { continue; };
                Some(zero)
            }
            _ => continue,
        };
        let axis = get_attr_opt::<i64>(node, "axis")?.copied().unwrap_or(1);
        let value = dequantize_linear(quantized, scale, zero, axis)?;
        initializers.insert(output.clone(), value);
        prepared += 1;
    }
    Ok(prepared)
}

fn dequantize_linear(
    quantized: &Tensor,
    scale: &Tensor,
    zero: Option<&Tensor>,
    axis: i64,
) -> Result<Tensor> {
    #[cfg(feature = "cuda")]
    if quantized.device().is_cuda()
        && quantized.dtype() == DType::I16
        && scale.dtype() == DType::F32
        && zero.is_none_or(|zero| zero.dtype() == DType::I16)
    {
        let zero = match zero {
            Some(zero) => zero.clone(),
            None => Tensor::zeros(scale.shape(), DType::I16, scale.device())?,
        };
        return quantized.apply_op3_no_bwd(
            scale,
            &zero,
            &DequantizeI16Cuda { axis: quantized.normalize_axis(axis)? },
        );
    }
    let quantized = quantized.to_dtype(DType::F32)?;
    let scale = scale.to_dtype(DType::F32)?;
    let zero = match zero {
        Some(zero) => zero.to_dtype(DType::F32)?,
        None => Tensor::zeros(scale.shape(), DType::F32, scale.device())?,
    };
    let (scale, zero) = if scale.rank() == 0 || scale.elem_count() == 1 {
        (scale, zero)
    } else {
        let axis = quantized.normalize_axis(axis)?;
        let mut shape = vec![1usize; quantized.rank()];
        shape[axis] = scale.elem_count();
        (scale.reshape(shape.as_slice())?, zero.reshape(shape.as_slice())?)
    };
    quantized.broadcast_sub(&zero)?.broadcast_mul(&scale)
}

trait Attr {
    const TYPE: AttributeType;
    fn get(attr: &onnx::AttributeProto) -> Result<&Self>;
}

trait AttrOwned: Sized {
    const TYPE: AttributeType;
    fn get(attr: &onnx::AttributeProto) -> Result<Self>;
}

impl Attr for i64 {
    const TYPE: AttributeType = AttributeType::Int;
    fn get(attr: &onnx::AttributeProto) -> Result<&Self> {
        Ok(&attr.i)
    }
}

impl Attr for f32 {
    const TYPE: AttributeType = AttributeType::Float;
    fn get(attr: &onnx::AttributeProto) -> Result<&Self> {
        Ok(&attr.f)
    }
}

impl Attr for [i64] {
    const TYPE: AttributeType = AttributeType::Ints;
    fn get(attr: &onnx::AttributeProto) -> Result<&Self> {
        Ok(attr.ints.as_slice())
    }
}

impl Attr for str {
    const TYPE: AttributeType = AttributeType::String;
    fn get(attr: &onnx::AttributeProto) -> Result<&Self> {
        std::str::from_utf8(&attr.s).map_err(candle::Error::wrap)
    }
}

impl Attr for GraphProto {
    const TYPE: AttributeType = AttributeType::Graph;
    fn get(attr: &onnx::AttributeProto) -> Result<&Self> {
        attr.g
            .as_ref()
            .ok_or_else(|| candle::Error::Msg("attribute does not contain graph".to_string()))
    }
}

impl AttrOwned for Vec<String> {
    const TYPE: AttributeType = AttributeType::Strings;
    fn get(attr: &onnx::AttributeProto) -> Result<Self> {
        let mut ret = vec![];
        for bytes in attr.strings.iter() {
            let s = String::from_utf8(bytes.clone()).map_err(candle::Error::wrap)?;
            ret.push(s);
        }
        Ok(ret)
    }
}

impl AttrOwned for Tensor {
    const TYPE: AttributeType = AttributeType::Tensor;
    fn get(attr: &onnx::AttributeProto) -> Result<Self> {
        let tensor_proto = match &attr.t {
            Some(value) => value,
            None => bail!(
                "attribute {} was of type TENSOR, but no tensor was found",
                attr.name
            ),
        };

        let data_type = match DataType::try_from(tensor_proto.data_type) {
            Ok(value) => value,
            Err(_) => bail!(
                "attribute {} of type TENSOR was an invalid data_type number {}",
                attr.name,
                tensor_proto.data_type
            ),
        };

        let dtype = match dtype(data_type) {
            Some(value) => value,
            None => bail!(
                "attribute {} of type TENSOR has an unsupported data_type {}",
                attr.name,
                data_type.as_str_name()
            ),
        };

        let mut dims = Vec::with_capacity(tensor_proto.dims.len());
        for dim in &tensor_proto.dims {
            if dim < &0 {
                bail!(
                    "attribute {} of type TENSOR has a negative dimension, which is unsupported",
                    attr.name
                )
            }
            dims.push(*dim as usize)
        }

        Tensor::from_raw_buffer(&tensor_proto.raw_data, dtype, &dims, &Device::Cpu)
    }
}

fn get_attr_<'a>(node: &'a onnx::NodeProto, name: &str) -> Result<&'a onnx::AttributeProto> {
    match node.attribute.iter().find(|attr| attr.name == name) {
        None => {
            bail!(
                "cannot find the '{name}' attribute in '{}' for {}",
                node.op_type,
                node.name
            )
        }
        Some(dt) => Ok(dt),
    }
}

fn get_attr<'a, T: Attr + ?Sized>(node: &'a onnx::NodeProto, name: &str) -> Result<&'a T> {
    let attr = get_attr_(node, name)?;
    if attr.r#type() != T::TYPE {
        bail!(
            "unsupported type {:?} for '{name}' attribute in '{}' for {}",
            attr.r#type,
            node.op_type,
            node.name
        )
    }
    T::get(attr)
}

fn get_attr_opt<'a, T: Attr + ?Sized>(
    node: &'a onnx::NodeProto,
    name: &str,
) -> Result<Option<&'a T>> {
    match node.attribute.iter().find(|attr| attr.name == name) {
        None => Ok(None),
        Some(attr) => {
            if attr.r#type() != T::TYPE {
                bail!(
                    "unsupported type {:?} for '{name}' attribute in '{}' for {}",
                    attr.r#type,
                    node.op_type,
                    node.name
                )
            }
            let val = T::get(attr)?;
            Ok(Some(val))
        }
    }
}

fn get_attr_opt_owned<T: AttrOwned>(node: &onnx::NodeProto, name: &str) -> Result<Option<T>> {
    match node.attribute.iter().find(|attr| attr.name == name) {
        None => Ok(None),
        Some(attr) => {
            if attr.r#type() != T::TYPE {
                bail!(
                    "unsupported type {:?} for '{name}' attribute in '{}' for {}",
                    attr.r#type,
                    node.op_type,
                    node.name
                )
            }
            let val = T::get(attr)?;
            Ok(Some(val))
        }
    }
}

pub fn get_tensor(t: &onnx::TensorProto, name: &str, device: &Device) -> Result<Tensor> {
    let dims: Vec<usize> = t.dims.iter().map(|&x| x as usize).collect();
    match DataType::try_from(t.data_type) {
        Ok(DataType::Int8) => {
            let values = t.raw_data.iter().map(|value| (*value as i8) as i16).collect::<Vec<_>>();
            Tensor::from_vec(values, dims, device)
        }
        Ok(DataType::Int16) => {
            let values = if !t.int32_data.is_empty() {
                t.int32_data.iter().map(|&value| value as i16).collect::<Vec<_>>()
            } else {
                t.raw_data.chunks_exact(2).map(|bytes| i16::from_le_bytes([bytes[0], bytes[1]])).collect::<Vec<_>>()
            };
            Tensor::from_vec(values, dims, device)
        }
        Ok(DataType::Int32) => {
            if t.int32_data.is_empty() {
                let len = t.raw_data.len() / 4;
                let data: &[i32] =
                    unsafe { std::slice::from_raw_parts(t.raw_data.as_ptr() as *const i32, len) };
                let data = data.iter().map(|v| *v as i64).collect::<Vec<_>>();
                Tensor::from_vec(data, len, device)
            } else {
                let data = t.int32_data.iter().map(|v| *v as i64).collect::<Vec<_>>();
                Tensor::from_vec(data, t.int32_data.len(), device)
            }
        }
        Ok(dt) => match dtype(dt) {
            Some(dt) => {
                if dt == DType::F32 && !t.float_data.is_empty() {
                    Tensor::from_slice(&t.float_data, dims.as_slice(), device)
                } else if dt == DType::F64 && !t.double_data.is_empty() {
                    Tensor::from_slice(&t.double_data, dims.as_slice(), device)
                } else if dt == DType::I64 && !t.int64_data.is_empty() {
                    Tensor::from_slice(&t.int64_data, dims.as_slice(), device)
                } else {
                    Tensor::from_raw_buffer(
                        t.raw_data.as_slice(),
                        dt,
                        dims.as_slice(),
                        device,
                    )
                }
            }
            None => {
                bail!("unsupported 'value' data-type {dt:?} for {name}")
            }
        },
        Err(_) => {
            bail!("unsupported 'value' data-type {} for {name}", t.data_type,)
        }
    }
}

fn tensor_proto_ints(tensor: &onnx::TensorProto) -> Option<Vec<i64>> {
    match DataType::try_from(tensor.data_type).ok()? {
        DataType::Int64 => {
            if !tensor.int64_data.is_empty() {
                Some(tensor.int64_data.clone())
            } else if !tensor.raw_data.is_empty() && tensor.raw_data.len() % 8 == 0 {
                Some(tensor.raw_data.chunks_exact(8).map(|bytes| {
                    i64::from_le_bytes(bytes.try_into().expect("eight-byte chunk"))
                }).collect())
            } else {
                Some(Vec::new())
            }
        }
        DataType::Int32 => {
            if !tensor.int32_data.is_empty() {
                Some(tensor.int32_data.iter().map(|&value| value as i64).collect())
            } else if !tensor.raw_data.is_empty() && tensor.raw_data.len() % 4 == 0 {
                Some(tensor.raw_data.chunks_exact(4).map(|bytes| {
                    i32::from_le_bytes(bytes.try_into().expect("four-byte chunk")) as i64
                }).collect())
            } else {
                Some(Vec::new())
            }
        }
        _ => None,
    }
}

fn collect_host_integer_constants(graph: &GraphProto) -> HashMap<String, Vec<i64>> {
    let mut needed = HashSet::new();
    for node in &graph.node {
        let input_range = match node.op_type.as_str() {
            "Gather" | "Reshape" | "Expand" | "Tile" | "CumSum" | "Squeeze"
            | "Unsqueeze" | "Pad" | "Split" | "ReduceSum" | "ReduceMax" | "ReduceL2" => 1..2,
            "ConstantOfShape" => 0..1,
            "Slice" => 1..5,
            "Range" => 0..3,
            _ => continue,
        };
        needed.extend(
            node.input.iter().skip(input_range.start).take(input_range.len())
                .filter(|name| !name.is_empty()).cloned(),
        );
    }
    let producers = graph.node.iter()
        .flat_map(|node| node.output.iter().map(move |output| (output.as_str(), node)))
        .collect::<HashMap<_, _>>();
    let mut pending = needed.iter().cloned().collect::<Vec<_>>();
    let mut visited = HashSet::new();
    while let Some(name) = pending.pop() {
        if !visited.insert(name.clone()) { continue; }
        let Some(node) = producers.get(name.as_str()) else { continue; };
        match node.op_type.as_str() {
            // Shape and Size depend on runtime tensor metadata, which the evaluator
            // already knows without reading tensor contents back from CUDA.
            "Shape" | "Size" => continue,
            "Gather" | "Add" | "Sub" | "Mul" | "Div" | "Mod" | "Unsqueeze"
            | "Squeeze" | "Cast" | "Identity" | "Slice" | "Concat" | "Range" => {
                pending.extend(node.input.iter().filter(|input| !input.is_empty()).cloned());
            }
            _ => continue,
        }
    }
    needed.extend(visited);
    let mut constants = HashMap::new();
    for tensor in &graph.initializer {
        if needed.contains(&tensor.name) {
            if let Some(values) = tensor_proto_ints(tensor) {
                constants.insert(tensor.name.clone(), values);
            }
        }
    }
    for node in graph.node.iter().filter(|node| node.op_type == "Constant") {
        let Some(output) = node.output.first() else { continue; };
        if !needed.contains(output) {
            continue;
        }
        let Some(tensor) = node.attribute.iter()
            .find(|attribute| attribute.name == "value")
            .and_then(|attribute| attribute.t.as_ref()) else { continue; };
        if let Some(values) = tensor_proto_ints(tensor) {
            constants.insert(output.clone(), values);
        }
    }
    constants
}

/// Prepare the graph's static integer sidecar once when loading an ONNX model.
/// Runtime Shape/Size and arithmetic results are still added per evaluation.
pub fn prepare_host_integer_constants(
    model: &onnx::ModelProto,
) -> Result<HashMap<String, Vec<i64>>> {
    let graph = match &model.graph {
        Some(graph) => graph,
        None => bail!("no graph defined in proto"),
    };
    Ok(collect_host_integer_constants(graph))
}

fn host_integer_binary(
    left: Option<&Vec<i64>>,
    right: Option<&Vec<i64>>,
    op: impl Fn(i64, i64) -> Option<i64>,
) -> Option<Vec<i64>> {
    let left = left?;
    let right = right?;
    let len = left.len().max(right.len());
    if (left.len() != 1 && left.len() != len) || (right.len() != 1 && right.len() != len) {
        return None;
    }
    (0..len)
        .map(|index| op(left[if left.len() == 1 { 0 } else { index }], right[if right.len() == 1 { 0 } else { index }]))
        .collect()
}

fn host_integer_binary_shape_compatible(left: &Tensor, right: &Tensor, host_count: usize) -> bool {
    let output_count = if left.rank() <= 1 && right.rank() <= 1 {
        left.elem_count().max(right.elem_count())
    } else if left.dims() == right.dims() {
        left.elem_count()
    } else {
        return false;
    };
    output_count == host_count
}

#[cfg(feature = "cuda")]
struct MulAddLeakyReluPlan {
    add_node_index: usize,
    relu_node_index: usize,
    shift_input: String,
    relu_output: String,
    alpha: f32,
}

/// Find single-use, adjacent Mul -> Add -> LeakyRelu subgraphs. The evaluator
/// only uses these plans when explicitly opted in on CUDA; the ordinary eager
/// path remains the fallback for all other graphs and tensor layouts.
#[cfg(feature = "cuda")]
fn collect_mul_add_leaky_relu_plans(graph: &GraphProto) -> HashMap<usize, MulAddLeakyReluPlan> {
    let mut use_count = HashMap::<&str, usize>::new();
    let mut consumers = HashMap::<&str, Vec<usize>>::new();
    for (node_index, node) in graph.node.iter().enumerate() {
        for name in &node.input {
            if !name.is_empty() {
                *use_count.entry(name.as_str()).or_default() += 1;
                consumers.entry(name.as_str()).or_default().push(node_index);
            }
        }
    }
    let graph_outputs = graph.output.iter().map(|value| value.name.as_str()).collect::<HashSet<_>>();
    let mut plans = HashMap::new();
    for (index, mul) in graph.node.iter().enumerate() {
        if mul.op_type != "Mul" || mul.input.len() != 2 || mul.output.len() != 1 {
            continue;
        }
        let mul_output = mul.output[0].as_str();
        let Some([add_index]) = consumers.get(mul_output).map(Vec::as_slice) else { continue; };
        if use_count.get(mul_output).copied() != Some(1) || graph_outputs.contains(mul_output) {
            continue;
        }
        let add = &graph.node[*add_index];
        if add.op_type != "Add" || add.input.len() != 2 || add.output.len() != 1 {
            continue;
        }
        let add_output = add.output[0].as_str();
        let Some([relu_index]) = consumers.get(add_output).map(Vec::as_slice) else { continue; };
        if add.input.iter().filter(|input| input.as_str() == mul_output).count() != 1
            || use_count.get(add_output).copied() != Some(1)
            || graph_outputs.contains(add_output)
        {
            continue;
        }
        let relu = &graph.node[*relu_index];
        if relu.op_type != "LeakyRelu" || relu.input.len() != 1 || relu.output.len() != 1 {
            continue;
        }
        if relu.input[0] != add_output {
            continue;
        }
        let Some(shift_input) = add.input.iter().find(|input| input.as_str() != mul_output) else {
            continue;
        };
        let alpha = match get_attr_opt::<f32>(relu, "alpha") {
            Ok(Some(alpha)) => *alpha,
            Ok(None) => 0.01,
            Err(_) => continue,
        };
        plans.insert(index, MulAddLeakyReluPlan {
            add_node_index: *add_index,
            relu_node_index: *relu_index,
            shift_input: shift_input.clone(),
            relu_output: relu.output[0].clone(),
            alpha,
        });
    }
    plans
}

// This function provides a direct evaluation of the proto.
// Longer-term, we should first convert the proto to an intermediate representation of the compute
// graph so as to make multiple evaluations more efficient.
// An example upside of this would be to remove intermediary values when they are not needed
// anymore.
pub fn simple_eval(
    model: &onnx::ModelProto,
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>> {
    let host_integer_constants = prepare_host_integer_constants(model)?;
    simple_eval_with_host_integer_constants(model, inputs, &host_integer_constants)
}

/// Evaluate literal-only portions of the graph once using the same operators
/// as the request evaluator. Keep the graph intact for callers that supply
/// their own inputs and do not prepare the model.
pub fn prepare_static_nodes(
    model: &onnx::ModelProto,
    initializers: &mut HashMap<String, Tensor>,
    host_integers: &mut HashMap<String, Vec<i64>>,
    device: &Device,
) -> Result<()> {
    let Some(graph) = model.graph.as_ref() else {
        bail!("no graph defined in proto");
    };
    let mut available = initializers.keys().map(String::as_str).collect::<HashSet<_>>();
    let mut plan = GraphProto::default();
    for node in &graph.node {
        if !matches!(
            node.op_type.as_str(),
            "Cast" | "ConstantOfShape" | "Identity" | "Concat" | "Slice"
                | "Transpose" | "Reshape" | "Unsqueeze" | "Squeeze"
                | "Add" | "Sub" | "Mul" | "Div" | "Mod" | "Neg" | "Sin" | "Cos"
        ) || !node.input.iter().all(|name| name.is_empty() || available.contains(name.as_str()))
            || node.output.iter().all(|name| name.is_empty() || available.contains(name.as_str()))
        {
            continue;
        }
        plan.node.push(node.clone());
        for name in node.output.iter().filter(|name| !name.is_empty()) {
            available.insert(name);
            plan.output.push(onnx::ValueInfoProto { name: name.clone(), ..Default::default() });
        }
    }
    if plan.node.is_empty() {
        return Ok(());
    }
    let mut values = initializers.clone();
    let prepared = simple_eval_(&plan, &mut values, device, &mut host_integers.clone(), None, false)?;
    for (name, tensor) in prepared {
        if tensor.dtype() == DType::I64 && tensor.elem_count() <= 64 {
            let metadata = tensor.flatten_all()?.force_contiguous()?.to_vec1::<i64>()?;
            host_integers.insert(name.clone(), metadata);
        }
        initializers.insert(name, tensor);
    }
    Ok(())
}

#[derive(Default)]
pub(crate) struct HostShapePlan {
    nodes: HashSet<usize>,
    constants: HashMap<String, Tensor>,
    controls: HashMap<String, Tensor>,
}

#[cfg(feature = "cuda")]
impl HostShapePlan {
    pub(crate) fn tail(&self, index: usize) -> Self {
        Self {
            nodes: self.nodes.iter().filter_map(|id| id.checked_sub(index)).collect(),
            constants: self.constants.clone(),
            controls: self.controls.clone(),
        }
    }
}

pub(crate) fn prepare_host_shape_plan(
    model: &onnx::ModelProto,
    initializers: &HashMap<String, Tensor>,
) -> Result<HostShapePlan> {
    let Some(graph) = model.graph.as_ref() else {
        bail!("no graph defined in proto");
    };
    if has_subgraphs(graph) {
        // A nested graph can capture a value without listing it as a node input.
        return Ok(HostShapePlan::default());
    }
    let mut known: HashSet<&str> = initializers.iter()
        .filter(|(_, tensor)| tensor.dtype() == DType::I64 && tensor.elem_count() <= 64)
        .map(|(name, _)| name.as_str())
        .collect();
    let mut users = HashMap::<&str, Vec<(usize, usize)>>::new();
    let mut candidates = HashSet::new();
    for (index, node) in graph.node.iter().enumerate() {
        for (slot, name) in node.input.iter().enumerate() {
            if !name.is_empty() {
                users.entry(name).or_default().push((index, slot));
            }
        }
        if node.output.iter().all(|name| name.is_empty() || known.contains(name.as_str())) {
            continue;
        }
        let literal_shape_op = matches!(
            node.op_type.as_str(),
            "Add" | "Sub" | "Mul" | "Div" | "Squeeze" | "Unsqueeze" |
            "Concat" | "Gather" | "Slice" | "Reshape" | "Identity" |
            "Equal" | "Where" | "Cast" | "ConstantOfShape"
        ) && node.input.iter().all(|name| name.is_empty() || known.contains(name.as_str()));
        if literal_shape_op || matches!(node.op_type.as_str(), "Shape" | "Size") {
            candidates.insert(index);
            known.extend(node.output.iter().filter(|name| !name.is_empty()).map(String::as_str));
        }
    }
    let exported: HashSet<_> = graph.output.iter().map(|output| output.name.as_str()).collect();
    let producers: HashMap<_, _> = graph.node.iter().enumerate()
        .flat_map(|(index, node)| node.output.iter().map(move |name| (name.as_str(), index)))
        .collect();
    // Remove nodes that have a device-bound consumer or require a device
    // input. A removal can affect either side of the graph boundary.
    loop {
        let rejected = candidates.iter().copied().filter(|&index| {
            let node = &graph.node[index];
            let needs_device = node.output.iter().any(|name| {
                exported.contains(name.as_str()) || users.get(name.as_str()).is_some_and(|uses| {
                    uses.iter().any(|&(user, slot)| {
                        !candidates.contains(&user) && !host_control_slot(&graph.node[user].op_type, slot)
                    })
                })
            });
            let has_device_input = !matches!(node.op_type.as_str(), "Shape" | "Size")
                && node.input.iter().any(|name| {
                    !name.is_empty() && !initializers.contains_key(name)
                        && !producers.get(name.as_str()).is_some_and(|index| candidates.contains(index))
                });
            needs_device || has_device_input
        }).collect::<Vec<_>>();
        if rejected.is_empty() {
            break;
        }
        for index in rejected {
            candidates.remove(&index);
        }
    }
    let mut plan = HostShapePlan { nodes: candidates, ..Default::default() };
    for node in &graph.node {
        for (slot, name) in node.input.iter().enumerate() {
            if (host_control_slot(&node.op_type, slot) || (node.op_type == "Pow" && slot == 1)) && !plan.controls.contains_key(name) {
                if let Some(tensor) = initializers.get(name).filter(|t| t.elem_count() <= 64) {
                    plan.controls.insert(name.clone(), tensor.to_device(&Device::Cpu)?);
                }
            }
        }
    }
    for &index in &plan.nodes {
        let node = &graph.node[index];
        if !matches!(node.op_type.as_str(), "Shape" | "Size") {
            for name in &node.input {
                if let Some(tensor) = initializers.get(name) {
                    if !plan.constants.contains_key(name) {
                        plan.constants.insert(name.clone(), tensor.to_device(&Device::Cpu)?);
                    }
                }
            }
        }
    }
    Ok(plan)
}

fn has_subgraphs(graph: &GraphProto) -> bool {
    graph.node.iter().any(|node| node.attribute.iter().any(|attribute| {
        matches!(attribute.r#type(), AttributeType::Graph | AttributeType::Graphs)
    }))
}

pub(crate) fn compact_prepared_graph(
    model: &mut onnx::ModelProto,
    initializers: &mut HashMap<String, Tensor>,
    host_integers: &mut HashMap<String, Vec<i64>>,
) {
    let Some(graph) = model.graph.as_mut() else { return; };
    if has_subgraphs(graph) {
        return;
    }
    graph.node.retain(|node| node.output.iter().all(String::is_empty)
        || node.output.iter().any(|name| !name.is_empty() && !initializers.contains_key(name)));
    graph.initializer.clear();
    let used: HashSet<_> = graph.node.iter().flat_map(|node| node.input.iter())
        .chain(graph.output.iter().map(|output| &output.name))
        .map(String::as_str)
        .collect();
    initializers.retain(|name, _| used.contains(name.as_str()) || name.starts_with("__kale_lstm_gates_preordered/"));
    host_integers.retain(|name, _| used.contains(name.as_str()));
}

fn host_control_slot(op: &str, slot: usize) -> bool {
    match op {
        "Slice" => slot > 0,
        "Reshape" | "Expand" | "Tile" | "Squeeze" | "Unsqueeze"
            | "Pad" | "Split" | "CumSum" | "ReduceSum" | "ReduceL2" | "ReduceMax" => slot == 1,
        "Resize" => slot == 2 || slot == 3,
        "Range" => slot < 3,
        "ConstantOfShape" => slot == 0,
        _ => false,
    }
}

pub fn simple_eval_with_host_integer_constants(
    model: &onnx::ModelProto,
    inputs: HashMap<String, Value>,
    prepared_host_integer_constants: &HashMap<String, Vec<i64>>,
) -> Result<HashMap<String, Value>> {
    eval_prepared(model, inputs, prepared_host_integer_constants, None, false)
}

pub(crate) fn eval_prepared(
    model: &onnx::ModelProto,
    mut inputs: HashMap<String, Value>,
    prepared_host_integer_constants: &HashMap<String, Vec<i64>>,
    host_shape_plan: Option<&HostShapePlan>,
    nano_cuda: bool,
) -> Result<HashMap<String, Value>> {
    let device = inputs
        .values()
        .next()
        .map(|value| value.device().clone())
        .unwrap_or(Device::Cpu);
    let graph = match &model.graph {
        None => bail!("no graph defined in proto"),
        Some(graph) => graph,
    };
    let mut host_integer_values = prepared_host_integer_constants.clone();
    simple_eval_(graph, &mut inputs, &device, &mut host_integer_values, host_shape_plan, nano_cuda)
}

fn simple_eval_(
    graph: &onnx::GraphProto,
    values: &mut HashMap<String, Value>,
    device: &Device,
    host_integer_values: &mut HashMap<String, Vec<i64>>,
    host_shape_plan: Option<&HostShapePlan>,
    _nano_cuda: bool,
) -> Result<HashMap<String, Value>> {
    for t in graph.initializer.iter() {
        if !values.contains_key(&t.name) {
            let tensor = get_tensor(t, t.name.as_str(), device)?;
            values.insert(t.name.to_string(), tensor);
        }
    }
    for input in graph.input.iter() {
        let input_type = match &input.r#type {
            Some(input_type) => input_type,
            None => continue,
        };
        let input_type = match &input_type.value {
            Some(input_type) => input_type,
            None => continue,
        };
        let tensor_type = match input_type {
            onnx::type_proto::Value::TensorType(tt) => tt,
            _ => continue,
        };

        let tensor = match values.get(&input.name) {
            None => bail!("missing input {}", input.name),
            Some(tensor) => tensor,
        };
        let dt = match DataType::try_from(tensor_type.elem_type) {
            Ok(dt) => match dtype(dt) {
                Some(dt) => dt,
                None => {
                    bail!("unsupported 'value' data-type {dt:?} for {}", input.name)
                }
            },
            type_ => bail!("unsupported input type {type_:?}"),
        };
        match &tensor_type.shape {
            None => continue,
            Some(shape) => {
                if shape.dim.len() != tensor.rank() {
                    bail!(
                        "unexpected rank for {}, got {:?}, expected {:?}",
                        input.name,
                        shape.dim,
                        tensor.shape()
                    )
                }
                for (idx, (d, &dim)) in shape.dim.iter().zip(tensor.dims().iter()).enumerate() {
                    match &d.value {
                        Some(onnx::tensor_shape_proto::dimension::Value::DimValue(v)) => {
                            if *v as usize != dim {
                                bail!(
                                    "unexpected dim {idx} for {}, got {:?}, expected {:?}",
                                    input.name,
                                    shape.dim,
                                    tensor.shape()
                                )
                            }
                        }
                        // We do not check equality constraints for the DimParam dimensions for now.
                        Some(onnx::tensor_shape_proto::dimension::Value::DimParam(_)) | None => (),
                    }
                }
            }
        };
        if dt != tensor.dtype() {
            bail!(
                "unexpected dtype for {}, got {:?}, expected {dt:?}",
                input.name,
                tensor.dtype()
            )
        }
    }
    // The nodes are topologically sorted so we can just process them in order.
    let profile_mode = std::env::var("KALE_TTS_PROFILE_OPS").ok();
    let profile_ops = match profile_mode.as_deref() {
        Some("warm") => PROFILE_EVAL_CALLS.fetch_add(1, Ordering::Relaxed) > 0,
        Some(_) => true,
        None => false,
    };
    let trace_ops = std::env::var_os("KALE_TTS_TRACE_OPS").is_some();
    #[cfg(feature = "cuda")]
    let pointwise_fusions = if device.is_cuda() && pointwise_fusion_enabled() {
        collect_mul_add_leaky_relu_plans(graph)
    } else {
        HashMap::new()
    };
    #[cfg(feature = "cuda")]
    let enable_fused_layernorm = device.is_cuda()
        && (_nano_cuda || std::env::var_os("KALE_TTS_ENABLE_FUSED_LAYERNORM").is_some())
        && std::env::var_os("KALE_TTS_DISABLE_FUSED_NORMS").is_none();
    #[cfg(feature = "cuda")]
    let enable_fused_instance_norm =
        device.is_cuda()
        && (_nano_cuda || std::env::var_os("KALE_TTS_ENABLE_FUSED_INSTANCE_NORM").is_some())
        && std::env::var_os("KALE_TTS_DISABLE_FUSED_NORMS").is_none();
    #[cfg(feature = "cuda")]
    let disable_fused_lstm_step = std::env::var_os("KALE_TTS_DISABLE_FUSED_LSTM_STEP").is_some();
    #[cfg(feature = "cuda")]
    let mut skipped_fusion_nodes = HashSet::new();
    #[cfg(feature = "cuda")]
    let persistent_lstm =
        (_nano_cuda || std::env::var_os("KALE_TTS_PERSISTENT_LSTM").is_some())
        && std::env::var_os("KALE_TTS_DISABLE_PERSISTENT_LSTM").is_none();
    #[cfg(feature = "cuda")]
    let inline_elementwise = std::env::var_os("KALE_TTS_DISABLE_INLINE_ELEMENTWISE").is_none();
    macro_rules! binary {
        ($left:expr, $right:expr, $op:ident, $fallback:ident) => {{
            #[cfg(feature = "cuda")]
            let fast = if _nano_cuda && inline_elementwise {
                try_elementwise($left, $right, Binary::$op, 0.0)?
            } else { None };
            #[cfg(not(feature = "cuda"))]
            let fast: Option<Tensor> = None;
            match fast {
                Some(output) => output,
                None => $left.$fallback($right)?,
            }
        }};
    }
    for (node_index, node) in graph.node.iter().enumerate() {
        #[cfg(feature = "cuda")]
        if skipped_fusion_nodes.remove(&node_index) {
            continue;
        }
        if node.output.iter().any(|name| !name.is_empty())
            && node.output.iter().all(|name| name.is_empty() || values.contains_key(name))
        {
            continue;
        }
        let op_start = profile_ops.then(std::time::Instant::now);
        if trace_ops {
            let args = node.input.iter().filter_map(|name| values.get(name).map(|v| format!("{name}={:?}{:?}", v.dtype(), v.dims()))).collect::<Vec<_>>();
            eprintln!("{} {} {:?}", node.op_type, node.name, args);
        }
        let host_node = host_shape_plan.filter(|plan| plan.nodes.contains(&node_index));
        let get = |input_name: &str| match host_node.and_then(|plan| plan.constants.get(input_name))
            .or_else(|| values.get(input_name)) {
            Some(value) => Ok(value),
            None => bail!("cannot find {input_name} for op '{}'", node.name),
        };
        let get_opt = |i: usize| {
            node.input
                .get(i)
                .filter(|s: &&String| !s.is_empty())
                .map(|s| get(s))
        };

        let get_control = |name: &str| match host_shape_plan.and_then(|plan| plan.controls.get(name)) {
            Some(tensor) => Ok(tensor),
            None => get(name),
        };

        let read_ints = |name: &str| -> Result<Vec<i64>> {
            match host_integer_values.get(name) {
                Some(values) => Ok(values.clone()),
                None => get(name)?.to_vec1::<i64>(),
            }
        };

        // TODO: Validate node.input for each operator.
        match node.op_type.as_str() {
            "Add" => {
                let input0 = get(&node.input[0])?;
                let input1 = get(&node.input[1])?;
                let output = binary!(input0, input1, Add, broadcast_add);
                let host_output = host_integer_binary(
                    host_integer_values.get(&node.input[0]),
                    host_integer_values.get(&node.input[1]),
                    i64::checked_add,
                ).filter(|host| host_integer_binary_shape_compatible(input0, input1, host.len()));
                values.insert(node.output[0].clone(), output);
                if let Some(host_output) = host_output {
                    host_integer_values.insert(node.output[0].clone(), host_output);
                }
            }
            "Sub" => {
                let input0 = get(&node.input[0])?;
                let input1 = get(&node.input[1])?;
                let output = binary!(input0, input1, Sub, broadcast_sub);
                let host_output = host_integer_binary(
                    host_integer_values.get(&node.input[0]),
                    host_integer_values.get(&node.input[1]),
                    i64::checked_sub,
                ).filter(|host| host_integer_binary_shape_compatible(input0, input1, host.len()));
                values.insert(node.output[0].clone(), output);
                if let Some(host_output) = host_output {
                    host_integer_values.insert(node.output[0].clone(), host_output);
                }
            }
            "Mul" => {
                if host_node.is_some() {
                    let left = get(&node.input[0])?;
                    let right = get(&node.input[1])?;
                    let output = left.broadcast_mul(right)?;
                    if let Some(data) = host_integer_binary(
                        host_integer_values.get(&node.input[0]),
                        host_integer_values.get(&node.input[1]),
                        i64::checked_mul,
                    ).filter(|data| host_integer_binary_shape_compatible(left, right, data.len())) {
                        host_integer_values.insert(node.output[0].clone(), data);
                    }
                    values.insert(node.output[0].clone(), output);
                    continue;
                }
                #[cfg(feature = "cuda")]
                let fused = if device.is_cuda() {
                    if let Some(plan) = pointwise_fusions.get(&node_index) {
                        let input0 = get(&node.input[0])?.clone();
                        let input1 = get(&node.input[1])?.clone();
                        let shift = get(&plan.shift_input)?.clone();
                        let output_shape = input0
                            .shape()
                            .broadcast_shape_binary_op(input1.shape(), "Mul")
                            .ok()
                            .and_then(|shape| shape.broadcast_shape_binary_op(shift.shape(), "Add").ok());
                        let output_is_rank3_broadcast = output_shape.as_ref().is_some_and(|shape| {
                            shape.dims().len() == 3
                                && [&input0, &input1, &shift].iter().all(|tensor| {
                                    tensor.rank() == 3
                                        && (0..3).all(|axis| {
                                            tensor.dims()[axis] == 1
                                                || tensor.dims()[axis] == shape.dims()[axis]
                                        })
                                })
                        });
                        if input0.dtype() == DType::F32
                            && input1.dtype() == DType::F32
                            && shift.dtype() == DType::F32
                            && output_is_rank3_broadcast
                        {
                            let output = input0.apply_op3_no_bwd(
                                &input1,
                                &shift,
                                &MulAddLeakyReluF32Cuda { alpha: plan.alpha },
                            )?;
                            if std::env::var_os("KALE_TTS_VALIDATE_FUSED_POINTWISE_TAIL").is_some() {
                                let product = input0.broadcast_mul(&input1)?;
                                let affine = product.broadcast_add(&shift)?;
                                let reference = candle_nn::ops::leaky_relu(&affine, plan.alpha as f64)?;
                                let difference = output.broadcast_sub(&reference)?.abs()?;
                                let max_abs = difference.max_all()?.to_scalar::<f32>()?;
                                let mean_abs = difference.mean_all()?.to_scalar::<f32>()?;
                                eprintln!(
                                    "fused pointwise-tail validation: mul={} add={} relu={} shape={:?} max_abs={max_abs} mean_abs={mean_abs}",
                                    node.name,
                                    graph.node[plan.add_node_index].name,
                                    graph.node[plan.relu_node_index].name,
                                    reference.dims(),
                                );
                                // Keep the generic result downstream so each following
                                // comparison sees the reference graph's intermediate values.
                                values.insert(plan.relu_output.clone(), reference);
                            } else {
                                values.insert(plan.relu_output.clone(), output);
                            }
                            skipped_fusion_nodes.insert(plan.add_node_index);
                            skipped_fusion_nodes.insert(plan.relu_node_index);
                            true
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                } else {
                    false
                };
                #[cfg(feature = "cuda")]
                if !fused {
                    let output = {
                        let input0 = values.get(&node.input[0]).ok_or_else(|| {
                            candle::Error::Msg(format!("cannot find {} for op '{}'", node.input[0], node.name))
                        })?;
                        let input1 = values.get(&node.input[1]).ok_or_else(|| {
                            candle::Error::Msg(format!("cannot find {} for op '{}'", node.input[1], node.name))
                        })?;
                        binary!(input0, input1, Mul, broadcast_mul)
                    };
                    values.insert(node.output[0].clone(), output);
                }
                #[cfg(not(feature = "cuda"))]
                {
                    let input0 = get(&node.input[0])?;
                    let input1 = get(&node.input[1])?;
                    let output = input0.broadcast_mul(input1)?;
                    values.insert(node.output[0].clone(), output);
                }
                let host_output = host_integer_binary(
                    host_integer_values.get(&node.input[0]),
                    host_integer_values.get(&node.input[1]),
                    i64::checked_mul,
                ).filter(|host| match (values.get(&node.input[0]), values.get(&node.input[1])) {
                    (Some(left), Some(right)) => host_integer_binary_shape_compatible(left, right, host.len()),
                    _ => false,
                });
                if let Some(host_output) = host_output {
                    host_integer_values.insert(node.output[0].clone(), host_output);
                }
            }
            "Div" => {
                let input0 = get(&node.input[0])?;
                let input1 = get(&node.input[1])?;
                let output = binary!(input0, input1, Div, broadcast_div);
                let host_output = host_integer_binary(
                    host_integer_values.get(&node.input[0]),
                    host_integer_values.get(&node.input[1]),
                    i64::checked_div,
                ).filter(|host| host_integer_binary_shape_compatible(input0, input1, host.len()));
                values.insert(node.output[0].clone(), output);
                if let Some(host_output) = host_output {
                    host_integer_values.insert(node.output[0].clone(), host_output);
                }
            }
            "Pow" => {
                let input0 = get(&node.input[0])?;
                let input1 = get(&node.input[1])?;
                // HACK: current implementation of broadcast_pow cannot handle negative base,
                // so we use powf where we can, which *does* correctly handle negative base.
                if let Ok(exp) = to_scalar_flexible::<f64>(&get_control(&node.input[1])?.to_dtype(DType::F64)?) {
                    let output = input0.powf(exp)?;
                    values.insert(node.output[0].clone(), output);
                } else {
                    let output = input0.broadcast_pow(input1)?;
                    values.insert(node.output[0].clone(), output);
                }
            }
            "Exp" => {
                let xs = get(&node.input[0])?;
                let output = xs.exp()?;
                values.insert(node.output[0].clone(), output);
            }
            "Equal" => {
                let input0 = get(&node.input[0])?;
                let input1 = get(&node.input[1])?;
                let output = input0.broadcast_eq(input1)?;
                values.insert(node.output[0].clone(), output);
            }
            "Not" => {
                let xs = get(&node.input[0])?;
                let xs = xs.eq(&xs.zeros_like()?)?;
                values.insert(node.output[0].clone(), xs);
            }
            "MatMul" => {
                let input0 = get(&node.input[0])?;
                let input1 = get(&node.input[1])?;
                let output = input0.broadcast_matmul(input1).map_err(|e| candle::Error::Msg(format!(
                    "MatMul {} ({:?} {:?} x {:?} {:?}): {e}", node.name, input0.dtype(), input0.dims(), input1.dtype(), input1.dims()
                )))?;
                values.insert(node.output[0].clone(), output);
            }
            "Reshape" => {
                let input0 = get(&node.input[0])?;
                let input1 = match host_integer_values.get(&node.input[1]) {
                    Some(shape) => shape.clone(),
                    None => get(&node.input[1])?.to_vec1::<i64>()?,
                };
                // A 0 in the target shape copies the corresponding input dimension, unless
                // allowzero=1, where it means a literal zero-length dimension.
                let allowzero = get_attr_opt::<i64>(node, "allowzero")?
                    .copied()
                    .unwrap_or(0)
                    == 1;
                if input1.iter().filter(|&&v| v == -1).count() > 1 {
                    bail!("Reshape: at most one dimension of the target shape can be -1")
                }
                // Resolve everything but -1 first: a copied 0 is part of the volume, so it
                // has to be in the product that -1 is inferred against.
                let mut resolved: Vec<Option<usize>> = Vec::with_capacity(input1.len());
                for (idx, &v) in input1.iter().enumerate() {
                    resolved.push(match v {
                        -1 => None,
                        0 if allowzero => Some(0),
                        0 => Some(input0.dim(idx)?),
                        v if v > 0 => Some(v as usize),
                        v => bail!("Reshape: invalid dimension {v} in target shape"),
                    });
                }
                let known: usize = resolved.iter().flatten().product();
                let input1 = resolved
                    .into_iter()
                    .map(|d| match d {
                        Some(d) => Ok(d),
                        // A -1 has no unique value when the rest of the volume is zero.
                        None if known == 0 => {
                            bail!("Reshape: -1 cannot be inferred when another dimension is zero")
                        }
                        None => Ok(input0.elem_count() / known),
                    })
                    .collect::<Result<Vec<usize>>>()?;
                let output = input0.reshape(input1)?;
                values.insert(node.output[0].clone(), output);
            }
            "LogSoftmax" => {
                let input = get(&node.input[0])?;
                let output = match get_attr_opt::<i64>(node, "axis")? {
                    None => candle_nn::ops::log_softmax(input, candle::D::Minus1)?,
                    Some(&axis) => {
                        let axis = input.normalize_axis(axis)?;
                        candle_nn::ops::log_softmax(input, axis)?
                    }
                };
                values.insert(node.output[0].clone(), output);
            }
            "Softmax" => {
                let input = get(&node.input[0])?;
                let output = match get_attr_opt::<i64>(node, "axis")? {
                    None => candle_nn::ops::softmax_last_dim(input)?,
                    Some(&axis) => {
                        let axis = input.normalize_axis(axis)?;
                        if input.device().is_cuda()
                            && axis + 1 == input.rank()
                            && input.is_contiguous()
                        {
                            candle_nn::ops::softmax_last_dim(input)?
                        } else {
                            candle_nn::ops::softmax(input, axis)?
                        }
                    }
                };
                values.insert(node.output[0].clone(), output);
            }
            "Transpose" => {
                let input = get(&node.input[0])?;
                let output = match get_attr_opt::<[i64]>(node, "perm")? {
                    None => input.t()?,
                    Some(perm) => {
                        let perm = perm.iter().map(|&v| v as usize).collect::<Vec<_>>();
                        input.permute(perm)?.contiguous()?
                    }
                };
                values.insert(node.output[0].clone(), output);
            }
            "Dropout" => {
                let input = get(&node.input[0])?;
                // Do not apply dropout at the moment, consider that we're only doing inference.
                values.insert(node.output[0].clone(), input.clone());
            }
            "MaxPool" => {
                // https://github.com/onnx/onnx/blob/main/docs/Operators.md#MaxPool
                let dilations = get_attr_opt::<[i64]>(node, "dilations")?;
                let kernel_shape = get_attr::<[i64]>(node, "kernel_shape")?;
                let pads = get_attr_opt::<[i64]>(node, "pads")?;
                let strides = get_attr_opt::<[i64]>(node, "strides")?;
                let auto_pad = get_attr_opt::<str>(node, "auto_pad")?;
                match auto_pad {
                    None | Some("NOTSET") => (),
                    Some(s) => bail!("unsupported auto_pad {s}"),
                };
                if let Some(d) = dilations {
                    if d.iter().any(|&v| v != 1) {
                        bail!("MaxPool with dilation != 1, {dilations:?}")
                    }
                }
                if let Some(d) = pads {
                    if d.iter().any(|&v| v != 0) {
                        bail!("MaxPool with pads != 0, {pads:?}")
                    }
                }
                let xs = get(&node.input[0])?;
                let (k1, k2) = match kernel_shape {
                    [k1, k2] => (*k1 as usize, *k2 as usize),
                    _ => bail!("only 2d MaxPool is supported, kernel shape {kernel_shape:?}"),
                };
                let ys = match strides {
                    None => xs.max_pool2d((k1, k2))?,
                    Some([s1, s2]) => {
                        xs.max_pool2d_with_stride((k1, k2), (*s1 as usize, *s2 as usize))?
                    }
                    Some(strides) => bail!("only 2d MaxPool is supported, strides {strides:?}"),
                };
                values.insert(node.output[0].clone(), ys);
            }
            "AveragePool" => {
                // https://github.com/onnx/onnx/blob/main/docs/Operators.md#AveragePool
                let dilations = get_attr_opt::<[i64]>(node, "dilations")?;
                let kernel_shape = get_attr::<[i64]>(node, "kernel_shape")?;
                let pads = get_attr_opt::<[i64]>(node, "pads")?;
                let strides = get_attr_opt::<[i64]>(node, "strides")?;
                let auto_pad = get_attr_opt::<str>(node, "auto_pad")?;
                match auto_pad {
                    None | Some("NOTSET") => (),
                    Some(s) => bail!("unsupported auto_pad {s}"),
                };
                if let Some(d) = dilations {
                    if d.iter().any(|&v| v != 1) {
                        bail!("AvgPool with dilation != 1, {dilations:?}")
                    }
                }
                if let Some(d) = pads {
                    if d.iter().any(|&v| v != 0) {
                        bail!("AvgPool with pads != 0, {pads:?}")
                    }
                }
                let xs = get(&node.input[0])?;
                let (k1, k2) = match kernel_shape {
                    [k1, k2] => (*k1 as usize, *k2 as usize),
                    _ => bail!("only 2d AvgPool is supported, kernel shape {kernel_shape:?}"),
                };
                let ys = match strides {
                    None => xs.avg_pool2d((k1, k2))?,
                    Some([s1, s2]) => {
                        xs.avg_pool2d_with_stride((k1, k2), (*s1 as usize, *s2 as usize))?
                    }
                    Some(strides) => bail!("only 2d AvgPool is supported, strides {strides:?}"),
                };
                values.insert(node.output[0].clone(), ys);
            }
            "BatchNormalization" => {
                let training_mode = get_attr_opt::<i64>(node, "training_mode")?;
                if training_mode.copied().unwrap_or(0) != 0 {
                    bail!("training mode is not supported for BatchNorm")
                }
                let eps = get_attr_opt::<f32>(node, "epsilon")?
                    .copied()
                    .unwrap_or(1e-5);
                let xs = get(&node.input[0])?;
                let weight = get(&node.input[1])?;
                let bias = get(&node.input[2])?;
                let running_mean = get(&node.input[3])?;
                let running_var = get(&node.input[4])?;
                let target_shape: Vec<usize> = xs
                    .dims()
                    .iter()
                    .enumerate()
                    .map(|(idx, v)| if idx == 1 { *v } else { 1 })
                    .collect();
                let target_shape = target_shape.as_slice();
                let xs = xs
                    .broadcast_sub(&running_mean.reshape(target_shape)?)?
                    .broadcast_div(&(running_var.reshape(target_shape)? + eps as f64)?.sqrt()?)?;
                let weight = weight.reshape(target_shape)?;
                let bias = bias.reshape(target_shape)?;
                let xs = xs.broadcast_mul(&weight)?.broadcast_add(&bias)?;
                values.insert(node.output[0].clone(), xs);
            }
            "Squeeze" => {
                let xs = get(&node.input[0])?;
                let mut axes = if node.input.len() <= 1 {
                    // contract all the dimensions with size 1 except the batch dim.
                    xs.dims()
                        .iter()
                        .enumerate()
                        .flat_map(|(idx, &s)| if s == 1 && idx > 0 { Some(idx) } else { None })
                        .collect()
                } else {
                    read_ints(&node.input[1])?
                        .iter()
                        .map(|&i| xs.normalize_axis(i))
                        .collect::<Result<Vec<_>>>()?
                };
                axes.sort();
                let mut xs = xs.clone();
                for &axis in axes.iter().rev() {
                    xs = xs.squeeze(axis)?
                }
                if let Some(host_data) = host_integer_values.get(&node.input[0]).cloned() {
                    host_integer_values.insert(node.output[0].clone(), host_data);
                }
                values.insert(node.output[0].clone(), xs);
            }
            // https://github.com/onnx/onnx/blob/main/docs/Operators.md#ConstantOfShape
            "ConstantOfShape" => {
                let output_device = if host_node.is_some() { &Device::Cpu } else { device };
                let value = get_attr_opt_owned::<Tensor>(node, "value")?
                    .map(|value| value.to_device(output_device))
                    .transpose()?
                    .unwrap_or(Tensor::zeros((), DType::F32, output_device)?);

                let shape_vec: Vec<usize> = read_ints(&node.input[0])?
                    .iter()
                    .map(|&x| x as usize)
                    .collect();

                let xs = Tensor::ones(shape_vec, value.dtype(), output_device)?
                    .broadcast_mul(&value)?;
                values.insert(node.output[0].clone(), xs);
            }
            "Unsqueeze" => {
                let xs = get(&node.input[0])?;
                let axes = match get_attr_opt::<[i64]>(node, "axes")? {
                    Some(axis) => axis.to_vec(),
                    None => read_ints(&node.input[1])?,
                };
                let mut axes = axes
                    .iter()
                    .map(|&i| {
                        if i == xs.rank() as i64 {
                            Ok(xs.rank())
                        } else if i < 0 {
                            // normalize_axis doesn't work correctly here
                            // because we actually want normalized with respect
                            // to the final size, not the current (off by one)
                            Ok(xs.rank() - (-i as usize) + 1)
                        } else {
                            xs.normalize_axis(i)
                        }
                    })
                    .collect::<Result<Vec<_>>>()?;
                axes.sort();
                let mut xs = xs.clone();
                for &axis in axes.iter().rev() {
                    xs = xs.unsqueeze(axis)?
                }
                if let Some(host_data) = host_integer_values.get(&node.input[0]).cloned() {
                    host_integer_values.insert(node.output[0].clone(), host_data);
                }
                values.insert(node.output[0].clone(), xs);
            }
            "Clip" => {
                let xs = get(&node.input[0])?;
                let xs = if let Some(mins) = get_opt(1) {
                    xs.broadcast_maximum(mins?)?
                } else {
                    xs.clone()
                };
                let xs = if let Some(maxs) = get_opt(2) {
                    xs.broadcast_minimum(maxs?)?
                } else {
                    xs.clone()
                };
                values.insert(node.output[0].clone(), xs);
            }
            "Gather" => {
                // https://github.com/onnx/onnx/blob/main/docs/Operators.md#Gather
                let xs = get(&node.input[0])?;
                let indices_tensor = get(&node.input[1])?;
                let axis = get_attr_opt::<i64>(node, "axis")?.copied().unwrap_or(0);
                let axis = xs.normalize_axis(axis)?;
                let host_indices = host_integer_values.get(&node.input[1]).cloned();

                // Scalar constant gathers are common for the attention head. Resolve
                // them on the host from the ONNX protobuf and return a view, avoiding
                // per-request CUDA index normalization and scalar readback.
                if indices_tensor.dims().is_empty() {
                    if let Some(host_indices) = host_indices.as_ref() {
                        let Some(&mut_index) = host_indices.first() else {
                            bail!("Gather scalar index is empty in {}", node.name);
                        };
                        let axis_size = xs.dims()[axis] as i64;
                        let mut index = mut_index;
                        if index < 0 {
                            index += axis_size;
                        }
                        if index < 0 || index >= axis_size {
                            bail!("Gather index {mut_index} is out of bounds for axis {axis} of size {axis_size}");
                        }
                        let host_value = if xs.rank() == 1 && axis == 0 {
                            host_integer_values.get(&node.input[0])
                                .and_then(|host_data| host_data.get(index as usize).copied())
                        } else {
                            None
                        };
                        let gathered = xs.narrow(axis, index as usize, 1)?.squeeze(axis)?;
                        values.insert(node.output[0].clone(), gathered);
                        if let Some(host_value) = host_value {
                            host_integer_values.insert(node.output[0].clone(), vec![host_value]);
                        }
                        continue;
                    }
                }

                // index_select does not support negative indices, so normalize them
                // to positive indices.
                let indices = if host_indices.as_ref().is_some_and(|indices| indices.iter().all(|&index| index >= 0)) {
                    indices_tensor.clone()
                } else {
                    let zeros = Tensor::zeros(indices_tensor.shape(), indices_tensor.dtype(), indices_tensor.device())?;
                    let max = Tensor::new(xs.dims()[axis] as i64, indices_tensor.device())?
                        .to_dtype(indices_tensor.dtype())?;
                    let mask = indices_tensor.lt(&zeros)?;
                    mask.to_dtype(indices_tensor.dtype())?
                        .broadcast_mul(&max)?
                        .add(indices_tensor)?
                };

                // In Pytorch or Numpy this can be done by indexing the xs tensor using the indices
                // tensor directly, but candle does not support tensor indexing at the moment, so
                // some workarounds must be done.
                let host_gathered = if xs.rank() == 1 && axis == 0 {
                    host_integer_values.get(&node.input[0]).and_then(|host_data| {
                        host_indices.as_ref().map(|indices| {
                            indices.iter().map(|&index| {
                                let index = if index < 0 { index + host_data.len() as i64 } else { index };
                                host_data.get(index as usize).copied()
                            }).collect::<Option<Vec<_>>>()
                        }).flatten()
                    })
                } else {
                    None
                };
                let xs = match indices.dims() {
                    [] => {
                        let index = indices.to_vec0::<i64>()? as usize;
                        xs.narrow(axis, index, 1)?.squeeze(axis)?
                    }
                    [_] => xs.index_select(&indices, axis)?,
                    [first, _] => {
                        let mut v = Vec::with_capacity(*first);
                        for i in 0..*first {
                            v.push(xs.index_select(&indices.get(i)?, axis)?)
                        }
                        Tensor::stack(&v, axis)?
                    }
                    _ => {
                        // TODO: Provide an op to handle the ONNX generalized gather op ideally in a
                        // differentiable way.
                        todo!("implement gather for {xs:?} {indices:?} axis {axis}")
                    }
                };
                values.insert(node.output[0].clone(), xs);
                if let Some(host_gathered) = host_gathered {
                    host_integer_values.insert(node.output[0].clone(), host_gathered);
                }
            }
            // https://onnx.ai/onnx/operators/onnx__GatherElements.html#gatherelements
            // A Note to fellow lurkers:
            // The numpy based `gather_elements` implementation in `onnx` tests [here](https://github.com/onnx/onnx/blob/main/onnx/backend/test/case/node/gatherelements.py)
            // and examples is incorrect.
            // Use `torch.gather` for the validating/ verifying against the proper behaviour
            "GatherElements" => {
                let data = get(&node.input[0])?;
                let indices = get(&node.input[1])?;

                let rank = data.rank();
                if rank != indices.rank() {
                    bail!("indices must have same rank as input data. Data rank [{}] != indices rank [{}]", data.rank(), indices.rank());
                }

                let axis = {
                    let axis_i64 = get_attr_opt::<i64>(node, "axis")?.copied().unwrap_or(0);
                    let axis = data.normalize_axis(axis_i64)?;

                    if axis >= rank {
                        bail!(
                            "axis ({}) out of accepted range [-rank, rank-1] which was [-{rank}, {}]",
                            axis_i64,
                            rank - 1
                        )
                    }

                    axis
                };

                // index_select does not support negative indices, so normalize them
                // to positive indices.
                let indices = &{
                    let zeros = Tensor::zeros(indices.shape(), indices.dtype(), indices.device())?;
                    let max = Tensor::new(data.dims()[axis] as i64, indices.device())?
                        .to_dtype(indices.dtype())?;
                    let mask = indices.lt(&zeros)?;
                    mask.to_dtype(indices.dtype())?
                        .broadcast_mul(&max)?
                        .add(indices)?
                };

                values.insert(node.output[0].clone(), data.gather(indices, axis)?);
            }
            "Shape" => {
                // https://github.com/onnx/onnx/blob/main/docs/Operators.md#Shape
                let xs = get(&node.input[0])?;
                let start = get_attr_opt::<i64>(node, "start")?.copied().unwrap_or(0);
                let end = get_attr_opt::<i64>(node, "end")?.copied().unwrap_or(xs.rank() as i64);
                let normalize_shape_bound = |axis: i64| -> usize {
                    if axis < 0 { (xs.rank() as i64 + axis).clamp(0, xs.rank() as i64) as usize }
                    else { axis.clamp(0, xs.rank() as i64) as usize }
                };
                let start = normalize_shape_bound(start);
                let end = normalize_shape_bound(end);
                let mut dims = vec![];
                for idx in start..end {
                    dims.push(xs.dim(idx)? as i64)
                }
                host_integer_values.insert(node.output[0].clone(), dims.clone());
                let shape_device = if host_node.is_some() { &Device::Cpu } else { xs.device() };
                let dims_tensor = Tensor::from_vec(dims.clone(), (dims.len(),), shape_device)?;
                values.insert(node.output[0].clone(), dims_tensor);
            }
            // https://github.com/onnx/onnx/blob/main/docs/Operators.md#Size
            "Size" => {
                let data = get(&node.input[0])?;
                let size: usize = data.dims().iter().product();
                let shape_device = if host_node.is_some() { &Device::Cpu } else { data.device() };
                let output = Tensor::from_slice(&[size as i64], (), shape_device)?;
                host_integer_values.insert(node.output[0].clone(), vec![size as i64]);
                values.insert(node.output[0].clone(), output);
            }
            // https://github.com/onnx/onnx/blob/main/docs/Operators.md#Sqrt
            "Sqrt" => {
                let xs = get(&node.input[0])?;
                let output = xs.sqrt()?;
                values.insert(node.output[0].clone(), output);
            }
            // https://github.com/onnx/onnx/blob/main/docs/Operators.md#Range
            "Range" => {
                let start = get_control(&node.input[0])?;
                let limit = get_control(&node.input[1])?;
                let delta = get_control(&node.input[2])?;

                macro_rules! arange_step {
                    ($t: ty) => {
                        Tensor::arange_step(
                            to_vec0_flexible::<$t>(start)?,
                            to_vec0_flexible::<$t>(limit)?,
                            to_vec0_flexible::<$t>(delta)?,
                            device,
                        )?
                    };
                }

                let output = match start.dtype() {
                    DType::U8 => arange_step!(u8),
                    DType::U32 => arange_step!(u32),
                    DType::I64 => match (
                        host_integer_values.get(&node.input[0]).map(Vec::as_slice),
                        host_integer_values.get(&node.input[1]).map(Vec::as_slice),
                        host_integer_values.get(&node.input[2]).map(Vec::as_slice),
                    ) {
                        #[cfg(feature = "cuda")]
                        (Some([0]), Some([limit]), Some([1]))
                            if _nano_cuda && (1..=i64::from(u32::MAX)).contains(limit) =>
                        {
                            super::nano_cuda::zero_range(get(&node.input[0])?, *limit as usize, DType::I64)?
                        }
                        (Some([start_value]), Some([limit]), Some([delta])) => {
                            Tensor::arange_step(*start_value, *limit, *delta, device)?
                        }
                        _ => arange_step!(i64),
                    },
                    DType::BF16 => arange_step!(f32),
                    DType::F16 => arange_step!(f32),
                    #[cfg(feature = "cuda")]
                    DType::F32 if _nano_cuda => {
                        let start = to_vec0_flexible::<f32>(start)?;
                        let limit = to_vec0_flexible::<f32>(limit)?;
                        let delta = to_vec0_flexible::<f32>(delta)?;
                        if start.to_bits() == 0 && delta == 1.0 && (1.0..=16_777_216.0).contains(&limit) {
                            super::nano_cuda::zero_range(get(&node.input[0])?, limit.ceil() as usize, DType::F32)?
                        } else { Tensor::arange_step(start, limit, delta, device)? }
                    }
                    DType::F32 => arange_step!(f32),
                    DType::F64 => arange_step!(f64),
                    DType::F8E4M3 => arange_step!(f32),
                    DType::I32
                    | DType::I16
                    | DType::F6E2M3
                    | DType::F6E3M2
                    | DType::F4
                    | DType::F8E8M0 => {
                        bail!("unsupported Range type i32/i16/f6e2m3/f6e3m2/f4/f8e8m0")
                    }
                    dt => bail!("unsupported Range dtype {dt:?}"),
                };

                values.insert(node.output[0].clone(), output);
            }
            // https://github.com/onnx/onnx/blob/main/docs/Operators.md#Greater
            "Greater" => {
                let a = get(&node.input[0])?;
                let b = get(&node.input[1])?;

                let output = a.broadcast_gt(b)?;
                values.insert(node.output[0].clone(), output);
            }
            // https://github.com/onnx/onnx/blob/main/docs/Operators.md#Less
            "Less" => {
                let a = get(&node.input[0])?;
                let b = get(&node.input[1])?;

                let output = a.broadcast_lt(b)?;
                values.insert(node.output[0].clone(), output);
            }
            // https://github.com/onnx/onnx/blob/main/docs/Operators.md#LessOrEqual
            "LessOrEqual" => {
                let a = get(&node.input[0])?;
                let b = get(&node.input[1])?;

                let output = a.broadcast_le(b)?;
                values.insert(node.output[0].clone(), output);
            }
            // https://github.com/onnx/onnx/blob/main/docs/Operators.md#GreaterOrEqual
            "GreaterOrEqual" => {
                let a = get(&node.input[0])?;
                let b = get(&node.input[1])?;

                let output = a.broadcast_ge(b)?;
                values.insert(node.output[0].clone(), output);
            }
            // https://github.com/onnx/onnx/blob/main/docs/Operators.md#Log
            "Log" => {
                let a = get(&node.input[0])?;

                let output = a.log()?;
                values.insert(node.output[0].clone(), output);
            }
            // https://github.com/onnx/onnx/blob/main/docs/Operators.md#Min
            "Min" => {
                let mut output = get(&node.input[0])?.clone();
                for input in node.input.iter() {
                    let input = get(input)?;
                    output = output.broadcast_minimum(input)?
                }

                values.insert(node.output[0].clone(), output);
            }
            "Max" => {
                let mut output = get(&node.input[0])?.clone();
                for input in node.input.iter().skip(1) {
                    output = output.broadcast_maximum(get(input)?)?;
                }
                values.insert(node.output[0].clone(), output);
            }
            // https://github.com/onnx/onnx/blob/main/docs/Operators.md#Where
            "Where" => {
                let cond = get(&node.input[0])?;
                let a = get(&node.input[1])?;
                let b = get(&node.input[2])?;

                // where_cond requires that all inputs are the same shape.
                // In contrast, the Where op in ONNX only requires that they are broadcastable.
                let shape = broadcast_shape_from_many(&[cond.dims(), a.dims(), b.dims()])?;
                let cond = cond.broadcast_as(shape.clone())?;
                let a = a.broadcast_as(shape.clone())?;
                let b = b.broadcast_as(shape)?;
                let output = cond.where_cond(&a, &b)?;
                values.insert(node.output[0].clone(), output);
            }
            "Conv" => {
                // https://github.com/onnx/onnx/blob/main/docs/Operators.md#Conv
                let dilations = get_attr_opt::<[i64]>(node, "dilations")?;
                let groups = get_attr_opt::<i64>(node, "group")?.copied().unwrap_or(1);
                let _kernel_shape = get_attr_opt::<[i64]>(node, "kernel_shape")?;
                let pads = get_attr_opt::<[i64]>(node, "pads")?;
                let strides = get_attr_opt::<[i64]>(node, "strides")?;
                let auto_pad = get_attr_opt::<str>(node, "auto_pad")?;
                match auto_pad {
                    None | Some("NOTSET") => (),
                    Some(s) => bail!("unsupported auto_pad {s}"),
                };
                let xs = get(&node.input[0])?;
                let ws = get(&node.input[1])?;
                let mut bias_fused = false;
                let ys = match ws.rank() {
                    3 => {
                        let (pads, xs) = match pads {
                            None => (0, xs.clone()),
                            Some([p]) => (*p as usize, xs.clone()),
                            Some([p1, p2]) => {
                                if p1 != p2 {
                                    (0usize, xs.pad_with_zeros(2, *p1 as usize, *p2 as usize)?)
                                } else {
                                    (*p1 as usize, xs.clone())
                                }
                            }
                            Some(pads) => {
                                bail!("more pads than expected in conv1d {pads:?} {}", node.name)
                            }
                        };
                        let strides = match strides {
                            None => 1,
                            Some([p]) => *p as usize,
                            Some(s) => {
                                bail!("more strides than expected in conv1d {s:?} {}", node.name)
                            }
                        };
                        let dilations = match dilations {
                            None => 1,
                            Some([p]) => *p as usize,
                            Some(s) => {
                                bail!("more dilations than expected in conv1d {s:?} {}", node.name)
                            }
                        };
                        let is_depthwise_f32 = groups as usize == xs.dims()[1]
                            && ws.dims()[0] == xs.dims()[1]
                            && ws.dims()[1] == 1
                            && xs.dtype() == DType::F32
                            && ws.dtype() == DType::F32
                            && (node.input.len() <= 2
                                || get(&node.input[2])?.dtype() == DType::F32);
                        if is_depthwise_f32 {
                            let bias = match node.input.get(2).filter(|name| !name.is_empty()) {
                                Some(name) => get(name)?.clone(),
                                None => Tensor::zeros((ws.dims()[0],), DType::F32, xs.device())?,
                            };
                            bias_fused = true;
                            xs.apply_op3_no_bwd(
                                ws,
                                &bias,
                                &DepthwiseConv1dF32 { padding: pads, stride: strides, dilation: dilations },
                            )?
                        } else {
                            xs.conv1d(ws, pads, strides, dilations, groups as usize)?
                        }
                    }
                    4 => {
                        let (pads, xs) = match pads {
                            None => (0, xs.clone()),
                            Some([p]) => (*p as usize, xs.clone()),
                            Some(&[p1, p2, p3, p4]) => {
                                let p1 = p1 as usize;
                                let p2 = p2 as usize;
                                let p3 = p3 as usize;
                                let p4 = p4 as usize;
                                if p1 != p2 || p1 != p3 || p1 != p4 {
                                    (0, xs.pad_with_zeros(2, p1, p3)?.pad_with_zeros(3, p2, p4)?)
                                } else {
                                    (p1, xs.clone())
                                }
                            }
                            Some(pads) => {
                                bail!("more pads than expected in conv2d {pads:?} {}", node.name)
                            }
                        };
                        let strides = match strides {
                            None => 1,
                            Some([p]) => *p as usize,
                            Some([p1, p2]) => {
                                if p1 != p2 {
                                    bail!(
                                        "strides have to be the same on both axis {pads:?} {}",
                                        node.name
                                    )
                                }
                                *p1 as usize
                            }
                            Some(s) => {
                                bail!("more strides than expected in conv2d {s:?} {}", node.name)
                            }
                        };
                        let dilations = match dilations {
                            None => 1,
                            Some([p]) => *p as usize,
                            Some([p1, p2]) => {
                                if p1 != p2 {
                                    bail!(
                                        "dilations have to be the same on both axis {pads:?} {}",
                                        node.name
                                    )
                                }
                                *p1 as usize
                            }
                            Some(s) => {
                                bail!("more dilations than expected in conv2d {s:?} {}", node.name)
                            }
                        };
                        xs.conv2d(ws, pads, strides, dilations, groups as usize)?
                    }
                    rank => bail!(
                        "unsupported rank for weight matrix {rank} in conv {}",
                        node.name
                    ),
                };
                let ys = if node.input.len() > 2 && !bias_fused {
                    let bs = get(&node.input[2])?;
                    let mut bs_shape = vec![1; ys.rank()];
                    bs_shape[1] = bs.elem_count();
                    let bias = bs.reshape(bs_shape)?;
                    binary!(&ys, &bias, Add, broadcast_add)
                } else {
                    ys
                };
                values.insert(node.output[0].clone(), ys);
            }
            "ConvTranspose" => {
                let xs = get(&node.input[0])?;
                let ws = get(&node.input[1])?;
                let stride = get_attr_opt::<[i64]>(node, "strides")?;
                let pads = get_attr_opt::<[i64]>(node, "pads")?;
                let output_padding = get_attr_opt::<[i64]>(node, "output_padding")?;
                let dilations = get_attr_opt::<[i64]>(node, "dilations")?;
                let groups = get_attr_opt::<i64>(node, "group")?.copied().unwrap_or(1) as usize;
                let mut bias_fused = false;
                let ys = match ws.rank() {
                    3 => {
                        let stride = stride.map_or(1, |v| v[0] as usize);
                        let dilation = dilations.map_or(1, |v| v[0] as usize);
                        let opad = output_padding.map_or(0, |v| v[0] as usize);
                        let (left, right) = match pads {
                            None => (0usize, 0usize),
                            Some([left, right]) => (*left as usize, *right as usize),
                            Some(v) => bail!("ConvTranspose1d expects 2 pads, got {v:?}"),
                        };
                        #[cfg(feature = "cuda")]
                        let is_depthwise_f32 = groups == xs.dims()[1]
                            && ws.dims()[0] == xs.dims()[1]
                            && ws.dims()[1] == 1
                            && xs.dtype() == DType::F32
                            && ws.dtype() == DType::F32
                            && (node.input.len() <= 2
                                || node.input[2].is_empty()
                                || get(&node.input[2])?.dtype() == DType::F32);
                        #[cfg(feature = "cuda")]
                        let fused = if device.is_cuda() && is_depthwise_f32 && left == right {
                            let bias = match node.input.get(2).filter(|name| !name.is_empty()) {
                                Some(name) => get(name)?.clone(),
                                None => Tensor::zeros((ws.dims()[0],), DType::F32, xs.device())?,
                            };
                            Some(xs.apply_op3_no_bwd(
                                ws,
                                &bias,
                                &DepthwiseConvTranspose1dF32Cuda {
                                    padding: left,
                                    stride,
                                    dilation,
                                    output_padding: opad,
                                },
                            )?)
                        } else {
                            None
                        };
                        #[cfg(not(feature = "cuda"))]
                        let fused: Option<Tensor> = None;
                        if let Some(ys) = fused {
                            bias_fused = true;
                            ys
                        } else {
                            let mut ys = xs.conv_transpose1d(ws, left, opad, stride, dilation, groups)?;
                            if right > left {
                                ys = ys.narrow(2, 0, ys.dim(2)? - (right - left))?;
                            } else if left > right {
                                ys = ys.narrow(2, left - right, ys.dim(2)? - (left - right))?;
                            }
                            ys
                        }
                    }
                    4 => {
                        if groups != 1 { bail!("grouped 2D ConvTranspose is not supported in {}", node.name); }
                        let stride = stride.map_or([1, 1], |v| [v[0] as usize, v[1] as usize]);
                        let dilation = dilations.map_or([1, 1], |v| [v[0] as usize, v[1] as usize]);
                        let opad = output_padding.map_or([0, 0], |v| [v[0] as usize, v[1] as usize]);
                        let (top, left, bottom, right) = match pads {
                            None => (0usize, 0usize, 0usize, 0usize),
                            Some([top, left, bottom, right]) => (*top as usize, *left as usize, *bottom as usize, *right as usize),
                            Some(v) => bail!("ConvTranspose2d expects 4 pads, got {v:?}"),
                        };
                        if top != left || top != bottom || top != right || stride[0] != stride[1]
                            || dilation[0] != dilation[1] || opad[0] != opad[1] {
                            bail!("asymmetric 2D ConvTranspose is not supported in {}", node.name);
                        }
                        let mut ys = xs.conv_transpose2d(ws, top, opad[0], stride[0], dilation[0])?;
                        if bottom > top { ys = ys.narrow(2, 0, ys.dim(2)? - (bottom - top))?; }
                        if top > bottom { ys = ys.narrow(2, top - bottom, ys.dim(2)? - (top - bottom))?; }
                        if right > left { ys = ys.narrow(3, 0, ys.dim(3)? - (right - left))?; }
                        if left > right { ys = ys.narrow(3, left - right, ys.dim(3)? - (left - right))?; }
                        ys
                    }
                    rank => bail!("unsupported ConvTranspose weight rank {rank} in {}", node.name),
                };
                let ys = if node.input.len() > 2 && !node.input[2].is_empty() && !bias_fused {
                    let bias = get(&node.input[2])?;
                    let mut shape = vec![1; ys.rank()];
                    shape[1] = bias.elem_count();
                    ys.broadcast_add(&bias.reshape(shape)?)?
                } else { ys };
                values.insert(node.output[0].clone(), ys);
            }
            "Concat" => {
                // https://github.com/onnx/onnx/blob/main/docs/Operators.md#Concat
                let inputs = node
                    .input
                    .iter()
                    .map(|n| Ok(get(n.as_str())?.clone()))
                    .collect::<Result<Vec<Value>>>()?;
                let axis: i64 = *get_attr(node, "axis")?;
                if inputs.is_empty() {
                    bail!("empty concat")
                };
                // Find minimum rank among inputs and squeeze trailing singleton dims to match
                let min_rank = inputs.iter().map(|t| t.rank()).min().unwrap();
                let inputs: Vec<_> = inputs
                    .into_iter()
                    .map(|t| {
                        let mut t = t;
                        while t.rank() > min_rank {
                            let last_dim = t.rank() - 1;
                            if t.dims()[last_dim] == 1 {
                                t = t.squeeze(last_dim).unwrap_or(t);
                            } else {
                                break;
                            }
                        }
                        t
                    })
                    .collect();
                let axis = inputs[0].normalize_axis(axis)?;
                let output = Tensor::cat(&inputs, axis).map_err(|e| {
                    let shapes: Vec<_> = inputs.iter().map(|t| format!("{:?}", t.dims())).collect();
                    candle::Error::Msg(format!(
                        "Concat failed for node '{}': {} (input shapes: {:?})",
                        node.name, e, shapes
                    ))
                })?;
                if output.rank() == 1 && output.dtype() == DType::I64 {
                    if let Some(parts) = node.input.iter().map(|name| host_integer_values.get(name)).collect::<Option<Vec<_>>>() {
                        let host_output = parts.into_iter().flatten().copied().collect::<Vec<_>>();
                        if host_output.len() == output.elem_count() {
                            host_integer_values.insert(node.output[0].clone(), host_output);
                        }
                    }
                }
                values.insert(node.output[0].clone(), output);
            }
            "Abs" => {
                let input = get(&node.input[0])?;
                let output = input.abs()?;
                values.insert(node.output[0].clone(), output);
            }
            "Cos" => {
                let input = get(&node.input[0])?;
                let output = input.cos()?;
                values.insert(node.output[0].clone(), output);
            }
            "Sin" => {
                let input = get(&node.input[0])?;
                let output = input.sin()?;
                values.insert(node.output[0].clone(), output);
            }
            "Neg" => {
                let input = get(&node.input[0])?;
                // neg() not implemented for i64, work around with multiply by -1
                let output = if input.dtype() == DType::I64 {
                    let minus_one =
                        Tensor::new(&[-1i64], input.device())?.broadcast_as(input.shape())?;
                    input.mul(&minus_one)?
                } else {
                    input.neg()?
                };
                values.insert(node.output[0].clone(), output);
            }
            "Erf" => {
                let input = get(&node.input[0])?;
                let output = input.erf()?;
                values.insert(node.output[0].clone(), output);
            }
            "Tanh" => {
                let input = get(&node.input[0])?;
                let output = input.tanh()?;
                values.insert(node.output[0].clone(), output);
            }
            "Sigmoid" => {
                let input = get(&node.input[0])?;
                let output = candle_nn::ops::sigmoid(input)?;
                values.insert(node.output[0].clone(), output);
            }
            "Gelu" => {
                let input = get(&node.input[0])?;
                let output = input.gelu_erf()?;
                values.insert(node.output[0].clone(), output);
            }
            "Relu" => {
                let input = get(&node.input[0])?;
                let output = input.relu()?;
                values.insert(node.output[0].clone(), output);
            }
            "PRelu" => {
                // https://onnx.ai/onnx/operators/onnx__PRelu.html
                let input = get(&node.input[0])?;
                let slope = get(&node.input[1])?;

                let output = PReLU::new(slope.clone(), false).forward(input)?;
                values.insert(node.output[0].clone(), output);
            }
            "Ceil" => {
                let input = get(&node.input[0])?;
                let output = input.ceil()?;
                values.insert(node.output[0].clone(), output);
            }
            "Floor" => {
                let input = get(&node.input[0])?;
                let output = input.floor()?;
                values.insert(node.output[0].clone(), output);
            }
            "Round" => {
                let input = get(&node.input[0])?;
                values.insert(node.output[0].clone(), input.round_to(0)?);
            }
            "Mod" => {
                let a = get(&node.input[0])?;
                let b = get(&node.input[1])?;
                let fmod = get_attr_opt::<i64>(node, "fmod")?.copied().unwrap_or(0) != 0;
                if fmod { bail!("Mod fmod=1 is not implemented in {}", node.name); }
                #[cfg(feature = "cuda")]
                if device.is_cuda() && a.dtype() == DType::I64 && b.dtype() == DType::I64 {
                    let output_shape = a.shape().broadcast_shape_binary_op(b.shape(), "Mod")?;
                    let output = a.apply_op2_no_bwd(
                        b,
                        &ModI64Cuda {
                            lhs_count: a.elem_count(),
                            rhs_count: b.elem_count(),
                            output_shape,
                        },
                    )?;
                    values.insert(node.output[0].clone(), output);
                    continue;
                }
                let q = a.broadcast_div(b)?;
                let q = q.floor()?;
                let output = a.broadcast_sub(&q.broadcast_mul(b)?)?;
                values.insert(node.output[0].clone(), output);
            }
            "DequantizeLinear" => {
                let axis = get_attr_opt::<i64>(node, "axis")?.copied().unwrap_or(1);
                let quantized_raw = get(&node.input[0])?;
                let scale_raw = get(&node.input[1])?;
                let zero_raw = if node.input.len() > 2 && !node.input[2].is_empty() {
                    Some(get(&node.input[2])?)
                } else {
                    None
                };
                let output = dequantize_linear(quantized_raw, scale_raw, zero_raw, axis)?;
                values.insert(node.output[0].clone(), output);
            }
            "LayerNormalization" => {
                let input = get(&node.input[0])?;
                let axis = get_attr_opt::<i64>(node, "axis")?.copied().unwrap_or(-1);
                let axis = input.normalize_axis(axis)?;
                #[cfg(feature = "cuda")]
                if enable_fused_layernorm
                    && input.dtype() == DType::F32
                    && input.rank() == 3
                    && axis == 2
                    && (1..=1024).contains(&input.dims()[2])
                    && input.elem_count() != 0
                    && node.output.get(1).is_none_or(String::is_empty)
                    && node.output.get(2).is_none_or(String::is_empty)
                    && node.input.get(1).is_some_and(|name| !name.is_empty())
                    && node.input.get(2).is_some_and(|name| !name.is_empty())
                {
                    let scale = get(&node.input[1])?;
                    let bias = get(&node.input[2])?;
                    let is_last_axis_vector = |tensor: &Tensor| {
                        tensor.dtype() == DType::F32
                            && tensor.rank() > 0
                            && tensor.dims().last() == input.dims().last()
                            && tensor.dims()[..tensor.rank() - 1].iter().all(|&dim| dim == 1)
                    };
                    if is_last_axis_vector(scale) && is_last_axis_vector(bias) {
                        let epsilon = get_attr_opt::<f32>(node, "epsilon")?
                            .copied()
                            .unwrap_or(1e-5);
                        let output = input.apply_op3_no_bwd(
                            scale,
                            bias,
                            &LayerNormLastF32Cuda { epsilon, per_channel_affine: false },
                        )?;
                        if std::env::var_os("KALE_TTS_VALIDATE_FUSED_LAYERNORM").is_some() {
                            let axes = [axis];
                            let mean = input.mean_keepdim(axes.as_slice())?;
                            let centered = input.broadcast_sub(&mean)?;
                            let variance = centered.sqr()?.mean_keepdim(axes.as_slice())?;
                            let inv_std = (variance + epsilon as f64)?.sqrt()?.recip()?;
                            let reference = centered
                                .broadcast_mul(&inv_std)?
                                .broadcast_mul(scale)?
                                .broadcast_add(bias)?;
                            let difference = output
                                .broadcast_sub(&reference)?
                                .abs()?;
                            let max_abs = difference.max_all()?.to_scalar::<f32>()?;
                            let mean_abs = difference.mean_all()?.to_scalar::<f32>()?;
                            eprintln!(
                                "fused LayerNormalization validation: node={} shape={:?} max_abs={max_abs} mean_abs={mean_abs}",
                                node.name,
                                input.dims()
                            );
                            // Advance using the generic output so each subsequent node's
                            // comparison is against the same reference-path input.
                            values.insert(node.output[0].clone(), reference);
                        } else {
                            values.insert(node.output[0].clone(), output);
                        }
                        continue;
                    }
                }
                let axes = (axis..input.rank()).collect::<Vec<_>>();
                let mean = input.mean_keepdim(axes.as_slice())?;
                let centered = input.broadcast_sub(&mean)?;
                let variance = centered.sqr()?.mean_keepdim(axes.as_slice())?;
                let epsilon = get_attr_opt::<f32>(node, "epsilon")?.copied().unwrap_or(1e-5);
                let inv_std = (variance + epsilon as f64)?.sqrt()?.recip()?;
                let mut output = centered.broadcast_mul(&inv_std)?;
                if node.input.len() > 1 && !node.input[1].is_empty() {
                    output = output.broadcast_mul(get(&node.input[1])?)?;
                }
                if node.input.len() > 2 && !node.input[2].is_empty() {
                    output = output.broadcast_add(get(&node.input[2])?)?;
                }
                values.insert(node.output[0].clone(), output);
                if node.output.len() > 1 && !node.output[1].is_empty() {
                    values.insert(node.output[1].clone(), mean);
                }
                if node.output.len() > 2 && !node.output[2].is_empty() {
                    values.insert(node.output[2].clone(), inv_std);
                }
            }
            "InstanceNormalization" => {
                let input = get(&node.input[0])?;
                if input.rank() < 3 { bail!("InstanceNormalization expects rank >= 3 in {}", node.name); }
                #[cfg(feature = "cuda")]
                if enable_fused_instance_norm && input.dtype() == DType::F32
                    && input.rank() == 3 && (1..=1024).contains(&input.dims()[2])
                    && input.elem_count() != 0
                {
                    let scale = get(&node.input[1])?;
                    let bias = get(&node.input[2])?;
                    let channels = input.dims()[1];
                    if scale.dtype() == DType::F32 && bias.dtype() == DType::F32
                        && scale.dims() == [channels] && bias.dims() == [channels]
                    {
                        let epsilon = get_attr_opt::<f32>(node, "epsilon")?.copied().unwrap_or(1e-5);
                        let output = input.apply_op3_no_bwd(
                            scale, bias, &LayerNormLastF32Cuda { epsilon, per_channel_affine: true },
                        )?;
                        values.insert(node.output[0].clone(), output);
                        continue;
                    }
                }
                let axes = (2..input.rank()).collect::<Vec<_>>();
                let mean = input.mean_keepdim(axes.as_slice())?;
                let centered = input.broadcast_sub(&mean)?;
                let variance = centered.sqr()?.mean_keepdim(axes.as_slice())?;
                let epsilon = get_attr_opt::<f32>(node, "epsilon")?.copied().unwrap_or(1e-5);
                let inv_std = (variance + epsilon as f64)?.sqrt()?.recip()?;
                let mut shape = vec![1usize; input.rank()];
                shape[1] = input.dim(1)?;
                let scale = get(&node.input[1])?.reshape(shape.as_slice())?;
                let bias = get(&node.input[2])?.reshape(shape.as_slice())?;
                let output = centered.broadcast_mul(&inv_std)?.broadcast_mul(&scale)?.broadcast_add(&bias)?;
                values.insert(node.output[0].clone(), output);
            }
            "RandomNormalLike" => {
                let input = get(&node.input[0])?;
                let mean = get_attr_opt::<f32>(node, "mean")?.copied().unwrap_or(0.0) as f64;
                let scale = get_attr_opt::<f32>(node, "scale")?.copied().unwrap_or(1.0) as f64;
                let output = input.randn_like(mean, scale)?;
                values.insert(node.output[0].clone(), output);
            }
            // https://github.com/onnx/onnx/blob/main/docs/Operators.md#Constant
            "Constant" => {
                let value = match node.attribute.iter().find(|attr| attr.name == "value") {
                    None => {
                        // TODO: support sparse_value etc.
                        bail!("cannot find 'value' attr in 'Constant' for {}", node.name)
                    }
                    Some(value) => value,
                };
                let output = match value.r#type() {
                    AttributeType::Tensor => {
                        let t = value.t.as_ref().unwrap();
                        get_tensor(t, &node.name, device)?
                    }
                    rtype => bail!("unsupported 'value' type {rtype:?} for {}", node.name),
                };

                values.insert(node.output[0].clone(), output);
            }
            // https://github.com/onnx/onnx/blob/main/docs/Operators.md#Cast
            "Cast" => {
                let input = get(&node.input[0])?;
                let dt: i64 = *get_attr(node, "to")?;
                let dtype = match DataType::try_from(dt as i32) {
                    Ok(DataType::Int32) => DType::I64,
                    Ok(dt) => match dtype(dt) {
                        Some(dt) => dt,
                        None => {
                            bail!("unsupported 'to' value {dt:?} for cast {}", node.name)
                        }
                    },
                    Err(_) => {
                        bail!("unsupported 'to' value {dt:?} for cast {}", node.name)
                    }
                };
                let output = input.to_dtype(dtype)?;
                if dtype == DType::I64 && input.dtype() == DType::I64 {
                    if let Some(data) = host_integer_values.get(&node.input[0]).cloned() {
                        host_integer_values.insert(node.output[0].clone(), data);
                    }
                }
                values.insert(node.output[0].clone(), output);
            }
            // https://github.com/onnx/onnx/blob/main/docs/Operators.md#CumSum
            "CumSum" => {
                let exclusive = get_attr_opt::<i64>(node, "exclusive")?
                    .copied()
                    .unwrap_or(0);
                let reverse = get_attr_opt::<i64>(node, "reverse")?.copied().unwrap_or(0);
                if exclusive != 0 {
                    bail!("only exclusive == 0 is supported in CumSum")
                }
                if reverse != 0 {
                    bail!("only reverse == 0 is supported in CumSum")
                }
                let input = get(&node.input[0])?;
                let axis = match host_integer_values.get(&node.input[1]).map(Vec::as_slice) {
                    Some([axis]) => u32::try_from(*axis).map_err(candle::Error::wrap)?,
                    _ => to_vec0_flexible::<u32>(&get(&node.input[1])?.to_dtype(DType::U32)?)?,
                };
                let output = match input.dtype() {
                    DType::I64 | DType::I32 | DType::I16 | DType::U32 | DType::U8 => input
                        .to_dtype(DType::F32)?.cumsum(axis as usize)?.to_dtype(input.dtype())?,
                    _ => input.cumsum(axis as usize)?,
                };
                values.insert(node.output[0].clone(), output);
            }
            //  https://github.com/onnx/onnx/blob/main/docs/Operators.md#flatten
            "Flatten" => {
                let axis = get_attr_opt::<i64>(node, "axis")?.copied().unwrap_or(1) as usize;
                let input = get(&node.input[0])?;
                let first_part: usize = input.shape().dims().iter().take(axis).product();
                let end_index = input.shape().dims().iter().product::<usize>();
                let new_shape = (first_part, end_index / first_part);
                let output = input.reshape(new_shape)?;
                values.insert(node.output[0].clone(), output);
            }
            // https://github.com/onnx/onnx/blob/main/docs/Operators.md#identity
            "Identity" => {
                let input = get(&node.input[0])?;
                values.insert(node.output[0].clone(), input.clone());
            }
            // https://github.com/onnx/onnx/blob/main/docs/Operators.md#if
            "If" => {
                // protobuf encodes boolean false as 0 and true as 1
                let cond = to_scalar_flexible::<u8>(&get(&node.input[0])?.get(0)?)?;
                let attr_name = if cond != 0 {
                    "then_branch"
                } else {
                    "else_branch"
                };
                let sub_graph = get_attr::<GraphProto>(node, attr_name)?;
                if sub_graph.output.len() != node.output.len() {
                    bail!(
                        "If node {:?} is malformed: branch outputs ({}) don't match node outputs ({})",
                        node.name,
                        sub_graph.output.len(),
                        node.output.len()
                    );
                }
                let mut branch_integer_values = collect_host_integer_constants(sub_graph);
                let branch_out = simple_eval_(sub_graph, values, device, &mut branch_integer_values, None, _nano_cuda)?;
                for (i, out) in node.output.iter().enumerate() {
                    values.insert(
                        out.clone(),
                        branch_out.get(&sub_graph.output[i].name).unwrap().clone(),
                    );
                }
            }
            // https://github.com/onnx/onnx/blob/main/docs/Operators.md#pad
            "Pad" => {
                let mode = get_attr_opt(node, "mode")?.unwrap_or("constant");
                let data = get(&node.input[0])?;
                let pads = get(&node.input[1])?;
                if node.input.len() > 2 {
                    bail!(
                        "unsupported number of inputs {} for Pad node {:?}, expected 2",
                        node.input.len(),
                        node.name
                    );
                }
                if pads.rank() != 1 {
                    bail!("Pad expects 'pads' input to be 1D vector: {pads:?}");
                }
                if pads.dim(0).unwrap() != 2 * data.rank() {
                    bail!("Pad expects 'pads' input len to be 2 * rank of 'data' input: pads: {}, data rank: {}", pads, data.rank());
                }

                let pads = read_ints(&node.input[1])?;
                let (pads_pre, pads_post) = pads.split_at(pads.len() / 2);

                match mode {
                    "reflect" => {
                        #[cfg(feature = "cuda")]
                        if let ([1, 1, length], [0, 0, left], [0, 0, right]) = (data.dims(), pads_pre, pads_post) {
                            if _nano_cuda && data.dtype() == DType::F32 && *left >= 0 && *right >= 0
                                && (*left as usize) < *length && (*right as usize) < *length
                            {
                                let out = data.apply_op1_no_bwd(&super::nano_cuda::ReflectPad {
                                    left: *left as usize, right: *right as usize,
                                })?;
                                values.insert(node.output[0].clone(), out);
                                continue;
                            }
                        }
                        let mut out = data.clone();
                        for (i, &dim) in data.dims().iter().enumerate().rev() {
                            if pads_pre[i] == 0 && pads_post[i] == 0 {
                                continue;
                            }
                            fn zigzag(min: i64, max: i64) -> impl Iterator<Item = i64> {
                                std::iter::repeat((min..max).chain((min + 1..=max).rev())).flatten()
                            }
                            let idx = if dim > 1 {
                                let cycle_len = dim * 2 - 2;
                                let skip = cycle_len - ((pads_pre[i] as usize) % cycle_len);
                                let idx = zigzag(0, (dim - 1) as i64)
                                    .skip(skip)
                                    .take((pads_pre[i] as usize) + dim + (pads_post[i] as usize));
                                Tensor::from_iter(idx, out.device())?
                            } else {
                                Tensor::full(0i64, (dim,), out.device())?
                            };

                            out = out.index_select(&idx, i)?;
                        }

                        values.insert(node.output[0].clone(), out);
                    }
                    _ => bail!(
                        "unsupported 'mode' value {mode:?} for Pad node {:?}",
                        node.name
                    ),
                }
            }
            // https://github.com/onnx/onnx/blob/main/docs/Operators.md#slice
            "Slice" => {
                let data = get(&node.input[0])?;
                let vector = |name: &str| -> Result<Vec<i64>> {
                    match host_integer_values.get(name) {
                        Some(values) => Ok(values.clone()),
                        None => get(name)?.to_vec1::<i64>(),
                    }
                };
                let starts = vector(&node.input[1])?;
                let ends = vector(&node.input[2])?;
                let axes = match node.input.len() {
                    3 => (0..starts.len()).map(|axis| axis as i64).collect(),
                    4 | 5 => vector(&node.input[3])?,
                    _ => bail!(
                        "Slice node is invalid, expected 3-5 inputs, got {}: {:?}",
                        node.input.len(),
                        node
                    ),
                };
                let steps = if node.input.len() == 5 {
                    vector(&node.input[4])?
                } else {
                    vec![1; starts.len()]
                };
                if starts.len() != ends.len() || starts.len() != axes.len() || starts.len() != steps.len() {
                    bail!("Slice parameter lengths do not match in {}", node.name);
                }

                let mut out = data.clone();
                let mut host_data = if data.rank() == 1 {
                    host_integer_values.get(&node.input[0]).cloned()
                } else {
                    None
                };
                for (i, axis) in axes.into_iter().enumerate() {
                    // All negative elements of axes are made non-negative by
                    // adding r to them, where r = rank(input).
                    let axis = if axis < 0 {
                        axis + data.rank() as i64
                    } else {
                        axis
                    } as usize;
                    if axis >= data.rank() {
                        bail!("Slice axis {axis} is out of range for rank {}", data.rank());
                    }

                    let data_dim = data.dims()[axis] as i64;
                    let mut s = starts[i];
                    let mut e = ends[i];
                    // All negative values in starts[i] and ends[i] have
                    // dims[axes[i]] added to them, where dims are the
                    // dimensions of input.
                    if s < 0 {
                        s += data_dim;
                    }
                    if e < 0 {
                        e += data_dim;
                    }

                    let p = steps[i];
                    if p == 0 {
                        bail!("Slice step cannot be zero in {}", node.name);
                    }
                    // starts[i] is clamped into the range [0, dims[axes[i]]]
                    // for positive stepping and [0, dims[axes[i]]-1] for
                    // negative stepping.
                    // for positive stepping ends[axes[i]] is clamped to
                    // [0, dims[axes[i]]], while for negative stepping it is
                    // clamped to [-1, dims[axes[i]]-1].
                    if p >= 0 {
                        s = s.clamp(0, data_dim);
                        e = e.clamp(0, data_dim);
                    } else {
                        s = s.clamp(0, data_dim - 1);
                        e = e.clamp(-1, data_dim - 1);
                    }

                    if p == 1 {
                        let count = (e - s).max(0) as usize;
                        let start = s as usize;
                        out = out.narrow(axis, start, count)?;
                        host_data = host_data.and_then(|data| data.get(start..start + count).map(<[i64]>::to_vec));
                    } else {
                        host_data = None;
                        #[cfg(feature = "cuda")]
                        let indexes = if _nano_cuda && data.device().is_cuda() {
                            super::nano_cuda::slice_indexes(data, s, e, p)?
                        } else { Tensor::arange_step(s, e, p, data.device())? };
                        #[cfg(not(feature = "cuda"))]
                        let indexes = Tensor::arange_step(s, e, p, data.device())?;
                        out = out.contiguous()?.index_select(&indexes, axis)?;
                    }
                }
                if let Some(host_data) = host_data {
                    host_integer_values.insert(node.output[0].clone(), host_data);
                }
                values.insert(node.output[0].clone(), out);
            }
            // https://onnx.ai/onnx/operators/onnx__ReduceMax.html#reducemax
            "ReduceMax" => {
                let input = get(&node.input[0])?;
                let axes = get_opt(1);
                let keepdims = get_attr_opt::<i64>(node, "keepdims")?.copied().unwrap_or(1) == 1;

                let axes = if let Some(Ok(axes)) = axes {
                    // Satisfies version 18+
                    axes.to_vec1::<i64>().ok()
                } else if let Ok(Some(axes)) = get_attr_opt::<[i64]>(node, "axes") {
                    // Backward compatibility with version 13 and below
                    Some(axes.to_vec())
                } else {
                    None
                };

                let axes = if let Some(axes) = axes {
                    let rank = input.rank();
                    let mut axes_set = HashSet::new();

                    let mut axes = axes
                        .iter()
                        .map(|a| {
                            let axis = if *a < 0 {
                                (rank as i64 + *a) as usize
                            } else {
                                *a as usize
                            };

                            axes_set.insert(axis);
                            axis
                        })
                        .collect::<Vec<_>>();

                    if axes_set.len() < axes.len() {
                        bail!("Duplicate value in 'axes'");
                    }

                    if axes.len() > 1 {
                        axes.sort();
                    }

                    Some(axes)
                } else {
                    None
                };

                // TODO: Handle empty set
                // Definition:
                // "Reduction over an empty set of values yields minus infinity (if supported by the datatype) or the minimum value of the data type otherwise"
                // For now, this will throw an error
                if input.elem_count() == 0 {
                    bail!("reduction over zero-size tensor not supported");
                }

                let output = if let Some(axes) = axes {
                    let mut result = input.clone();
                    for &axis in axes.iter().rev() {
                        result = if keepdims {
                            result.max_keepdim(axis)?
                        } else {
                            result.max(axis)?
                        }
                    }

                    result
                } else {
                    // If `axes` is empty and `noop_with_empty_axes` is set to `true (1)`
                    // ""input tensor will not be reduced,and the output tensor would be equivalent to input tensor.""
                    if get_attr_opt::<i64>(node, "noop_with_empty_axes")?.copied() == Some(1) {
                        input.clone()
                    } else {
                        let mut result = input.flatten_all()?;
                        if keepdims {
                            result = result.max_keepdim(0)?;
                            // If keepdims is true, reshape to match input dimensions
                            let shape = vec![1; input.rank()];
                            result.reshape(shape)?
                        } else {
                            result.max(0)?
                        }
                    }
                };

                values.insert(node.output[0].clone(), output);
            }
            // https://onnx.ai/onnx/operators/onnx__ReduceMean.html#reducemean-13
            // TODO: This version is only compatible with ReduceMean V13 and below.
            "ReduceMean" => {
                let input = get(&node.input[0])?;
                let axes = get_attr_opt::<[i64]>(node, "axes")?;
                let keepdims = get_attr_opt::<i64>(node, "keepdims")?.copied().unwrap_or(1);

                let n_dims = input.dims().len();

                let axes: Vec<usize> = if let Some(axes) = axes {
                    axes.iter()
                        .map(|e| (if e < &0 { (n_dims as i64) + *e } else { *e }) as usize)
                        .collect()
                } else {
                    (0..n_dims).collect()
                };
                let output = if keepdims == 1 {
                    input.mean_keepdim(axes)?
                } else {
                    input.mean(axes)?
                };
                values.insert(node.output[0].clone(), output);
            }
            // https://onnx.ai/onnx/operators/onnx__ReduceMin.html#reducemin
            "ReduceMin" => {
                let input = get(&node.input[0])?;
                let axes = get_opt(1);
                let keepdims = get_attr_opt::<i64>(node, "keepdims")?.copied().unwrap_or(1) == 1;

                let axes = if let Some(Ok(axes)) = axes {
                    // Satisfies version 18+
                    axes.to_vec1::<i64>().ok()
                } else if let Ok(Some(axes)) = get_attr_opt::<[i64]>(node, "axes") {
                    // Backward compatibility with version 13 and below
                    Some(axes.to_vec())
                } else {
                    None
                };

                let axes = if let Some(axes) = axes {
                    let rank = input.rank();
                    let mut axes_set = HashSet::new();

                    let mut axes = axes
                        .iter()
                        .map(|a| {
                            let axis = if *a < 0 {
                                (rank as i64 + *a) as usize
                            } else {
                                *a as usize
                            };

                            axes_set.insert(axis);
                            axis
                        })
                        .collect::<Vec<_>>();

                    if axes_set.len() < axes.len() {
                        bail!("Duplicate value in 'axes'");
                    }

                    if axes.len() > 1 {
                        axes.sort();
                    }

                    Some(axes)
                } else {
                    None
                };

                // TODO: Handle empty set
                // Definition:
                // "Reduction over an empty set of values yields positive infinity (if supported by the datatype) or the max value of the data type otherwise"
                // For now, this will throw an error
                if input.elem_count() == 0 {
                    bail!("reduction over zero-size tensor not supported");
                }

                let output = if let Some(axes) = axes {
                    let mut result = input.clone();
                    for &axis in axes.iter().rev() {
                        result = if keepdims {
                            result.min_keepdim(axis)?
                        } else {
                            result.min(axis)?
                        }
                    }

                    result
                } else {
                    // If `axes` is empty and `noop_with_empty_axes` is set to `true (1)`
                    // ""input tensor will not be reduced,and the output tensor would be equivalent to input tensor.""
                    if get_attr_opt::<i64>(node, "noop_with_empty_axes")?.copied() == Some(1) {
                        input.clone()
                    } else {
                        let mut result = input.flatten_all()?;
                        if keepdims {
                            result = result.min_keepdim(0)?;
                            // If keepdims is true, reshape to match input dimensions
                            let shape = vec![1; input.rank()];
                            result.reshape(shape)?
                        } else {
                            result.min(0)?
                        }
                    }
                };

                values.insert(node.output[0].clone(), output);
            }
            //https://github.com/onnx/onnx/blob/main/docs/Operators.md#Split
            // Version 18 impl
            "Split" => {
                let input_tensor = get(&node.input[0])?;
                let axis = get_attr_opt::<i64>(node, "axis")?.copied().unwrap_or(0);
                let axis = input_tensor.normalize_axis(axis)?;

                // Determine split sizes
                let splits = if node.input.len() > 1 {
                    // If the split tensor is provided, use it to determine sizes
                    let split_tensor = read_ints(&node.input[1])?;
                    split_tensor.iter().map(|&x| x as usize).collect::<Vec<_>>()
                } else {
                    let num_outputs = if let Some(&num_outputs_attrib) =
                        get_attr_opt::<i64>(node, "num_outputs")?
                    {
                        num_outputs_attrib as usize
                    } else {
                        node.output.len()
                    };

                    let input_dim = input_tensor.dim(axis)?;

                    let mut split_sizes =
                        vec![input_dim / num_outputs as usize; num_outputs as usize];
                    let remainder = input_dim % num_outputs as usize;
                    if remainder > 0 {
                        // If there's a remainder, add it to the last split size
                        split_sizes[num_outputs as usize - 1] += remainder;
                    }

                    split_sizes
                };

                // Perform the split operation
                let mut outputs = vec![];
                let mut start = 0;
                for &size in &splits {
                    let end = start + size;
                    let slice = input_tensor.narrow(axis, start, size)?;
                    outputs.push(slice);
                    start = end;
                }

                // Insert the split outputs into the values map
                for (output, slice) in node.output.iter().zip(outputs.into_iter()) {
                    values.insert(output.clone(), slice);
                }
            }
            //https://github.com/onnx/onnx/blob/main/docs/Operators.md#Expand
            // Version 13 impl
            "Expand" => {
                // unlike broadcast_to, expand allows for the output shape to
                // be different from the specified shape.
                let input_tensor = get(&node.input[0])?;
                let input_shape = get(&node.input[1])?;

                // Check that the shape tensor is 1D
                if input_shape.rank() != 1 {
                    bail!(
                        "Expand expects 'shape' input to be 1D tensor: {:?}",
                        input_shape
                    );
                }
                let input_tensor_dims = input_tensor.dims();
                let input_shape_values = match host_integer_values.get(&node.input[1]) {
                    Some(shape) => shape.clone(),
                    None => input_shape.to_vec1::<i64>()?,
                };
                let input_shape_dims = input_shape_values
                    .into_iter()
                    .map(|x| x as usize)
                    .collect::<Vec<_>>();

                let target_shape = broadcast_shape(input_tensor_dims, input_shape_dims.as_slice())?;

                let expanded_tensor = input_tensor.broadcast_as(target_shape)?;

                values.insert(node.output[0].clone(), expanded_tensor);
            }
            // https://github.com/onnx/onnx/blob/main/docs/Operators.md#Tile
            "Tile" => {
                let input = get(&node.input[0])?;
                let repeats = match host_integer_values.get(&node.input[1]) {
                    Some(repeats) => repeats.clone(),
                    None => get(&node.input[1])?.to_vec1::<i64>()?,
                };

                let mut result = input.clone();
                for (dim, &repeat) in repeats.iter().enumerate() {
                    if repeat > 1 {
                        let repeat = repeat as usize;
                        let tensors: Vec<_> = (0..repeat).map(|_| result.clone()).collect();
                        result = Tensor::cat(&tensors, dim)?;
                    }
                }
                values.insert(node.output[0].clone(), result);
            }
            //https://github.com/onnx/onnx/blob/main/docs/Operators.md#ReduceSum
            // Version 13 impl
            "ReduceSum" => {
                let input = get(&node.input[0])?;
                let axes = get_opt(1);
                let keepdims = get_attr_opt::<i64>(node, "keepdims")?.copied().unwrap_or(1);
                let noop_with_empty_axes = get_attr_opt::<i64>(node, "noop_with_empty_axes")?
                    .copied()
                    .unwrap_or(0);

                let axes = match axes {
                    Some(Ok(_)) => read_ints(&node.input[1])?
                        .into_iter()
                        .map(|x| input.normalize_axis(x))
                        .collect::<Result<Vec<_>>>()?,
                    Some(Err(_)) | None => {
                        if noop_with_empty_axes == 1 {
                            vec![]
                        } else {
                            (0..input.rank()).collect()
                        }
                    }
                };

                let output = if keepdims == 1 {
                    input.sum_keepdim(axes)?
                } else {
                    input.sum(axes)?
                };

                values.insert(node.output[0].clone(), output);
            }
            // https://github.com/onnx/onnx/blob/main/docs/Operators.md#ReduceL2
            // Version 18 impl
            "ReduceL2" => {
                let input = get(&node.input[0])?;
                let axes = get_opt(1);
                let keepdims = get_attr_opt::<i64>(node, "keepdims")?.copied().unwrap_or(1);
                let noop_with_empty_axes = get_attr_opt::<i64>(node, "noop_with_empty_axes")?
                    .copied()
                    .unwrap_or(0);

                let input_sq = input.sqr()?;

                let axes = match axes {
                    Some(axes) => axes?
                        .to_vec1::<i64>()?
                        .into_iter()
                        .map(|x| input_sq.normalize_axis(x))
                        .collect::<Result<Vec<_>>>()?,
                    None => {
                        if noop_with_empty_axes == 1 {
                            vec![]
                        } else {
                            (0..input_sq.rank()).collect()
                        }
                    }
                };

                let output = if keepdims == 1 {
                    input_sq.sum_keepdim(axes)?.sqrt()?
                } else {
                    input_sq.sum(axes)?.sqrt()?
                };

                values.insert(node.output[0].clone(), output);
            }
            random_type @ ("RandomUniform" | "RandomNormal") => {
                let dt: i64 = get_attr_opt(node, "dtype")?.copied().unwrap_or(1); // 1 is float
                                                                                  // type by
                                                                                  // default
                let dtype = match DataType::try_from(dt as i32) {
                    Ok(dt) => match dtype(dt) {
                        Some(DType::U8 | DType::U32 | DType::I64) => {
                            bail!(
                                "unsupported 'dtype' value {dt:?}, only floats are allowed, for {random_type} {}",
                                node.name
                            )
                        }
                        Some(dt) => dt,
                        None => {
                            bail!(
                                "unsupported 'dtype' value {dt:?} for {random_type} {}",
                                node.name
                            )
                        }
                    },
                    Err(_) => {
                        bail!(
                            "unsupported 'dtype' value {dt:?} for {random_type} {}",
                            node.name
                        )
                    }
                };
                let seed: Option<f32> = get_attr_opt(node, "seed")?.copied();
                if seed.is_some() {
                    bail!("seed for {random_type} is currently not supported")
                };
                let shape: Vec<usize> = get_attr::<[i64]>(node, "shape")?
                    .iter()
                    .map(|x| *x as usize)
                    .collect();
                let output = if random_type == "RandomUniform" {
                    let low: f32 = get_attr_opt(node, "low")?.copied().unwrap_or(0.0);
                    let high: f32 = get_attr_opt(node, "high")?.copied().unwrap_or(1.0);
                    Tensor::rand(low, high, shape, device)?.to_dtype(dtype)?
                } else {
                    let mean: f32 = get_attr_opt(node, "mean")?.copied().unwrap_or(0.0);
                    let scale: f32 = get_attr_opt(node, "scale")?.copied().unwrap_or(1.0);
                    Tensor::randn(mean, scale, shape, device)?.to_dtype(dtype)?
                };
                values.insert(node.output[0].clone(), output);
            }
            "ArgMin" => {
                let input = get(&node.input[0])?;
                let axis_i64: i64 = get_attr_opt(node, "axis")?.copied().unwrap_or(0);
                let rank_i64: i64 = input.rank().try_into().unwrap();
                if axis_i64 < -rank_i64 || axis_i64 >= rank_i64 {
                    bail!(
                        "axis ({}) out of accepted range [-rank, rank-1] which was [{}, {}]",
                        axis_i64,
                        -rank_i64,
                        rank_i64 - 1
                    )
                }
                let axis = input.normalize_axis(axis_i64)?;
                let keepdims: i64 = get_attr_opt(node, "keepdims")?.copied().unwrap_or(1);
                let select_last_index: i64 = get_attr_opt(node, "select_last_index")?
                    .copied()
                    .unwrap_or(0);
                if select_last_index == 1 {
                    bail!("select_last_index for ArgMin is currently not supported")
                }
                let output = if keepdims == 1 {
                    input.argmin_keepdim(axis)?
                } else {
                    input.argmin(axis)?
                }
                .to_dtype(DType::I64)?;
                values.insert(node.output[0].clone(), output);
            }
            "ArgMax" => {
                let input = get(&node.input[0])?;
                let axis_i64: i64 = get_attr_opt(node, "axis")?.copied().unwrap_or(0);
                let rank_i64: i64 = input.rank().try_into().unwrap();
                if axis_i64 < -rank_i64 || axis_i64 >= rank_i64 {
                    bail!(
                        "axis ({}) out of accepted range [-rank, rank-1] which was [{}, {}]",
                        axis_i64,
                        -rank_i64,
                        rank_i64 - 1
                    )
                }
                let axis = input.normalize_axis(axis_i64)?;
                let keepdims: i64 = get_attr_opt(node, "keepdims")?.copied().unwrap_or(1);
                let select_last_index: i64 = get_attr_opt(node, "select_last_index")?
                    .copied()
                    .unwrap_or(0);
                if select_last_index == 1 {
                    bail!("select_last_index for ArgMin is currently not supported")
                }
                let output = if keepdims == 1 {
                    input.argmax_keepdim(axis)?
                } else {
                    input.argmax(axis)?
                }
                .to_dtype(DType::I64)?;
                values.insert(node.output[0].clone(), output);
            }
            "LeakyRelu" => {
                let input = get(&node.input[0])?;
                let dt = input.dtype();
                match dt {
                    DType::U8
                    | DType::U32
                    | DType::I64
                    | DType::I32
                    | DType::I16
                    | DType::F6E2M3
                    | DType::F6E3M2
                    | DType::F4
                    | DType::F8E8M0 => {
                        bail!(
                            "unsupported dtype {}, only float types are allowed for LeakyRelu",
                            dt.as_str()
                        )
                    }
                    DType::BF16 | DType::F16 | DType::F32 | DType::F64 | DType::F8E4M3 => {}
                    dt => bail!("unsupported dtype {dt:?} for LeakyRelu"),
                }
                let alpha = get_attr_opt::<f32>(node, "alpha")?.copied().unwrap_or(0.01);
                #[cfg(feature = "cuda")]
                let fast = if _nano_cuda && inline_elementwise {
                    try_elementwise(input, input, Binary::LeakyRelu, alpha)?
                } else { None };
                #[cfg(not(feature = "cuda"))]
                let fast: Option<Tensor> = None;
                let output = match fast {
                    Some(output) => output,
                    None => candle_nn::ops::leaky_relu(input, alpha.into())?,
                };
                values.insert(node.output[0].clone(), output);
            }
            // https://github.com/onnx/onnx/blob/main/docs/Operators.md#Gemm
            "Gemm" => {
                let a = get(&node.input[0])?;
                let b = get(&node.input[1])?;
                let c = get(&node.input[2])?;

                let alpha = get_attr_opt::<f32>(node, "alpha")?.copied().unwrap_or(1.0);
                let beta = get_attr_opt::<f32>(node, "beta")?.copied().unwrap_or(1.0);

                let trans_a = get_attr_opt::<i64>(node, "transA")?.copied().unwrap_or(0);
                let trans_b = get_attr_opt::<i64>(node, "transB")?.copied().unwrap_or(0);

                let a = if trans_a == 0 { a.clone() } else { a.t()? };
                let b = if trans_b == 0 { b.clone() } else { b.t()? };

                let a = if alpha == 1.0 { a } else { (a * f64::from(alpha))? };
                let c = if beta == 1.0 { c.clone() } else { (c * f64::from(beta))? };
                let product = a.broadcast_matmul(&b).map_err(|e| candle::Error::Msg(format!("Gemm {} ({:?} {:?} x {:?} {:?}): {e}", node.name, a.dtype(), a.dims(), b.dtype(), b.dims())))?;
                let output = binary!(&product, &c, Add, broadcast_add);
                values.insert(node.output[0].clone(), output);
            }
            "LSTM" => {
                let direction = get_attr_opt(node, "direction")?.unwrap_or("forward");
                if direction != "forward" && direction != "reverse" && direction != "bidirectional" {
                    bail!("LSTM direction '{direction}' is not supported in {}", node.name);
                }
                let num_directions = if direction == "bidirectional" { 2 } else { 1 };
                let hidden_size: i64 = get_attr(node, "hidden_size").copied()?;
                let input_forget = get_attr_opt(node, "input_forget")?.copied().unwrap_or(0);
                if input_forget != 0 {
                    bail!("LSTM currently only supports input_forget == 0");
                }
                let activations_default = vec![
                    "Sigmoid".to_string(),
                    "Tanh".to_string(),
                    "Tanh".to_string(),
                ];
                let activations = get_attr_opt_owned::<Vec<String>>(node, "activations")?
                    .unwrap_or(activations_default.clone());
                if activations != activations_default {
                    bail!("LSTM currently only supports default activations ({activations_default:?})");
                }
                // activation_alpha and activation_beta don't apply to (Sigmoid, Tanh, Tanh) so ignoring them is okay
                if get_attr_opt::<f32>(node, "clip")?.is_some() {
                    bail!("LSTM does not currently support clip attribute");
                }

                // The shape format of inputs X, initial_h and outputs Y, Y_h.
                // If 0, the following shapes are expected:
                //     X.shape = [seq_length, batch_size, input_size],
                //     Y.shape = [seq_length, num_directions, batch_size, hidden_size],
                //     initial_h.shape = Y_h.shape = [num_directions, batch_size, hidden_size].
                // If 1, the following shapes are expected:
                //     X.shape = [batch_size, seq_length, input_size],
                //     Y.shape = [batch_size, seq_length, num_directions, hidden_size],
                //     initial_h.shape = Y_h.shape = [batch_size, num_directions, hidden_size].
                let layout = get_attr_opt(node, "layout")?.copied().unwrap_or(0);
                if layout != 0 {
                    bail!("LSTM currently only supports layout == 0");
                }

                // The input sequences packed (and potentially padded) into one 3-D tensor
                // with the shape of `[seq_length, batch_size, input_size]`.
                let x = get(&node.input[0])?;
                // XXX: depends on layout
                let (seq_length, batch_size, _input_size) = x.dims3()?;
                // The weight tensor for the gates.
                // Concatenation of `W[iofc]` and `WB[iofc]` (if bidirectional) along dimension 0.
                // The tensor has shape `[num_directions, 4*hidden_size, input_size]`.
                let w = get(&node.input[1])?;
                // The recurrence weight tensor.
                // Concatenation of `R[iofc]` and `RB[iofc]` (if bidirectional) along dimension 0.
                // This tensor has shape `[num_directions, 4*hidden_size, hidden_size]`.
                let r = get(&node.input[2])?;

                // The bias tensor for input gate.
                // Concatenation of `[Wb[iofc], Rb[iofc]]`, and `[WBb[iofc], RBb[iofc]]` (if bidirectional) along dimension 0.
                // This tensor has shape `[num_directions, 8*hidden_size]`.
                // Optional: If not specified - assumed to be 0.
                let b_default: Tensor;
                let b = match get_opt(3) {
                    Some(n) => n?,
                    None => {
                        b_default = Tensor::zeros(
                            (num_directions, 8 * hidden_size as usize),
                            DType::F32,
                            x.device(),
                        )?;
                        &b_default
                    }
                };

                // Optional tensor specifying lengths of the sequences in a batch.
                // If not specified - assumed all sequences in the batch to have length `seq_length`.
                // It has shape `[batch_size]`.
                let seq_lens_is_default = match get_opt(4) {
                    Some(n) => n?.to_vec1::<i64>()?.iter().all(|e| *e as usize == seq_length),
                    None => true,
                };
                if !seq_lens_is_default {
                    bail!("LSTM currently only supports default value of seq_lens");
                }

                // Optional initial value of the hidden. If not specified - assumed to be 0.
                // It has shape `[num_directions, batch_size, hidden_size]`.
                let initial_h_default: Tensor;
                let initial_h = match get_opt(5) {
                    Some(n) => n?,
                    _ => {
                        initial_h_default = Tensor::zeros(
                            (num_directions, batch_size, hidden_size as usize),
                            DType::F32,
                            x.device(),
                        )?;
                        &initial_h_default
                    }
                };

                // Optional initial value of the cell.
                // If not specified - assumed to be 0.
                // It has shape `[num_directions, batch_size, hidden_size]`.
                let initial_c_default: Tensor;
                let initial_c = match node.input.get(6) {
                    Some(n) if !n.is_empty() => get(n)?,
                    _ => {
                        initial_c_default = Tensor::zeros(
                            (num_directions, batch_size, hidden_size as usize),
                            DType::F32,
                            x.device(),
                        )?;
                        &initial_c_default
                    }
                };

                // The weight tensor for peepholes.
                // Concatenation of `P[iof]` and `PB[iof]` (if bidirectional) along dimension 0.
                // It has shape `[num_directions, 3*hidde_size]`. Optional: If not specified - assumed to be 0.
                let gates_preordered = values.contains_key(&format!("__kale_lstm_gates_preordered/{}", node.name));
                if !gates_preordered {
                    if let Some(p) = get_opt(7) {
                        if p?.to_vec2::<f32>()?.iter().flatten().any(|e| *e != 0.0) {
                            bail!("LSTM currently only supports default value of p (a Tensor of all zeroes)");
                        }
                    }
                }

                let directions: Vec<bool> = match direction {
                    "forward" => vec![false],
                    "reverse" => vec![true],
                    _ => vec![false, true],
                };
                let mut sequences = Vec::with_capacity(num_directions);
                let mut final_hs = Vec::with_capacity(num_directions);
                let mut final_cs = Vec::with_capacity(num_directions);
                for (direction_index, reverse) in directions.iter().copied().enumerate() {
                    let b_dir = b.get(direction_index)?;
                    let (w_dir, r_dir, wb, rb) = if gates_preordered {
                        (
                            w.get(direction_index)?,
                            r.get(direction_index)?,
                            b_dir.narrow(0, 0, 4 * hidden_size as usize)?,
                            b_dir.narrow(0, 4 * hidden_size as usize, 4 * hidden_size as usize)?,
                        )
                    } else {
                        let idx_wb = Tensor::arange(0, 4 * hidden_size, x.device())?;
                        let idx_rb = Tensor::arange(4 * hidden_size, 8 * hidden_size, x.device())?;
                        let wb = b_dir.index_select(&idx_wb, 0)?;
                        let rb = b_dir.index_select(&idx_rb, 0)?;
                        let idx_i = Tensor::arange(0, hidden_size, x.device())?;
                        let idx_o = Tensor::arange(hidden_size, 2 * hidden_size, x.device())?;
                        let idx_f = Tensor::arange(2 * hidden_size, 3 * hidden_size, x.device())?;
                        let idx_c = Tensor::arange(3 * hidden_size, 4 * hidden_size, x.device())?;
                        let idx_ifco = Tensor::cat(&[&idx_i, &idx_f, &idx_c, &idx_o], 0)?;
                        (
                            w.get(direction_index)?.index_select(&idx_ifco, 0)?,
                            r.get(direction_index)?.index_select(&idx_ifco, 0)?,
                            wb.index_select(&idx_ifco, 0)?,
                            rb.index_select(&idx_ifco, 0)?,
                        )
                    };
                    // Project all input timesteps in one batched GEMM instead of
                    // launching an input-weight matmul separately for every step.
                    let projected = x.broadcast_matmul(&w_dir.t()?)?;
                    let input_gates = binary!(&projected, &wb, Add, broadcast_add);
                    let recurrent_weights = r_dir.t()?;
                    #[cfg(feature = "cuda")]
                    let use_fused_step = device.is_cuda()
                        && !disable_fused_lstm_step;
                    let mut hidden = initial_h.get(direction_index)?;
                    let mut cell = initial_c.get(direction_index)?;
                    #[cfg(feature = "cuda")]
                    if use_fused_step && persistent_lstm && seq_length > 0 {
                        let output = input_gates.apply_op1_no_bwd(&FusedLstmStepCuda {
                            recurrent_weights: r_dir.clone(),
                            recurrent_bias: rb.clone(),
                            previous_hidden: hidden,
                            previous_cell: cell,
                            hidden_size: hidden_size as usize,
                            sequence_reverse: Some(reverse),
                            cooperative: _nano_cuda && std::env::var_os("KALE_TTS_DISABLE_COOPERATIVE_LSTM").is_none(),
                        })?;
                        sequences.push(output.narrow(0, 0, seq_length)?);
                        final_hs.push(output.get(if reverse { 0 } else { seq_length - 1 })?);
                        final_cs.push(output.get(seq_length)?);
                        continue;
                    }
                    let steps: Box<dyn Iterator<Item = usize>> = if reverse { Box::new((0..seq_length).rev()) } else { Box::new(0..seq_length) };
                    let mut states = Vec::with_capacity(seq_length);
                    for t in steps {
                        #[cfg(feature = "cuda")]
                        let fused_state = if use_fused_step {
                            Some(input_gates.get(t)?.apply_op1_no_bwd(&FusedLstmStepCuda {
                                recurrent_weights: r_dir.clone(),
                                recurrent_bias: rb.clone(),
                                previous_hidden: hidden.clone(),
                                previous_cell: cell.clone(),
                                hidden_size: hidden_size as usize,
                                sequence_reverse: None,
                                cooperative: false,
                            })?)
                        } else {
                            None
                        };
                        #[cfg(not(feature = "cuda"))]
                        let fused_state: Option<Tensor> = None;
                        if let Some(next_state) = fused_state {
                            hidden = next_state.get(0)?;
                            cell = next_state.get(1)?;
                        } else {
                            let recurrent_gates = hidden.matmul(&recurrent_weights)?.broadcast_add(&rb)?;
                            let gates = input_gates.get(t)?.broadcast_add(&recurrent_gates)?;
                            let chunks = gates.chunk(4, 1)?;
                            let input_gate = candle_nn::ops::sigmoid(&chunks[0])?;
                            let forget_gate = candle_nn::ops::sigmoid(&chunks[1])?;
                            let cell_gate = chunks[2].tanh()?;
                            let output_gate = candle_nn::ops::sigmoid(&chunks[3])?;
                            cell = forget_gate
                                .broadcast_mul(&cell)?
                                .broadcast_add(&input_gate.broadcast_mul(&cell_gate)?)?;
                            hidden = output_gate.broadcast_mul(&cell.tanh()?)?;
                        }
                        states.push(hidden.clone());
                    }
                    if reverse { states.reverse(); }
                    let sequence = Tensor::stack(&states, 1)?.transpose(0, 1)?;
                    sequences.push(sequence);
                    final_hs.push(hidden);
                    final_cs.push(cell);
                }
                if let Some(name) = node.output.first().filter(|n| !n.is_empty()) {
                    let output = Tensor::stack(&sequences, 1)?;
                    values.insert(name.clone(), output);
                }
                if let Some(name) = node.output.get(1).filter(|n| !n.is_empty()) {
                    values.insert(name.clone(), Tensor::stack(&final_hs, 0)?);
                }
                if let Some(name) = node.output.get(2).filter(|n| !n.is_empty()) {
                    values.insert(name.clone(), Tensor::stack(&final_cs, 0)?);
                }
            }
            "RNN" => {
                // activation_alpha and activation_beta don't apply to (Tanh, Tanh) so ignoring them is okay
                let activations_default = vec!["Tanh".to_string(), "Tanh".to_string()];
                let activations = get_attr_opt_owned::<Vec<String>>(node, "activations")?
                    .unwrap_or(activations_default.clone());
                let clip = get_attr_opt::<f32>(node, "clip")?.copied();
                if clip.is_some() {
                    bail!("RNN does not currently support clip attribute");
                }
                let direction = get_attr_opt(node, "direction")?.unwrap_or("forward");
                if direction != "forward" {
                    bail!("RNN currently only supports direction == \"forward\"");
                }
                let num_directions = if direction == "bidirectional" { 2 } else { 1 };
                let hidden_size: i64 = get_attr(node, "hidden_size").copied()?;

                // The shape format of inputs X, initial_h and outputs Y, Y_h.
                // If 0, the following shapes are expected:
                //    X.shape = [seq_length, batch_size, input_size],
                //    Y.shape = [seq_length, num_directions, batch_size, hidden_size],
                //    initial_h.shape = Y_h.shape = [num_directions, batch_size, hidden_size].
                // If 1, the following shapes are expected:
                //    X.shape = [batch_size, seq_length, input_size],
                //    Y.shape = [batch_size, seq_length, num_directions, hidden_size],
                //    initial_h.shape = Y_h.shape = [batch_size, num_directions, hidden_size].
                let layout = get_attr_opt(node, "layout")?.copied().unwrap_or(0);
                if layout != 0 {
                    bail!("RNN currently only supports layout == 0");
                }

                // The input sequences packed (and potentially padded) into one 3-D tensor
                // with the shape of `[seq_length, batch_size, input_size]`.
                let x = get(&node.input[0])?;
                // XXX: depends on layout
                let (seq_length, batch_size, _) = x.dims3()?;
                // The weight tensor for the input gate.
                // Concatenation of `Wi` and `WBi` (if bidirectional).
                // The tensor has shape `[num_directions, hidden_size, input_size]`.
                let w = get(&node.input[1])?;
                // The recurrence weight tensor.
                // Concatenation of `Ri` and `RBi` (if bidirectional).
                // This tensor has shape `[num_directions, hidden_size, hidden_size]`.
                let r = get(&node.input[2])?;

                // The bias tensor for input gate.
                // Concatenation of `[Wbi, Rbi]` and `[WBbi, RBbi]` (if bidirectional).
                // This tensor has shape `[num_directions, 2*hidden_size]`.
                // Optional: If not specified - assumed to be 0.
                let b_default: Tensor;
                let b = match get_opt(3) {
                    Some(n) => n?,
                    None => {
                        b_default = Tensor::zeros(
                            (num_directions, 2 * hidden_size as usize),
                            DType::F32,
                            x.device(),
                        )?;
                        &b_default
                    }
                };

                // Optional tensor specifying lengths of the sequences in a batch.
                // If not specified - assumed all sequences in the batch to have length `seq_length`.
                // It has shape `[batch_size]`.
                let seq_lens_default: Tensor;
                let seq_lens = match get_opt(4) {
                    Some(n) => n?,
                    None => {
                        seq_lens_default =
                            Tensor::full(seq_length as i64, (batch_size,), x.device())?;
                        &seq_lens_default
                    }
                };
                let seq_lens_is_default =
                    (seq_lens.to_vec1::<i64>()?.iter()).all(|e| *e as usize == seq_length);
                if !seq_lens_is_default {
                    bail!("RNN currently does not support variable-length sequences. All sequences must use the full sequence length of {}", seq_length);
                }

                // Optional initial value of the hidden. If not specified - assumed to be 0.
                // It has shape `[num_directions, batch_size, hidden_size]`.
                let initial_h_default: Tensor;
                let initial_h = match get_opt(5) {
                    Some(n) => n?,
                    _ => {
                        initial_h_default = Tensor::zeros(
                            (num_directions, batch_size, hidden_size as usize),
                            DType::F32,
                            x.device(),
                        )?;
                        &initial_h_default
                    }
                };

                fn choose_activation(activation: &str, x: &Tensor) -> Result<Tensor> {
                    match activation {
                        "Tanh" => x.tanh(),
                        _ => bail!("unsupported activation {activation}"),
                    }
                }

                // these all have [num_directions, ...] shapes
                let w = w.get(0)?;
                let r = r.get(0)?;
                let b = b.get(0)?;
                let idx_wb = Tensor::arange(0, hidden_size, x.device())?;
                let idx_rb = Tensor::arange(hidden_size, 2 * hidden_size, x.device())?;
                let wb = b.index_select(&idx_wb, 0)?;
                let rb = b.index_select(&idx_rb, 0)?;
                let mut h_t = initial_h.get(0)?;
                let mut h_list: Vec<Tensor> = vec![];
                for i in 0..seq_length {
                    let xs = x.get(i)?;
                    let h = xs
                        .matmul(&w.t()?)?
                        .add(&h_t.matmul(&r.t()?)?)?
                        .add(&wb.unsqueeze(0)?)?
                        .add(&rb.unsqueeze(0)?)?;
                    let h = choose_activation(&activations[0], &h)?;
                    h_list.push(h.to_owned());
                    h_t = h;
                }
                let h = Tensor::stack(&h_list, 0)?;
                let h =
                    h.reshape((seq_length, num_directions, batch_size, hidden_size as usize))?;
                values.insert(node.output[0].clone(), h);
                values.insert(
                    node.output[1].clone(),
                    h_t.reshape((num_directions, batch_size, hidden_size as usize))?,
                );
            }
            // https://onnx.ai/onnx/operators/onnx__Xor.html
            "Xor" => {
                // Since we don't have a `DType::Bool` yet, this ensures that we are working with `0`(False) & `1`(True)
                let a = get(&node.input[0])?.gt(0_u8)?;
                let b = get(&node.input[1])?.gt(0_u8)?;

                let out = a.broadcast_add(&b)?.eq(1_u8)?;

                values.insert(node.output[0].clone(), out);
            }
            // https://onnx.ai/onnx/operators/onnx__And.html
            "And" => {
                let a = get(&node.input[0])?.gt(0_u8)?;
                let b = get(&node.input[1])?.gt(0_u8)?;

                let out = a.broadcast_mul(&b)?;

                values.insert(node.output[0].clone(), out);
            }
            // https://onnx.ai/onnx/operators/onnx__Or.html
            "Or" => {
                let a = get(&node.input[0])?.gt(0_u8)?;
                let b = get(&node.input[1])?.gt(0_u8)?;

                let out = a.broadcast_add(&b)?.gt(0_u8)?;

                values.insert(node.output[0].clone(), out);
            }
            // https://onnx.ai/onnx/operators/onnx__Sign.html
            "Sign" => {
                let input = get(&node.input[0])?;
                let output = input.sign()?;
                values.insert(node.output[0].clone(), output);
            }
            // https://onnx.ai/onnx/operators/onnx__Selu.html
            "Selu" => {
                let input = get(&node.input[0])?;
                let alpha = get_attr_opt::<f32>(node, "alpha")?
                    .copied()
                    .unwrap_or(1.6732632);
                let gamma = get_attr_opt::<f32>(node, "gamma")?
                    .copied()
                    .unwrap_or(1.050701);
                let out = candle_nn::ops::selu(input, alpha as f32, gamma as f32)?;
                values.insert(node.output[0].clone(), out);
            }

            // https://onnx.ai/onnx/operators/onnx__OneHot.html
            "OneHot" => {
                let indices = get(&node.input[0])?;
                let orig_shape = get(&node.input[0])?.dims().to_vec();
                let depth_tensor = get(&node.input[1])?;
                let values_tensor = get(&node.input[2])?;

                let depth = to_scalar_flexible::<i64>(depth_tensor)? as usize;
                let values_vec = values_tensor.to_vec1::<f32>()?;
                if values_vec.len() != 2 {
                    return Err(candle::Error::Msg(
                        "OneHot: expected 2-element values tensor".to_string(),
                    ));
                }
                let off_value = values_vec[0];
                let on_value = values_vec[1];

                let mut axis = node
                    .attribute
                    .iter()
                    .find(|attr| attr.name == "axis")
                    .map(|attr| attr.i)
                    .unwrap_or(-1);

                let rank = indices.rank();
                if axis < -((rank as i64) + 1) || axis > (rank as i64) {
                    return Err(candle::Error::Msg(format!(
                        "OneHot: invalid axis {axis} for rank {rank}"
                    )));
                }
                if axis < 0 {
                    axis += rank as i64 + 1;
                }

                let indices = indices.flatten_all()?;
                let indices_vec = indices.to_vec1::<i64>()?;
                let mut out = vec![off_value; depth * indices.elem_count()];
                for (i, &index) in indices_vec.iter().enumerate() {
                    let idx = if index < 0 {
                        (index + depth as i64) as usize
                    } else {
                        index as usize
                    };
                    if idx >= depth {
                        continue;
                    }
                    out[i * depth + idx] = on_value;
                }

                let mut target_shape = orig_shape;
                target_shape.push(depth);
                let output = Tensor::from_vec(out, target_shape, indices.device())?;

                let final_output = if axis as usize == output.rank() - 1 {
                    output
                } else {
                    fn move_axis_to(rank: usize, from: usize, to: usize) -> Vec<usize> {
                        let mut dims: Vec<usize> = (0..rank).collect();
                        let axis = dims.remove(from);
                        dims.insert(to, axis);
                        dims
                    }

                    let perm = move_axis_to(output.rank(), output.rank() - 1, axis as usize);
                    output.permute(&*perm)?
                };
                values.insert(node.output[0].clone(), final_output);
            }
            "HardSwish" => {
                let input = get(&node.input[0])?;
                let hard_sigmoid = candle_nn::ops::hard_sigmoid(&input)?;
                let output = input * hard_sigmoid;
                values.insert(node.output[0].clone(), output?);
            }
            "Resize" => {
                let input = get(&node.input[0])?;

                if input.rank() != 3 && input.rank() != 4 {
                    bail!("Unsupported rank for nearest resize: {} in {}", input.rank(), node.name);
                }

                let scales = if node.input.len() > 2 && !node.input[2].is_empty() {
                    Some(get_control(&node.input[2])?)
                } else {
                    None
                };

                let sizes = if node.input.len() > 3 && !node.input[3].is_empty() {
                    Some(get_control(&node.input[3])?)
                } else {
                    None
                };

                let output_dims = match (scales, sizes) {
                    (Some(_), Some(_)) => {
                        bail!("Scales and sizes cannot both be set for Resize operation")
                    }
                    (Some(scales_tensor), None) => {
                        let scale_values = scales_tensor.to_vec1::<f32>()?;
                        input
                            .dims()
                            .iter()
                            .enumerate()
                            .map(|(i, &d)| (d as f32 * scale_values[i]) as usize)
                            .collect::<Vec<_>>()
                    }
                    (None, Some(sizes_tensor)) => sizes_tensor
                        .to_vec1::<i64>()?
                        .iter()
                        .map(|&d| d as usize)
                        .collect::<Vec<_>>(),
                    (None, None) => bail!("Either scales or sizes should be present"),
                };

                let coordinate_transformation_mode =
                    get_attr_opt::<str>(node, "coordinate_transformation_mode")?
                        .unwrap_or("half_pixel");
                // Interpolation mode: nearest, linear, or cubic.
                let mode = get_attr_opt::<str>(node, "mode")?.unwrap_or("nearest");
                // How to determine the "nearest" pixel in nearest interpolation mode.
                let nearest_mode =
                    get_attr_opt::<str>(node, "nearest_mode")?.unwrap_or("round_prefer_floor");

                if mode != "nearest" {
                    bail!("Unsupported resize mode: {}", mode);
                }

                if nearest_mode != "floor" {
                    bail!("Unsupported nearest_mode for resize: {}", nearest_mode);
                }

                if coordinate_transformation_mode != "asymmetric" {
                    bail!(
                        "Unsupported coordinate_transformation_mode for resize: {}",
                        coordinate_transformation_mode
                    );
                }

                let output = match input.rank() {
                    3 => {
                        #[cfg(feature = "cuda")]
                        if device.is_cuda() && input.dtype() == DType::F32 {
                            input.apply_op1_no_bwd(&UpsampleNearest1dF32Cuda {
                                target_size: output_dims[2],
                            })?
                        } else {
                            input.upsample_nearest1d(output_dims[2])?
                        }
                        #[cfg(not(feature = "cuda"))]
                        input.upsample_nearest1d(output_dims[2])?
                    }
                    4 => input.upsample_nearest2d(output_dims[2], output_dims[3])?,
                    _ => unreachable!(),
                };

                values.insert(node.output[0].clone(), output);
            }
            "Trilu" => {
                let input = get(&node.input[0])?;

                // Get the diagonal offset 'k' from the second input if provided
                let k = if node.input.len() > 1 && !node.input[1].is_empty() {
                    to_vec0_flexible::<i64>(get(&node.input[1])?)?
                } else {
                    0
                };

                // Get the 'upper' attribute
                let upper = get_attr_opt::<i64>(node, "upper")?.copied().unwrap_or(1);

                // For batched inputs, we need to handle each matrix separately
                let dims = input.dims();
                if dims.len() < 2 {
                    bail!("Trilu expects input with at least 2 dimensions: {:?}", dims);
                }

                // Get the last two dimensions which represent the matrix
                let n = dims[dims.len() - 2];
                let m = dims[dims.len() - 1];
                let max_dim = std::cmp::max(n, m);

                // Handle the diagonal offset k
                let mask = if k != 0 {
                    let mut data = vec![0u32; n * m];
                    for i in 0..n {
                        for j in 0..m {
                            if (upper != 0 && (j as i64) >= (i as i64) + k)
                                || (upper == 0 && (j as i64) <= (i as i64) + k)
                            {
                                data[i * m + j] = 1u32;
                            }
                        }
                    }
                    Tensor::from_vec(data, (n, m), input.device())?.to_dtype(input.dtype())?
                } else if upper == 0 {
                    Tensor::tril2(max_dim, input.dtype(), input.device())?
                } else {
                    Tensor::triu2(max_dim, input.dtype(), input.device())?
                };

                let final_mask = if n != m {
                    mask.narrow(0, 0, n)?.narrow(1, 0, m)?
                } else {
                    mask
                };

                let output = (input * &final_mask)?;

                values.insert(node.output[0].clone(), output);
            }
            "ScatterND" => {
                let data = get(&node.input[0])?;

                let indices = get(&node.input[1])?;
                let indices = indices.to_dtype(DType::I64)?;

                let updates = get(&node.input[2])?;

                let reduction = get_attr_opt::<str>(node, "reduction")?.unwrap_or("none");

                let indices_shape = indices.dims();
                let data_shape = data.dims();
                let _updates_shape = updates.dims();

                // Last dimension of indices represents the depth of indexing
                let k = indices_shape.last().unwrap().clone();

                if k > data.rank() {
                    bail!("ScatterND expects k (indices.shape[-1]) to be at most the rank of data");
                }

                let num_updates = indices_shape[..indices_shape.len() - 1]
                    .iter()
                    .product::<usize>();

                let flat_indices = if indices.rank() == 1 && k == 1 {
                    indices.unsqueeze(0)?
                } else {
                    indices.reshape((num_updates, k))?
                };

                // Calculate the shape of each update element
                let update_element_shape = if k < data_shape.len() {
                    data_shape[k..].to_vec()
                } else {
                    vec![]
                };

                // Expected shape for updates based on indices and target tensor
                let expected_updates_shape = {
                    let mut shape = indices_shape[..indices_shape.len() - 1].to_vec();
                    shape.extend(&update_element_shape);
                    shape
                };

                // Validate or reshape updates to expected shape
                let updates = if updates.dims() != expected_updates_shape {
                    if updates.rank() == 0 {
                        // Handle scalar updates
                        let mut target_shape = vec![num_updates];
                        target_shape.extend(&update_element_shape);
                        updates.broadcast_as(target_shape)?
                    } else {
                        // Try to broadcast or reshape updates to expected shape
                        let flat_shape =
                            vec![num_updates, update_element_shape.iter().product::<usize>()];
                        let flattened = updates.reshape(flat_shape)?;
                        flattened.reshape(expected_updates_shape)?
                    }
                } else {
                    updates.clone()
                };

                let mut output = data.clone();

                // convert indices to flat indices
                let mut flat_output = output.flatten_all()?;
                let flat_updates = if update_element_shape.is_empty() {
                    updates.reshape(num_updates)?
                } else {
                    let product = update_element_shape.iter().product::<usize>();
                    updates.reshape((num_updates, product))?
                };

                // Calculate strides for the output tensor
                let mut strides: Vec<usize> = vec![1];
                for i in (0..data_shape.len() - 1).rev() {
                    strides.push(strides.last().unwrap() * data_shape[i + 1]);
                }
                strides.reverse();

                // Process each update
                for i in 0..num_updates {
                    let index_slice = flat_indices.narrow(0, i, 1)?;
                    let indices_vec = index_slice.squeeze(0)?.to_vec1::<i64>()?;

                    // Convert multi-dimensional indices to flat index
                    let mut flat_idx: usize = 0;
                    for (dim, &idx) in indices_vec.iter().enumerate() {
                        let dim_size = data_shape[dim] as i64;
                        let norm_idx = if idx < 0 { dim_size + idx } else { idx };

                        if norm_idx < 0 || norm_idx >= dim_size {
                            bail!(
                                "Index {} out of bounds for dimension {} with size {}",
                                idx,
                                dim,
                                dim_size
                            );
                        }

                        flat_idx += (norm_idx as usize) * strides[dim];
                    }

                    // Extract current update
                    let update_slice = if update_element_shape.is_empty() {
                        flat_updates.narrow(0, i, 1)?.squeeze(0)?
                    } else {
                        flat_updates.narrow(0, i, 1)?
                    };

                    match reduction {
                        "add" => {
                            if update_element_shape.is_empty() {
                                let existing = flat_output.narrow(0, flat_idx, 1)?;
                                let new_value = existing.add(&update_slice.unsqueeze(0)?)?;
                                flat_output = flat_output.slice_scatter(&new_value, 0, flat_idx)?;
                            } else {
                                let slice_size = update_element_shape.iter().product::<usize>();
                                let existing = flat_output.narrow(0, flat_idx, slice_size)?;
                                let new_value = existing.add(&update_slice)?;
                                flat_output = flat_output.slice_scatter(&new_value, 0, flat_idx)?;
                            }
                        }
                        "none" | _ => {
                            if update_element_shape.is_empty() {
                                flat_output = flat_output.slice_scatter(
                                    &update_slice.unsqueeze(0)?,
                                    0,
                                    flat_idx,
                                )?;
                            } else {
                                flat_output =
                                    flat_output.slice_scatter(&update_slice, 0, flat_idx)?;
                            }
                        }
                    }
                }

                // Reshape flat output back to original shape
                output = flat_output.reshape(data_shape.to_vec())?;

                values.insert(node.output[0].clone(), output);
            }
            op_type => bail!("unsupported op_type {op_type} for op {node:?}"),
        }
        if let Some(start) = op_start {
            if device.is_cuda() {
                device.synchronize()?;
            }
            eprintln!(
                "KALE_TTS_PROFILE_OP,{},{},{:.3}",
                node.op_type,
                node.name,
                start.elapsed().as_secs_f64() * 1000.0,
            );
        }
    }
    graph
        .output
        .iter()
        .map(|output| match values.remove(&output.name) {
            None => bail!("cannot find output {}", output.name),
            Some(value) => Ok((output.name.clone(), value)),
        })
        .collect()
}

fn broadcast_shape(shape_a: &[usize], shape_b: &[usize]) -> Result<Vec<usize>> {
    let (longest, shortest) = if shape_a.len() > shape_b.len() {
        (shape_a, shape_b)
    } else {
        (shape_b, shape_a)
    };
    let diff = longest.len() - shortest.len();
    let mut target_shape = longest[0..diff].to_vec();
    for (dim1, dim2) in longest[diff..].iter().zip(shortest.iter()) {
        if *dim1 == *dim2 || *dim2 == 1 || *dim1 == 1 {
            target_shape.push(usize::max(*dim1, *dim2));
        } else {
            bail!(
                "Expand: incompatible shapes for broadcast, {:?} and {:?}",
                shape_a,
                shape_b
            );
        }
    }
    Ok(target_shape)
}

fn broadcast_shape_from_many(shapes: &[&[usize]]) -> Result<Vec<usize>> {
    if shapes.is_empty() {
        return Ok(Vec::new());
    }
    let mut shape_out = shapes[0].to_vec();
    for shape in shapes[1..].iter() {
        shape_out = broadcast_shape(&shape_out, shape)?;
    }
    Ok(shape_out)
}

/// Extract scalar from tensors that may be wrapped in extra dimensions.
/// Some ONNX exports use shape [1] or [1,1] where scalars are expected.
/// Only accepts single-element tensors; multi-element tensors still fail.
fn to_scalar_flexible<T: candle::WithDType>(t: &Tensor) -> Result<T> {
    if t.rank() > 0 && t.elem_count() == 1 {
        t.flatten_all()?.i(0)?.to_scalar::<T>()
    } else {
        t.to_scalar::<T>()
    }
}

/// Same as to_scalar_flexible but returns via to_vec0 for types that need it.
fn to_vec0_flexible<T: candle::WithDType>(t: &Tensor) -> Result<T> {
    if t.rank() > 0 && t.elem_count() == 1 {
        t.flatten_all()?.i(0)?.to_vec0::<T>()
    } else {
        t.to_vec0::<T>()
    }
}
