# Inter-Process Video Sharing Design

## Overview

This document outlines the architecture for adding Syphon (macOS), Spout (Windows), and v4l2loopback (Linux) support to RustJay Waaaves. The goal is a unified, performant abstraction that enables zero-copy GPU texture sharing where possible.

## Platform Comparison

| Feature | Syphon (macOS) | Spout (Windows) | v4l2loopback (Linux) |
|---------|---------------|-----------------|---------------------|
| API | OpenGL-based | DirectX/OpenGL | V4L2 Video API |
| GPU Sharing | Yes (IOSurface) | Yes (DX11/DX12/GL) | No (CPU buffers) |
| Discovery | Bonjour/Local | Active sender list | Device nodes (/dev/videoX) |
| Format | RGBA/BGRA | RGBA/BGRA/NV12 | Various (RGB24, YUYV) |
| Latency | Ultra-low | Ultra-low | Low |
| Rust Crates | `syphon` (objc) | `spout-rs` | `v4l2-sys`, `v4l-rs` |

## Architecture Design

### Core Abstraction

```
┌─────────────────────────────────────────────────────────────────┐
│                     IPC Video Framework                          │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐             │
│  │   Input     │  │   Output    │  │  Discovery  │             │
│  │   Trait     │  │   Trait     │  │   Trait     │             │
│  └──────┬──────┘  └──────┬──────┘  └─────────────┘             │
│         │                │                                       │
│  ┌──────┴──────┐  ┌──────┴──────┐                               │
│  │ Platform    │  │ Platform    │                               │
│  │ Input Impls │  │ Output Impls│                               │
│  ├─────────────┤  ├─────────────┤                               │
│  │ • Syphon    │  │ • Syphon    │  (macOS)                      │
│  │ • Spout     │  │ • Spout     │  (Windows)                    │
│  │ • V4L2      │  │ • V4L2      │  (Linux)                      │
│  └─────────────┘  └─────────────┘                               │
└─────────────────────────────────────────────────────────────────┘
```

### Trait Definitions

```rust
// Core input trait - unified interface for all IPC inputs
pub trait IpcInput: Send {
    fn connect(&mut self, source: &str) -> Result<()>;
    fn disconnect(&mut self);
    fn is_connected(&self) -> bool;
    fn receive_frame(&mut self) -> Option<IpcFrame>;
    fn resolution(&self) -> Option<(u32, u32)>;
}

// Core output trait - unified interface for all IPC outputs
pub trait IpcOutput: Send {
    fn create_server(&mut self, name: &str, width: u32, height: u32) -> Result<()>;
    fn destroy_server(&mut self);
    fn is_active(&self) -> bool;
    fn send_frame(&mut self, texture: &wgpu::Texture, encoder: &mut wgpu::CommandEncoder) -> Result<()>;
}

// Frame data that works across all platforms
pub enum IpcFrame {
    // GPU texture handle (Syphon/Spout)
    GpuTexture(GpuTextureHandle),
    // CPU buffer (v4l2loopback, fallback)
    CpuBuffer {
        data: Vec<u8>,
        format: PixelFormat,
        width: u32,
        height: u32,
    },
}

// Platform-specific GPU handle
pub enum GpuTextureHandle {
    #[cfg(target_os = "macos")]
    Syphon(SyphonHandle),
    #[cfg(target_windows)]
    Spout(SpoutHandle),
}
```

## Implementation Strategy

### Phase 1: macOS Syphon Support

**Input (Syphon Client)**
- Use `objc` crate for Objective-C interop
- Bind to Syphon framework (`/System/Library/Frameworks/Syphon.framework`)
- Implement `SyphonInput` struct wrapping `SyphonClient`

**Output (Syphon Server)**
- Create `SyphonOutput` wrapping `SyphonServer`
- Share wgpu Metal texture directly via `IOSurface`

**Key Implementation Details:**
```rust
#[cfg(target_os = "macos")]
pub mod syphon;

// Syphon uses IOSurface for zero-copy texture sharing
// wgpu Metal backend uses IOSurface-backed textures
// We can extract the IOSurface from wgpu and pass to Syphon
```

### Phase 2: Windows Spout Support

**Input (Spout Receiver)**
- Use `spout-rs` or custom bindings to Spout SDK
- Support both DirectX and OpenGL backends

**Output (Spout Sender)**
- Create shared DirectX texture
- wgpu DX12 backend can share textures via `ID3D12Resource` handles

**Key Implementation Details:**
```rust
#[cfg(target_os = "windows")]
pub mod spout;

// Spout supports DX11, DX12, and OpenGL sharing
// wgpu on Windows uses DX12 by default
// Need to use Spout's DX12 interface
```

### Phase 3: Linux v4l2loopback Support

**Input (V4L2 Capture)**
- Use `v4l-rs` for video4linux2 API
- Read from virtual video device

**Output (V4L2 Output)**
- Write to v4l2loopback device (`/dev/videoX`)
- Requires `v4l2loopback` kernel module

**Key Implementation Details:**
```rust
#[cfg(target_os = "linux")]
pub mod v4l2;

// v4l2loopback uses CPU buffers
// Need RGB24 or YUYV format conversion
// Higher latency than GPU sharing but standard Linux approach
```

## GPU Texture Sharing Details

### macOS (Metal + IOSurface)

```rust
// Extract IOSurface from wgpu Metal texture
// Share with Syphon

use wgpu::hal::metal::Surface as MetalSurface;
use core_graphics::iosurface::IOSurface;

pub fn share_wgpu_texture_with_syphon(
    texture: &wgpu::Texture,
    syphon_server: &mut SyphonServer,
) {
    // wgpu Metal textures are backed by IOSurface
    // Need to access the underlying CAMetalDrawable/MTLTexture
    // Then get the IOSurfaceRef
    
    // This requires wgpu HAL access or a custom surface
    // Alternative: Create texture with shared IOSurface
}
```

### Windows (DirectX 12)

```rust
// Share wgpu DX12 texture with Spout
use wgpu::hal::dx12;

pub fn share_wgpu_texture_with_spout(
    texture: &wgpu::Texture,
    spout_sender: &mut SpoutSender,
) {
    // wgpu DX12 textures are ID3D12Resource
    // Spout can use D3D12 shared handles
    // Need to create texture with D3D12_HEAP_FLAG_SHARED
}
```

## Integration with Existing Code

### Input Integration

Update `InputType` enum in `src/input/mod.rs`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputType {
    None,
    Webcam,
    Ndi,
    Syphon,    // macOS only
    Spout,     // Windows only
    V4L2,      // Linux only
    VideoFile,
}
```

Update `InputSource` struct:

```rust
pub struct InputSource {
    // ... existing fields ...
    
    /// IPC input receiver (Syphon/Spout/V4L2)
    #[cfg(target_os = "macos")]
    syphon_input: Option<SyphonInput>,
    #[cfg(target_os = "windows")]
    spout_input: Option<SpoutInput>,
    #[cfg(target_os = "linux")]
    v4l2_input: Option<V4L2Input>,
}
```

### Output Integration

Update `src/output/mod.rs`:

```rust
pub mod ndi_sender;
pub use ndi_sender::{NdiOutputSender, is_ndi_output_available};

pub mod ndi_async;
pub use ndi_async::AsyncNdiOutput;

// Platform-specific IPC outputs
#[cfg(target_os = "macos")]
pub mod syphon_output;
#[cfg(target_os = "macos")]
pub use syphon_output::SyphonOutput;

#[cfg(target_os = "windows")]
pub mod spout_output;
#[cfg(target_os = "windows")]
pub use spout_output::SpoutOutput;

#[cfg(target_os = "linux")]
pub mod v4l2_output;
#[cfg(target_os = "linux")]
pub use v4l2_output::V4L2Output;
```

## Dependencies to Add

### Cargo.toml updates:

```toml
[dependencies]
# Platform-specific IPC support
[target.'cfg(target_os = "macos")'.dependencies]
objc = "0.2"
core-foundation = "0.9"
core-graphics = "0.23"
foreign-types = "0.5"

[target.'cfg(target_os = "windows")'.dependencies]
spout-rs = { version = "0.1", optional = true }
winapi = { version = "0.3", features = ["d3d11", "d3d12", "dxgi"] }

[target.'cfg(target_os = "linux")'.dependencies]
v4l = "0.14"
v4l2-sys = "0.3"

[features]
default = ["webcam", "ipc"]
webcam = ["nokhwa"]
ipc = ["spout"]  # Platform-specific, enabled by default on Windows
```

## GUI Integration

### Inputs Tab

Add platform-specific input sections:

```rust
// macOS
ui.text("Syphon Sources:");
if ui.button("Refresh Syphon") {
    // Scan for Syphon servers
}
// List available Syphon servers

// Windows
ui.text("Spout Senders:");
if ui.button("Refresh Spout") {
    // Scan for Spout senders
}
// List available Spout senders

// Linux
ui.text("V4L2 Devices:");
if ui.button("Refresh V4L2") {
    // List /dev/video* devices
}
```

### Settings Tab

Add IPC output controls:

```rust
// macOS Syphon Output
checkbox("Enable Syphon Output", &mut config.syphon_output_enabled);
input_text("Syphon Server Name", &mut config.syphon_server_name);

// Windows Spout Output
checkbox("Enable Spout Output", &mut config.spout_output_enabled);
input_text("Spout Sender Name", &mut config.spout_sender_name);

// Linux v4l2loopback Output
checkbox("Enable V4L2 Output", &mut config.v4l2_output_enabled);
combo_box("V4L2 Device", &v4l2_devices, &mut selected_device);
```

## Performance Considerations

### Zero-Copy Path (Syphon/Spout)

```
App A (RustJay)          Shared GPU Texture          App B (Resolume/OBS)
     │                            │                            │
     │ wgpu render ───────────────▶│                            │
     │ (Metal/DX12 texture)       │                            │
     │                            │◀───────────────────────────│
     │                            │       Syphon/Spout read    │
     │                            │       (zero copy)          │
```

### CPU Copy Path (v4l2loopback/NDI fallback)

```
App A (RustJay)         GPU Readback         CPU Buffer           v4l2 Device
     │                       │                    │                    │
     │ wgpu render ─────────▶│                    │                    │
     │                       │ async map ────────▶│                    │
     │                       │                    │ write to /dev/video│
     │                       │                    │◀───────────────────│
```

## Implementation Timeline

### Week 1: Foundation
- Create `src/ipc/` module structure
- Define core traits (`IpcInput`, `IpcOutput`)
- Add platform-specific feature flags

### Week 2: macOS Syphon
- Implement `SyphonInput` using objc bindings
- Implement `SyphonOutput` with IOSurface sharing
- Test with Resolume Avenue, MadMapper

### Week 3: Windows Spout
- Research Spout SDK integration options
- Implement `SpoutInput` and `SpoutOutput`
- Test with TouchDesigner, Arena

### Week 4: Linux v4l2loopback
- Implement `V4L2Input` and `V4L2Output`
- Handle format conversion (RGB→YUYV)
- Test with OBS, FFmpeg

### Week 5: Integration & Polish
- GUI integration for all platforms
- Performance optimization
- Documentation and examples

## Testing Strategy

### macOS Syphon Test Matrix
- [ ] Receive from Resolume Avenue
- [ ] Receive from MadMapper
- [ ] Receive from Quartz Composer
- [ ] Send to Resolume Avenue
- [ ] Send to OBS (via Syphon plugin)
- [ ] Send to Millumin

### Windows Spout Test Matrix
- [ ] Receive from TouchDesigner
- [ ] Receive from Arena
- [ ] Receive from Magic Music Visuals
- [ ] Send to TouchDesigner
- [ ] Send to OBS (built-in Spout support)
- [ ] Send to Resolume

### Linux v4l2loopback Test Matrix
- [ ] Receive from FFmpeg
- [ ] Receive from GStreamer
- [ ] Send to OBS
- [ ] Send to FFmpeg capture
- [ ] Send to browsers (via webcam API)

## Open Questions

1. **Metal/DX12 Interop**: Can we access wgpu's underlying Metal/DX12 textures for sharing?
   - Option A: Fork wgpu to expose IOSurface/D3D12Resource
   - Option B: Use custom surface implementation
   - Option C: Accept one GPU→CPU→GPU copy for compatibility

2. **Spout SDK**: Use existing Rust bindings or create new ones?
   - `spout-rs` exists but may need updates
   - Could use FFI to official Spout SDK

3. **Syphon Framework**: Handle framework loading gracefully on non-macOS systems?
   - Use weak framework linking
   - Runtime availability check

## Next Steps

1. **Prototype macOS Syphon** - Start with CPU copy path to validate architecture
2. **Research wgpu texture interop** - Investigate HAL access for zero-copy
3. **Set up CI for all platforms** - Ensure cross-platform builds work
4. **Create minimal test app** - Verify IPC works before full integration
