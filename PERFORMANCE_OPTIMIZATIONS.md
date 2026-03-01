# Performance Optimization Plan

## Overview

This document outlines the performance optimization strategy for RustJay WAAAVES. The application currently functions well but has several opportunities for GPU efficiency improvements.

**Current Architecture:**
- Modular 3-stage pipeline (Stage 1: Input Sampling, Stage 2: Effects, Stage 3: Mixing)
- Multiple render passes per block with ping-pong buffers
- CPU-side feedback/delay buffer management
- Synchronous uniform buffer updates

---

## Phase 1: Quick Wins (Low Risk, Immediate Impact)

### 1.1 Remove Block1DebugView Redundancy
**Status:** 🔄 Ready to implement
**Impact:** Low (code cleanup + minor performance)

The `Block1DebugView` enum and associated code was used for visualizing intermediate stages. This is now redundant since the Preview Window with source selection provides the same functionality.

**Files to modify:**
- `src/core/mod.rs` - Remove `Block1DebugView` enum
- `src/core/mod.rs` - Remove `block1_debug_view` from SharedState
- `src/engine/blocks/block1.rs` - Remove debug_view field and all conditional logic
- `src/engine/mod.rs` - Remove debug view sync code

**Benefits:**
- Cleaner code path (no conditional render pass skipping)
- Reduced branch complexity in hot render loop
- ~50-100 lines of code removed

---

### 1.2 Conditional Stage Skipping
**Status:** 📋 Planned
**Impact:** High (skip unnecessary work)

Expand the existing Stage 2 conditional logic to all stages:

```rust
fn should_skip_stage2(params: &Block1Params) -> bool {
    params.ch1_blur_amount == 0.0 
        && params.ch1_sharpen_amount == 0.0
        && params.ch1_hsb_attenuate == Vec3::ONE
        && !params.ch1_hue_invert
        && !params.ch1_solarize
        && !params.ch1_posterize
        // ... all effects disabled
}

fn should_skip_stage3_ch2_mix(params: &Block1Params) -> bool {
    params.ch2_mix_amount == 0.0 || params.ch2_input_select == 0 // "None"
}

fn should_skip_fb1(params: &Block1Params) -> bool {
    params.fb1_mix_amount == 0.0
}
```

**Implementation notes:**
- Add `copy_texture_to_texture` fast path for skipped stages
- Store "dirty" flags to detect when params change
- Avoid uniform buffer writes when stage is skipped

**Expected gain:** 20-30% when effects disabled

---

### 1.3 Bind Group Caching
**Status:** 📋 Planned
**Impact:** Medium (reduce per-frame allocations)

**Current:** Create bind groups every frame:
```rust
let stage1_ch1_bind_group = self.create_stage1_bind_group(device, ...);
```

**Optimized:** Cache and reuse:
```rust
pub struct ModularBlock1 {
    cached_bind_groups: HashMap<u64, wgpu::BindGroup>, // key = input texture id
}
```

**Key considerations:**
- Use texture view's internal ID as cache key
- Clear cache when pipeline recreated
- Maximum cache size to prevent unbounded growth

**Expected gain:** 10-15% reduction in CPU overhead

---

## Phase 2: GPU Efficiency (Medium Risk, High Impact)

### 2.1 Uniform Buffer Pooling
**Status:** 📋 Planned
**Impact:** Medium (reduce CPU→GPU transfers)

**Current:** 6+ separate uniform buffers written every frame
**Optimized:** Single large buffer with dynamic offsets

```rust
// Single 4KB buffer with all uniforms
let uniform_pool = device.create_buffer(&BufferDescriptor {
    size: 4096,
    usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
    mapped_at_creation: false,
});

// Write all at once or in batches
queue.write_buffer(&uniform_pool, ch1_stage1_offset, ch1_stage1_bytes);
queue.write_buffer(&uniform_pool, ch2_stage1_offset, ch2_stage1_bytes);
// etc.

// Bind with dynamic offset
render_pass.set_bind_group(0, &bind_group, &[ch1_stage1_offset]);
```

**Benefits:**
- Fewer write_buffer calls
- Better cache locality
- Reduced memory fragmentation

**Expected gain:** 10-15%

---

### 2.2 Compute Shaders for Feedback/Delay
**Status:** 📋 Planned
**Impact:** High (eliminate copies, batch operations)

**Current pipeline per frame:**
```
Stage1 Render → Buffer A
Stage2 Render → Buffer B (conditional)
Stage3 Render → Output
Copy → Feedback buffer
Copy → Delay ring buffer slot
```

**Optimized compute approach:**
```
Compute dispatch:
  - Sample inputs with transforms
  - Apply effects (if needed)
  - Mix to output
  - Write feedback buffer
  - Write delay buffer slot
```

**Benefits:**
- Single dispatch vs 3-4 render passes
- No intermediate texture copies
- Direct imageLoad/imageStore (no vertex overhead)
- Can batch Block 1, 2, 3 in single command buffer more efficiently

**Implementation:**
```wgsl
@compute @workgroup_size(8, 8)
fn block1_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let uv = vec2<f32>(global_id.xy) / vec2<f32>(uniforms.width, uniforms.height);
    
    // Stage 1: Sample with transforms
    let ch1 = sample_input(uv, uniforms.ch1_transform);
    let ch2 = sample_input(uv, uniforms.ch2_transform);
    let fb = sample_feedback(uv, uniforms.fb_transform);
    
    // Stage 2: Effects (conditional)
    let ch1_effected = apply_effects(ch1, uniforms.effects);
    
    // Stage 3: Mix
    let output = mix(ch1_effected, ch2, fb, uniforms.mix_params);
    
    // Write outputs
    textureStore(output_tex, global_id.xy, output);
    textureStore(feedback_tex, global_id.xy, output);
    textureStore(delay_tex_array, vec3(global_id.xy, delay_index), output);
}
```

**Expected gain:** 25-40% overall

---

### 2.3 Async Preview Readback Optimization
**Status:** 📋 Planned
**Impact:** Medium (reduce CPU polling)

**Current:**
```rust
preview.process_readback(&self.device); // Calls device.poll()
```

**Optimized:**
- Use `PollType::Wait` with timeout instead of `Poll`
- Only readback when preview window visible AND needs update
- Throttle to 15 FPS when idle (no color pick pending)

```rust
// Only update preview every N frames when idle
let update_interval = if color_pick_requested { 1 } else { 4 }; // 60fps → 15fps
if frame_count % update_interval == 0 {
    preview.update();
}

// In process_readback - use Wait instead of Poll
device.poll(wgpu::PollType::wait(Duration::from_micros(100))).ok();
```

**Expected gain:** 5-10% CPU reduction

---

## Phase 3: Architecture Improvements (Higher Risk, Long-term)

### 3.1 Texture Arrays for Inputs
**Status:** 📋 Planned
**Impact:** Medium (simpler bind groups)

**Current:** Switch bind groups when changing input
**Optimized:** Single bind group with texture array

```wgsl
@group(0) @binding(0) var input_textures: texture_2d_array<f32>;

// In shader - index selects input
let color = textureSample(input_textures, sampler, uv, input_index);
```

**Benefits:**
- No bind group switching
- Cache-friendly
- Can batch multiple inputs in single dispatch

---

### 3.2 Frame Graph / Render Graph
**Status:** 📋 Research
**Impact:** Very High (automatic optimization)

Implement a frame graph for automatic:
- Barrier insertion
- Resource aliasing (transient textures)
- Render pass merging
- Culling

```rust
let mut graph = FrameGraph::new();

graph.add_pass("Block1", |builder| {
    builder.read("input1");
    builder.read("input2");
    builder.write("block1_output");
});

graph.compile(); // Automatic optimization
graph.execute(encoder);
```

**Reference implementations:**
- Bevy's render graph
- FidelityFX FrameGraph
- RenderGraph in various engines

---

## Phase 4: Memory & Bandwidth

### 4.1 Reduced Internal Resolution
**Status:** 📋 Consideration
**Impact:** High (quadratic reduction in work)

**Current:** 1920x1080 internal resolution
**Option:** 1280x720 or 960x540 with good upscaling

**Trade-offs:**
- +50-100% performance
- - Some fine detail in feedback patterns

**Implementation:**
- Render at `output_size / scale_factor`
- Final upscale pass with bicubic/bilinear filtering
- User-configurable quality setting

---

### 4.2 Temporal Feedback Optimization
**Status:** 📋 Research
**Impact:** Medium

**Current:** Full resolution feedback every frame
**Option:** Checkerboard / interleaved feedback

Render half the pixels each frame, reconstruct from history.

---

## Implementation Priority

| Priority | Optimization | Effort | Impact | Risk |
|----------|--------------|--------|--------|------|
| 1 | Remove Block1DebugView | Low | Low | None |
| 2 | Conditional stage skipping | Low | High | Low |
| 3 | Bind group caching | Low | Medium | Low |
| 4 | Uniform buffer pooling | Low | Medium | Low |
| 5 | Async preview optimization | Low | Medium | Low |
| 6 | Compute shaders | Medium | High | Medium |
| 7 | Texture arrays | Medium | Medium | Medium |
| 8 | Frame graph | High | Very High | High |
| 9 | Reduced resolution | Low | High | Low* |

*User-visible change

---

## Profiling Strategy

Before and after each optimization:

```rust
// GPU timestamps
render_pass.write_timestamp(&query_set, index);

// Readback and log
let timestamps = readback_buffer.get_mapped_range();
let duration_ns = (timestamps[1] - timestamps[0]) * timestamp_period;
log::info!("Block1 render: {:.2}ms", duration_ns as f32 / 1_000_000.0);
```

**Metrics to track:**
1. Frame time (ms)
2. GPU utilization %
3. Memory bandwidth (texture copies)
4. CPU time in render loop
5. Number of render passes

---

## Notes

- Test on both integrated and discrete GPUs
- macOS Metal has different performance characteristics than Vulkan
- Always profile before optimizing - measure actual bottlenecks
- Keep the OF-compatible code path working for reference
