// Fullscreen triangle vertex shader
// Outputs UVs in [0, 1] range

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    // Generate fullscreen triangle
    // Position: (-1, -1), (3, -1), (-1, 3)
    // UV: (0, 1), (2, 1), (0, -1)
    var out: VertexOutput;
    
    let x = f32(vertex_index % 2u) * 4.0 - 1.0; // 0 -> -1, 1 -> 3
    let y = f32(vertex_index / 2u) * 4.0 - 1.0; // 0 -> -1, 1 -> 3
    
    out.position = vec4<f32>(x, y, 0.0, 1.0);
    out.uv = vec2<f32>(
        (x + 1.0) * 0.5,  // -1->1 to 0->1
        (1.0 - y) * 0.5   // Flip Y for texture coords
    );
    
    return out;
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@group(0) @binding(0)
var source_texture: texture_2d<f32>;

@group(0) @binding(1)
var source_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(source_texture, source_sampler, in.uv);
}
