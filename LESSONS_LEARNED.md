# Lessons Learned - Block 1 & 2 Implementation

## Critical Bug Patterns and Solutions

### 1. Uniform Buffer Alignment Issues

#### The Problem
WGSL `vec4<f32>` requires 16-byte alignment. When placed at misaligned offsets, the shader reads garbage values.

**Example - Broken:**
```rust
// Rust struct
fb2_filters_boost: f32,        // offset 184
_pad8: f32,                    // offset 188
_pad9: f32,                    // offset 192
fb2_shear_matrix: [f32; 4],    // offset 196 - NOT 16-byte aligned!
```

**WGSL reads vec4 at offset 196:**
```wgsl
fb2_shear_matrix: vec4<f32>,   // Expects 16-byte alignment
```

**Result:** 196 % 16 = 4, so shader reads from wrong memory location.

#### The Fix
Ensure vec4 fields are at offsets divisible by 16:

```rust
// Fixed - shear matrix at offset 192 (16-byte aligned)
fb2_filters_boost: f32,        // offset 184
_pad8: f32,                    // offset 188 (4 bytes padding)
fb2_shear_matrix: [f32; 4],    // offset 192 - 16-byte aligned! ✓
```

**Verification:**
```rust
let offset = mem::offset_of!(Stage3Uniforms, fb2_shear_matrix);
assert!(offset % 16 == 0, "vec4 must be 16-byte aligned");
```

---

### 2. Hardcoded Values in Shader Bindings

#### The Problem
When setting up texture bindings, using hardcoded values instead of actual parameters:

```rust
// WRONG - Always uses 1-frame delay
let delay_view = self.resources.get_delay_view(1);
```

Even though `fb2_delay_time` uniform was correct, the bound texture was always 1 frame behind.

#### The Fix
Always use actual parameter values:

```rust
// CORRECT - Uses actual delay setting
let delay_frames = params.fb2_delay_time as usize;
let (fb2_view, delay_view) = if delay_frames > 0 {
    let delayed = self.resources.get_delay_view(delay_frames);
    (delayed, delayed)
} else {
    (self.resources.get_feedback_view(), self.resources.get_delay_view(1))
};
```

---

### 3. Transform Order Matters

#### The Problem
Applying rotate before scale produces different results than scale before rotate.

**Broken (Rotate → Scale):**
```wgsl
result = rotate(result, angle);     // Rotates first
result = result / scale;            // Then scales
```

**Fixed (Scale → Rotate):**
```wgsl
result = result / scale;            // Scale first
result = rotate(result, angle);     // Then rotate
```

#### Lesson
Match the transform order of the reference implementation (OpenFrameworks version). When in doubt, check the original shader code.

---

### 4. Feedback Texture Source

#### The Problem
Block 2 renders to an external texture (`block2_texture`), not internal ping-pong buffers. Using the wrong source for feedback:

```rust
// WRONG - Copies from internal buffer (empty)
self.resources.update_delay_buffer(&mut encoder);
```

#### The Fix
Copy from the correct external texture:

```rust
// CORRECT - Copy from block2_texture
self.modular_block2.resources.update_feedback_from_external(
    &mut encoder, 
    &self.block2_texture.texture
);
self.modular_block2.resources.update_delay_buffer_from_external(
    &mut encoder, 
    &self.block2_texture.texture
);
```

---

### 5. Vec3 Size Mismatch

#### The Problem
Rust `Vec3` (from glam) has size 16 bytes (12 data + 4 padding), but WGSL `vec3<f32>` has size 12 bytes.

**Rust:**
```rust
pub struct Vec3([f32; 3]);  // glam adds 4 bytes padding = 16 bytes total
```

**WGSL:**
```wgsl
vec3<f32>  // 12 bytes, 16-byte aligned
```

#### The Fix
Use `[f32; 3]` in Rust with explicit padding:

```rust
#[repr(C, align(16))]
pub struct Stage3Uniforms {
    fb2_hsb_offset: [f32; 3],   // 12 bytes
    _pad3: f32,                  // 4 bytes padding
    fb2_hsb_attenuate: [f32; 3], // Next field at 16-byte offset
    // ...
}
```

---

### 6. WGSL Struct Layout

#### The Problem
WGSL adds implicit padding that must be explicitly matched in Rust.

**WGSL:**
```wgsl
struct Uniforms {
    mix_amount: f32,      // offset 0
    mix_type: i32,        // offset 4
    mix_overflow: i32,    // offset 8
    // implicit 4 bytes padding to align next field
    key_value: vec3<f32>, // offset 16 (16-byte aligned)
}
```

**Rust must match:**
```rust
mix_amount: f32,       // offset 0
mix_type: i32,         // offset 4
mix_overflow: i32,     // offset 8
_pad0: f32,            // offset 12 (explicit padding)
key_value: [f32; 3],   // offset 16
```

---

### 7. Debug Visualization

#### Technique
Use color tints to identify which code path is executing:

```wgsl
// In fragment shader - tint based on conditions
if (no_input_signal) {
    return vec4<f32>(1.0, 0.0, 0.0, 1.0);  // Red = no input
}
if (using_delayed_feedback) {
    return vec4<f32>(0.0, 1.0, 0.0, 1.0);  // Green = delay active
}
```

**Remove all debug tints before committing!**

---

### 8. Delay Buffer Indexing

#### The Problem
Off-by-one errors in ring buffer indexing:

```rust
// WRONG - Reads from wrong position
let read_index = (write_index - delay) % max_frames;
```

#### The Fix
Handle wrap-around correctly:

```rust
// CORRECT
let read_index = if write_index >= delay {
    write_index - delay
} else {
    max_frames - (delay - write_index)
};
```

---

## Testing Checklist

When implementing a new block, verify:

### Geometry
- [ ] X/Y displace moves texture correctly
- [ ] Z displace (scale) zooms from center
- [ ] Rotate rotates around center
- [ ] Rotate direction matches reference (CW vs CCW)
- [ ] Kaleidoscope creates symmetrical patterns
- [ ] H/V mirrors reflect correctly
- [ ] H/V flips invert correctly
- [ ] Shear matrix skews as expected

### Color
- [ ] HSB offset shifts hue/saturation/brightness
- [ ] HSB attenuate multiplies values
- [ ] HSB powmap applies curves
- [ ] Inverts (hue/sat/bright/RGB) work
- [ ] Posterize reduces color levels
- [ ] Hue shaper applies special function

### Mixing
- [ ] All mix types (lerp, add, diff, mult, dodge)
- [ ] Keying (luma and chroma)
- [ ] Key threshold/soft control edge softness
- [ ] Key order (key→mix vs mix→key)

### Feedback
- [ ] Delay time selection (0, 1, 10, 60 frames)
- [ ] Feedback persists across frames
- [ ] Feedback transforms apply correctly

---

## Code Patterns

### Modular Block Structure

```rust
pub struct ModularBlockX {
    // Resources (ping-pong buffers, feedback, delay)
    pub resources: BlockResources,
    
    // Stage 1: Input sampling
    stage1_pipeline: wgpu::RenderPipeline,
    stage1_bind_group_layout: wgpu::BindGroupLayout,
    stage1_uniforms: wgpu::Buffer,
    
    // Stage 2: Effects (optional)
    stage2_pipeline: wgpu::RenderPipeline,
    // ...
    
    // Stage 3: Mixing + Feedback
    stage3_pipeline: wgpu::RenderPipeline,
    // ...
}
```

### Uniform Buffer Update Pattern

```rust
fn update_stageX_uniforms(&self, queue: &wgpu::Queue, params: &BlockXParams) {
    let uniforms = StageXUniforms {
        // Map params to shader uniforms
        x_displace: params.x_displace,
        // ...
    };
    
    queue.write_buffer(
        &self.stageX_uniforms, 
        0, 
        bytemuck::cast_slice(&[uniforms])
    );
}
```

### Render Pass Pattern

```rust
fn render_stageX(&self, encoder: &mut wgpu::CommandEncoder) {
    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: self.resources.get_output_view(),
            // ...
        })],
        // ...
    });
    
    render_pass.set_pipeline(&self.stageX_pipeline);
    render_pass.set_bind_group(0, &bind_group, &[]);
    render_pass.draw(0..6, 0..1);
}
```

---

## Reference

- `BLOCK1_IMPLEMENTATION_GUIDE.md` - Detailed Block 1 implementation
- `BLOCK2_PROGRESS.md` - Block 2 status and tasks
- `src/engine/blocks/block1.rs` - Reference implementation
- `src/engine/blocks/block2.rs` - Latest patterns
