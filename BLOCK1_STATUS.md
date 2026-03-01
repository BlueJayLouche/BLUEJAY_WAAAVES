# Block 1 Implementation Status

## Overview

Block 1 is the primary channel mixing and feedback block in the RustJay Waaaves application. It uses a 3-stage modular architecture:
1. **Stage 1**: Input sampling with geometric transforms
2. **Stage 2**: Effects processing (HSB, blur, sharpen, posterize)
3. **Stage 3**: Mixing with keying and feedback

## Critical Bug Fixed: Shared Uniform Buffers

### The Problem
Both CH1 and CH2 were sharing the **same uniform buffer** for each stage:

```
Stage 1: Single `stage1_uniforms` buffer
  CH1 writes params → buffer
  CH2 writes params → buffer (OVERWRITES CH1!)
  CH1 renders using buffer (now has CH2 data!)
  CH2 renders using buffer

Result: CH1 shows CH2's transforms, CH2 works correctly
```

### The Solution
Created **separate uniform buffers** for each channel:

```
Stage 1: `stage1_uniforms_ch1` + `stage1_uniforms_ch1`
  CH1 writes params → stage1_uniforms_ch1
  CH1 renders using stage1_uniforms_ch1 ✓
  CH2 writes params → stage1_uniforms_ch2
  CH2 renders using stage1_uniforms_ch2 ✓

Result: Each channel uses its own parameters
```

Same fix applied to Stage 2 for color effects.

## Working Features ✅

### Input System
- ✅ Input 1 and Input 2 texture upload to GPU
- ✅ Input selection (Input 1 vs Input 2) for both CH1 and CH2
- ✅ Full color rendering (no longer monochrome)
- ✅ Frame draining to prevent queue buildup
- ✅ Webcam capture with auto-start on app launch

### Stage 1: Input Sampling (CH1 and CH2 Independent)
- ✅ Geometric transforms (X/Y/Z displace, rotate)
- ✅ Kaleidoscope effect
- ✅ Horizontal/Vertical mirrors
- ✅ Horizontal/Vertical flip
- ✅ Input selection working correctly
- ✅ **Fixed**: Separate uniform buffers for CH1 and CH2

### Stage 2: Effects Processing (CH1 and CH2 Independent)
- ✅ HSB attenuation (multiply HSB values)
- ✅ Hue/Sat/Bright inverts
- ✅ RGB invert
- ✅ Blur effect
- ✅ Sharpen effect
- ✅ Filters boost
- ✅ Solarize
- ✅ Posterize
- ✅ **Fixed**: Separate uniform buffers for CH1 and CH2

### Stage 3: Mixing & Feedback
- ✅ **Fixed**: Mix Amount slider (0%=CH1, 50%=blend, 100%=CH2)
- ✅ Mix types: Lerp, Add, Diff, Mult, Dodge
- ✅ Overflow modes: Wrap, Clamp, Mirror
- ✅ **Fixed**: Keying only applies when threshold < 1.0
- ✅ FB1 geometric transforms
- ✅ FB1 color adjustments (HSB offset/attenuate)
- ✅ FB1 hue shaper
- ✅ FB1 posterize (RGB and HSB modes)
- ✅ FB1 blur/sharpen on feedback
- ✅ Delay buffer (0-120 frames)

## Architecture Notes

### Buffer Flow
```
Stage 1 CH1: Input → Transform → Buffer A (using stage1_uniforms_ch1)
Stage 2 CH1: Buffer A → Effects → Buffer B (using stage2_uniforms_ch1)

Stage 1 CH2: Input → Transform → Buffer A (using stage1_uniforms_ch2)
Stage 2 CH2: Buffer A → Effects → CH2 Buffer (using stage2_uniforms_ch2)

Stage 3: Buffer B (CH1) + CH2 Buffer + FB1 → Mix → Output
```

### Key Insight
GPU command encoding is deferred - writes to buffers happen immediately on CPU, but render passes execute later on GPU. If you write CH2 uniforms before CH1's render pass executes, CH1 will use CH2's data.

**Solution**: Use separate buffers OR ensure render pass is submitted before writing next channel's uniforms.

### Uniform Buffer Pattern
```rust
// WRONG - shared buffer
stage1_uniforms: wgpu::Buffer,
write_stage1_uniforms(queue, &ch1_params);  // Write CH1
write_stage1_uniforms(queue, &ch2_params);  // Overwrites CH1!
render_ch1();  // Uses CH2 params 😢
render_ch2();  // Uses CH2 params

// CORRECT - separate buffers
stage1_uniforms_ch1: wgpu::Buffer,
stage1_uniforms_ch2: wgpu::Buffer,
write_stage1_uniforms(queue, &stage1_uniforms_ch1, &ch1_params);
write_stage1_uniforms(queue, &stage1_uniforms_ch2, &ch2_params);
render_ch1();  // Uses CH1 params ✓
render_ch2();  // Uses CH2 params ✓
```

## Remaining Work

### FB1 May Have Same Issue
FB1 (feedback) uses its own uniform buffer. If FB1 mixing shows similar issues (feedback parameters affecting wrong channel), apply the same fix:
- Create separate uniform buffer for FB1
- Update `write_stage3_uniforms` to accept target buffer
- Update bind group creation to use correct buffer

## Testing Checklist

- [x] CH1 transforms (displace, rotate, kaleidoscope) work independently
- [x] CH2 transforms work independently  
- [x] CH1 color effects (HSB, blur, posterize) work independently
- [x] CH2 color effects work independently
- [x] Mix Amount blends correctly (0%, 50%, 100%)
- [x] Keying activates only when threshold < 1.0
- [x] FB1 transforms work correctly
- [x] FB1 color effects work correctly
- [x] FB1 mixing works correctly

## Uniform Buffer Layout Fix (2025-02-25)

### Problem
FB1 color transforms (HSB offset/attenuate/powmap) were not working correctly due to uniform buffer layout mismatch between Rust and WGSL.

### Root Cause
Rust `Vec3` type with `#[repr(C, align(16))]` has size 16 (12 bytes data + 4 bytes padding), but WGSL `vec3<f32>` has size 12. This caused a 4-byte offset drift for all fields after the vec3 fields.

### Solution
Changed `Vec3` fields to `[f32; 3]` and added explicit padding to match WGSL layout:

```rust
// Before (16 bytes each, caused offset drift)
fb1_hsb_offset: Vec3,       // Size 16
fb1_hsb_attenuate: Vec3,    // Size 16
fb1_hsb_powmap: Vec3,       // Size 16

// After (12 bytes each with explicit padding)
_pad_before_hsb: [f32; 2],      // Offsets 88-96
fb1_hsb_offset: [f32; 3],       // Offset 96, size 12
_pad_after_hsb_offset: f32,     // Offset 108-112
fb1_hsb_attenuate: [f32; 3],    // Offset 112, size 12
_pad_after_hsb_attenuate: f32,  // Offset 124-128
fb1_hsb_powmap: [f32; 3],       // Offset 128, size 12
```

### Verified Offsets
| Field | Rust Offset | WGSL Offset |
|-------|-------------|-------------|
| fb1_key_order | 84 | 84 |
| fb1_hsb_offset | 96 | 96 |
| fb1_hsb_attenuate | 112 | 112 |
| fb1_hsb_powmap | 128 | 128 |
| fb1_hue_shaper | 140 | 140 |
| fb1_posterize | 144 | 144 |
| fb1_posterize_switch | 148 | 148 |
| fb1_hue_invert | 152 | 152 |

✅ **All offsets now match!**

## References

- OF Original: `bin/data/shadersGL3/shader1.frag` - `mixnKeyVideo()` function
- OF Keying: Uses distance from key color, mixes based on threshold/soft
