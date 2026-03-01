//! # Modular Shader Stages
//!
//! Three-stage shader architecture for each block:
//! 1. Input Sampling - Transform UVs and sample textures
//! 2. Effects Processing - HSB, blur, filters (optional)
//! 3. Mixing - Combine inputs and feedback

use std::sync::Arc;

/// Common vertex shader for all stages
pub const COMMON_VERTEX_SHADER: &str = r#"
struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) texcoord: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) texcoord: vec2<f32>,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.position = vec4<f32>(input.position, 0.0, 1.0);
    output.texcoord = input.texcoord;
    return output;
}
"#;

/// Stage 1: Input Sampling Shader
/// Transforms UV coordinates and samples all input textures
pub const STAGE1_INPUT_SAMPLING: &str = r#"
// Uniforms for input sampling stage
struct Stage1Uniforms {
    // Resolution
    width: f32,
    height: f32,
    
    // Input texture dimensions
    input1_width: f32,
    input1_height: f32,
    input2_width: f32,
    input2_height: f32,
    fb_width: f32,
    fb_height: f32,
    
    // Which inputs to sample (1.0 = sample, 0.0 = skip)
    sample_input1: f32,
    sample_input2: f32,
    sample_feedback: f32,
    
    // Input 1 transforms
    in1_scale: f32,
    in1_rotate: f32,
    in1_x_displace: f32,
    in1_y_displace: f32,
    in1_aspect: f32,
    
    // Input 2 transforms
    in2_scale: f32,
    in2_rotate: f32,
    in2_x_displace: f32,
    in2_y_displace: f32,
    in2_aspect: f32,
    
    // Feedback transforms
    fb_scale: f32,
    fb_rotate: f32,
    fb_x_displace: f32,
    fb_y_displace: f32,
    fb_aspect: f32,
    
    // Padding to 16-byte alignment
    _pad: f32,
};

@group(0) @binding(0)
var<uniform> uniforms: Stage1Uniforms;

// Input textures
@group(0) @binding(1)
var input1_tex: texture_2d<f32>;
@group(0) @binding(2)
var input1_sampler: sampler;

@group(0) @binding(3)
var input2_tex: texture_2d<f32>;
@group(0) @binding(4)
var input2_sampler: sampler;

@group(0) @binding(5)
var fb_tex: texture_2d<f32>;
@group(0) @binding(6)
var fb_sampler: sampler;

// Transform UV coordinates
fn transform_uv(uv: vec2<f32>, scale: f32, rotate: f32, displace: vec2<f32>) -> vec2<f32> {
    // Center
    var result = uv - vec2<f32>(0.5);
    
    // Rotate
    let cos_r = cos(rotate);
    let sin_r = sin(rotate);
    let rot_x = result.x * cos_r - result.y * sin_r;
    let rot_y = result.x * sin_r + result.y * cos_r;
    result = vec2<f32>(rot_x, rot_y);
    
    // Scale
    result = result / scale;
    
    // Uncenter and displace
    result = result + vec2<f32>(0.5) + displace;
    
    return result;
}

// Sample texture with transform, return black if out of bounds
fn sample_transformed(tex: texture_2d<f32>, tex_sampler: sampler, 
                      uv: vec2<f32>, scale: f32, rotate: f32, 
                      displace: vec2<f32>) -> vec4<f32> {
    let transformed_uv = transform_uv(uv, scale, rotate, displace);
    
    // Check bounds
    if (transformed_uv.x < 0.0 || transformed_uv.x > 1.0 ||
        transformed_uv.y < 0.0 || transformed_uv.y > 1.0) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }
    
    return textureSample(tex, tex_sampler, transformed_uv);
}

@fragment
fn fs_main(@location(0) texcoord: vec2<f32>) -> @location(0) vec4<f32> {
    // Sample Input 1 (always needed for Block 1)
    let input1_color = sample_transformed(
        input1_tex, input1_sampler, texcoord,
        uniforms.in1_scale, uniforms.in1_rotate,
        vec2<f32>(uniforms.in1_x_displace, uniforms.in1_y_displace)
    );
    
    // Sample Input 2 (if enabled)
    var input2_color = vec4<f32>(0.0);
    if (uniforms.sample_input2 > 0.5) {
        input2_color = sample_transformed(
            input2_tex, input2_sampler, texcoord,
            uniforms.in2_scale, uniforms.in2_rotate,
            vec2<f32>(uniforms.in2_x_displace, uniforms.in2_y_displace)
        );
    }
    
    // Sample Feedback (if enabled)
    var fb_color = vec4<f32>(0.0);
    if (uniforms.sample_feedback > 0.5) {
        fb_color = sample_transformed(
            fb_tex, fb_sampler, texcoord,
            uniforms.fb_scale, uniforms.fb_rotate,
            vec2<f32>(uniforms.fb_x_displace, uniforms.fb_y_displace)
        );
    }
    
    // Pack all samples into output for next stages
    // R = Input 1 R, G = Input 1 G, B = Input 1 B, A = Input 2 R
    // (We need multiple outputs or multiple passes for all samples)
    // For now, just show Input 1
    return input1_color;
}
"#;

/// Stage 2: Effects Processing Shader
/// Applies HSB adjustments, blur, filters
pub const STAGE2_EFFECTS: &str = r#"
struct Stage2Uniforms {
    // HSB adjustments
    hsb_hue_attenuate: f32,
    hsb_sat_attenuate: f32,
    hsb_bright_attenuate: f32,
    
    // Blur/Sharpen
    blur_amount: f32,
    blur_radius: f32,
    sharpen_amount: f32,
    sharpen_radius: f32,
    
    // Switches (packed as bits)
    switches: u32,  // bit 0=hue_invert, 1=sat_invert, 2=bright_invert, 3=posterize, 4=solarize
    
    // Posterize
    posterize_levels: f32,
    posterize_inv: f32,
    
    // Resolution for blur sampling
    width: f32,
    height: f32,
    
    _pad: f32,
};

@group(0) @binding(0)
var<uniform> uniforms: Stage2Uniforms;

// Input from Stage 1
@group(0) @binding(1)
var input_tex: texture_2d<f32>;
@group(0) @binding(2)
var input_sampler: sampler;

// RGB to HSB conversion
fn rgb2hsb(c: vec3<f32>) -> vec3<f32> {
    let K = vec4<f32>(0.0, -1.0 / 3.0, 2.0 / 3.0, -1.0);
    let p = mix(vec4<f32>(c.bg, K.wz), vec4<f32>(c.gb, K.xy), step(c.b, c.g));
    let q = mix(vec4<f32>(p.xyw, c.r), vec4<f32>(c.r, p.yzx), step(p.x, c.r));
    let d = q.x - min(q.w, q.y);
    let e = 1.0e-10;
    return vec3<f32>(abs(q.z + (q.w - q.y) / (6.0 * d + e)), d / (q.x + e), q.x);
}

// HSB to RGB conversion  
fn hsb2rgb(c: vec3<f32>) -> vec3<f32> {
    let K = vec4<f32>(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    let p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, clamp(p - K.xxx, vec3<f32>(0.0), vec3<f32>(1.0)), c.y);
}

fn get_switch(switches: u32, bit: u32) -> bool {
    return (switches & (1u << bit)) != 0u;
}

// Box blur
fn blur(tex: texture_2d<f32>, tex_sampler: sampler, uv: vec2<f32>, radius: f32) -> vec3<f32> {
    let texel = vec2<f32>(radius) / vec2<f32>(uniforms.width, uniforms.height);
    
    var result = vec3<f32>(0.0);
    result += textureSample(tex, tex_sampler, uv + texel * vec2<f32>(-1.0, -1.0)).rgb;
    result += textureSample(tex, tex_sampler, uv + texel * vec2<f32>( 0.0, -1.0)).rgb;
    result += textureSample(tex, tex_sampler, uv + texel * vec2<f32>( 1.0, -1.0)).rgb;
    result += textureSample(tex, tex_sampler, uv + texel * vec2<f32>(-1.0,  0.0)).rgb;
    result += textureSample(tex, tex_sampler, uv + texel * vec2<f32>( 0.0,  0.0)).rgb;
    result += textureSample(tex, tex_sampler, uv + texel * vec2<f32>( 1.0,  0.0)).rgb;
    result += textureSample(tex, tex_sampler, uv + texel * vec2<f32>(-1.0,  1.0)).rgb;
    result += textureSample(tex, tex_sampler, uv + texel * vec2<f32>( 0.0,  1.0)).rgb;
    result += textureSample(tex, tex_sampler, uv + texel * vec2<f32>( 1.0,  1.0)).rgb;
    
    return result / 9.0;
}

@fragment
fn fs_main(@location(0) texcoord: vec2<f32>) -> @location(0) vec4<f32> {
    let input_color = textureSample(input_tex, input_sampler, texcoord);
    
    // Check if any processing needed
    let needs_blur = uniforms.blur_amount > 0.001;
    let needs_hsb = uniforms.hsb_hue_attenuate != 1.0 || 
                    uniforms.hsb_sat_attenuate != 1.0 || 
                    uniforms.hsb_bright_attenuate != 1.0 ||
                    get_switch(uniforms.switches, 0u) ||  // hue_invert
                    get_switch(uniforms.switches, 1u) ||  // sat_invert  
                    get_switch(uniforms.switches, 2u);    // bright_invert
    let needs_posterize = get_switch(uniforms.switches, 3u);
    
    // Early exit: no processing needed
    if (!needs_blur && !needs_hsb && !needs_posterize) {
        return input_color;
    }
    
    var color = input_color.rgb;
    
    // Blur
    if (needs_blur) {
        let blurred = blur(input_tex, input_sampler, texcoord, uniforms.blur_radius);
        color = mix(color, blurred, uniforms.blur_amount);
    }
    
    // HSB Processing
    if (needs_hsb) {
        var hsb = rgb2hsb(color);
        
        // Attenuate
        hsb.x = pow(hsb.x, uniforms.hsb_hue_attenuate);
        hsb.y = pow(hsb.y, uniforms.hsb_sat_attenuate);
        hsb.z = pow(hsb.z, uniforms.hsb_bright_attenuate);
        
        // Inverts
        if (get_switch(uniforms.switches, 0u)) { hsb.x = 1.0 - hsb.x; }
        if (get_switch(uniforms.switches, 1u)) { hsb.y = 1.0 - hsb.y; }
        if (get_switch(uniforms.switches, 2u)) { hsb.z = 1.0 - hsb.z; }
        
        hsb.x = fract(hsb.x);
        color = hsb2rgb(hsb);
    }
    
    // Posterize
    if (needs_posterize) {
        color = floor(color * uniforms.posterize_levels) * uniforms.posterize_inv;
    }
    
    return vec4<f32>(color, input_color.a);
}
"#;

/// Stage 3: Mixing Shader
/// Combines inputs with feedback using blend modes
pub const STAGE3_MIXING: &str = r#"
struct Stage3Uniforms {
    // Mix amounts
    input2_amount: f32,
    feedback_amount: f32,
    
    // Mix types (0=lerp, 1=add, 2=diff, 3=mult, 4=dodge)
    input2_mix_type: i32,
    feedback_mix_type: i32,
    
    // Keying
    key_threshold: f32,
    key_soft: f32,
    key_r: f32,
    key_g: f32,
    key_b: f32,
    
    _pad: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> uniforms: Stage3Uniforms;

// Inputs from previous stages
@group(0) @binding(1)
var input1_tex: texture_2d<f32>;  // Input 1 processed
@group(0) @binding(2)
var input1_sampler: sampler;

@group(0) @binding(3)
var input2_tex: texture_2d<f32>;  // Input 2 processed
@group(0) @binding(4)
var input2_sampler: sampler;

@group(0) @binding(5)
var fb_tex: texture_2d<f32>;        // Feedback processed
@group(0) @binding(6)
var fb_sampler: sampler;

// Mix two colors with blend mode
fn mix_colors(a: vec3<f32>, b: vec3<f32>, amount: f32, mix_type: i32) -> vec3<f32> {
    switch mix_type {
        case 0: { // Lerp
            return mix(a, b, amount);
        }
        case 1: { // Add
            return a + amount * b;
        }
        case 2: { // Difference
            return abs(a - amount * b);
        }
        case 3: { // Multiply
            return mix(a, a * b, amount);
        }
        case 4: { // Dodge
            return mix(a, a / (1.00001 - b), amount);
        }
        default: {
            return mix(a, b, amount);
        }
    }
}

@fragment
fn fs_main(@location(0) texcoord: vec2<f32>) -> @location(0) vec4<f32> {
    let input1 = textureSample(input1_tex, input1_sampler, texcoord).rgb;
    
    // Mix with Input 2
    var mixed = input1;
    if (uniforms.input2_amount > 0.001) {
        let input2 = textureSample(input2_tex, input2_sampler, texcoord).rgb;
        mixed = mix_colors(input1, input2, uniforms.input2_amount, uniforms.input2_mix_type);
    }
    
    // Keying (simple chroma key)
    let key_color = vec3<f32>(uniforms.key_r, uniforms.key_g, uniforms.key_b);
    let key_dist = distance(mixed, key_color);
    if (key_dist < uniforms.key_threshold) {
        let alpha = smoothstep(uniforms.key_threshold - uniforms.key_soft, 
                               uniforms.key_threshold, key_dist);
        mixed = mix(key_color, mixed, alpha);
    }
    
    // Mix with Feedback
    var final_color = mixed;
    if (uniforms.feedback_amount > 0.001) {
        let fb = textureSample(fb_tex, fb_sampler, texcoord).rgb;
        final_color = mix_colors(mixed, fb, uniforms.feedback_amount, uniforms.feedback_mix_type);
    }
    
    return vec4<f32>(final_color, 1.0);
}
"#;

/// Create a render pipeline from WGSL shader code
pub fn create_stage_pipeline(
    device: &wgpu::Device,
    shader_code: &str,
    format: wgpu::TextureFormat,
    label: &str,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(label),
        source: wgpu::ShaderSource::Wgsl(shader_code.into()),
    });
    
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some(&format!("{} Layout", label)),
        bind_group_layouts: &[], // Will be set per-stage
        push_constant_ranges: &[],
    });
    
    // This is a placeholder - actual implementation will need proper bind group layouts
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(label),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[], // Will add vertex buffer layout
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        cache: None,
    })
}
