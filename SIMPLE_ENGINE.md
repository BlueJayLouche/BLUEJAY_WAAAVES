# Simple Feedback Engine - Technical Reference

This document provides detailed technical information about the simplified single-block feedback engine implementation. It serves as a reference for understanding the architecture, troubleshooting issues, and extending the system.

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Dual-Window System](#dual-window-system)
3. [Feedback Loop Design](#feedback-loop-design)
4. [Tap Tempo System](#tap-tempo-system)
5. [Texture Format Handling](#texture-format-handling)
6. [Shader Pipeline](#shader-pipeline)
7. [Common Issues & Solutions](#common-issues--solutions)
8. [Extension Guide](#extension-guide)

---

## Architecture Overview

The simple feedback engine is a minimal implementation designed for:
- Single shader feedback loop
- Auto-webcam startup
- Tempo-synced LFOs
- Real-time parameter control via ImGui

### Key Components

```
┌─────────────────────────────────────────────────────────────────┐
│                     SimpleEngine                                │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────────────┐  │
│  │   Output    │    │  Feedback   │    │   Input Manager     │  │
│  │  Texture    │◄──►│  Textures   │    │  (Webcam/NDI/etc)   │  │
│  │ (display)   │    │(ping-pong)  │    │                     │  │
│  └──────┬──────┘    └──────┬──────┘    └─────────────────────┘  │
│         │                  │                                     │
│         └──────────────────┘                                     │
│              │                                                   │
│              ▼                                                   │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │           SimpleFeedbackPipeline (Shader)                │   │
│  │  - Combines input + feedback                            │   │
│  │  - Applies hue shift (LFO-modulated)                    │   │
│  │  - Applies rotate/zoom (tempo-synced LFO)               │   │
│  └──────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                  Control Window (ImGui)                         │
│  ┌─────────────────┐    ┌─────────────────────────────────────┐ │
│  │   Hue Slider    │    │         Tap Tempo                   │ │
│  │  (0.0 - 1.0)    │    │  - TAP button resets LFO phase      │ │
│  │                 │    │  - 4+ taps calculates BPM           │ │
│  │                 │    │  - Auto-reset after 2s inactivity   │ │
│  └─────────────────┘    └─────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

---

## Dual-Window System

### Design Rationale

The dual-window architecture separates rendering from control:

1. **Output Window**: Full wgpu surface for shader rendering
2. **Control Window**: Separate wgpu surface for ImGui interface

### Implementation Details

**Critical Constraint (macOS Metal)**:
- Cannot create multiple `wgpu::Device` instances on macOS
- Must share device/queue between windows
- Control window creates its own surface using shared device

```rust
// Engine stores shared resources
pub struct SimpleEngine {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    instance: wgpu::Instance,  // Needed for control window surface
    // ...
}

// Control window gets shared references
let imgui_renderer = pollster::block_on(async {
    let device = engine.device();  // Arc::clone
    let queue = engine.queue();    // Arc::clone
    let instance = engine.instance();
    
    ImGuiRenderer::new(instance, device, queue, control_window).await
});
```

### Window Event Handling

Events are routed based on window ID:

```rust
fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
    // Check if event is for control window
    if let Some(ref control_window) = self.control_window {
        if window_id == control_window.id() {
            // Pass to ImGui renderer for input handling
            if let Some(ref mut renderer) = self.imgui_renderer {
                renderer.handle_event(&event, control_window);
            }
            // Handle control window events...
            return;
        }
    }
    // Handle output window events...
}
```

### ImGui Renderer Integration

The `ImGuiRenderer` wraps `imgui-wgpu` with winit event handling:

```rust
pub struct ImGuiRenderer {
    context: Context,
    renderer: Renderer,
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    surface: wgpu::Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,
}
```

**Important**: Must set initial display size before first frame:
```rust
imgui_renderer.set_display_size(width as f32, height as f32);
```

---

## Feedback Loop Design

### Ping-Pong Buffering

The engine uses two feedback textures that alternate as read/write targets:

```
Frame N:   Read from A ──► Shader ──► Write to B
Frame N+1: Read from B ──► Shader ──► Write to A
```

This prevents read-after-write hazards and enables continuous feedback.

### Implementation

```rust
pub struct SimpleEngine {
    feedback_texture_a: Texture,           // Always exists
    feedback_texture_b: Option<Texture>,   // For ping-pong mode
    current_feedback_idx: usize,           // 0 = read A/write B, 1 = read B/write A
}

fn render(&mut self) {
    // Determine read/write targets based on current index
    let feedback_read_view = if self.current_feedback_idx == 0 {
        &self.feedback_texture_a.view
    } else {
        &self.feedback_texture_b.as_ref().unwrap().view
    };
    
    let feedback_write_view = if self.current_feedback_idx == 0 {
        &self.feedback_texture_b.as_ref().unwrap().view
    } else {
        &self.feedback_texture_a.view
    };
    
    // Render to write target
    self.do_render_pass(feedback_write_view);
    
    // Swap for next frame
    self.current_feedback_idx = 1 - self.current_feedback_idx;
}
```

### Shader Feedback Processing

The shader applies transformations to feedback UVs:

```wgsl
// Rotate and zoom UV coordinates
fn transform_uv(uv: vec2<f32>, rotate_lfo: f32, zoom_lfo: f32, ...) -> vec2<f32> {
    // Center UVs
    var result = uv - vec2<f32>(0.5);
    
    // Apply rotation (LFO + manual)
    let total_rotate = rotate_lfo * 2.0 * 3.14159265 * rotate_amount + manual_rotate;
    let cos_r = cos(total_rotate);
    let sin_r = sin(total_rotate);
    // ... rotation matrix
    
    // Apply zoom (LFO + manual)
    let lfo_zoom = 1.0 + (zoom_lfo - 0.5) * zoom_amount;
    let total_zoom = lfo_zoom * manual_zoom;
    result = result / total_zoom;
    
    // Apply translation
    result = result - manual_translate;
    
    // Uncenter
    return result + vec2<f32>(0.5);
}
```

### Feedback Mix Equation

```wgsl
// Sample input and transformed feedback
let input_color = textureSample(input_tex, input_sampler, texcoord);
let feedback_color = textureSample(feedback_tex, feedback_sampler, feedback_uv);

// Apply hue shift to feedback
let shifted_feedback = apply_hue_shift(feedback_color.rgb, uniforms.hue_lfo, uniforms.hue_amount);

// Mix input with hue-shifted feedback
let output_rgb = mix(input_color.rgb, shifted_feedback, uniforms.feedback_amount);
```

---

## Tap Tempo System

### Algorithm

The tap tempo uses a sliding window of tap times:

1. **Record Tap**: Store timestamp in `VecDeque<Instant>` (max 8 entries)
2. **Auto-Reset**: Clear if >2 seconds since last tap
3. **LFO Phase Reset**: Every tap resets LFO phase to 0
4. **BPM Calculation**: Requires 4+ taps

```rust
fn handle_tap_tempo(&mut self) {
    let now = Instant::now();
    
    // Clear if too long since last tap
    if let Some(&last_tap) = self.tap_times.back() {
        if now.duration_since(last_tap).as_secs_f32() > 2.0 {
            self.tap_times.clear();
        }
    }
    
    self.tap_times.push_back(now);
    
    // Keep only last 8 taps
    if self.tap_times.len() > 8 {
        self.tap_times.pop_front();
    }
    
    // Reset LFO phase on every tap
    engine.reset_lfo_phase();
    
    // Calculate BPM with 4+ taps
    if self.tap_times.len() >= 4 {
        let intervals: Vec<f32> = taps.windows(2)
            .map(|w| w[1].duration_since(w[0]).as_secs_f32())
            .collect();
        
        let avg_interval = intervals.iter().sum::<f32>() / intervals.len() as f32;
        let bpm = 60.0 / avg_interval;
        self.current_bpm = bpm.clamp(40.0, 200.0);
    }
}
```

### LFO Phase Reset

Resetting phase on each tap creates rhythmic synchronization:

```rust
pub fn reset_lfo_phase(&mut self) {
    self.lfo_bank.reset_all();  // Sets all LFO phases to 0
}
```

This means:
- Hue shift starts from 0 on each tap
- Rotation starts from center position
- Zoom starts from neutral

### BPM to LFO Frequency

```rust
// LFO frequency = BPM / 60.0 * beats_per_cycle
// For 1 cycle per beat at 120 BPM: 120/60 * 1 = 2.0 Hz
pub fn update(&mut self, delta_time: f32) {
    let beats_per_second = self.bpm / 60.0;
    let phase_increment = delta_time * beats_per_second * self.cycles_per_beat;
    self.phase = (self.phase + phase_increment) % 1.0;
    
    // Map to sine wave 0-1 range
    self.value = (self.phase * 2.0 * PI).sin() * 0.5 + 0.5;
}
```

---

## Texture Format Handling

### The Format Mismatch Problem

On macOS with Metal:
- Surface format is typically `Bgra8UnormSrgb`
- Default texture format is `Rgba8Unorm`
- These are incompatible for `copy_texture_to_texture`

### Solution: Unified Format

All textures use the surface format:

```rust
let surface_format = surface_caps.formats[0]; // Bgra8UnormSrgb on macOS

// All render targets use surface_format
let output_texture = Texture::create_render_target_with_format(
    &device, width, height, "Output", surface_format,
);
let feedback_texture = Texture::create_render_target_with_format(
    &device, width, height, "Feedback", surface_format,
);
let input_texture = Texture::create_render_target_with_format(
    &device, width, height, "Input", surface_format,
);
```

### Texture Usage Flags

```rust
wgpu::TextureUsages::TEXTURE_BINDING |  // Can be sampled in shader
wgpu::TextureUsages::RENDER_ATTACHMENT   // Can be render target
```

**Note**: Surface textures cannot have `COPY_DST` - must use render pass for blitting.

### Blit Pipeline

For presenting feedback texture to surface:

```rust
// Create blit pipeline with surface format
let blit_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
    // ...
    fragment: Some(wgpu::FragmentState {
        targets: &[Some(wgpu::ColorTargetState {
            format: surface_format,  // Match surface
            // ...
        })],
    }),
});

// Render pass blits texture to surface
fn present_output(&self) {
    let blit_bind_group = self.create_blit_bind_group(source_view);
    
    let mut render_pass = encoder.begin_render_pass(...);
    render_pass.set_pipeline(&self.blit_pipeline);
    render_pass.set_bind_group(0, &blit_bind_group, &[]);
    render_pass.draw(0..6, 0..1);  // Full-screen quad
}
```

---

## Shader Pipeline

### WGSL Struct Alignment

Critical: WGSL struct layout must match Rust exactly:

```rust
#[repr(C, align(16))]
pub struct SimpleFeedbackUniforms {
    // 16 bytes - all f32, no padding needed
    pub width: f32,
    pub height: f32,
    pub inv_width: f32,
    pub inv_height: f32,
    
    // 16 bytes
    pub hue_lfo: f32,
    pub rotate_lfo: f32,
    pub zoom_lfo: f32,
    pub _pad1: f32,  // Explicit padding
    
    // 16 bytes
    pub feedback_amount: f32,
    pub hue_amount: f32,
    pub rotate_amount: f32,
    pub zoom_amount: f32,
    
    // ... more fields
}
```

```wgsl
struct SimpleFeedbackUniforms {
    width: f32,
    height: f32,
    inv_width: f32,
    inv_height: f32,
    
    hue_lfo: f32,
    rotate_lfo: f32,
    zoom_lfo: f32,
    _pad1: f32,  // Must match Rust
    
    feedback_amount: f32,
    hue_amount: f32,
    rotate_amount: f32,
    zoom_amount: f32,
    
    // ...
}
```

### Vertex Buffer Layout

```rust
// Position (x, y) + Texcoord (u, v) = 4 floats per vertex
let vertices: &[f32] = &[
    // Tri 1
    -1.0, -1.0, 0.0, 1.0,  // bottom-left
     1.0, -1.0, 1.0, 1.0,  // bottom-right
    -1.0,  1.0, 0.0, 0.0,  // top-left
    // Tri 2
     1.0, -1.0, 1.0, 1.0,  // bottom-right
     1.0,  1.0, 1.0, 0.0,  // top-right
    -1.0,  1.0, 0.0, 0.0,  // top-left
];

fn simple_vertex_desc() -> wgpu::VertexBufferLayout<'static> {
    wgpu::VertexBufferLayout {
        array_stride: 4 * std::mem::size_of::<f32>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &[
            wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x2, // position
            },
            wgpu::VertexAttribute {
                offset: 2 * std::mem::size_of::<f32>() as wgpu::BufferAddress,
                shader_location: 1,
                format: wgpu::VertexFormat::Float32x2, // texcoord
            },
        ],
    }
}
```

### HSB/HSL Color Space

Hue shift uses HSB (Hue-Saturation-Brightness):

```wgsl
fn rgb2hsb(c: vec3<f32>) -> vec3<f32> {
    // ... conversion logic
    return vec3<f32>(hue, saturation, brightness);
}

fn hsb2rgb(c: vec3<f32>) -> vec3<f32> {
    // ... conversion logic
    return rgb;
}

fn apply_hue_shift(color: vec3<f32>, lfo_value: f32, amount: f32) -> vec3<f32> {
    var hsb = rgb2hsb(color);
    let hue_shift = lfo_value * 2.0 * 3.14159265 * amount;
    hsb.x = fract(hsb.x + hue_shift);
    return hsb2rgb(hsb);
}
```

---

## Common Issues & Solutions

### "Buffer size mismatch" Error

**Symptom**: `Buffer is bound with size X where the shader expects Y`

**Cause**: Rust struct size doesn't match WGSL struct size

**Solution**:
1. Check `#[repr(C, align(16))]` on Rust struct
2. Ensure explicit padding fields match
3. Verify no `Vec3` (use `[f32; 3]` with alignment)

### "Render pipeline targets are incompatible" Error

**Symptom**: `Incompatible color attachments at indices [0]`

**Cause**: Render pass texture format doesn't match pipeline format

**Solution**: Ensure all textures use the same format (surface format):
```rust
// When recreating textures on resize
let surface_format = self.config.format;
self.output_texture = Texture::create_render_target_with_format(
    &self.device, width, height, "Output", surface_format,
);
```

### "Device lost" Error (macOS)

**Symptom**: `RequestDeviceError { inner: Core(Device(Lost)) }`

**Cause**: Creating multiple wgpu devices on macOS Metal

**Solution**: Share device/queue between windows:
```rust
// Don't do this:
let (device2, queue2) = adapter.request_device(...).await; // CRASH!

// Do this:
let device = Arc::clone(&engine.device());
let queue = Arc::clone(&engine.queue());
```

### "Invalid DisplaySize value" Error (ImGui)

**Symptom**: `Assertion failed: (g.IO.DisplaySize.x >= 0.0f)`

**Cause**: ImGui context not initialized with display size

**Solution**: Set display size before first frame:
```rust
let size = window.inner_size();
imgui_renderer.set_display_size(size.width as f32, size.height as f32);
```

### Black Screen / No Feedback

**Possible causes**:
1. Shader returning debug output (raw input only)
2. Feedback texture never cleared (starts with garbage)
3. Feedback amount set to 0.0
4. UV transforms pushing feedback outside 0-1 range

**Solutions**:
```rust
// Clear feedback textures on startup
feedback_texture_a.clear_to_black(&queue);
feedback_texture_b.clear_to_black(&queue);

// In shader, check UV bounds
if (feedback_uv.x < 0.0 || feedback_uv.x > 1.0 ||
    feedback_uv.y < 0.0 || feedback_uv.y > 1.0) {
    // Out of bounds - show input only
    return vec4<f32>(input_color.rgb, 1.0);
}
```

### Camera Not Starting

**Symptom**: Black screen, no camera input

**Causes**:
1. Camera permission not granted (macOS)
2. Wrong resolution requested (camera may not support 640x480)
3. YUYV format not converted properly

**Solution**: Check camera capabilities and convert format:
```rust
// Camera may report 1280x720 even when requesting 640x480
let (width, height) = self.input_manager.get_input1_resolution();
// Create texture with actual resolution, not requested
```

---

## Extension Guide

### Adding New Parameters

1. **Add to Uniforms Struct**:
```rust
// In simple_feedback.rs
#[repr(C, align(16))]
pub struct SimpleFeedbackUniforms {
    // ... existing fields
    pub new_param: f32,
    pub _pad: f32,  // Maintain 16-byte alignment
}
```

2. **Add to WGSL Shader**:
```wgsl
struct SimpleFeedbackUniforms {
    // ... existing fields
    new_param: f32,
    _pad: f32,
}

@fragment
fn fs_main(@location(0) texcoord: vec2<f32>) -> @location(0) vec4<f32> {
    // Use new_param in shader
    let adjusted = input_color * uniforms.new_param;
    // ...
}
```

3. **Add UI Control**:
```rust
// In simple_main.rs ImGui render
ui.slider("##new_param", 0.0, 1.0, &mut self.new_param);
if let Some(ref mut engine) = self.engine {
    engine.set_new_param(self.new_param);
}
```

### Adding Audio Reactivity

1. **Access FFT Data**:
```rust
// In render loop
let fft_values = self.input_manager.get_audio_fft();
let bass_energy = fft_values[0];  // Low frequencies
```

2. **Modulate Parameter**:
```rust
let modulated_hue = base_hue + bass_energy * modulation_amount;
params.hue_lfo = modulated_hue;
```

### Supporting Multiple Inputs

The engine supports Input 1 and Input 2:

```rust
// In shader, add second input texture
@group(0) @binding(5)
var input2_tex: texture_2d<f32>;
@group(0) @binding(6)
var input2_sampler: sampler;

// Mix inputs
let input1 = textureSample(input_tex, input_sampler, texcoord);
let input2 = textureSample(input2_tex, input2_sampler, texcoord);
let mixed_input = mix(input1, input2, input_mix);
```

### Switching Feedback Modes

Add different feedback algorithms:

```rust
pub enum FeedbackMode {
    Standard,      // Current implementation
    Kaleidoscope,  // Mirror/rotate quadrants
    Delay,         // Temporal delay buffer
    Trails,        // Motion blur accumulation
}

fn apply_feedback(mode: FeedbackMode, uv: vec2<f32>) -> vec2<f32> {
    switch mode {
        case Kaleidoscope: return kaleidoscope_uv(uv);
        case Delay: return delay_uv(uv);
        // ...
    }
}
```

---

## Performance Considerations

### Texture Size

- Match output resolution (1280x720 = ~3.5MB per texture)
- Ping-pong uses 2x memory
- Consider half-resolution feedback for performance

### Shader Complexity

- HSB conversion is expensive (trigonometry)
- Consider LUT (Look-Up Table) for hue shift
- Minimize branching in fragment shader

### Update Frequency

- LFOs update every frame
- Parameter updates only when changed
- Uniform buffer updates via `queue.write_buffer()`

---

## Debugging Tips

### Enable Debug Logging

```bash
RUST_LOG=info cargo run -- --simple
```

### Visualize Feedback UVs

```wgsl
// In shader, uncomment to debug:
return vec4<f32>(feedback_uv.x, feedback_uv.y, 0.0, 1.0);
```

### Check Texture Contents

```rust
// Save texture to file for inspection
let texture_data = self.read_texture_data(&feedback_texture);
save_as_png(&texture_data, "debug_feedback.png");
```

### Frame Timing

```rust
let start = Instant::now();
engine.render();
let frame_time = start.elapsed();
println!("Frame: {:?} ({:.1} FPS)", frame_time, 1.0 / frame_time.as_secs_f32());
```

---

## References

- [WGSL Spec](https://www.w3.org/TR/WGSL/)
- [wgpu Documentation](https://docs.rs/wgpu/)
- [imgui-rs Documentation](https://docs.rs/imgui/)
- [nokhwa Webcam Library](https://docs.rs/nokhwa/)

---

## Glossary

- **Ping-pong**: Alternating between two buffers for read/write
- **LFO**: Low Frequency Oscillator (slow cyclic modulation)
- **HSB**: Hue-Saturation-Brightness color space
- **UV Coordinates**: Texture coordinates (0-1 range)
- **Blit**: Copying/rendering one texture to another
- **Feedback**: Using previous frame output as current input
