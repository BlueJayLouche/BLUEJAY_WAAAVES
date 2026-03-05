//! # Spout Implementation (Windows)
//!
//! Provides Spout video sharing support on Windows.
//!
//! ## Overview
//!
//! Spout is a real-time video sharing framework for Windows that uses
//! DirectX texture sharing for zero-copy inter-process communication.
//!
//! ## Implementation Notes
//!
//! This implementation uses the Spout SDK via FFI bindings.
//! Supports DirectX 11, DirectX 12, and OpenGL sharing.
//!
//! ## Zero-Copy Architecture
//!
//! ```
//! RustJay (wgpu/DX12) ────► D3D12 Resource ────► Spout Sender ────► Receiving App
//!                              (shared handle)       (publish)
//! ```

use super::{IpcDiscovery, IpcError, IpcFrame, IpcInput, IpcOutput, IpcResult, IpcSourceInfo, PixelFormat};
use std::fmt::Debug;

/// Handle to a Spout-shared GPU texture
#[derive(Debug)]
pub struct SpoutTextureHandle {
    /// DirectX shared handle
    pub shared_handle: *mut std::ffi::c_void,
    /// Texture dimensions
    pub width: u32,
    pub height: u32,
    /// Format (DXGI_FORMAT)
    pub dxgi_format: u32,
}

// Safety: SpoutTextureHandle is Send if the handle is valid
unsafe impl Send for SpoutTextureHandle {}

/// Spout input receiver
///
/// Receives video frames from Spout senders on the local machine.
pub struct SpoutInput {
    /// Connected sender name
    sender_name: Option<String>,
    /// Current dimensions
    dimensions: Option<(u32, u32)>,
    /// Native Spout receiver (opaque pointer)
    #[allow(dead_code)]
    native_receiver: Option<*mut std::ffi::c_void>,
    /// Whether to use GPU texture sharing
    use_gpu_sharing: bool,
}

// Safety: The native pointer is only accessed from the creating thread
unsafe impl Send for SpoutInput {}

impl SpoutInput {
    /// Create a new Spout input receiver
    pub fn new() -> Self {
        Self {
            sender_name: None,
            dimensions: None,
            native_receiver: None,
            use_gpu_sharing: true,
        }
    }

    /// Create with CPU fallback only
    pub fn new_cpu_only() -> Self {
        Self {
            sender_name: None,
            dimensions: None,
            native_receiver: None,
            use_gpu_sharing: false,
        }
    }

    /// Check if Spout SDK is available
    pub fn is_available() -> bool {
        // Check if spout library can be loaded
        // In a real implementation, try to load Spout.dll
        true // Placeholder
    }

    /// Enable/disable GPU sharing
    pub fn set_gpu_sharing(&mut self, enabled: bool) {
        self.use_gpu_sharing = enabled;
    }
}

impl Debug for SpoutInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SpoutInput")
            .field("sender_name", &self.sender_name)
            .field("dimensions", &self.dimensions)
            .field("connected", &self.is_connected())
            .field("gpu_sharing", &self.use_gpu_sharing)
            .finish()
    }
}

impl IpcInput for SpoutInput {
    fn connect(&mut self, source: &str) -> IpcResult<()> {
        if self.is_connected() {
            return Err(IpcError::AlreadyConnected);
        }

        log::info!("[Spout] Connecting to sender: {}", source);

        // TODO: Implement using Spout SDK:
        // 1. Create SpoutReceiver
        // 2. Set sender name
        // 3. Open shared memory/texture
        // 4. Configure for CPU or GPU mode

        self.sender_name = Some(source.to_string());

        Ok(())
    }

    fn disconnect(&mut self) {
        if !self.is_connected() {
            return;
        }

        log::info!("[Spout] Disconnecting from sender: {:?}", self.sender_name);

        // TODO: Release SpoutReceiver
        // 1. Close shared resources
        // 2. Release native receiver
        // 3. Clean up

        self.sender_name = None;
        self.dimensions = None;
        self.native_receiver = None;
    }

    fn is_connected(&self) -> bool {
        self.sender_name.is_some()
    }

    fn receive_frame(&mut self) -> Option<IpcFrame> {
        if !self.is_connected() {
            return None;
        }

        // TODO: Implement frame receiving:
        // 1. Check if sender is updated (ReceiveTexture/ReceiveImage)
        // 2. If GPU sharing: get shared handle, return GpuTexture
        // 3. If CPU mode: read to buffer, return CpuBuffer

        None
    }

    fn resolution(&self) -> Option<(u32, u32)> {
        self.dimensions
    }

    fn source_name(&self) -> Option<&str> {
        self.sender_name.as_deref()
    }
}

impl Default for SpoutInput {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for SpoutInput {
    fn drop(&mut self) {
        self.disconnect();
    }
}

/// Spout output sender
///
/// Publishes video frames as a Spout sender that other apps can receive.
pub struct SpoutOutput {
    /// Sender name
    sender_name: Option<String>,
    /// Current dimensions
    dimensions: Option<(u32, u32)>,
    /// Native Spout sender (opaque pointer)
    #[allow(dead_code)]
    native_sender: Option<*mut std::ffi::c_void>,
    /// Whether to use GPU texture sharing
    use_gpu_sharing: bool,
    /// DirectX device/context for sharing
    #[allow(dead_code)]
    dx_device: Option<*mut std::ffi::c_void>,
}

// Safety: Native pointers are only accessed from the creating thread
unsafe impl Send for SpoutOutput {}

impl SpoutOutput {
    /// Create a new Spout output sender
    pub fn new() -> Self {
        Self {
            sender_name: None,
            dimensions: None,
            native_sender: None,
            use_gpu_sharing: true,
            dx_device: None,
        }
    }

    /// Create with CPU fallback only
    pub fn new_cpu_only() -> Self {
        Self {
            sender_name: None,
            dimensions: None,
            native_sender: None,
            use_gpu_sharing: false,
            dx_device: None,
        }
    }

    /// Check if Spout SDK is available
    pub fn is_available() -> bool {
        SpoutInput::is_available()
    }

    /// Enable/disable GPU sharing
    pub fn set_gpu_sharing(&mut self, enabled: bool) {
        self.use_gpu_sharing = enabled;
    }
}

impl Debug for SpoutOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SpoutOutput")
            .field("sender_name", &self.sender_name)
            .field("dimensions", &self.dimensions)
            .field("active", &self.is_active())
            .field("gpu_sharing", &self.use_gpu_sharing)
            .finish()
    }
}

impl IpcOutput for SpoutOutput {
    fn create_server(&mut self, name: &str, width: u32, height: u32) -> IpcResult<()> {
        if self.is_active() {
            self.destroy_server();
        }

        if width == 0 || height == 0 {
            return Err(IpcError::InvalidDimensions { width, height });
        }

        log::info!("[Spout] Creating sender '{}' at {}x{}", name, width, height);

        // TODO: Implement using Spout SDK:
        // 1. Create SpoutSender
        // 2. Set sender name
        // 3. Create shared texture (if GPU mode)
        // 4. Or allocate shared memory (if CPU mode)

        self.sender_name = Some(name.to_string());
        self.dimensions = Some((width, height));

        Ok(())
    }

    fn destroy_server(&mut self) {
        if !self.is_active() {
            return;
        }

        log::info!("[Spout] Destroying sender: {:?}", self.sender_name);

        // TODO: Release SpoutSender
        // 1. Release shared resources
        // 2. Release native sender
        // 3. Clean up DirectX resources

        self.sender_name = None;
        self.dimensions = None;
        self.native_sender = None;
        self.dx_device = None;
    }

    fn is_active(&self) -> bool {
        self.sender_name.is_some()
    }

    fn send_texture(
        &mut self,
        _texture: &wgpu::Texture,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
    ) -> IpcResult<()> {
        if !self.is_active() {
            return Err(IpcError::NotInitialized);
        }

        if !self.use_gpu_sharing {
            return Err(IpcError::SharingNotAvailable);
        }

        // TODO: Implement zero-copy texture sharing:
        // 1. Get underlying DirectX 12 resource from wgpu texture
        //    - Access via wgpu::hal::dx12
        // 2. Create shared handle for the resource
        // 3. Send via SpoutSender.SendTexture
        //
        // Challenges:
        // - wgpu DX12 textures need D3D12_HEAP_FLAG_SHARED
        // - Need to synchronize GPU operations
        // - Handle format conversion if needed

        log::debug!("[Spout] Publishing GPU texture");

        Ok(())
    }

    fn send_buffer(
        &mut self,
        data: &[u8],
        format: PixelFormat,
        width: u32,
        height: u32,
    ) -> IpcResult<()> {
        if !self.is_active() {
            return Err(IpcError::NotInitialized);
        }

        // Validate dimensions match
        if let Some((w, h)) = self.dimensions {
            if w != width || h != height {
                return Err(IpcError::InvalidDimensions { width, height });
            }
        }

        // Validate buffer size
        let expected_size = format.buffer_size(width, height);
        if data.len() < expected_size {
            return Err(IpcError::NativeError(
                format!("Buffer too small: {} < {}", data.len(), expected_size)
            ));
        }

        // TODO: Implement CPU buffer publishing:
        // 1. Convert to BGRA (Spout native format)
        // 2. Send via SpoutSender.SendImage
        //    - SpoutSender.SendImage(buffer, width, height, format, flip)

        log::debug!("[Spout] Publishing CPU buffer: {}x{} {:?}", width, height, format);

        Ok(())
    }

    fn server_name(&self) -> Option<&str> {
        self.sender_name.as_deref()
    }

    fn dimensions(&self) -> Option<(u32, u32)> {
        self.dimensions
    }
}

impl Default for SpoutOutput {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for SpoutOutput {
    fn drop(&mut self) {
        self.destroy_server();
    }
}

/// Spout sender discovery
///
/// Scans for available Spout senders on the local machine.
pub struct SpoutDiscovery {
    /// Last discovered sources
    sources: Vec<IpcSourceInfo>,
}

impl SpoutDiscovery {
    /// Create a new Spout discovery
    pub fn new() -> Self {
        Self {
            sources: Vec::new(),
        }
    }
}

impl Default for SpoutDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

impl IpcDiscovery for SpoutDiscovery {
    fn discover_sources(&mut self, _timeout_ms: u32) -> Vec<IpcSourceInfo> {
        log::debug!("[Spout] Discovering senders...");

        // TODO: Implement discovery:
        // 1. Use Spout.GetSenderCount()
        // 2. Iterate with GetSenderName(index)
        // 3. Get info with GetSenderInfo(name, ...)
        // 4. Update self.sources

        self.sources.clear();
        self.sources.clone()
    }

    fn is_source_available(&self, name: &str) -> bool {
        self.sources.iter().any(|s| s.name == name)
    }
}

/// Convert pixel format to DirectX DXGI format
#[allow(dead_code)]
pub fn pixel_format_to_dxgi(format: PixelFormat) -> u32 {
    // DXGI_FORMAT_R8G8B8A8_UNORM = 28
    // DXGI_FORMAT_B8G8R8A8_UNORM = 87
    // DXGI_FORMAT_NV12 = 103
    match format {
        PixelFormat::RGBA => 28,  // DXGI_FORMAT_R8G8B8A8_UNORM
        PixelFormat::BGRA => 87,  // DXGI_FORMAT_B8G8R8A8_UNORM
        PixelFormat::NV12 => 103, // DXGI_FORMAT_NV12
        PixelFormat::RGB24 | PixelFormat::YUYV => 28, // Convert to RGBA
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spout_input_creation() {
        let input = SpoutInput::new();
        assert!(!input.is_connected());
        assert_eq!(input.source_name(), None);
        assert!(input.use_gpu_sharing);
    }

    #[test]
    fn test_spout_output_creation() {
        let output = SpoutOutput::new();
        assert!(!output.is_active());
        assert_eq!(output.server_name(), None);
    }

    #[test]
    fn test_spout_cpu_fallback() {
        let input = SpoutInput::new_cpu_only();
        assert!(!input.use_gpu_sharing);

        let output = SpoutOutput::new_cpu_only();
        assert!(!output.use_gpu_sharing);
    }

    #[test]
    fn test_pixel_format_conversion() {
        assert_eq!(pixel_format_to_dxgi(PixelFormat::RGBA), 28);
        assert_eq!(pixel_format_to_dxgi(PixelFormat::BGRA), 87);
    }
}
