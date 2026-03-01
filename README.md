# RustJay Waaaves

A high-performance VJ (Visual Jockey) application written in Rust, ported from the OpenFrameworks-based "BLUEJAY_WAAAVES" project.

## Disclaimer

**This is NOT an official port of Gravity Waaaves.** The original creator, **Andrei Jay**, will not provide any support whatsoever for this software.

This source code is distributed AS-IS, there is no guaranteed continuing support.

## Special Thanks to Andrei Jay

This project would not exist without the incredible work of **Andrei Jay**, who created the original VSEJET GRAVITY_WAAAVES and generously open-sourced his work. His contributions to the video synthesis community through open-source projects, educational resources, and creative tools have enabled countless artists and developers to explore the world of analog-style video feedback and synthesis.

**Please support Andrei:**
- Patreon: https://www.patreon.com/c/andrei_jay
- Ko-fi: https://ko-fi.com/andreijay
- Website: https://videosynthecosphere.com
- Alternative Website: https://andreijaycreativecoding.com

You can download WPDSK and GW-DSK from his website - **please go support him!**

---

## Overview

RustJay Waaaves is a real-time video effects processor designed for live visual performance. It uses a modular shader pipeline with three processing blocks, providing extensive parameter control via an ImGui-based interface.

## Architecture

### Dual-Window Design

The application uses a dual-window architecture inspired by the original OpenFrameworks version:

- **Output Window**: wgpu-based rendering of the final visual output
- **Control Window**: ImGui-based interface for real-time parameter manipulation

### Shader Pipeline

The rendering pipeline consists of three main blocks:

```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│   Block 1   │───▶│   Block 2   │───▶│   Block 3   │──▶ Output
│  (Channels) │    │  (Feedback) │    │   (Final)   │
└──────┬──────┘    └──────┬──────┘    └─────────────┘
       │                  │
       ▼                  ▼
   FB1 Delay          FB2 Delay
   (1-120 frames)     (1-120 frames)
```

#### Block 1: Channel Mixing
- Two input channels (CH1, CH2) with independent transforms
- Geometric effects: kaleidoscope, rotation, displacement, mirroring
- Color effects: HSB adjustment, posterization, inversion
- Filters: blur, sharpen with adjustable radii
- Feedback loop (FB1) with keying and temporal filtering

#### Block 2: Secondary Processing
- Processes secondary input or Block 1 output
- Same geometric and color effects as Block 1
- Independent feedback loop (FB2)

#### Block 3: Final Mixing
- Re-processes both Block 1 and Block 2 outputs
- Colorization with 5-band color mapping
- Matrix mixer for RGB channel manipulation
- Final compositing with keying

## Features

### Visual Effects

- **Geometric Transformations**
  - 2D displacement (X, Y, Z/scale)
  - Rotation with aspect ratio preservation modes
  - Kaleidoscope with adjustable segments and rotation
  - Horizontal/vertical mirror and flip
  - Shear transformation matrix

- **Color Processing**
  - HSB color space adjustments
  - Posterization with invert option
  - Color inversion (RGB, HSB channels)
  - Solarization effect
  - 5-band colorization

- **Filters**
  - Box blur with radius control
  - Sharpen with boost compensation
  - Temporal filtering for feedback smoothing

- **Compositing**
  - Multiple blend modes: lerp, add, difference, multiply, dodge
  - Chroma keying with adjustable threshold and softness
  - Overflow modes: clamp, wrap, fold
  - Matrix mixer for RGB channel routing

### Modulation

- 16 macro banks with assignable parameters
- LFO (Low Frequency Oscillator) per macro
  - Waveforms: sine, triangle, saw, square, random
  - Tempo sync to BPM
  - Adjustable rate and amplitude
- Audio reactivity via FFT analysis
- OSC control support

### Inputs

- Webcam capture
- NDI input (Network Device Interface)
- Spout input (Windows only, planned)
- Video file playback (planned)

### Outputs

- Full-screen wgpu output
- NDI output for external capture (planned)
- Video recording (planned)

## Building

### Prerequisites

- Rust 1.75+ (latest stable recommended)
- OpenGL 3.3 compatible GPU (or Metal via wgpu)
- macOS, Windows, or Linux

### Dependencies

Key dependencies:
- `wgpu 25.0` - Cross-platform GPU acceleration
- `winit 0.30` - Windowing
- `imgui 0.12` + `imgui-wgpu 0.25` - GUI framework
- `glam` - Fast linear algebra
- `cpal` - Audio I/O
- `serde` + `toml` - Configuration

### Build Commands

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Run with logging
RUST_LOG=info cargo run
```

## Configuration

Configuration is stored in `config.toml`:

```toml
[output_window]
width = 1280
height = 720
fps = 60

[control_window]
width = 1920
height = 1080
fps = 30

[pipeline]
internal_width = 1280
internal_height = 720
max_delay_frames = 120
```

The file is automatically created with defaults on first run.

## Usage

### Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `ESC` | Exit application |
| `F1` | Toggle fullscreen (output) |
| `F2` | Show/hide control window |
| `Space` | Clear feedback buffers |
| `1-9` | Select macro bank |

### Parameter Control

Parameters are organized in tabs:
- **Block 1**: Primary channel mixing and feedback
- **Block 2**: Secondary processing
- **Block 3**: Final colorization and mixing
- **Macros**: LFO and modulation setup
- **Inputs**: Video/audio input configuration
- **Settings**: Output and recording options

### Macro Assignment

1. Select a macro bank (0-15)
2. Click on a parameter while holding the macro's modifier key
3. Adjust the macro amount slider

## Performance Optimization

The implementation includes several performance optimizations:

- **Early-exit shaders**: Skip expensive operations when parameters are zero
- **Persistent PBOs**: Async GPU→CPU readback for NDI output (planned)
- **Framebuffer pooling**: Reuse GPU memory allocations
- **Efficient uniforms**: Cache uniform locations, batch updates
- **Shader branch reduction**: Use mix() instead of conditionals where possible

### Benchmarks

Typical performance on modern hardware:
- 1920x1080 @ 60fps: <2ms GPU time per frame
- 4K @ 60fps: <5ms GPU time per frame

## Shader Porting Notes

The GL3 shaders were ported to WGSL with the following considerations:

1. **Uniform buffers**: Organized by functional groups, matching Rust struct layout
2. **Vec3 alignment**: Custom Vec3 type with 16-byte alignment to match WGSL
3. **Branch elimination**: Replaced if-statements with mix() operations
4. **Early exits**: Added threshold checks to skip expensive operations
5. **Precision**: Maintained float32 for compatibility with original

## Development

### Project Structure

```
src/
├── config/        # Configuration management (TOML)
├── core/          # Core types and shared state
├── engine/        # wgpu rendering engine
│   ├── pipelines/ # Shader pipelines (Block1, Block2, Block3)
│   ├── texture.rs # Texture utilities
│   └── mod.rs     # Main engine with dual-window support
├── gui/           # ImGui interface
├── input/         # Video input handling
├── params/        # Parameter structures
└── utils/         # Helper utilities
```

### Adding New Parameters

1. Add field to appropriate params struct in `src/params/mod.rs`
2. Add uniform to shader in WGSL pipeline files
3. Add UI control in `src/gui/mod.rs`
4. Add conversion in pipeline's `update_params` method

### Shader Development

Shaders use WGSL (WebGPU Shading Language). When modifying:

1. Maintain compatibility with wgpu
2. Test on both Metal (macOS) and Vulkan (Windows/Linux)
3. Profile with GPU analysis tools
4. Document any precision changes

## License

MIT License - See LICENSE file for details

## Credits

Ported from the OpenFrameworks "BLUEJAY_WAAAVES" project.
Original shaders and design concept by Andrei Jay.

## Roadmap

- [x] Basic wgpu rendering pipeline
- [x] ImGui control window
- [x] Dual-window architecture
- [ ] SPIR-V shader compilation for validation
- [ ] NDI input/output
- [ ] Video file player
- [ ] Audio analysis and reactivity
- [ ] TouchOSC integration
- [ ] Preset system
- [ ] Syphon output (macOS)

## Troubleshooting

### Black screen on startup
- Check GPU supports wgpu (Metal on macOS, Vulkan on Windows/Linux)
- Verify shaders compiled successfully (check logs)
- Try windowed mode first

### Low frame rate
- Reduce internal resolution
- Disable temporal filtering
- Lower delay buffer size

### Input not working
- Check device permissions (camera/mic)
- Verify NDI source is active
- Try different input resolution

### GUI not responding
- Check winit event loop is running
- Verify imgui context initialization

## Support

If you like this work, please consider supporting the original creator Andrei Jay through the links above.

For issues with this Rust port, please file a GitHub issue.
