# Block 2 Implementation Plan

## Overview

Block 2 processes a secondary input (Block 1 output, Input 1, or Input 2) with its own feedback loop (FB2). Following the Block 1 modular architecture, Block 2 will use a 3-stage pipeline.

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        BLOCK 2 PIPELINE                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  STAGE 1: Input Sampling                                         │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │ Select input based on block2_input_select:               │    │
│  │   0 = Block 1 output                                     │    │
│  │   1 = Input 1 (camera/media)                             │    │
│  │   2 = Input 2 (camera/media)                             │    │
│  │ Apply geometric transforms:                              │    │
│  │   - X/Y/Z displace                                       │    │
│  │   - Rotate                                               │    │
│  │   - Kaleidoscope                                         │    │
│  │   - H/V mirrors and flips                                │    │
│  │ Output: Transformed input                                │    │
│  └─────────────────────────────────────────────────────────┘    │
│                              ↓                                   │
│  STAGE 2: Effects Processing (Optional)                          │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │ Apply color/filter effects:                              │    │
│  │   - HSB attenuate                                        │    │
│  │   - Blur                                                 │    │
│  │   - Sharpen                                              │    │
│  │   - Posterize                                            │    │
│  │   - Solarize                                             │    │
│  │   - Hue/Bright/Sat inverts                               │    │
│  │ SKIPPED if no effects enabled                            │    │
│  │ Output: Processed input                                  │    │
│  └─────────────────────────────────────────────────────────┘    │
│                              ↓                                   │
│  STAGE 3: Mixing & FB2 Feedback                                  │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │ Mix processed input with FB2:                            │    │
│  │   - Mix amount (0-1)                                     │    │
│  │   - Mix types: lerp, add, diff, mult, dodge              │    │
│  │   - Keying on input                                      │    │
│  │                                                          │    │
│  │ FB2 transforms (applied to feedback UV):                 │    │
│  │   - X/Y/Z displace                                       │    │
│  │   - Rotate                                               │    │
│  │   - Kaleidoscope                                         │    │
│  │   - H/V mirrors                                          │    │
│  │   - Shear matrix                                         │    │
│  │                                                          │    │
│  │ FB2 color adjustments:                                   │    │
│  │   - HSB offset/attenuate/powmap                          │    │
│  │   - Hue shaper                                           │    │
│  │   - Posterize                                            │    │
│  │   - Inverts                                              │    │
│  │                                                          │    │
│  │ FB2 filters:                                             │    │
│  │   - Blur                                                 │    │
│  │   - Sharpen                                              │    │
│  │                                                          │    │
│  │ Delay: 0-120 frames (0 = immediate feedback)             │    │
│  │ Output: Final Block 2 result                             │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

## Key Differences from Block 1

| Aspect | Block 1 | Block 2 |
|--------|---------|---------|
| **Inputs** | 2 channels (CH1 + CH2) | 1 input (selectable source) |
| **Feedback** | FB1 | FB2 |
| **Channel mixing** | CH1 + CH2 mix | Input + FB2 mix |
| **Separate per-channel effects** | Yes (CH1 and CH2 independent) | No (single input chain) |
| **Keying** | Separate CH2 and FB1 keying | Input keying only |

## Implementation Tasks

### Phase 1: Create Modular Block 2 Structure

**Files to create/modify:**
- `src/engine/blocks/block2.rs` - New modular Block 2 implementation
- `src/engine/blocks/mod.rs` - Export ModularBlock2
- `src/engine/mod.rs` - Replace Block2Pipeline with ModularBlock2

**Components needed:**

1. **Stage 1 Pipeline** - Input sampling with transforms
   - One uniform buffer (not two like Block 1)
   - WGSL shader with input selection logic
   - Bind group with input textures (Block1, Input1, Input2)

2. **Stage 2 Pipeline** - Effects processing
   - One uniform buffer
   - WGSL shader with blur, sharpen, HSB, etc.
   - Conditional rendering (skip if no effects)

3. **Stage 3 Pipeline** - Mixing & FB2
   - One uniform buffer
   - WGSL shader with mixn_key_video function
   - FB2 transforms and color adjustments
   - Delay buffer sampling

4. **BlockResources** - Reuse existing struct
   - `buffer_a` / `buffer_b` - Ping-pong for Stage 1/2
   - `feedback` - FB2 texture
   - `delay_buffers` - Ring buffer for delay

### Phase 2: Uniform Buffer Layout (CRITICAL)

Based on Block 1 lessons, use `[f32; 3]` with explicit padding for vec3 fields:

```rust
#[repr(C, align(16))]
struct Block2Stage3Uniforms {
    // Mix params
    mix_amount: f32,
    mix_type: i32,
    mix_overflow: i32,
    _pad0: f32,
    
    // Input keying
    key_value_r: f32,
    key_value_g: f32,
    key_value_b: f32,
    key_threshold: f32,
    key_soft: f32,
    key_mode: i32,
    key_order: i32,
    
    // FB2 Color (use [f32; 3] not Vec3!)
    _pad_before_hsb: [f32; 2],      // Align to 16 bytes
    fb2_hsb_offset: [f32; 3],
    _pad_after_hsb_offset: f32,
    fb2_hsb_attenuate: [f32; 3],
    _pad_after_hsb_attenuate: f32,
    fb2_hsb_powmap: [f32; 3],
    
    fb2_hue_shaper: f32,
    fb2_posterize: f32,
    fb2_posterize_switch: i32,
    
    // FB2 Inverts
    fb2_hue_invert: f32,
    fb2_saturation_invert: f32,
    fb2_bright_invert: f32,
    _pad1: f32,
    
    // FB2 Geometric
    fb2_x_displace: f32,
    fb2_y_displace: f32,
    fb2_z_displace: f32,
    fb2_rotate: f32,
    fb2_kaleidoscope_amount: f32,
    fb2_kaleidoscope_slice: f32,
    fb2_h_mirror: f32,
    fb2_v_mirror: f32,
    
    // FB2 Filters
    fb2_blur_amount: f32,
    fb2_blur_radius: f32,
    fb2_sharpen_amount: f32,
    fb2_sharpen_radius: f32,
    fb2_filters_boost: f32,
    _pad2: f32,
    _pad3: f32,
    _pad4: f32,
    
    // FB2 Shear matrix (vec4)
    fb2_shear_matrix: [f32; 4],
    
    // Delay
    fb2_delay_time: i32,
    fb2_rotate_mode: i32,
    fb2_geo_overflow: i32,
    _pad5: i32,
}
```

**VERIFY OFFSETS** using the offset_of function before running!

### Phase 3: WGSL Shaders

**Stage 1 Shader** (`block2_stage1.wgsl` inline):
- Input selection based on `block2_input_select` uniform
- Geometric transforms (same as Block 1 Stage 1 but for single input)
- Output to buffer

**Stage 2 Shader** (`block2_stage2.wgsl` inline):
- Effects processing (same pattern as Block 1 Stage 2)
- Early exit if no effects enabled

**Stage 3 Shader** (`block2_stage3.wgsl` inline):
- Mix input with FB2
- Keying support
- FB2 transforms (UV-based like Block 1)
- FB2 color adjustments
- FB2 filters
- Delay buffer sampling

### Phase 4: Integration

**In `WgpuEngine`:**

1. Replace `Block2Pipeline` with `ModularBlock2`
2. Update render loop:
   ```rust
   // Current (old pipeline):
   self.block2_pipeline.update_textures(...);
   // render block2...
   
   // New (modular):
   self.modular_block2.render(
       &mut encoder,
       block2_input_view,  // Selected input
       &self.fb2_texture.view,
       &modulated_block2,
   );
   ```

3. Update feedback copy:
   ```rust
   // Current:
   encoder.copy_texture_to_texture(
       block2_texture → fb2_texture
   );
   
   // New:
   self.modular_block2.resources.update_feedback(&mut encoder);
   self.modular_block2.resources.update_delay_buffer(&mut encoder);
   ```

### Phase 5: Testing Checklist

**Input Selection:**
- [ ] Block 2 with Block 1 as input (default)
- [ ] Block 2 with Input 1 as input
- [ ] Block 2 with Input 2 as input
- [ ] Switch between inputs at runtime

**Stage 1 Transforms:**
- [ ] X/Y Displace
- [ ] Z Displace (scale)
- [ ] Rotate
- [ ] Kaleidoscope
- [ ] H/V mirrors
- [ ] H/V flips

**Stage 2 Effects:**
- [ ] HSB attenuate
- [ ] Blur
- [ ] Sharpen
- [ ] Posterize
- [ ] Solarize
- [ ] Inverts

**Stage 3 Mixing:**
- [ ] Mix amount 0-100%
- [ ] Mix types (lerp, add, diff, mult, dodge)
- [ ] Keying threshold
- [ ] Key soft

**FB2 Transforms:**
- [ ] X/Y/Z displace
- [ ] Rotate
- [ ] Kaleidoscope
- [ ] Mirrors
- [ ] Shear matrix

**FB2 Color:**
- [ ] HSB offset
- [ ] HSB attenuate
- [ ] HSB powmap
- [ ] Hue shaper
- [ ] Posterize
- [ ] Inverts

**FB2 Filters:**
- [ ] Blur
- [ ] Sharpen
- [ ] Filters boost

**Delay:**
- [ ] Delay 0 (immediate)
- [ ] Delay 1-120 frames
- [ ] Delay with mix

**LFO Integration:**
- [ ] All LFO-assigned parameters respond correctly
- [ ] Waveform selection works

## Critical Lessons from Block 1

1. **Uniform Buffer Layout**: Use `[f32; 3]` with explicit padding for WGSL vec3
2. **Single Buffer for Single Channel**: Block 2 only needs one buffer per stage (not two like Block 1's CH1/CH2)
3. **Input Selection**: Do in shader (Stage 1) based on uniform, not CPU-side
4. **Verify Offsets**: Always run layout verification before testing
5. **Keying Logic**: Only apply when threshold < 0.999

## Timeline Estimate

| Phase | Task | Estimated Time |
|-------|------|----------------|
| 1 | Create modular structure | 2-3 hours |
| 2 | Define uniform buffers | 1-2 hours |
| 3 | Write WGSL shaders | 3-4 hours |
| 4 | Engine integration | 2-3 hours |
| 5 | Testing & debugging | 2-3 hours |
| **Total** | | **10-15 hours** |

## Reference Files

- `src/engine/blocks/block1.rs` - Copy structure from here
- `src/engine/pipelines/block2.rs` - Reference for current Block 2 logic
- `BLOCK1_IMPLEMENTATION_GUIDE.md` - Critical bugs to avoid
- `BLOCK1_STATUS.md` - Testing patterns

## Next Steps

1. Create `src/engine/blocks/block2.rs` with modular structure
2. Copy Stage 1/2/3 patterns from Block 1, adapt for single input
3. Define uniform structs with proper padding
4. Write WGSL shaders
5. Integrate into engine
6. Test incrementally (Stage 1 → Stage 2 → Stage 3)
