//! # Syphon Implementation (macOS)
//!
//! Provides Syphon video sharing support on macOS.
//!
//! ## Overview
//!
//! Syphon is an open-source Mac OS X technology that allows applications to share frames
//! with one another in realtime. It uses IOSurface for zero-copy GPU texture sharing.
//!
//! ## Implementation Notes
//!
//! This implementation uses Objective-C runtime bindings via the `objc` crate.
//! The Syphon framework must be available at runtime (included with many VJ apps).
//!
//! ## Zero-Copy Architecture
//!
//! ```
//! RustJay (wgpu/Metal) ────► IOSurface ────► Syphon Server ────► Receiving App
//!                              (shared)          (publish)
//! ```

use super::{IpcDiscovery, IpcError, IpcFrame, IpcInput, IpcOutput, IpcResult, IpcSourceInfo, PixelFormat};
use std::fmt::Debug;

/// Handle to a Syphon-shared GPU texture
#[derive(Debug)]
pub struct SyphonTextureHandle {
    /// IOSurface ID for sharing
    pub iosurface_id: u32,
    /// Texture dimensions
    pub width: u32,
    pub height: u32,
}

/// Syphon input client
///
/// Receives video frames from Syphon servers on the local machine.
pub struct SyphonInput {
    /// Connected server name
    server_name: Option<String>,
    /// Current dimensions
    dimensions: Option<(u32, u32)>,
    /// Native Syphon client (opaque pointer)
    #[allow(dead_code)]
    native_client: Option<*mut std::ffi::c_void>,
}

// Safety: The native client pointer is only accessed from the thread that created it
// and is properly synchronized via the option wrapper
unsafe impl Send for SyphonInput {}

impl SyphonInput {
    /// Create a new Syphon input client
    pub fn new() -> Self {
        Self {
            server_name: None,
            dimensions: None,
            native_client: None,
        }
    }

    /// Check if Syphon framework is available
    pub fn is_available() -> bool {
        // Check if Syphon framework can be loaded
        // In a real implementation, this would try to load the framework
        true // Placeholder
    }
}

impl Debug for SyphonInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SyphonInput")
            .field("server_name", &self.server_name)
            .field("dimensions", &self.dimensions)
            .field("connected", &self.is_connected())
            .finish()
    }
}

impl IpcInput for SyphonInput {
    fn connect(&mut self, source: &str) -> IpcResult<()> {
        if self.is_connected() {
            return Err(IpcError::AlreadyConnected);
        }

        log::info!("[Syphon] Connecting to server: {}", source);

        // TODO: Implement using objc crate:
        // 1. Create SyphonClient with server name
        // 2. Set up frame callback or polling
        // 3. Store native client pointer

        self.server_name = Some(source.to_string());
        
        // Placeholder: Would return Err on actual failure
        Ok(())
    }

    fn disconnect(&mut self) {
        if !self.is_connected() {
            return;
        }

        log::info!("[Syphon] Disconnecting from server: {:?}", self.server_name);

        // TODO: Release SyphonClient
        // 1. Stop frame callbacks
        // 2. Release native client
        // 3. Clean up resources

        self.server_name = None;
        self.dimensions = None;
        self.native_client = None;
    }

    fn is_connected(&self) -> bool {
        self.server_name.is_some()
    }

    fn receive_frame(&mut self) -> Option<IpcFrame> {
        if !self.is_connected() {
            return None;
        }

        // TODO: Implement frame receiving:
        // 1. Check if new frame available from SyphonClient
        // 2. Get IOSurface reference
        // 3. Option A: Return GPU handle (zero-copy)
        // 4. Option B: Read to CPU buffer (fallback)

        // Placeholder: Return None for now
        None
    }

    fn resolution(&self) -> Option<(u32, u32)> {
        self.dimensions
    }

    fn source_name(&self) -> Option<&str> {
        self.server_name.as_deref()
    }
}

impl Default for SyphonInput {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for SyphonInput {
    fn drop(&mut self) {
        self.disconnect();
    }
}

/// Syphon output server
///
/// Publishes video frames as a Syphon server that other apps can receive.
pub struct SyphonOutput {
    /// Server name
    server_name: Option<String>,
    /// Current dimensions
    dimensions: Option<(u32, u32)>,
    /// Native Syphon server (opaque pointer)
    #[allow(dead_code)]
    native_server: Option<*mut std::ffi::c_void>,
    /// Whether to use IOSurface sharing (zero-copy)
    use_gpu_sharing: bool,
}

// Safety: The native server pointer is only accessed from the thread that created it
unsafe impl Send for SyphonOutput {}

impl SyphonOutput {
    /// Create a new Syphon output server
    pub fn new() -> Self {
        Self {
            server_name: None,
            dimensions: None,
            native_server: None,
            use_gpu_sharing: true,
        }
    }

    /// Create with GPU sharing disabled (CPU fallback)
    pub fn new_cpu_only() -> Self {
        Self {
            server_name: None,
            dimensions: None,
            native_server: None,
            use_gpu_sharing: false,
        }
    }

    /// Check if Syphon framework is available
    pub fn is_available() -> bool {
        SyphonInput::is_available()
    }

    /// Enable/disable GPU sharing
    pub fn set_gpu_sharing(&mut self, enabled: bool) {
        self.use_gpu_sharing = enabled;
    }
}

impl Debug for SyphonOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SyphonOutput")
            .field("server_name", &self.server_name)
            .field("dimensions", &self.dimensions)
            .field("active", &self.is_active())
            .field("gpu_sharing", &self.use_gpu_sharing)
            .finish()
    }
}

impl IpcOutput for SyphonOutput {
    fn create_server(&mut self, name: &str, width: u32, height: u32) -> IpcResult<()> {
        if self.is_active() {
            self.destroy_server();
        }

        if width == 0 || height == 0 {
            return Err(IpcError::InvalidDimensions { width, height });
        }

        log::info!("[Syphon] Creating server '{}' at {}x{}", name, width, height);

        // TODO: Implement using objc crate:
        // 1. Create SyphonServer with name
        // 2. Configure for IOSurface publishing if GPU sharing enabled
        // 3. Store native server pointer

        self.server_name = Some(name.to_string());
        self.dimensions = Some((width, height));

        Ok(())
    }

    fn destroy_server(&mut self) {
        if !self.is_active() {
            return;
        }

        log::info!("[Syphon] Destroying server: {:?}", self.server_name);

        // TODO: Release SyphonServer
        // 1. Stop publishing
        // 2. Release native server
        // 3. Clean up resources

        self.server_name = None;
        self.dimensions = None;
        self.native_server = None;
    }

    fn is_active(&self) -> bool {
        self.server_name.is_some()
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
        // 1. Get underlying Metal texture from wgpu texture
        //    - Access via wgpu::hal::metal or custom surface
        // 2. Get IOSurface from Metal texture
        // 3. Publish IOSurface to SyphonServer
        //
        // This requires either:
        // - Access to wgpu's HAL layer
        // - Custom Metal surface implementation
        // - Or: Copy to CPU and use send_buffer (fallback)

        log::debug!("[Syphon] Publishing GPU texture");

        // Placeholder: Would return Err on actual failure
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
        // 1. Convert to Syphon-compatible format (usually RGBA/BGRA)
        // 2. Create CGImage or texture from buffer
        // 3. Publish to SyphonServer

        log::debug!("[Syphon] Publishing CPU buffer: {}x{} {:?}", width, height, format);

        // Placeholder
        Ok(())
    }

    fn server_name(&self) -> Option<&str> {
        self.server_name.as_deref()
    }

    fn dimensions(&self) -> Option<(u32, u32)> {
        self.dimensions
    }
}

impl Default for SyphonOutput {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for SyphonOutput {
    fn drop(&mut self) {
        self.destroy_server();
    }
}

/// Syphon source discovery
///
/// Scans for available Syphon servers on the local machine.
pub struct SyphonDiscovery {
    /// Last discovered sources
    sources: Vec<IpcSourceInfo>,
}

impl SyphonDiscovery {
    /// Create a new Syphon discovery
    pub fn new() -> Self {
        Self {
            sources: Vec::new(),
        }
    }
}

impl Default for SyphonDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

impl IpcDiscovery for SyphonDiscovery {
    fn discover_sources(&mut self, _timeout_ms: u32) -> Vec<IpcSourceInfo> {
        log::debug!("[Syphon] Discovering sources...");

        // TODO: Implement discovery:
        // 1. Use SyphonServerDirectory (NSNotification-based)
        // 2. Get list of available servers
        // 3. Filter by type if needed
        // 4. Update self.sources

        // Placeholder: Return empty list
        self.sources.clear();
        self.sources.clone()
    }

    fn is_source_available(&self, name: &str) -> bool {
        self.sources.iter().any(|s| s.name == name)
    }
}

/// Convert pixel format to CoreGraphics format
#[allow(dead_code)]
fn pixel_format_to_core_graphics(format: PixelFormat) -> u32 {
    // kCGImageAlphaPremultipliedLast = 1 (RGBA)
    // kCGImageAlphaPremultipliedFirst = 2 (ARGB)
    // kCGImageAlphaNoneSkipLast = 5 (RGBX)
    // etc.
    match format {
        PixelFormat::RGBA => 1, // kCGImageAlphaPremultipliedLast
        PixelFormat::BGRA => 2, // kCGImageAlphaPremultipliedFirst (needs swap)
        _ => 1, // Default to RGBA
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_syphon_input_creation() {
        let input = SyphonInput::new();
        assert!(!input.is_connected());
        assert_eq!(input.source_name(), None);
    }

    #[test]
    fn test_syphon_output_creation() {
        let output = SyphonOutput::new();
        assert!(!output.is_active());
        assert_eq!(output.server_name(), None);
    }

    #[test]
    fn test_syphon_output_cpu_fallback() {
        let output = SyphonOutput::new_cpu_only();
        assert!(!output.use_gpu_sharing);
    }
}
