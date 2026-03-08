// BGRA to RGBA conversion compute shader
// Processes pixels in parallel using GPU compute

struct Uniforms {
    width: u32,
    height: u32,
    stride: u32,
    _padding: u32,
};

@group(0) @binding(0)
var<storage, read> input_buffer: array<u8>;

@group(0) @binding(1)
var<storage, read_write> output_buffer: array<u8>;

@group(0) @binding(2)
var<uniform> uniforms: Uniforms;

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let x = global_id.x;
    let y = global_id.y;
    
    // Bounds check
    if (x >= uniforms.width || y >= uniforms.height) {
        return;
    }
    
    // Calculate source and destination indices
    let src_idx = y * uniforms.stride + x * 4u;
    let dst_idx = (y * uniforms.width + x) * 4u;
    
    // Bounds check for buffer access
    if (src_idx + 3u >= arrayLength(&input_buffer) || 
        dst_idx + 3u >= arrayLength(&output_buffer)) {
        return;
    }
    
    // BGRA -> RGBA conversion
    // Input: B G R A
    // Output: R G B A
    output_buffer[dst_idx + 0u] = input_buffer[src_idx + 2u]; // R <- B
    output_buffer[dst_idx + 1u] = input_buffer[src_idx + 1u]; // G <- G
    output_buffer[dst_idx + 2u] = input_buffer[src_idx + 0u]; // B <- R
    output_buffer[dst_idx + 3u] = input_buffer[src_idx + 3u]; // A <- A
}
