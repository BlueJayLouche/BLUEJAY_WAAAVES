# IPC Video Sharing Implementation Guide

## Overview

This guide explains how to complete the implementation of Syphon (macOS), Spout (Windows), and v4l2loopback (Linux) support in RustJay Waaaves.

## Current Status

✅ **Completed:**
- Core trait abstractions (`IpcInput`, `IpcOutput`, `IpcDiscovery`)
- Module structure with platform-specific implementations
- Framework implementations for all three platforms
- Cargo.toml with platform-specific dependencies
- **Syphon Output Framework:**
  - `SyphonSender` with background publish thread (mirrors `NdiOutputSender`)
  - `AsyncSyphonOutput` with triple-buffered GPU readback (mirrors `AsyncNdiOutput`)
  - `SyphonOutputIntegration` for engine integration
- **Syphon Input Framework:**
  - `SyphonInputReceiver` with background receive thread
  - `SyphonDiscovery` for server enumeration
  - `SyphonInputIntegration` for engine integration

🔄 **Remaining:**
- **Objective-C bindings** for Syphon.framework
- GPU texture interop with wgpu Metal backend
- GUI integration (Inputs/Settings tabs)
- Testing and validation with Resolume/MadMapper
- Spout and v4l2loopback implementation (follow Syphon pattern)

---

## Platform-Specific Implementation

### macOS Syphon

#### Prerequisites

```bash
# Syphon.framework is included with many VJ apps
# Or install via: https://github.com/Syphon/Syphon-Framework
```

#### Implementation Steps

1. **Add Objective-C Runtime Bindings**

Create `src/ipc/syphon_bind.rs`:

```rust
//! Objective-C bindings for Syphon framework

use objc::runtime::{Object, Class, Sel};
use objc::{msg_send, sel, sel_impl};
use core_foundation::base::TCFType;
use core_graphics::iosurface::IOSurface;

/// Opaque type for SyphonServer
pub enum SyphonServerRef {}

/// Opaque type for SyphonClient
pub enum SyphonClientRef {}

/// Opaque type for SyphonServerDirectory
pub enum SyphonServerDirectoryRef {}

/// Create a new Syphon server
pub unsafe fn create_syphon_server(name: &str) -> *mut Object {
    let cls = Class::get("SyphonServer").expect("SyphonServer class not found");
    let server: *mut Object = msg_send![cls, alloc];
    let nsstring = NSString::from_str(name);
    let server: *mut Object = msg_send![server, initWithName:nsstring];
    server
}

/// Publish an IOSurface to a Syphon server
pub unsafe fn publish_frame_texture(
    server: *mut Object,
    iosurface: &IOSurface,
    dims: (u32, u32),
    flip: bool,
) {
    let (width, height) = dims;
    let _: () = msg_send![server, 
        publishFrameTexture:iosurface.as_concrete_TypeRef()
        textureTarget:0  // GL_TEXTURE_RECTANGLE_EXT
        imageRegion:NSMakeRect(0.0, 0.0, width as f64, height as f64)
        textureDimensions:NSMakeSize(width as f64, height as f64)
        flipped:flip
    ];
}

/// Create a Syphon client
pub unsafe fn create_syphon_client(server_name: &str) -> *mut Object {
    let cls = Class::get("SyphonClient").expect("SyphonClient class not found");
    
    // Get server from directory
    let directory_cls = Class::get("SyphonServerDirectory").unwrap();
    let servers: *mut Object = msg_send![directory_cls, servers];
    
    // Find server by name...
    
    let client: *mut Object = msg_send![cls, alloc];
    // Initialize with server...
    client
}
```

2. **Integrate with wgpu Metal Textures**

The challenge is accessing wgpu's underlying Metal textures. Two approaches:

**Approach A: Using wgpu HAL (Direct Access)**

```rust
use wgpu::hal::metal::Device as MetalDevice;
use wgpu::hal::Texture as HalTexture;

pub fn get_metal_texture(wgpu_texture: &wgpu::Texture) -> *mut metal::MTLTexture {
    // Access HAL texture
    let hal_texture: &HalTexture<metal::Api> = unsafe {
        // This requires wgpu to expose HAL types
        std::mem::transmute(wgpu_texture)
    };
    
    // Get Metal texture from HAL texture
    hal_texture.raw
}
```

**Approach B: Custom Surface with IOSurface**

Create textures backed by IOSurface that can be shared:

```rust
use core_graphics::iosurface::{IOSurface, IOSurfaceRef};
use metal::{Device, TextureDescriptor, Texture};

pub fn create_iosurface_backed_texture(
    device: &Device,
    width: u32,
    height: u32,
) -> (Texture, IOSurface) {
    // Create IOSurface
    let properties = CFDictionary::from_CFType_pairs(&[
        // ... set IOSurface properties
    ]);
    
    let iosurface = IOSurface::new(&properties);
    
    // Create Metal texture from IOSurface
    let texture = device.new_texture_with_iosurface(
        &iosurface,
        // ... descriptor
    );
    
    (texture, iosurface)
}
```

3. **Update SyphonInput Implementation**

Replace the stub methods in `src/ipc/syphon.rs`:

```rust
impl IpcInput for SyphonInput {
    fn connect(&mut self, source: &str) -> IpcResult<()> {
        unsafe {
            let client = create_syphon_client(source);
            if client.is_null() {
                return Err(IpcError::ConnectionFailed(
                    format!("Could not connect to Syphon server: {}", source)
                ));
            }
            
            self.native_client = Some(client as *mut c_void);
            self.server_name = Some(source.to_string());
            
            // Set up frame handler
            // ...
            
            Ok(())
        }
    }
    
    fn receive_frame(&mut self) -> Option<IpcFrame> {
        unsafe {
            let client = self.native_client? as *mut Object;
            
            // Check if new frame available
            let has_new_frame: bool = msg_send![client, hasNewFrame];
            if !has_new_frame {
                return None;
            }
            
            // Get IOSurface from client
            let surface: *mut IOSurfaceRef = msg_send![client, newFrameImage];
            if surface.is_null() {
                return None;
            }
            
            let iosurface = IOSurface::wrap_under_get_rule(surface);
            
            // Lock and read pixels (or use GPU path)
            iosurface.lock(0);
            // ... read pixels
            iosurface.unlock(0);
            
            Some(IpcFrame::GpuTexture(SyphonTextureHandle {
                iosurface_id: iosurface.get_id(),
                width: self.dimensions?.0,
                height: self.dimensions?.1,
            }))
        }
    }
    // ...
}
```

---

### Windows Spout

#### Prerequisites

1. Download Spout SDK from: https://github.com/leadedge/Spout2
2. Place `Spout.dll` in project or ensure it's in PATH

#### Implementation Steps

1. **Create FFI Bindings**

Create `src/ipc/spout_ffi.rs`:

```rust
//! FFI bindings for Spout SDK

use std::os::raw::{c_char, c_int, c_void};

#[repr(C)]
pub struct SpoutSender {
    _private: [u8; 0],
}

#[repr(C)]
pub struct SpoutReceiver {
    _private: [u8; 0],
}

#[link(name = "Spout")]
extern "C" {
    // Sender functions
    pub fn GetSpout() -> *mut c_void;
    
    pub fn CreateSender(
        name: *const c_char,
        width: c_int,
        height: c_int,
        dwFormat: u32,
    ) -> bool;
    
    pub fn UpdateSender(
        name: *const c_char,
        width: c_int,
        height: c_int,
    ) -> bool;
    
    pub fn SendImage(
        buffer: *const u8,
        width: c_int,
        height: c_int,
        glFormat: u32,
        bInvert: bool,
    ) -> bool;
    
    pub fn SendTexture(
        textureID: u32,
        textureTarget: u32,
        width: c_int,
        height: c_int,
        bInvert: bool,
        hostFBO: u32,
    ) -> bool;
    
    pub fn ReleaseSender();
    
    // Receiver functions
    pub fn CreateReceiver(
        name: *const c_char,
        width: *mut c_int,
        height: *mut c_int,
        bUseActive: bool,
    ) -> bool;
    
    pub fn ReceiveImage(
        name: *const c_char,
        buffer: *mut u8,
        glFormat: u32,
        width: c_int,
        height: c_int,
        bInvert: bool,
        hostFBO: u32,
    ) -> bool;
    
    pub fn ReceiveTexture(
        name: *const c_char,
        textureID: u32,
        textureTarget: u32,
        width: c_int,
        height: c_int,
        bInvert: bool,
        hostFBO: u32,
    ) -> bool;
    
    pub fn ReleaseReceiver();
    
    // Shared texture (GPU sharing)
    pub fn EnableFrameCount();
    pub fn DisableFrameCount();
    pub fn CheckFrameCount(name: *const c_char) -> bool;
    
    // Sender list
    pub fn GetSenderCount() -> c_int;
    pub fn GetSenderNameByIndex(index: c_int, name: *mut c_char, maxLen: c_int);
    pub fn GetSenderInfo(
        name: *const c_char,
        width: *mut c_int,
        height: *mut c_int,
        dwFormat: *mut u32,
        bActive: *mut bool,
    ) -> bool;
}

// DXGI_FORMAT constants
pub const DXGI_FORMAT_B8G8R8A8_UNORM: u32 = 87;
pub const DXGI_FORMAT_R8G8B8A8_UNORM: u32 = 28;
pub const DXGI_FORMAT_NV12: u32 = 103;
```

2. **DX12 Interop for wgpu**

wgpu on Windows uses DirectX 12. To share textures with Spout:

```rust
use winapi::um::d3d12::*;
use winapi::shared::dxgi::*;
use winapi::shared::dxgiformat::*;
use winapi::shared::winerror::*;

/// Get D3D12 resource from wgpu texture
pub fn get_d3d12_resource(texture: &wgpu::Texture) -> *mut ID3D12Resource {
    // This requires wgpu HAL access
    // Similar to Metal approach
    unsafe {
        let hal_texture: &wgpu::hal::dx12::Texture = 
            std::mem::transmute(texture);
        hal_texture.resource.as_mut_ptr()
    }
}

/// Create shared handle for D3D12 resource
pub unsafe fn create_shared_handle(
    device: *mut ID3D12Device,
    resource: *mut ID3D12Resource,
) -> HANDLE {
    let mut handle = std::ptr::null_mut();
    
    // Create shared NT handle
    let result = (*device).CreateSharedHandle(
        resource as *mut _,
        std::ptr::null(),
        0x10000000, // GENERIC_ALL
        std::ptr::null(),
        &mut handle,
    );
    
    if SUCCEEDED(result) {
        handle
    } else {
        std::ptr::null_mut()
    }
}
```

3. **Update SpoutOutput Implementation**

```rust
impl IpcOutput for SpoutOutput {
    fn create_server(&mut self, name: &str, width: u32, height: u32) -> IpcResult<()> {
        let name_c = CString::new(name).unwrap();
        
        unsafe {
            let success = spout_ffi::CreateSender(
                name_c.as_ptr(),
                width as c_int,
                height as c_int,
                DXGI_FORMAT_B8G8R8A8_UNORM,
            );
            
            if !success {
                return Err(IpcError::NativeError(
                    "Failed to create Spout sender".to_string()
                ));
            }
        }
        
        self.sender_name = Some(name.to_string());
        self.dimensions = Some((width, height));
        
        Ok(())
    }
    
    fn send_buffer(
        &mut self,
        data: &[u8],
        _format: PixelFormat,
        width: u32,
        height: u32,
    ) -> IpcResult<()> {
        unsafe {
            // Convert RGBA to BGRA if needed
            let bgra_data = rgba_to_bgra(data);
            
            let success = spout_ffi::SendImage(
                bgra_data.as_ptr(),
                width as c_int,
                height as c_int,
                0x80E1, // GL_BGRA (Spout accepts GL formats)
                false,  // Don't invert
            );
            
            if !success {
                return Err(IpcError::NativeError(
                    "Failed to send Spout frame".to_string()
                ));
            }
        }
        
        Ok(())
    }
    // ...
}
```

---

### Linux v4l2loopback

#### Prerequisites

```bash
# Install v4l2loopback
sudo apt-get install v4l2loopback-dkms v4l-utils

# Load module with multiple devices
sudo modprobe v4l2loopback devices=2 video_nr=10,11 card_label="RustJay Out","RustJay Out 2"

# Verify
v4l2-ctl --list-devices
```

#### Implementation Steps

1. **Add V4L2 ioctl Bindings**

```rust
//! V4L2 ioctl definitions

use nix::ioctl_readwrite;
use std::os::raw::{c_char, c_uint, c_ulong, c_void};

// V4L2 constants
pub const VIDIOC_QUERYCAP: c_ulong = 0xc0045600;
pub const VIDIOC_S_FMT: c_ulong = 0xc0d05605;
pub const VIDIOC_G_FMT: c_ulong = 0xc0d05604;
pub const VIDIOC_REQBUFS: c_ulong = 0xc0145608;
pub const VIDIOC_QBUF: c_ulong = 0xc044560f;
pub const VIDIOC_DQBUF: c_ulong = 0xc0445611;
pub const VIDIOC_STREAMON: c_ulong = 0x40045612;
pub const VIDIOC_STREAMOFF: c_ulong = 0x40045613;

// Pixel formats (FourCC)
pub const V4L2_PIX_FMT_RGB24: u32 = fourcc(b'R', b'G', b'B', b'3');
pub const V4L2_PIX_FMT_BGR24: u32 = fourcc(b'B', b'G', b'R', b'3');
pub const V4L2_PIX_FMT_RGB32: u32 = fourcc(b'R', b'G', b'B', b'4');
pub const V4L2_PIX_FMT_BGR32: u32 = fourcc(b'B', b'G', b'R', b'4');
pub const V4L2_PIX_FMT_YUYV: u32 = fourcc(b'Y', b'U', b'Y', b'V');

const fn fourcc(a: u8, b: u8, c: u8, d: u8) -> u32 {
    (a as u32) | ((b as u32) << 8) | ((c as u32) << 16) | ((d as u32) << 24)
}

#[repr(C)]
pub struct v4l2_capability {
    pub driver: [c_char; 16],
    pub card: [c_char; 32],
    pub bus_info: [c_char; 32],
    pub version: u32,
    pub capabilities: u32,
    pub device_caps: u32,
    pub reserved: [u32; 3],
}

#[repr(C)]
pub struct v4l2_pix_format {
    pub width: u32,
    pub height: u32,
    pub pixelformat: u32,
    pub field: u32,
    pub bytesperline: u32,
    pub sizeimage: u32,
    pub colorspace: u32,
    pub priv_: u32,
    pub flags: u32,
    pub ycbcr_enc: u32,
    pub quantization: u32,
    pub xfer_func: u32,
}

#[repr(C)]
pub struct v4l2_format {
    pub type_: u32,
    pub fmt: v4l2_pix_format, // Simplified, actually a union
}

// Define ioctl wrappers using nix
ioctl_readwrite!(vidioc_querycap, VIDIOC_QUERYCAP, v4l2_capability);
ioctl_readwrite!(vidioc_s_fmt, VIDIOC_S_FMT, v4l2_format);
ioctl_readwrite!(vidioc_g_fmt, VIDIOC_G_FMT, v4l2_format);
```

2. **Complete V4L2Output Implementation**

```rust
impl IpcOutput for V4L2Output {
    fn create_server(&mut self, name: &str, width: u32, height: u32) -> IpcResult<()> {
        // Open device
        let device_path = if name.starts_with("/dev/video") {
            name.to_string()
        } else {
            find_loopback_device()
                .ok_or_else(|| IpcError::ServerNotFound(
                    "No v4l2loopback device available".to_string()
                ))?
        };
        
        let file = OpenOptions::new()
            .read(false)
            .write(true)
            .open(&device_path)
            .map_err(|e| IpcError::ConnectionFailed(e.to_string()))?;
        
        // Configure format using ioctl
        unsafe {
            let mut fmt: v4l2_format = std::mem::zeroed();
            fmt.type_ = 1; // V4L2_BUF_TYPE_VIDEO_OUTPUT
            fmt.fmt.width = width;
            fmt.fmt.height = height;
            fmt.fmt.pixelformat = V4L2_PIX_FMT_RGB24;
            fmt.fmt.sizeimage = width * height * 3;
            
            let fd = file.as_raw_fd();
            vidioc_s_fmt(fd, &mut fmt)
                .map_err(|e| IpcError::NativeError(format!("VIDIOC_S_FMT failed: {}", e)))?;
        }
        
        self.device_path = Some(device_path);
        self.file = Some(file);
        self.dimensions = Some((width, height));
        
        Ok(())
    }
    
    fn send_buffer(
        &mut self,
        data: &[u8],
        format: PixelFormat,
        width: u32,
        height: u32,
    ) -> IpcResult<()> {
        let file = self.file.as_mut().ok_or(IpcError::NotInitialized)?;
        
        // Convert to RGB24 if needed
        let rgb24_data = match format {
            PixelFormat::RGB24 => data.to_vec(),
            PixelFormat::RGBA => rgba_to_rgb24(data),
            PixelFormat::YUYV => yuyv_to_rgb24(data, width, height),
            _ => return Err(IpcError::UnsupportedFormat(format!("{:?}", format))),
        };
        
        // Write to device
        file.write_all(&rgb24_data)
            .map_err(|e| IpcError::NativeError(e.to_string()))?;
        
        Ok(())
    }
    // ...
}

fn find_loopback_device() -> Option<String> {
    // Read /sys/class/video4linux/ to find v4l2loopback devices
    std::fs::read_dir("/sys/class/video4linux").ok()?.find_map(|entry| {
        let entry = entry.ok()?;
        let path = entry.path();
        let name = entry.file_name();
        
        // Check if it's a loopback device
        let driver_path = path.join("device/driver");
        if let Ok(link) = std::fs::read_link(&driver_path) {
            if link.to_string_lossy().contains("v4l2loopback") {
                return Some(format!("/dev/{}", name.to_string_lossy()));
            }
        }
        
        None
    })
}
```

---

## GPU Texture Interop Strategy

### The Challenge

wgpu abstracts over multiple GPU APIs (Metal, DX12, Vulkan). To share textures with Syphon/Spout, we need access to the underlying native textures.

### Solution: Custom Surface/Texture

1. **Create platform-specific textures** that can be shared
2. **Import into wgpu** as external textures
3. **Share native handle** with IPC

```rust
/// Platform-specific shared texture
pub enum SharedTexture {
    #[cfg(target_os = "macos")]
    Metal {
        metal_texture: metal::Texture,
        iosurface: IOSurface,
        wgpu_texture: wgpu::Texture,
    },
    #[cfg(target_os = "windows")]
    DirectX {
        d3d_resource: *mut ID3D12Resource,
        shared_handle: HANDLE,
        wgpu_texture: wgpu::Texture,
    },
    #[cfg(target_os = "linux")]
    Vulkan {
        // Vulkan external memory
    },
}

impl SharedTexture {
    /// Create a new shared texture
    pub fn new(
        device: &wgpu::Device,
        width: u32,
        height: u32,
    ) -> Self {
        #[cfg(target_os = "macos")]
        {
            create_metal_shared_texture(device, width, height)
        }
        #[cfg(target_os = "windows")]
        {
            create_dx12_shared_texture(device, width, height)
        }
        // ...
    }
    
    /// Get the wgpu texture for rendering
    pub fn wgpu_texture(&self) -> &wgpu::Texture {
        match self {
            Self::Metal { wgpu_texture, .. } => wgpu_texture,
            Self::DirectX { wgpu_texture, .. } => wgpu_texture,
        }
    }
    
    /// Share with IPC
    pub fn share_with_ipc(&self, ipc_output: &mut dyn IpcOutput) {
        match self {
            #[cfg(target_os = "macos")]
            Self::Metal { iosurface, .. } => {
                // Publish IOSurface to Syphon
            }
            #[cfg(target_os = "windows")]
            Self::DirectX { shared_handle, .. } => {
                // Share handle with Spout
            }
            _ => {}
        }
    }
}
```

---

## GUI Integration

### Inputs Tab

Update `src/gui/mod.rs` to add IPC input controls:

```rust
fn build_inputs_tab(&mut self, ui: &mut Ui) {
    // Existing webcam controls...
    
    // Platform-specific IPC controls
    #[cfg(target_os = "macos")]
    self.build_syphon_controls(ui);
    
    #[cfg(target_os = "windows")]
    self.build_spout_controls(ui);
    
    #[cfg(target_os = "linux")]
    self.build_v4l2_controls(ui);
}

#[cfg(target_os = "macos")]
fn build_syphon_controls(&mut self, ui: &mut Ui) {
    ui.separator();
    ui.heading("Syphon Sources (macOS)");
    
    if ui.button("Refresh Syphon Servers").clicked() {
        self.discover_syphon_sources();
    }
    
    ComboBox::from_label("Input 1 Source")
        .selected_text(&self.selected_syphon_1)
        .show_ui(ui, |ui| {
            for source in &self.syphon_sources {
                ui.selectable_value(
                    &mut self.selected_syphon_1,
                    source.name.clone(),
                    &source.name
                );
            }
        });
    
    if ui.button("Connect Input 1").clicked() {
        self.connect_syphon_input(0, &self.selected_syphon_1);
    }
}
```

### Settings Tab

Add IPC output controls:

```rust
fn build_settings_tab(&mut self, ui: &mut Ui) {
    // Existing settings...
    
    ui.separator();
    ui.heading("IPC Output");
    
    #[cfg(target_os = "macos")]
    {
        ui.checkbox(&mut self.config.syphon_output_enabled, "Enable Syphon Output");
        if self.config.syphon_output_enabled {
            ui.input_text("Syphon Server Name", &mut self.config.syphon_server_name);
        }
    }
    
    #[cfg(target_os = "windows")]
    {
        ui.checkbox(&mut self.config.spout_output_enabled, "Enable Spout Output");
        if self.config.spout_output_enabled {
            ui.input_text("Spout Sender Name", &mut self.config.spout_sender_name);
        }
    }
    
    #[cfg(target_os = "linux")]
    {
        ui.checkbox(&mut self.config.v4l2_output_enabled, "Enable V4L2 Output");
        if self.config.v4l2_output_enabled {
            ComboBox::from_label("V4L2 Device")
                .selected_text(&self.selected_v4l2_device)
                .show_ui(ui, |ui| {
                    for (path, name) in &self.v4l2_devices {
                        ui.selectable_value(
                            &mut self.selected_v4l2_device,
                            path.clone(),
                            format!("{} ({})", name, path)
                        );
                    }
                });
        }
    }
}
```

---

## Testing Plan

### macOS Syphon Test Matrix

| Test | Source App | Expected Result |
|------|-----------|-----------------|
| Receive | Resolume Avenue | Video appears in RustJay |
| Receive | MadMapper | Video appears in RustJay |
| Receive | Millumin | Video appears in RustJay |
| Send | To Resolume | RustJay output visible |
| Send | To OBS | RustJay output visible |
| GPU Share | 4K60 | No CPU overhead |

### Windows Spout Test Matrix

| Test | Source App | Expected Result |
|------|-----------|-----------------|
| Receive | TouchDesigner | Video appears in RustJay |
| Receive | Arena | Video appears in RustJay |
| Send | To TouchDesigner | RustJay output visible |
| Send | To OBS | RustJay output visible |

### Linux v4l2loopback Test Matrix

| Test | Command/App | Expected Result |
|------|------------|-----------------|
| Send | `ffmpeg -i /dev/video10` | Receives RustJay output |
| Send | OBS (V4L2 source) | Receives RustJay output |
| Receive | `ffmpeg -f v4l2 -i /dev/video0` | Can read camera |

---

## Timeline

| Week | Task |
|------|------|
| 1 | Complete macOS Syphon with CPU fallback |
| 2 | Implement wgpu Metal interop for zero-copy |
| 3 | Windows Spout implementation |
| 4 | Linux v4l2loopback implementation |
| 5 | GUI integration and testing |
| 6 | Performance optimization and documentation |

---

## Resources

### Documentation
- [Syphon Framework](https://github.com/Syphon/Syphon-Framework)
- [Spout SDK](https://github.com/leadedge/Spout2)
- [v4l2loopback](https://github.com/umlaeute/v4l2loopback)
- [wgpu HAL Documentation](https://docs.rs/wgpu-hal/)

### Related Crates
- `objc` - Objective-C runtime
- `metal-rs` - Metal bindings
- `windows-rs` - Windows API
- `v4l-rs` - V4L2 bindings

### Example Projects
- [Spout Rust](https://github.com/cyberjunk/spout-rs)
- [Syphon Rust](https://github.com/nick-parker/Syphon)
- [V4L2 Rust](https://github.com/raymanfx/libv4l-rs)
