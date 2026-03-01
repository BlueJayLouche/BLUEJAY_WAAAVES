# Modular Shader Architecture Plan

## Philosophy
- **Not bound by OpenFrameworks limitations**
- **Debuggable** - Each stage can be visualized independently
- **Performant** - Skip stages when not needed
- **Maintainable** - Small, focused shaders instead of monolithic blocks

---

## Current vs Proposed Architecture

### Current (Monolithic)
```
┌─────────────────────────────────────────────────────────────┐
│                         BLOCK 1                              │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌────────┐ │
│  │ Input 1 │→│ Transform│→│  HSB   │→│ Mix CH2│→│ Mix FB1│→│
│  │ Sample  │ │  + Kaleido│  │Process │ │        │ │        │ │
│  └─────────┘ └─────────┘ └─────────┘ └─────────┘ └────────┘ │
│         ↑___________________________________________│        │
│                    Feedback Loop (implicit)                   │
└─────────────────────────────────────────────────────────────┘
```

**Problems:**
- One 1000+ line shader
- Can't debug intermediate stages
- Always runs all operations
- When black output: which of 20 operations caused it?

### Proposed (3-Stage Modular)
```
┌─────────────────────────────────────────────────────────────────┐
│                         BLOCK 1                                  │
│                                                                  │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │ STAGE 1: INPUT SAMPLING                                     │ │
│  │  - Sample Input 1, Input 2, FB textures                     │ │
│  │  - Apply coordinate transforms (UV space)                   │ │
│  │  - Output: Transformed samples (no color processing)        │ │
│  └────────────────────────────────────────────────────────────┘ │
│                              ↓                                   │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │ STAGE 2: EFFECT PROCESSING (Optional - can skip)            │ │
│  │  - HSB adjustments (hue shift, saturation, brightness)      │ │
│  │  - Blur/Sharpen filters                                     │ │
│  │  - Kaleidoscope/mirror effects                              │ │
│  │  - Posterize, Solarize                                      │ │
│  │  - Output: Processed colors                                 │ │
│  └────────────────────────────────────────────────────────────┘ │
│                              ↓                                   │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │ STAGE 3: MIXING & FEEDBACK                                  │ │
│  │  - Mix Channel 1 + Channel 2                                │ │
│  │  - Mix with Feedback (FB1)                                  │ │
│  │  - Keying, Blend modes                                      │ │
│  │  - Output: Final Block 1 result                             │ │
│  └────────────────────────────────────────────────────────────┘ │
│                              ↓                                   │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │ FEEDBACK BUFFER UPDATE                                      │ │
│  │  - Copy output to FB1 texture for next frame                │ │
│  └────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

**Benefits:**
- Each stage ~100-200 lines
- Can visualize output of each stage
- Skip Stage 2 entirely if no effects enabled
- Easier to test and debug

---

## Stage Details

### Stage 1: Input Sampling
**Purpose:** Get pixel data from all input sources with coordinate transforms

**Inputs:**
- Input 1 texture (camera/media)
- Input 2 texture (camera/media)  
- FB1 texture (previous frame feedback)
- Transform parameters (scale, rotate, displace, kaleidoscope)

**Output:** 
- `sampled_input1: vec4<f32>`
- `sampled_input2: vec4<f32>`
- `sampled_fb1: vec4<f32>`

**Operations:**
```wgsl
// For each input:
1. Apply aspect ratio correction
2. Apply scale/zoom
3. Apply rotation
4. Apply displacement
5. Apply kaleidoscope (if enabled)
6. Sample texture at transformed UV
```

**Early Exit Conditions:**
- If Input 2 mix amount = 0, skip Input 2 sampling
- If FB1 mix amount = 0, skip FB1 sampling

---

### Stage 2: Effect Processing (Optional)
**Purpose:** Color adjustments and filters

**Inputs:**
- Sampled colors from Stage 1
- Effect parameters (HSB adjustments, blur amount, etc.)

**Output:**
- Processed colors (same structure as input)

**Operations:**
```wgsl
// Only if effect enabled:
1. Blur/Sharpen (if amount > 0)
2. HSB conversion (if any HSB param != 1.0 or invert enabled)
3. Apply HSB adjustments
4. Convert back to RGB
5. Posterize (if enabled)
6. Solarize (if enabled)
```

**Early Exit Conditions:**
```wgsl
// Check if any effect is enabled
let needs_processing = 
    blur_amount > 0.001 ||
    sharpen_amount > 0.001 ||
    hsb_attenuate != vec3(1.0) ||
    hue_invert || sat_invert || bright_invert ||
    posterize_enabled || solarize_enabled;

if (!needs_processing) {
    output = input;  // Skip entire stage
}
```

---

### Stage 3: Mixing & Feedback
**Purpose:** Combine all processed inputs

**Inputs:**
- Processed Input 1
- Processed Input 2
- Processed FB1
- Mix parameters (amounts, blend modes, keying)

**Output:**
- Final color for this block

**Operations:**
```wgsl
1. Mix Input 1 + Input 2 (if Input 2 amount > 0)
2. Apply keying (if enabled)
3. Mix with FB1 (if FB1 amount > 0)
4. Apply final clamp/wrap/fold
```

---

## Performance Optimizations

### 1. Stage Skipping
```rust
pub struct BlockPipeline {
    stage1_sampler: RenderPipeline,      // Always run
    stage2_effects: Option<RenderPipeline>, // None = skip
    stage3_mixer: RenderPipeline,        // Always run
    
    // Track which effects are enabled
    effects_enabled: bool,
}

impl BlockPipeline {
    pub fn render(&self, encoder: &mut CommandEncoder) {
        // Stage 1: Always run
        self.run_stage1(encoder);
        
        // Stage 2: Only if effects enabled
        if self.effects_enabled {
            self.run_stage2(encoder);
        }
        
        // Stage 3: Always run
        self.run_stage3(encoder);
    }
}
```

### 2. Ping-Pong Buffers
Each stage uses ping-pong textures:
```
Frame N:   Input → BufferA → BufferB → Output
Frame N+1: Input → BufferB → BufferA → Output
```

### 3. Feedback Explicit
Feedback is now explicit in Stage 1 (sample FB texture) and after Stage 3 (copy output to FB texture):
```
Stage 1 samples: Input1, Input2, FB_texture
Stage 3 outputs: Block_result
After Stage 3:   Block_result → FB_texture (for next frame)
```

---

## Debug Visualization

Each stage output can be viewed:
```rust
enum DebugView {
    Normal,           // Show final output
    Stage1_Samples,   // Show transformed input samples
    Stage2_Effects,   // Show after HSB/blur processing
    Stage3_Mix,       // Show after mixing (before feedback write)
    FeedbackBuffer,   // Show what's in the feedback texture
}
```

In GUI:
- Dropdown to select debug view
- Shows intermediate textures
- Helps diagnose where black/purple output originates

---

## Implementation Phases

### Phase 1: Stage 1 (Input Sampling)
- Create separate shader for input sampling
- Support Input 1, Input 2, and FB1 texture sampling
- Apply coordinate transforms
- Test: Can view sampled inputs directly

### Phase 2: Stage 2 (Effects)
- Create effects shader
- HSB processing, blur, posterize
- Enable/disable based on parameters
- Test: Toggle effects on/off, see immediate change

### Phase 3: Stage 3 (Mixing)
- Create mixing shader
- Support all blend modes
- Keying support
- Test: Mix different inputs

### Phase 4: Integration
- Chain stages together
- Ping-pong buffer management
- Feedback loop
- Debug visualization

### Phase 5: Optimization
- Skip Stage 2 when not needed
- Early exit in shaders
- Texture format optimization

---

## Code Structure

```
src/engine/
├── mod.rs                    # Main engine
├── blocks/                   # Block implementations
│   ├── mod.rs               # Block trait/common code
│   ├── block1.rs            # Block 1 setup (stages 1-2-3)
│   ├── block2.rs            # Block 2 setup
│   └── block3.rs            # Block 3 setup
├── stages/                   # Individual stage shaders
│   ├── input_sampling.wgsl  # Stage 1
│   ├── effects.wgsl         # Stage 2
│   └── mixing.wgsl          # Stage 3
└── pipelines/
    ├── mod.rs               # Pipeline creation helpers
    └── common.wgsl          # Shared shader functions
```

---

## Benefits Summary

| Aspect | Monolithic (Current) | Modular (Proposed) |
|--------|---------------------|-------------------|
| Debuggability | Hard (1000+ lines) | Easy (3 stages, ~150 lines each) |
| Performance | Always runs everything | Skip unused stages |
| Maintainability | Difficult | Easy |
| Extensibility | Hard to add effects | Just add a new stage variant |
| Testability | Integration tests only | Unit test each stage |
| GPU overhead | 1 render pass | 2-3 render passes |
| Memory | Few textures | More intermediate textures |

**Trade-off:** Slightly more GPU overhead (multiple passes) for much better developer experience and maintainability.

---

## Next Steps

1. **Implement Stage 1** for Block 1
2. **Add debug visualization** for Stage 1 output
3. **Implement Stage 2** with enable/disable toggle
4. **Implement Stage 3** 
5. **Wire up feedback loop**
6. **Test and profile**
7. **Apply to Block 2 and Block 3**

---

## Notes

- This is **not** bound by OF limitations
- This is **better** than OF's approach
- Focus on **debuggability** and **maintainability**
- Performance optimizations come after correctness
