// Fuses the int16-backed ONNX int8 quantized-weight cast, zero-point subtraction,
// and scale multiplication. Candle has no native CUDA cast from I16 to F32.
extern "C" __global__ void kale_dequantize_i16(
    const short* quantized,
    const float* scales,
    const short* zero_points,
    float* output,
    int count,
    int axis_dim,
    int inner_size,
    int scale_count
) {
    const int i = blockIdx.x * blockDim.x + threadIdx.x;
    if (i >= count) return;
    const int channel = scale_count == 1 ? 0 : (i / inner_size) % axis_dim;
    const int scale_index = scale_count == 1 ? 0 : channel;
    output[i] = ((float)quantized[i] - (float)zero_points[scale_index]) * scales[scale_index];
}

// ONNX integer Mod (fmod=0) needs floor-division semantics. Fuse it into one
// kernel because Candle's generic CUDA floor path does not support I64.
extern "C" __global__ void kale_mod_i64(
    const long long* lhs,
    const long long* rhs,
    long long* output,
    int count,
    int lhs_count,
    int rhs_count
) {
    const int i = blockIdx.x * blockDim.x + threadIdx.x;
    if (i >= count) return;
    const long long a = lhs[lhs_count == 1 ? 0 : i];
    const long long b = rhs[rhs_count == 1 ? 0 : i];
    long long remainder = a % b;
    if (remainder != 0 && ((remainder < 0) != (b < 0))) remainder += b;
    output[i] = remainder;
}

extern "C" __global__ void kale_upsample_nearest1d_f32(
    const float* input,
    float* output,
    size_t batches,
    size_t channels,
    size_t source_size,
    size_t target_size,
    size_t stride_batch,
    size_t stride_channel,
    size_t stride_sample,
    size_t start_offset,
    size_t count
) {
    const size_t i = (size_t)blockIdx.x * blockDim.x + threadIdx.x;
    if (i >= count) return;
    const size_t sample = i % target_size;
    const size_t channel = (i / target_size) % channels;
    const size_t batch = i / (target_size * channels);
    const size_t source_sample = min(source_size - 1, (sample * source_size) / target_size);
    const size_t source_index = start_offset + batch * stride_batch
        + channel * stride_channel + source_sample * stride_sample;
    output[i] = input[source_index];
}

template<bool Variance>
__device__ __forceinline__ float norm_element(
    const float* input, size_t base, size_t stride, size_t index, float mean
) {
    const float value = input[base + index * stride];
    if (!Variance) return value;
    const float centered = __fsub_rn(value, mean);
    return __fmul_rn(centered, centered);
}

// Match Candle's scalar or float4 summation order before fusing the affine
// operations. Variance sums the virtual contiguous array of squared values.
template<bool Variance>
__device__ float norm_sum(
    const float* input, size_t base, size_t stride, size_t hidden,
    size_t row, float mean, size_t contiguous_mode, float* partial
) {
    const unsigned int tid = threadIdx.x;
    float sum = 0.0f;
    if (hidden <= 32) {
        if (tid == 0) {
            for (size_t i = 0; i < hidden; ++i) {
                sum += norm_element<Variance>(input, base, stride, i, mean);
            }
        }
        return sum;
    }
    if (contiguous_mode != 0) {
        const size_t head = min((4 - (row * hidden) % 4) % 4, hidden);
        const size_t tail = head + ((hidden - head) / 4) * 4;
        if (tid < head) sum += norm_element<Variance>(input, base, stride, tid, mean);
        if (contiguous_mode == 1) {
            const size_t i = head + 4 * tid;
            if (i < tail) {
                sum += norm_element<Variance>(input, base, stride, i, mean)
                    + norm_element<Variance>(input, base, stride, i + 1, mean)
                    + norm_element<Variance>(input, base, stride, i + 2, mean)
                    + norm_element<Variance>(input, base, stride, i + 3, mean);
            }
        } else if (head + tid < tail) {
            sum += norm_element<Variance>(input, base, stride, head + tid, mean);
        }
        if (tail + tid < hidden) sum += norm_element<Variance>(input, base, stride, tail + tid, mean);
    } else if (tid < hidden) {
        sum += norm_element<Variance>(input, base, stride, tid, mean);
    }

    for (unsigned int delta = 16; delta > 0; delta >>= 1)
        sum += __shfl_xor_sync(0xffffffff, sum, delta);
    if ((tid & 31) == 0) partial[tid >> 5] = sum;
    __syncthreads();
    if (tid < 32) {
        sum = tid < (blockDim.x >> 5) ? partial[tid] : 0.0f;
        for (unsigned int delta = 16; delta > 0; delta >>= 1)
            sum += __shfl_xor_sync(0xffffffff, sum, delta);
    }
    return sum;
}

// Last-axis LayerNormalization for small F32 rows. One CTA computes the full
// row statistics and affine transform, replacing the sequence of mean,
// subtract, square, variance, epsilon, sqrt/reciprocal, scale, and bias ops.
// `rows` is the second dimension of a rank-3 input; arbitrary input strides
// allow transposed Candle views without materializing a contiguous copy.
extern "C" __global__ void kale_layernorm_last_f32(
    const float* input,
    const float* scale,
    const float* bias,
    float* output,
    size_t batches,
    size_t rows,
    size_t hidden,
    size_t stride_batch,
    size_t stride_row,
    size_t stride_hidden,
    size_t input_offset,
    size_t scale_offset,
    size_t scale_stride,
    size_t bias_offset,
    size_t bias_stride,
    float epsilon,
    size_t per_channel_affine,
    size_t contiguous_mode
) {
    const size_t row = blockIdx.x;
    if (row >= batches * rows) return;
    const size_t batch = row / rows;
    const size_t row_in_batch = row % rows;
    const size_t base = input_offset + batch * stride_batch + row_in_batch * stride_row;
    const unsigned int tid = threadIdx.x;
    __shared__ float partial[32];
    __shared__ float row_mean;
    __shared__ float inv_std;

    const float scale_mean = __fdiv_rn(1.0f, (float)hidden);
    const float sum = norm_sum<false>(
        input, base, stride_hidden, hidden, row, 0.0f, contiguous_mode, partial);
    if (tid == 0) row_mean = __fmul_rn(sum, scale_mean);
    __syncthreads();

    const float square_sum = norm_sum<true>(
        input, base, stride_hidden, hidden, row, row_mean, 1, partial);
    if (tid == 0) {
        const float variance = __fmul_rn(square_sum, scale_mean);
        inv_std = __fdiv_rn(1.0f, __fsqrt_rn(__fadd_rn(variance, epsilon)));
    }
    __syncthreads();
    if (tid < hidden) {
        const size_t affine_index = per_channel_affine ? row_in_batch : tid;
        const float gamma = scale[scale_offset + affine_index * scale_stride];
        const float beta = bias[bias_offset + affine_index * bias_stride];
        const float centered = __fsub_rn(input[base + (size_t)tid * stride_hidden], row_mean);
        output[row * hidden + tid] = __fadd_rn(__fmul_rn(__fmul_rn(centered, inv_std), gamma), beta);
    }
}

// Preserve the eager order of an elementwise Mul -> Add -> LeakyRelu tail
// while combining its three output passes into one launch. Explicit round-to-
// nearest intrinsics prevent the compiler from contracting the affine steps.
extern "C" __global__ void kale_mul_add_leaky_relu_f32(
    const float* lhs,
    const float* rhs,
    const float* bias,
    float* output,
    size_t dim0,
    size_t dim1,
    size_t dim2,
    size_t lhs_dim0,
    size_t lhs_dim1,
    size_t lhs_dim2,
    size_t lhs_stride0,
    size_t lhs_stride1,
    size_t lhs_stride2,
    size_t lhs_offset,
    size_t rhs_dim0,
    size_t rhs_dim1,
    size_t rhs_dim2,
    size_t rhs_stride0,
    size_t rhs_stride1,
    size_t rhs_stride2,
    size_t rhs_offset,
    size_t bias_dim0,
    size_t bias_dim1,
    size_t bias_dim2,
    size_t bias_stride0,
    size_t bias_stride1,
    size_t bias_stride2,
    size_t bias_offset,
    float alpha,
    size_t count
) {
    const size_t i = (size_t)blockIdx.x * blockDim.x + threadIdx.x;
    if (i >= count) return;
    const size_t coord2 = i % dim2;
    const size_t coord1 = (i / dim2) % dim1;
    const size_t coord0 = i / (dim1 * dim2);
    const size_t lhs_index = lhs_offset
        + (lhs_dim0 == 1 ? 0 : coord0 * lhs_stride0)
        + (lhs_dim1 == 1 ? 0 : coord1 * lhs_stride1)
        + (lhs_dim2 == 1 ? 0 : coord2 * lhs_stride2);
    const size_t rhs_index = rhs_offset
        + (rhs_dim0 == 1 ? 0 : coord0 * rhs_stride0)
        + (rhs_dim1 == 1 ? 0 : coord1 * rhs_stride1)
        + (rhs_dim2 == 1 ? 0 : coord2 * rhs_stride2);
    const size_t bias_index = bias_offset
        + (bias_dim0 == 1 ? 0 : coord0 * bias_stride0)
        + (bias_dim1 == 1 ? 0 : coord1 * bias_stride1)
        + (bias_dim2 == 1 ? 0 : coord2 * bias_stride2);
    const float affine = __fadd_rn(__fmul_rn(lhs[lhs_index], rhs[rhs_index]), bias[bias_index]);
    const float positive = fmaxf(affine, 0.0f);
    const float negative = __fmul_rn(fminf(affine, 0.0f), alpha);
    output[i] = __fadd_rn(positive, negative);
}

extern "C" __global__ void kale_depthwise_conv1d_f32(
    const float* input,
    const float* weights,
    const float* bias,
    float* output,
    size_t batches,
    size_t channels,
    size_t source_size,
    size_t output_size,
    size_t kernel_size,
    size_t padding,
    size_t stride,
    size_t dilation,
    size_t input_stride_batch,
    size_t input_stride_channel,
    size_t input_stride_sample,
    size_t input_offset,
    size_t weight_stride_channel,
    size_t weight_stride_sample,
    size_t weight_offset,
    size_t bias_stride,
    size_t bias_offset,
    size_t count
) {
    const size_t i = (size_t)blockIdx.x * blockDim.x + threadIdx.x;
    if (i >= count) return;
    const size_t out_sample = i % output_size;
    const size_t channel = (i / output_size) % channels;
    const size_t batch = i / (output_size * channels);
    float sum = bias[bias_offset + channel * bias_stride];
    for (size_t k = 0; k < kernel_size; ++k) {
        const long long source_sample = (long long)(out_sample * stride + k * dilation)
            - (long long)padding;
        if (source_sample >= 0 && (size_t)source_sample < source_size) {
            const size_t x_index = input_offset + batch * input_stride_batch
                + channel * input_stride_channel + (size_t)source_sample * input_stride_sample;
            const size_t w_index = weight_offset + channel * weight_stride_channel
                + k * weight_stride_sample;
            sum += input[x_index] * weights[w_index];
        }
    }
    output[i] = sum;
}

extern "C" __global__ void kale_lstm_step_f32(
    const float* input_gates,
    const float* recurrent_weights,
    const float* recurrent_bias,
    const float* previous_hidden,
    const float* previous_cell,
    float* output,
    size_t batch_size,
    size_t hidden_size,
    size_t gate_stride_batch,
    size_t gate_stride_hidden,
    size_t gate_offset,
    size_t weight_stride_row,
    size_t weight_stride_col,
    size_t weight_offset,
    size_t bias_stride,
    size_t bias_offset,
    size_t hidden_stride_batch,
    size_t hidden_stride_element,
    size_t hidden_offset,
    size_t cell_stride_batch,
    size_t cell_stride_element,
    size_t cell_offset
) {
    __shared__ float recurrent[4096];
    const size_t batch = blockIdx.x;
    const unsigned int lane = threadIdx.x & 31;
    const unsigned int warp = threadIdx.x >> 5;
    const unsigned int num_warps = blockDim.x >> 5;

    for (size_t row = warp; row < 4 * hidden_size; row += num_warps) {
        float sum = 0.0f;
        for (size_t col = lane; col < hidden_size; col += 32) {
            const float h = previous_hidden[hidden_offset + batch * hidden_stride_batch
                + col * hidden_stride_element];
            const float w = recurrent_weights[weight_offset + row * weight_stride_row
                + col * weight_stride_col];
            sum += h * w;
        }
        for (unsigned int offset = 16; offset > 0; offset >>= 1) {
            sum += __shfl_down_sync(0xffffffff, sum, offset);
        }
        if (lane == 0) {
            recurrent[row] = sum + recurrent_bias[bias_offset + row * bias_stride];
        }
    }
    __syncthreads();

    for (size_t hidden = threadIdx.x; hidden < hidden_size; hidden += blockDim.x) {
        const size_t gate_base = gate_offset + batch * gate_stride_batch;
        const size_t hidden_base = hidden_offset + batch * hidden_stride_batch;
        const size_t cell_base = cell_offset + batch * cell_stride_batch;
        const float input_gate = input_gates[gate_base + hidden * gate_stride_hidden]
            + recurrent[hidden];
        const float forget_gate = input_gates[gate_base + (hidden_size + hidden) * gate_stride_hidden]
            + recurrent[hidden_size + hidden];
        const float cell_gate = input_gates[gate_base + (2 * hidden_size + hidden) * gate_stride_hidden]
            + recurrent[2 * hidden_size + hidden];
        const float output_gate = input_gates[gate_base + (3 * hidden_size + hidden) * gate_stride_hidden]
            + recurrent[3 * hidden_size + hidden];
        const float i = 1.0f / (1.0f + expf(-input_gate));
        const float f = 1.0f / (1.0f + expf(-forget_gate));
        const float c = tanhf(cell_gate);
        const float o = 1.0f / (1.0f + expf(-output_gate));
        const float next_cell = f * previous_cell[cell_base + hidden * cell_stride_element] + i * c;
        const float next_hidden = o * tanhf(next_cell);
        output[batch * hidden_size + hidden] = next_hidden;
        output[batch_size * hidden_size + batch * hidden_size + hidden] = next_cell;
    }
}


extern "C" __global__ void kale_depthwise_convtranspose1d_f32(
    const float* input,
    const float* weights,
    const float* bias,
    float* output,
    size_t batches,
    size_t channels,
    size_t source_size,
    size_t output_size,
    size_t kernel_size,
    size_t padding,
    size_t stride,
    size_t dilation,
    size_t input_stride_batch,
    size_t input_stride_channel,
    size_t input_stride_sample,
    size_t input_offset,
    size_t weight_stride_channel,
    size_t weight_stride_sample,
    size_t weight_offset,
    size_t bias_stride,
    size_t bias_offset,
    size_t count
) {
    const size_t i = (size_t)blockIdx.x * blockDim.x + threadIdx.x;
    if (i >= count) return;
    const size_t out_sample = i % output_size;
    const size_t channel = (i / output_size) % channels;
    const size_t batch = i / (output_size * channels);
    float sum = bias[bias_offset + channel * bias_stride];
    for (size_t k = 0; k < kernel_size; ++k) {
        const long long numerator = (long long)(out_sample + padding)
            - (long long)(k * dilation);
        if (numerator >= 0 && numerator % (long long)stride == 0) {
            const size_t source_sample = (size_t)(numerator / (long long)stride);
            if (source_sample < source_size) {
                const size_t input_index = input_offset + batch * input_stride_batch
                    + channel * input_stride_channel + source_sample * input_stride_sample;
                const size_t weight_index = weight_offset + channel * weight_stride_channel
                    + k * weight_stride_sample;
                sum += input[input_index] * weights[weight_index];
            }
        }
    }
    output[i] = sum;
}

extern "C" __global__ void kale_lstm_sequence_f32(
    const float* input_gates, const float* recurrent_weights,
    const float* recurrent_bias, const float* initial_hidden,
    const float* initial_cell, float* output,
    size_t batch_size, size_t hidden_size,
    size_t gate_stride_batch, size_t gate_stride_hidden, size_t gate_offset,
    size_t weight_stride_row, size_t weight_stride_col, size_t weight_offset,
    size_t bias_stride, size_t bias_offset,
    size_t hidden_stride_batch, size_t hidden_stride_element, size_t hidden_offset,
    size_t cell_stride_batch, size_t cell_stride_element, size_t cell_offset,
    size_t steps, size_t gate_stride_step, size_t reverse
) {
    __shared__ float recurrent[4096];
    __shared__ float hidden[1024];
    __shared__ float cell[1024];
    const size_t batch = blockIdx.x;
    const size_t unit = threadIdx.x;
    const unsigned int lane = threadIdx.x & 31;
    const unsigned int warp = threadIdx.x >> 5;
    const unsigned int num_warps = blockDim.x >> 5;
    if (unit < hidden_size) {
        hidden[unit] = initial_hidden[hidden_offset + batch * hidden_stride_batch + unit * hidden_stride_element];
        cell[unit] = initial_cell[cell_offset + batch * cell_stride_batch + unit * cell_stride_element];
    }
    __syncthreads();
    for (size_t step = 0; step < steps; ++step) {
        const size_t t = reverse ? steps - 1 - step : step;
        for (size_t row = warp; row < 4 * hidden_size; row += num_warps) {
            float sum = 0.0f;
            for (size_t col = lane; col < hidden_size; col += 32) {
                const float w = recurrent_weights[weight_offset + row * weight_stride_row + col * weight_stride_col];
                sum += hidden[col] * w;
            }
            for (unsigned int offset = 16; offset > 0; offset >>= 1) {
                sum += __shfl_down_sync(0xffffffff, sum, offset);
            }
            if (lane == 0) {
                recurrent[row] = sum + recurrent_bias[bias_offset + row * bias_stride];
            }
        }
        __syncthreads();
        if (unit < hidden_size) {
            const size_t base = gate_offset + t * gate_stride_step + batch * gate_stride_batch;
            const float input_gate = input_gates[base + unit * gate_stride_hidden] + recurrent[unit];
            const float forget_gate = input_gates[base + (hidden_size + unit) * gate_stride_hidden] + recurrent[hidden_size + unit];
            const float cell_gate = input_gates[base + (2 * hidden_size + unit) * gate_stride_hidden] + recurrent[2 * hidden_size + unit];
            const float output_gate = input_gates[base + (3 * hidden_size + unit) * gate_stride_hidden] + recurrent[3 * hidden_size + unit];
            const float i = 1.0f / (1.0f + expf(-input_gate));
            const float f = 1.0f / (1.0f + expf(-forget_gate));
            const float c = tanhf(cell_gate);
            const float o = 1.0f / (1.0f + expf(-output_gate));
            const float next_cell = f * cell[unit] + i * c;
            const float next_hidden = o * tanhf(next_cell);
            hidden[unit] = next_hidden;
            cell[unit] = next_cell;
            output[(t * batch_size + batch) * hidden_size + unit] = next_hidden;
        }
        __syncthreads();
    }
    if (unit < hidden_size) {
        output[(steps * batch_size + batch) * hidden_size + unit] = cell[unit];
    }
}

#include <cooperative_groups.h>

// Distribute the recurrent matrix-vector product over several SMs. Each
// warp owns one unit and retains its cell state across timesteps.
extern "C" __global__ void kale_lstm_cooperative_f32(
    const float* input_gates, const float* recurrent_weights,
    const float* recurrent_bias, const float* initial_hidden,
    const float* initial_cell, float* output,
    size_t batch_size, size_t hidden_size,
    size_t gate_stride_batch, size_t gate_stride_hidden, size_t gate_offset,
    size_t weight_stride_row, size_t weight_stride_col, size_t weight_offset,
    size_t bias_stride, size_t bias_offset,
    size_t hidden_stride_batch, size_t hidden_stride_element, size_t hidden_offset,
    size_t cell_stride_batch, size_t cell_stride_element, size_t cell_offset,
    size_t steps, size_t gate_stride_step, size_t reverse
) {
    const auto grid = cooperative_groups::this_grid();
    const size_t batch = blockIdx.y;
    const size_t unit = blockIdx.x * (blockDim.x / 32) + threadIdx.x / 32;
    const unsigned int lane = threadIdx.x % 32;
    // Nano has 96 units, so each lane retains exactly three columns.
    float weights[3][4];
    float bias[4];
    #pragma unroll
    for (int gate = 0; gate < 4; ++gate) {
        const size_t row = gate * 96 + unit;
        #pragma unroll
        for (int segment = 0; segment < 3; ++segment) {
            const size_t col = lane + 32 * segment;
            weights[segment][gate] = recurrent_weights[weight_offset + row * weight_stride_row + col * weight_stride_col];
        }
        bias[gate] = lane == 0 ? recurrent_bias[bias_offset + row * bias_stride] : 0.0f;
    }
    float cell = (unit < hidden_size && lane == 0)
        ? initial_cell[cell_offset + batch * cell_stride_batch + unit * cell_stride_element]
        : 0.0f;

    for (size_t step = 0; step < steps; ++step) {
        const size_t t = reverse ? steps - 1 - step : step;
        if (unit < hidden_size) {
            float sums[4] = {};
            const size_t previous = reverse ? t + 1 : t - 1;
            #pragma unroll
            for (int segment = 0; segment < 3; ++segment) {
                const size_t col = lane + 32 * segment;
                const float h = step == 0
                    ? initial_hidden[hidden_offset + batch * hidden_stride_batch + col * hidden_stride_element]
                    : output[(previous * batch_size + batch) * hidden_size + col];
                #pragma unroll
                for (int gate = 0; gate < 4; ++gate) {
                    sums[gate] += h * weights[segment][gate];
                }
            }
            for (unsigned int delta = 16; delta > 0; delta >>= 1) {
                #pragma unroll
                for (int gate = 0; gate < 4; ++gate) {
                    sums[gate] += __shfl_down_sync(0xffffffff, sums[gate], delta);
                }
            }
            if (lane == 0) {
                const size_t base = gate_offset + t * gate_stride_step + batch * gate_stride_batch;
                #pragma unroll
                for (int gate = 0; gate < 4; ++gate) {
                    const size_t row = gate * hidden_size + unit;
                    sums[gate] = input_gates[base + row * gate_stride_hidden]
                        + (sums[gate] + bias[gate]);
                }
                const float i = 1.0f / (1.0f + expf(-sums[0]));
                const float f = 1.0f / (1.0f + expf(-sums[1]));
                const float c = tanhf(sums[2]);
                const float o = 1.0f / (1.0f + expf(-sums[3]));
                cell = __fmaf_rn(i, c, __fmul_rn(f, cell));
                output[(t * batch_size + batch) * hidden_size + unit] = o * tanhf(cell);
            }
        }
        grid.sync();
    }
    if (unit < hidden_size && lane == 0) {
        output[(steps * batch_size + batch) * hidden_size + unit] = cell;
    }
}

struct ElementwiseLayout {
    unsigned int dims[4];
    unsigned int left[4];
    unsigned int right[4];
};

// Direct launch parameters avoid uploading a separate array of strides for
// each broadcast, including scalar and per-channel broadcasts in the decoder.
extern "C" __global__ void kale_elementwise_f32(
    const float* left, const float* right, float* output,
    unsigned int count, unsigned int op, float alpha,
    unsigned int contiguous, ElementwiseLayout layout
) {
    const unsigned int i = blockIdx.x * blockDim.x + threadIdx.x;
    if (i >= count) return;
    unsigned int li = i;
    unsigned int ri = i;
    if (!contiguous) {
        li = 0;
        ri = 0;
        unsigned int remainder = i;
        #pragma unroll
        for (int axis = 3; axis >= 0; --axis) {
            unsigned int coord = remainder;
            if (axis != 0) {
                coord %= layout.dims[axis];
                remainder /= layout.dims[axis];
            }
            li += coord * layout.left[axis];
            ri += coord * layout.right[axis];
        }
    }
    const float a = left[li];
    const float b = right[ri];
    switch (op) {
        case 0: output[i] = a + b; break;
        case 1: output[i] = a - b; break;
        case 2: output[i] = a * b; break;
        case 3: output[i] = a / b; break;
        case 4: output[i] = __fadd_rn(fmaxf(a, 0.0f), __fmul_rn(fminf(a, 0.0f), alpha)); break;
    }
}

extern "C" __global__ void kale_range_i64(long long* output, unsigned int count, long long start, long long step) {
    const unsigned int i = blockIdx.x * blockDim.x + threadIdx.x;
    if (i < count) output[i] = start + i * step;
}

extern "C" __global__ void kale_range_f32(float* output, unsigned int count, long long start, long long step) {
    const unsigned int i = blockIdx.x * blockDim.x + threadIdx.x;
    if (i < count) output[i] = (float)(start + i * step);
}

extern "C" __global__ void kale_reflect_pad_f32(
    const float* input, float* output, unsigned int count,
    size_t length, size_t left, size_t stride
) {
    const size_t i = (size_t)blockIdx.x * blockDim.x + threadIdx.x;
    if (i >= count) return;
    size_t source = i < left ? left - i : i - left;
    if (source >= length) source = 2 * (length - 1) - source;
    output[i] = input[source * stride];
}
