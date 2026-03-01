# Block 1 Implementation Guide

A comprehensive guide documenting the Block 1 implementation, including architecture, bugs fixed, and lessons learned. Use this as a reference for implementing Block 2 and Block 3.

---

## Architecture Overview

### 3-Stage Modular Pipeline

```
┌─────────────────────────────────────────────────────────────────┐
│                        BLOCK 1 PIPELINE                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  STAGE 1: Input Sampling          STAGE 2: Effects              │
│  ┌─────────────────────┐          ┌─────────────────────┐       │
│  │ CH1: Transform UV   │          │ CH1: HSB/Blur/etc   │       │
│  │  - Scale            │─────────▶│  - Color effects    │────┐  │
│  │  - Rotate           │          │  - Filters          │    │  │
│  │  - Displace         │          └─────────────────────┘    │  │
│  │  - Kaleidoscope     │                                     │  │
│  │  - Mirrors          │          ┌─────────────────────┐    │  │
│  └─────────────────────┘          │ CH2: HSB/Blur/etc   │    │  │
│                                   │  - Color effects    │────┤  │
│  ┌─────────────────────┐          │  - Filters          │    │  │
│  │ CH2: Transform UV   │          └─────────────────────┘    │  │
│  │  (same transforms)  │─────────────────────────────────────▶│  │
│  └─────────────────────┘                                     │  │
│                                                              ▼  │
│  STAGE 3: Mixing & Feedback                             Output  │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │  CH1 Output + CH2 Output + FB1 ──▶ Mix ──▶ Final Output │   │
│  │                                                         │   │
│  │  Features:                                              │   │
│  │  - Mix modes (lerp, add, diff, mult, dodge)            │   │
│  │  - Keying (CH2 and FB1 separate)                       │   │
│  │  - FB1 transforms and color                            │   │
│  │  - Delay buffer                                        │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### Key Design Decisions

1. **Separate Uniform Buffers**: Each channel (CH1, CH2) has its own uniform buffers for Stage 1 and Stage 2
2. **Single Stage 3 Buffer**: One uniform buffer for mixing (no separate CH1/CH2 needed)
3. **Ping-Pong Feedback**: FB1 is read from previous frame, written to current frame
4. **Delay Buffer**: Separate texture for delayed feedback (configurable 0-120 frames)

---

## Critical Bugs Fixed

### Bug 1: Shared Uniform Buffer Overwrite

**Severity**: 🔴 Critical  
**Impact**: CH1 showed CH2's parameters  
**Root Cause**: Both channels wrote to the same uniform buffer

#### The Problem

```rust
// WRONG - Single buffer shared between CH1 and CH2
struct Block1Pipeline {
    stage1_uniforms: wgpu::Buffer,  // Shared!
    stage2_uniforms: wgpu::Buffer,  // Shared!
}

fn render(&self, ch1_params, ch2_params) {
    // Write CH1 params
    queue.write_buffer(&self.stage1_uniforms, 0, &ch1_bytes);
    // CH1 render not submitted yet...
    
    // Write CH2 params (OVERWRITES CH1!)
    queue.write_buffer(&self.stage1_uniforms, 0, &ch2_bytes);
    
    // Now submit both render passes
    render_ch1();  // Uses CH2 params 😢
    render_ch2();  // Uses CH2 params
}
```

GPU command encoding is deferred. Both `write_buffer` calls happen immediately on CPU, but render passes execute later on GPU. The second write overwrites the first before either render executes.

#### The Solution

```rust
// CORRECT - Separate buffers for each channel
struct Block1Pipeline {
    stage1_uniforms_ch1: wgpu::Buffer,
    stage1_uniforms_ch2: wgpu::Buffer,
    stage2_uniforms_ch1: wgpu::Buffer,
    stage2_uniforms_ch2: wgpu::Buffer,
}

fn render(&self, ch1_params, ch2_params) {
    // Write to separate buffers
    queue.write_buffer(&self.stage1_uniforms_ch1, 0, &ch1_bytes);
    queue.write_buffer(&self.stage1_uniforms_ch2, 0, &ch2_bytes);
    
    // Each channel uses its own buffer
    render_ch1();  // Binds stage1_uniforms_ch1 ✓
    render_ch2();  // Binds stage1_uniforms_ch2 ✓
}
```

#### Bind Group Setup

```rust
// CH1 bind group uses ch1 buffers
let bind_group_ch1 = device.create_bind_group(&wgpu::BindGroupDescriptor {
    layout: &bind_group_layout,
    entries: &[
        wgpu::BindGroupEntry {
            binding: 0,
            resource: stage1_uniforms_ch1.as_entire_binding(),
        },
        // ... other bindings
    ],
});

// CH2 bind group uses ch2 buffers
let bind_group_ch2 = device.create_bind_group(&wgpu::BindGroupDescriptor {
    layout: &bind_group_layout,
    entries: &[
        wgpu::BindGroupEntry {
            binding: 0,
            resource: stage1_uniforms_ch2.as_entire_binding(),
        },
        // ... other bindings
    ],
});
```

---

### Bug 2: Uniform Buffer Layout Mismatch

**Severity**: 🔴 Critical  
**Impact**: FB1 transforms/color effects didn't work  
**Root Cause**: Rust `Vec3` size (16 bytes) didn't match WGSL `vec3` size (12 bytes)

#### The Problem

```rust
// Rust struct
#[repr(C, align(16))]
struct Vec3([f32; 3]);  // Size: 16 bytes (12 data + 4 padding)

#[repr(C, align(16))]
struct Stage3Uniforms {
    // ... fields before
    fb1_hsb_offset: Vec3,       // Offset 88, size 16
    fb1_hsb_attenuate: Vec3,    // Offset 104, size 16
    fb1_hsb_powmap: Vec3,       // Offset 120, size 16
    fb1_hue_shaper: f32,        // Offset 136
    // ...
}
```

```wgsl
// WGSL struct
struct Uniforms {
    // ... fields before
    fb1_hsb_offset: vec3<f32>,      // Offset 88, size 12
    fb1_hsb_attenuate: vec3<f32>,   // Offset 100, size 12
    fb1_hsb_powmap: vec3<f32>,      // Offset 112, size 12
    fb1_hue_shaper: f32,            // Offset 124
    // ...
}
```

The 4-byte size difference caused all fields after the first `vec3` to be at wrong offsets.

#### The Solution

Use `[f32; 3]` with explicit padding to match WGSL layout:

```rust
#[repr(C, align(16))]
struct Stage3Uniforms {
    // ... fields before (ends at offset 88)
    
    // Explicit padding to align to 16-byte boundary
    _pad_before_hsb: [f32; 2],      // Offsets 88-96
    
    // vec3 fields (12 bytes each)
    fb1_hsb_offset: [f32; 3],       // Offset 96, size 12
    _pad_after_hsb_offset: f32,     // Offset 108-112
    
    fb1_hsb_attenuate: [f32; 3],    // Offset 112, size 12
    _pad_after_hsb_attenuate: f32,  // Offset 124-128
    
    fb1_hsb_powmap: [f32; 3],       // Offset 128, size 12
    
    fb1_hue_shaper: f32,            // Offset 140
    // ...
}
```

#### WGSL vec3 Alignment Rules

- `vec3<f32>` has **16-byte alignment** (same as `vec4`)
- `vec3<f32>` has **12-byte size**
- After a `vec3`, next field starts at: `offset + 12` rounded up to 16-byte boundary

#### Verification Script

```rust
// Always verify your struct layout!
fn offset_of<T, F>(f: fn(&T) -> &F) -> usize {
    let uninit = std::mem::MaybeUninit::<T>::uninit();
    let base_ptr = uninit.as_ptr();
    let field_ptr = f(unsafe { &*base_ptr });
    (field_ptr as *const F as usize) - (base_ptr as usize)
}

// Usage:
println!("fb1_hsb_offset: {}", 
    offset_of::<Stage3Uniforms, _>(|s| &s.fb1_hsb_offset));
// Compare with WGSL offset
```

---

### Bug 3: Keying Always Active

**Severity**: 🟡 Medium  
**Impact**: CH2 was always keyed, never blended  
**Root Cause**: Keying function ran unconditionally

#### The Problem

```wgsl
fn mixn_key_video(fg: vec4<f32>, bg: vec4<f32>, params: KeyParams) -> vec4<f32> {
    let key_mix = calculate_key(fg, params);
    // Always applies keying, even at threshold = 1.0
    return mix(bg, fg, key_mix);
}
```

When `threshold = 1.0` (full range, no keying), the key was still being calculated and applied.

#### The Solution

Only apply keying when threshold is less than 1.0:

```wgsl
fn mixn_key_video(fg: vec4<f32>, bg: vec4<f32>, params: KeyParams) -> vec4<f32> {
    // Keying only when threshold < 1.0 (not full range)
    if (params.threshold >= 0.999) {
        return fg;  // No keying, just use foreground
    }
    
    let key_mix = calculate_key(fg, params);
    return mix(bg, fg, key_mix);
}
```

---

### Bug 4: Key Order Logic Reversed

**Severity**: 🟡 Medium  
**Impact**: "Key then Mix" vs "Mix then Key" produced wrong results  
**Root Cause**: Logic swapped foreground/background instead of operation order

#### The Problem

Original code swapped fg/bg for key_order == 1:
```wgsl
if (key_order == 1) {
    // Swapped fg and bg - wrong!
    return mixn_key_video(bg, fg, params);
}
```

This just swapped the inputs, not the order of operations.

#### The Solution

Proper "Key then Mix" vs "Mix then Key":

```wgsl
fn mixn_key_video_ch2(ch1: vec4<f32>, ch2: vec4<f32>, params: Uniforms) -> vec4<f32> {
    if (params.ch2_key_order == 0) {
        // Key CH2, then mix with CH1
        let keyed_ch2 = apply_keying(ch2, params.ch2_key_threshold);
        return mix(ch1, keyed_ch2, params.mix_amount);
    } else {
        // Mix CH1 and CH2, then key the result
        let mixed = mix(ch1, ch2, params.mix_amount);
        return apply_keying(mixed, params.ch2_key_threshold);
    }
}
```

---

### Bug 5: LFO Waveform Not Synced

**Severity**: 🟡 Medium  
**Impact**: LFO always used sine wave regardless of selection  
**Root Cause**: GUI-local state not synced to shared state

#### The Problem

```rust
// In draw_lfo_control()
let lfo = lfo_map.entry(param_id).or_insert(LfoState::default());

// User changes waveform
lfo.waveform = 3;  // Saw - only updates GUI-local state!

// Visualization uses local state (correct)
let value = calculate_lfo_value(bank.phase, lfo.waveform);

// But engine reads from shared state (stale value)
// state.lfo_banks[bank_index].waveform is still 0 (Sine)
```

#### The Solution

Sync GUI state to shared state when changed:

```rust
// When waveform changes
let new_waveform = wave_idx.clamp(0, WAVEFORM_NAMES.len() - 1) as i32;
if new_waveform != lfo.waveform {
    lfo.waveform = new_waveform;
    // Sync to shared state
    if lfo.bank_index >= 0 {
        if let Ok(mut state) = self.shared_state.lock() {
            if (lfo.bank_index as usize) < state.lfo_banks.len() {
                state.lfo_banks[lfo.bank_index as usize].waveform = new_waveform;
            }
        }
    }
}

// Also sync rate, tempo_sync, division at end of controls
if lfo.bank_index >= 0 {
    if let Ok(mut state) = self.shared_state.lock() {
        if (lfo.bank_index as usize) < state.lfo_banks.len() {
            let bank = &mut state.lfo_banks[lfo.bank_index as usize];
            bank.rate = lfo.rate;
            bank.tempo_sync = lfo.tempo_sync;
            bank.division = lfo.division;
        }
    }
}
```

---

## Implementation Checklist for Block 2/3

### Stage 1 & 2 Pattern (Per-Channel Processing)

- [ ] Create separate uniform buffers for each channel (CH1, CH2)
- [ ] Create separate bind groups for each channel
- [ ] Update struct initialization to populate both buffers
- [ ] Verify struct field order matches WGSL
- [ ] Run layout verification script

### Stage 3 Pattern (Mixing & Feedback)

- [ ] Single uniform buffer (no per-channel needed)
- [ ] Match WGSL layout for vec3 fields (use `[f32; 3]` + padding)
- [ ] Implement key_order logic correctly (key then mix vs mix then key)
- [ ] Add threshold check to disable keying at 1.0

### LFO Integration

- [ ] Sync waveform from GUI to shared state
- [ ] Sync rate, tempo_sync, division to shared state
- [ ] Verify visualization matches actual output

---

## Key Takeaways

1. **Always use separate uniform buffers** for per-channel processing
2. **Always verify struct layout** matches WGSL using offset calculation
3. **Remember GPU command encoding is deferred** - writes happen immediately, renders happen later
4. **Test with extreme parameter values** (0, 1, -1) to catch edge cases
5. **Sync GUI state to shared state** for LFOs and other time-based parameters

---

## Files to Reference

- `src/engine/blocks/block1.rs` - Main Block 1 implementation
- `src/core/lfo_engine.rs` - LFO calculation and application
- `src/gui/mod.rs` - GUI controls and state sync
- `BLOCK1_STATUS.md` - Current status and testing checklist
