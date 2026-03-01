# Block 3 Implementation Plan

## Current State

Block 3 has a working **pipeline** and **GUI**, but the **shader is incomplete**. It currently:
- ✅ Samples Block 1 and Block 2 textures
- ✅ Matrix mixes them
- ✅ Final mix with keying

**Missing:** Re-processing transforms on Block 1 and Block 2 before mixing.

## What's Missing

### Block 1 Re-processing (before matrix mix)
The shader uniforms exist but aren't used:
- Geometric: xy_displace, z_displace, rotate, shear_matrix, kaleidoscope, h/v_mirror, h/v_flip
- Filters: blur_amount, blur_radius, sharpen_amount, sharpen_radius, filters_boost
- Colorize: bands 1-5, colorize_mode, switches
- Dither: dither_amount, dither_type, switches

### Block 2 Re-processing (before matrix mix)
Same features as Block 1 re-processing.

## Implementation Strategy

### Option 1: Monolithic Shader (Quick)
Add all re-processing functions directly into the existing single shader.

**Pros:**
- Fast to implement
- No architectural changes

**Cons:**
- Shader gets large (~500+ lines)
- Harder to debug
- Always runs all re-processing even when not needed

### Option 2: Modular 3-Stage (Recommended)
Follow Block 1/2 pattern with separate stages:

```
Stage 1a: Block 1 Re-process (transforms + filters)
Stage 1b: Block 2 Re-process (transforms + filters)  
Stage 2:  Matrix Mixer + Final Mix
```

**Pros:**
- Can skip re-processing when not needed (performance)
- Debug each stage independently
- Consistent with Block 1/2 architecture

**Cons:**
- More code to write
- More complex resource management

## Recommended Approach: Modular

### Stage 1a: Block 1 Re-process
**Input:** `block1_texture`  
**Output:** `block1_processed_texture`

**Features:**
- Geometric transforms (displace, scale, rotate, shear, kaleidoscope, mirrors, flips)
- Blur/sharpen filters
- Colorize (5-band gradient mapping)
- Dither

### Stage 1b: Block 2 Re-process  
**Input:** `block2_texture`  
**Output:** `block2_processed_texture`

Same features as Stage 1a.

### Stage 2: Matrix Mix + Final Mix
**Inputs:** `block1_processed`, `block2_processed`  
**Output:** Surface

**Features:**
- Matrix mixer (existing code)
- Final mix with keying (existing code)

## Implementation Checklist

### Stage 1a & 1b Shaders
- [ ] Vertex shader (standard full-screen quad)
- [ ] Fragment shader with transforms
  - [ ] UV coordinate transforms (scale, rotate, displace, shear)
  - [ ] Kaleidoscope effect
  - [ ] Mirrors and flips
  - [ ] Blur (box or gaussian)
  - [ ] Sharpen (Laplacian)
  - [ ] Colorize (gradient mapping)
  - [ ] Dither (Bayer matrix)
- [ ] Uniform buffer layout matching Block3Params
- [ ] Bind group setup

### Stage 2 Shader (Existing)
- [ ] Update to use processed textures
- [ ] Keep matrix mixer
- [ ] Keep final mix + keying

### Resource Management
- [ ] Create `Block3Resources` with ping-pong buffers
- [ ] Stage 1a/1b output textures (or reuse Block 1/2 texture slots)
- [ ] Proper format handling (Rgba8Unorm)

### Pipeline Integration
- [ ] Create `ModularBlock3` struct
- [ ] Render method with stage chaining
- [ ] Conditional rendering (skip stages when not needed)

### Engine Integration
- [ ] Replace `Block3Pipeline` with `ModularBlock3`
- [ ] Update `WgpuEngine` to use new block
- [ ] Texture binding management

## Key Differences from Block 1/2

1. **No Feedback Loop:** Block 3 doesn't have FB3 - it's purely feed-forward
2. **Two Parallel Inputs:** Block 1 and Block 2 are processed independently
3. **Output to Surface:** Block 3 renders directly to output (not intermediate texture)
4. **Simpler Resource Management:** No delay buffers needed

## Testing Plan

### Stage 1 (Re-process)
- [ ] Block 1 displace moves texture
- [ ] Block 1 rotate works
- [ ] Block 1 blur softens image
- [ ] Block 1 sharpen enhances edges
- [ ] Block 1 kaleidoscope creates symmetry
- [ ] Block 1 colorize applies gradient
- [ ] Same for Block 2

### Stage 2 (Mixing)
- [ ] Matrix mixer blends colors (existing)
- [ ] Final mix amount controls blend (existing)
- [ ] Final keying works (existing)

## Reference Code

### Block 1 Blur Pattern
```wgsl
fn apply_blur(uv: vec2<f32>, amount: f32, radius: f32) -> vec3<f32> {
    let texel_size = vec2<f32>(1.0 / uniforms.width, 1.0 / uniforms.height);
    let blur_radius = radius * 5.0;
    
    var sum = vec3<f32>(0.0);
    var total_weight = 0.0;
    
    for (var x: i32 = -2; x <= 2; x = x + 1) {
        for (var y: i32 = -2; y <= 2; y = y + 1) {
            let offset = vec2<f32>(f32(x), f32(y)) * texel_size * blur_radius;
            let weight = 1.0 - length(vec2<f32>(f32(x), f32(y))) * 0.2;
            sum = sum + textureSample(input_tex, input_sampler, uv + offset).rgb * weight;
            total_weight = total_weight + weight;
        }
    }
    
    return mix(textureSample(input_tex, input_sampler, uv).rgb, sum / total_weight, amount);
}
```

### Transform Pattern
```wgsl
fn transform_uv(uv: vec2<f32>, displace: vec2<f32>, scale: f32, rotate: f32, shear: vec4<f32>) -> vec2<f32> {
    var result = uv - vec2<f32>(0.5);
    
    // Scale
    if (scale > 0.001) {
        result = result / scale;
    }
    
    // Rotate
    if (abs(rotate) > 0.001) {
        let angle = radians(rotate);
        let cos_r = cos(angle);
        let sin_r = sin(angle);
        let rot_x = result.x * cos_r - result.y * sin_r;
        let rot_y = result.x * sin_r + result.y * cos_r;
        result = vec2<f32>(rot_x, rot_y);
    }
    
    // Shear
    result = vec2<f32>(
        result.x * shear.x + result.y * shear.y,
        result.x * shear.z + result.y * shear.w
    );
    
    // Displace
    result = result + vec2<f32>(0.5) + displace;
    
    return result;
}
```

## Estimated Timeline

| Task | Time |
|------|------|
| Stage 1a shader (Block 1 re-process) | 2-3 hrs |
| Stage 1b shader (Block 2 re-process) | 1 hr (copy of 1a) |
| Stage 2 shader update | 30 min |
| ModularBlock3 struct | 1-2 hrs |
| Engine integration | 1 hr |
| Testing & debugging | 2-3 hrs |
| **Total** | **8-12 hrs** |

## Files to Modify

1. **New:** `src/engine/blocks/block3.rs` - Modular Block 3 implementation
2. **Update:** `src/engine/blocks/mod.rs` - Export ModularBlock3
3. **Update:** `src/engine/mod.rs` - Replace Block3Pipeline with ModularBlock3
4. **Optional:** `src/engine/pipelines/block3.rs` - Can be removed after migration
