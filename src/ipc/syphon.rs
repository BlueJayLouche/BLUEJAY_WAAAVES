//! # Syphon Implementation using syphon-core crate (macOS)
//!
//! Provides Syphon video sharing support on macOS using the well-tested
//! syphon-core and syphon-wgpu crates for zero-copy GPU texture sharing.
//!
//! ## Architecture
//!
//! ```
//! RustJay Waaaves
//!      │
//!      ▼
//! IpcInput/IpcOutput traits
//!      │
//!      ▼
//! SyphonInput/SyphonOutput (wrappers)
//!      │
//!      ▼
//! syphon-core / syphon-wgpu crates
//!      │
//!      ▼
//! Syphon.framework (macOS)
//! ```
//!
//! ## GPU Device Compatibility
//!
//! On multi-GPU systems (e.g., MacBook Pro with Intel + AMD), it's important
//! that the Syphon server uses the same GPU as your rendering. This module
//! provides device compatibility checking to help diagnose issues.
//!
//! Example:
//! ```rust,no_run
//! use crate::ipc::syphon::{SyphonOutput, check_gpu_compatibility};
//!
//! // Before creating output, check available GPUs
//! if let Ok(gpus) = list_available_gpus() {
//!     for gpu in gpus {
//!         log::info!("Available GPU: {} (high-performance: {})", 
//!             gpu.name, gpu.is_high_performance());
//!     }
//! }
//!
//! // Create output with automatic device selection
//! let output = SyphonOutput::new_with_wgpu("My Server", &device, &queue, 1920, 1080)?;
//! ```

use super::{IpcDiscovery, IpcError, IpcFrame, IpcInput, IpcOutput, IpcResult, IpcSourceInfo, PixelFormat};
use std::fmt::Debug;

/// Re-export the external crate types for advanced users
pub use syphon_core::{SyphonServer, SyphonClient, SyphonServerDirectory, ServerInfo, is_available};
pub use syphon_wgpu::SyphonWgpuOutput;

/// Re-export Metal device utilities for GPU compatibility checking
pub use syphon_core::{
    MetalDeviceInfo,
    default_device,
    available_devices,
    recommended_high_performance_device,
    check_device_compatibility,
    validate_device_match,
};

/// Check if Syphon framework is available
pub fn is_syphon_available() -> bool {
    is_available()
}

/// Syphon input client
///
/// Receives video frames from Syphon servers on the local machine.
/// Wraps syphon_core::SyphonClient to implement the IpcInput trait.
pub struct SyphonInput {
    /// Connected server name
    server_name: Option<String>,
    /// Current dimensions
    dimensions: Option<(u32, u32)>,
    /// Native Syphon client from external crate
    client: Option<SyphonClient>,
    /// Pending frame data from last receive call
    pending_frame: Option<Vec<u8>>,
}

impl SyphonInput {
    /// Create a new Syphon input client
    pub fn new() -> Self {
        Self {
            server_name: None,
            dimensions: None,
            client: None,
            pending_frame: None,
        }
    }

    /// Check if Syphon framework is available
    pub fn is_available() -> bool {
        is_available()
    }

    /// Get direct access to the underlying client (for advanced usage)
    pub fn client(&self) -> Option<&SyphonClient> {
        self.client.as_ref()
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

        // Use the external crate to connect
        match SyphonClient::connect(source) {
            Ok(client) => {
                self.server_name = Some(source.to_string());
                // Get initial frame to determine dimensions
                if let Ok(Some(frame)) = client.try_receive() {
                    self.dimensions = Some((frame.width, frame.height));
                }
                self.client = Some(client);
                log::info!("[Syphon] Connected to '{}'", source);
                Ok(())
            }
            Err(e) => {
                log::error!("[Syphon] Failed to connect to '{}': {}", source, e);
                Err(IpcError::ConnectionFailed(e.to_string()))
            }
        }
    }

    fn disconnect(&mut self) {
        if !self.is_connected() {
            return;
        }

        log::info!("[Syphon] Disconnecting from server: {:?}", self.server_name);

        // Drop the client (SyphonClient implements Drop)
        self.client = None;
        self.server_name = None;
        self.dimensions = None;
        self.pending_frame = None;
    }

    fn is_connected(&self) -> bool {
        self.client.is_some()
    }

    fn receive_frame(&mut self) -> Option<IpcFrame> {
        let client = self.client.as_ref()?;

        // Try to receive a frame
        match client.try_receive() {
            Ok(Some(mut frame)) => {
                // Update dimensions
                self.dimensions = Some((frame.width, frame.height));

                // Convert to CPU buffer (we could optimize this for GPU path later)
                match frame.to_vec() {
                    Ok(data) => {
                        Some(IpcFrame::CpuBuffer {
                            data,
                            format: PixelFormat::BGRA, // Syphon uses BGRA on macOS
                            width: frame.width,
                            height: frame.height,
                        })
                    }
                    Err(e) => {
                        log::warn!("[Syphon] Failed to read frame data: {}", e);
                        None
                    }
                }
            }
            Ok(None) => {
                // No new frame available
                None
            }
            Err(e) => {
                log::warn!("[Syphon] Error receiving frame: {}", e);
                None
            }
        }
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
/// Uses syphon_wgpu::SyphonWgpuOutput for zero-copy GPU texture sharing.
pub struct SyphonOutput {
    /// Server name
    server_name: Option<String>,
    /// Current dimensions
    dimensions: Option<(u32, u32)>,
    /// Zero-copy wgpu output (only available when created with device/queue)
    wgpu_output: Option<SyphonWgpuOutput>,
    /// Whether we have a valid server
    has_server: bool,
}

impl SyphonOutput {
    /// Create a new Syphon output server (for CPU fallback mode)
    pub fn new() -> Self {
        Self {
            server_name: None,
            dimensions: None,
            wgpu_output: None,
            has_server: false,
        }
    }

    /// Create with GPU sharing enabled (requires device and queue)
    /// 
    /// This is the recommended way to create a Syphon output for zero-copy operation.
    /// 
    /// # GPU Device Compatibility
    /// 
    /// On multi-GPU systems, this method will attempt to use the same GPU as the
    /// wgpu device. If the Syphon framework has loading issues (e.g., incorrect
    /// install name), it will fall back to the framework's internal device selection
    /// with a warning.
    ///
    /// # Errors
    /// Returns `IpcError::NativeError` if the Syphon framework is not available
    /// or if server creation fails.
    pub fn new_with_wgpu(
        name: &str,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
    ) -> IpcResult<Self> {
        // Log available GPUs for debugging (helpful for multi-GPU systems)
        #[cfg(target_os = "macos")]
        if log::log_enabled!(log::Level::Debug) {
            match available_devices() {
                gpus if !gpus.is_empty() => {
                    log::debug!("[Syphon] Available GPUs:");
                    for gpu in &gpus {
                        log::debug!("  - {} (default={}, low_power={}, unified={})",
                            gpu.name, gpu.is_default, gpu.is_low_power, gpu.has_unified_memory);
                    }
                }
                _ => {
                    log::debug!("[Syphon] Could not enumerate GPUs");
                }
            }
        }
        
        match SyphonWgpuOutput::new(name, device, queue, width, height) {
            Ok(output) => {
                let is_zero_copy = output.is_zero_copy();
                log::info!("[Syphon] Created wgpu output '{}' at {}x{} (zero-copy: {})",
                    name, width, height, is_zero_copy);
                
                if !is_zero_copy {
                    log::warn!("[Syphon] Zero-copy not available - falling back to CPU readback. \
                        This may impact performance.");
                }
                
                Ok(Self {
                    server_name: Some(name.to_string()),
                    dimensions: Some((width, height)),
                    wgpu_output: Some(output),
                    has_server: true,
                })
            }
            Err(syphon_core::SyphonError::FrameworkNotFound(ref msg)) => {
                log::error!("[Syphon] Framework not found: {}", msg);
                log::error!("[Syphon] Ensure Syphon.framework is installed at /Library/Frameworks/");
                log::error!("[Syphon] Download from: https://github.com/Syphon/Syphon-Framework/releases");
                Err(IpcError::NativeError(format!(
                    "Syphon framework not found. Install from https://github.com/Syphon/Syphon-Framework/releases: {}", 
                    msg
                )))
            }
            Err(e) => {
                log::error!("[Syphon] Failed to create wgpu output: {}", e);
                Err(IpcError::NativeError(format!("Failed to create Syphon output: {}", e)))
            }
        }
    }

    /// Check if Syphon framework is available
    pub fn is_available() -> bool {
        is_available()
    }

    /// Get the underlying wgpu output (for advanced usage)
    pub fn wgpu_output(&self) -> Option<&SyphonWgpuOutput> {
        self.wgpu_output.as_ref()
    }

    /// Get mutable reference to wgpu output
    pub fn wgpu_output_mut(&mut self) -> Option<&mut SyphonWgpuOutput> {
        self.wgpu_output.as_mut()
    }

    /// Publish a texture using zero-copy path (requires wgpu output)
    ///
    /// This is the most efficient way to publish frames when using wgpu.
    pub fn publish_texture(&mut self, texture: &wgpu::Texture, device: &wgpu::Device, queue: &wgpu::Queue) {
        if let Some(ref mut output) = self.wgpu_output {
            output.publish(texture, device, queue);
        }
    }
}

impl Debug for SyphonOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SyphonOutput")
            .field("server_name", &self.server_name)
            .field("dimensions", &self.dimensions)
            .field("active", &self.is_active())
            .field("zero_copy", &self.wgpu_output.as_ref().map_or(false, |o| o.is_zero_copy()))
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

        log::info!("[Syphon] Creating server '{}' at {}x{} (CPU fallback mode)", name, width, height);

        // Without device/queue, we can only do CPU fallback
        // The wgpu output should be created via new_with_wgpu() for GPU sharing
        self.server_name = Some(name.to_string());
        self.dimensions = Some((width, height));
        self.has_server = true;

        Ok(())
    }

    fn destroy_server(&mut self) {
        if !self.is_active() {
            return;
        }

        log::info!("[Syphon] Destroying server: {:?}", self.server_name);

        self.wgpu_output = None;
        self.server_name = None;
        self.dimensions = None;
        self.has_server = false;
    }

    fn is_active(&self) -> bool {
        self.has_server
    }

    fn send_texture(
        &mut self,
        _texture: &wgpu::Texture,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> IpcResult<()> {
        if !self.is_active() {
            return Err(IpcError::NotInitialized);
        }

        // If we have a wgpu output, use zero-copy path
        if let Some(ref mut output) = self.wgpu_output {
            output.publish(_texture, device, queue);
            return Ok(());
        }

        // Otherwise, we need to use CPU fallback
        Err(IpcError::SharingNotAvailable)
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

        // TODO: Implement CPU buffer publishing through syphon-core
        // This requires creating a Metal texture from the buffer and publishing it
        log::debug!("[Syphon] CPU buffer publishing not yet implemented ({}x{} {:?})", 
            width, height, format);

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
/// Wraps syphon_core::SyphonServerDirectory.
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

    /// Refresh the server list immediately
    pub fn refresh(&mut self) {
        self.sources = Self::discover_now();
    }

    /// Discover servers synchronously
    fn discover_now() -> Vec<IpcSourceInfo> {
        let servers = SyphonServerDirectory::servers();
        
        log::debug!("[Syphon] Discovered {} servers", servers.len());
        
        servers
            .into_iter()
            .map(|info| IpcSourceInfo {
                name: info.name,
                app_name: info.app_name,
                dimensions: None, // Could be fetched from a connected client
                handle: info.uuid,
            })
            .collect()
    }
}

impl Default for SyphonDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

impl IpcDiscovery for SyphonDiscovery {
    fn discover_sources(&mut self, _timeout_ms: u32) -> Vec<IpcSourceInfo> {
        self.refresh();
        self.sources.clone()
    }

    fn is_source_available(&self, name: &str) -> bool {
        self.sources.iter().any(|s| s.name == name)
    }
}

/// Conversion from external crate's ServerInfo to our IpcSourceInfo
impl From<ServerInfo> for IpcSourceInfo {
    fn from(info: ServerInfo) -> Self {
        Self {
            name: info.name,
            app_name: info.app_name,
            dimensions: None,
            handle: info.uuid,
        }
    }
}

/// List all available GPUs on the system
///
/// Useful for debugging multi-GPU setups and verifying which GPUs
/// are available for Syphon texture sharing.
///
/// # Example
/// ```rust,no_run
/// let gpus = list_available_gpus()?;
/// for gpu in gpus {
///     println!("GPU: {} (high-performance: {})", 
///         gpu.name, gpu.is_high_performance());
/// }
/// ```
pub fn list_available_gpus() -> IpcResult<Vec<MetalDeviceInfo>> {
    let devices = available_devices();
    if devices.is_empty() {
        return Err(IpcError::NativeError(
            "No Metal GPUs found. This may indicate a system issue.".to_string()
        ));
    }
    Ok(devices)
}

/// Get the recommended GPU for high-performance rendering
///
/// On multi-GPU systems, this returns the discrete/high-performance GPU
/// rather than the integrated/low-power GPU.
pub fn get_recommended_gpu() -> IpcResult<MetalDeviceInfo> {
    recommended_high_performance_device()
        .ok_or_else(|| IpcError::NativeError(
            "No suitable GPU found for high-performance rendering".to_string()
        ))
}

/// Check GPU compatibility for Syphon texture sharing
///
/// Returns information about GPU compatibility. This is useful for
/// diagnosing performance issues on multi-GPU systems.
///
/// # Returns
/// - `Ok(())` if GPUs are compatible
/// - `Err` with details if there may be performance issues
pub fn check_gpu_compatibility() -> IpcResult<String> {
    let devices = available_devices();
    
    if devices.is_empty() {
        return Err(IpcError::NativeError(
            "No Metal GPUs found".to_string()
        ));
    }
    
    if devices.len() == 1 {
        let gpu = &devices[0];
        return Ok(format!(
            "Single GPU system: {} (high-performance: {})",
            gpu.name,
            gpu.is_high_performance()
        ));
    }
    
    // Multi-GPU system
    let mut report = format!("Multi-GPU system ({} GPUs):\n", devices.len());
    
    for gpu in &devices {
        report.push_str(&format!(
            "  - {} (default={}, high-performance={})\n",
            gpu.name,
            gpu.is_default,
            gpu.is_high_performance()
        ));
    }
    
    // Check for potential issues
    let high_perf_gpus: Vec<_> = devices.iter()
        .filter(|d| d.is_high_performance())
        .collect();
    
    if high_perf_gpus.is_empty() {
        report.push_str("\nWarning: No high-performance GPU detected.");
    } else if high_perf_gpus.len() > 1 {
        report.push_str(&format!(
            "\nNote: Multiple high-performance GPUs detected ({}). \
             Ensure rendering and Syphon use the same GPU for optimal performance.",
            high_perf_gpus.len()
        ));
    }
    
    Ok(report)
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
    fn test_discovery_creation() {
        let discovery = SyphonDiscovery::new();
        assert!(discovery.sources.is_empty());
    }

    #[test]
    fn test_availability() {
        // Just make sure it doesn't panic
        let _available = is_syphon_available();
    }
}
