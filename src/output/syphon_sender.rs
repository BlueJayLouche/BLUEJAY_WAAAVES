//! # Syphon Output Sender (macOS)
//!
//! Sends video frames to a Syphon server for consumption by other macOS apps.
//!
//! This module provides two approaches:
//! 1. **SyphonSender** - CPU-based frame submission (compatibility mode)
//! 2. **SyphonWgpuSender** - Zero-copy GPU texture publishing (recommended)
//!
//! The zero-copy implementation is provided by the syphon-wgpu crate and uses
//! IOSurface-backed textures with Metal compute shaders for efficient Y-flip.

use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::thread::{self, JoinHandle};
use std::time::Instant;
use crossbeam::channel::{self, Sender as ChannelSender, Receiver};

// Re-export the high-performance wgpu output from external crate
pub use syphon_wgpu::SyphonWgpuOutput;

/// Syphon video frame data (CPU side)
pub struct SyphonFrameData {
    pub width: u32,
    pub height: u32,
    /// BGRA pixel data (native macOS format)
    pub data: Vec<u8>,
    pub timestamp: Instant,
}

/// CPU-based Syphon sender
///
/// This is a compatibility wrapper that uses CPU buffer submission.
/// For zero-copy GPU output, use `SyphonWgpuOutput` directly from the external crate.
///
/// **Deprecated**: Use `SyphonWgpuOutput` for better performance.
pub struct SyphonSender {
    name: String,
    width: u32,
    height: u32,
    frame_tx: ChannelSender<SyphonFrameData>,
    running: Arc<AtomicBool>,
    is_owner: bool,
}

impl SyphonSender {
    /// Create and start a new Syphon sender
    ///
    /// # Arguments
    /// * `name` - The Syphon server name (must be unique on the system)
    /// * `width` - Output width in pixels
    /// * `height` - Output height in pixels
    ///
    /// # Deprecated
    /// This creates a CPU-based sender. For zero-copy GPU output, use `SyphonWgpuOutput`.
    pub fn new(name: impl Into<String>, width: u32, height: u32) -> anyhow::Result<Self> {
        let name = name.into();
        
        if width == 0 || height == 0 {
            return Err(anyhow::anyhow!("Invalid dimensions: {}x{}", width, height));
        }
        
        // Check if Syphon is available
        if !Self::is_syphon_available() {
            return Err(anyhow::anyhow!(
                "Syphon.framework not available. Install from https://github.com/Syphon/Syphon-Framework"
            ));
        }
        
        // Create bounded channel for frame queue
        let (frame_tx, frame_rx) = channel::bounded(2);
        
        let running = Arc::new(AtomicBool::new(true));
        let running_clone = Arc::clone(&running);
        
        let name_clone = name.clone();
        
        // Spawn publish thread
        let thread_handle = thread::spawn(move || {
            Self::publish_thread(
                name_clone,
                width,
                height,
                frame_rx,
                running_clone,
            );
        });
        
        // Leak the thread handle to keep it running
        Box::leak(Box::new(thread_handle));
        
        log::info!("[Syphon] CPU Sender '{}' created at {}x{} (consider using SyphonWgpuOutput for zero-copy)", 
            name, width, height);
        
        Ok(Self {
            name,
            width,
            height,
            frame_tx,
            running,
            is_owner: true,
        })
    }
    
    /// Publish thread that owns the Syphon server
    #[cfg(target_os = "macos")]
    fn publish_thread(
        name: String,
        width: u32,
        height: u32,
        frame_rx: Receiver<SyphonFrameData>,
        running: Arc<AtomicBool>,
    ) {
        use syphon_core::SyphonServer;
        
        log::info!("[Syphon] Publish thread started for '{}'", name);
        
        // Create Syphon server using external crate
        let server = match SyphonServer::new(&name, width, height) {
            Ok(s) => s,
            Err(e) => {
                log::error!("[Syphon] Failed to create server '{}': {}", name, e);
                return;
            }
        };
        
        log::info!("[Syphon] Server '{}' created successfully", name);
        
        let mut frame_count = 0u64;
        let mut last_log = Instant::now();
        
        while running.load(Ordering::SeqCst) {
            match frame_rx.recv_timeout(std::time::Duration::from_millis(100)) {
                Ok(frame_data) => {
                    frame_count += 1;
                    
                    // For CPU-based publishing, we need to create a Metal texture
                    // from the buffer and publish it. This is not yet fully implemented
                    // in this compatibility wrapper.
                    //
                    // TODO: Implement CPU buffer publishing by:
                    // 1. Creating a Metal texture
                    // 2. Uploading the BGRA data
                    // 3. Publishing via server.publish_metal_texture()
                    
                    // For now, just log frame receipt
                    log::debug!("[Syphon] Frame {} received ({}x{}) - CPU publishing not fully implemented",
                        frame_count, frame_data.width, frame_data.height);
                    
                    // Log stats periodically
                    if last_log.elapsed().as_secs() >= 30 {
                        log::info!("[Syphon] {} frames received for '{}' (CPU mode)", frame_count, name);
                        last_log = Instant::now();
                    }
                }
                Err(channel::RecvTimeoutError::Timeout) => {
                    // No frame available, continue loop
                }
                Err(channel::RecvTimeoutError::Disconnected) => {
                    break;
                }
            }
        }
        
        // Cleanup - server is dropped automatically
        log::info!("[Syphon] Publish thread stopped for '{}' ({} frames total)", name, frame_count);
    }
    
    /// Publish thread stub for non-macOS platforms
    #[cfg(not(target_os = "macos"))]
    fn publish_thread(
        name: String,
        _width: u32,
        _height: u32,
        frame_rx: Receiver<SyphonFrameData>,
        running: Arc<AtomicBool>,
    ) {
        log::warn!("[Syphon] Publish thread for '{}' not available on this platform", name);
        
        // Just drain the queue
        while running.load(Ordering::SeqCst) {
            match frame_rx.recv_timeout(std::time::Duration::from_millis(100)) {
                Ok(_) => {}
                Err(channel::RecvTimeoutError::Timeout) => {}
                Err(channel::RecvTimeoutError::Disconnected) => break,
            }
        }
    }
    
    /// Submit a frame for publishing
    ///
    /// The data should be in BGRA format (native macOS).
    pub fn submit_frame(&self, bgra_data: &[u8], width: u32, height: u32) {
        // Validate dimensions
        if width != self.width || height != self.height {
            log::warn!("[Syphon] Frame size mismatch: expected {}x{}, got {}x{}",
                self.width, self.height, width, height);
            return;
        }
        
        if bgra_data.is_empty() {
            log::warn!("[Syphon] Empty frame data received");
            return;
        }
        
        let frame = SyphonFrameData {
            width,
            height,
            data: bgra_data.to_vec(),
            timestamp: Instant::now(),
        };
        
        // Try to send (non-blocking)
        match self.frame_tx.try_send(frame) {
            Ok(_) => {
                log::debug!("[Syphon] Frame queued: {}x{}", width, height);
            }
            Err(channel::TrySendError::Full(_)) => {
                log::debug!("[Syphon] Frame dropped - queue full");
            }
            Err(channel::TrySendError::Disconnected(_)) => {
                log::warn!("[Syphon] Frame channel disconnected");
            }
        }
    }
    
    /// Stop the sender
    pub fn stop(&mut self) {
        if !self.is_owner {
            return;
        }
        self.running.store(false, Ordering::SeqCst);
    }
    
    /// Check if sender is running
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }
    
    /// Get server name
    pub fn name(&self) -> &str {
        &self.name
    }
    
    /// Get dimensions
    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }
    
    /// Check if Syphon framework is available
    pub fn is_syphon_available() -> bool {
        syphon_core::is_available()
    }
}

impl Clone for SyphonSender {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            width: self.width,
            height: self.height,
            frame_tx: self.frame_tx.clone(),
            running: Arc::clone(&self.running),
            is_owner: false,
        }
    }
}

impl Drop for SyphonSender {
    fn drop(&mut self) {
        if self.is_owner {
            self.stop();
        }
    }
}

/// High-performance zero-copy Syphon sender using syphon-wgpu
///
/// This is a convenience wrapper around `SyphonWgpuOutput` from the external crate.
/// It provides the same interface as other output modules for consistency.
pub struct SyphonWgpuSender {
    output: Option<SyphonWgpuOutput>,
    name: String,
    width: u32,
    height: u32,
}

impl SyphonWgpuSender {
    /// Create a new zero-copy Syphon sender
    ///
    /// # Arguments
    /// * `name` - Server name visible to Syphon clients
    /// * `device` - wgpu device
    /// * `queue` - wgpu queue
    /// * `width` - Frame width
    /// * `height` - Frame height
    pub fn new(
        name: impl Into<String>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
    ) -> anyhow::Result<Self> {
        let name = name.into();
        
        let output = SyphonWgpuOutput::new(&name, device, queue, width, height)?;
        
        log::info!("[Syphon] Zero-copy sender '{}' created at {}x{} (zero-copy: {})",
            name, width, height, output.is_zero_copy());
        
        Ok(Self {
            output: Some(output),
            name,
            width,
            height,
        })
    }
    
    /// Publish a texture to Syphon (zero-copy)
    pub fn publish(&mut self, texture: &wgpu::Texture, device: &wgpu::Device, queue: &wgpu::Queue) {
        if let Some(ref mut output) = self.output {
            output.publish(texture, device, queue);
        }
    }
    
    /// Check if zero-copy is active
    pub fn is_zero_copy(&self) -> bool {
        self.output.as_ref().map_or(false, |o| o.is_zero_copy())
    }
    
    /// Get number of connected clients
    pub fn client_count(&self) -> usize {
        self.output.as_ref().map_or(0, |o| o.client_count())
    }
    
    /// Check if any clients are connected
    pub fn has_clients(&self) -> bool {
        self.client_count() > 0
    }
    
    /// Get server name
    pub fn name(&self) -> &str {
        &self.name
    }
    
    /// Get dimensions
    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}

impl Drop for SyphonWgpuSender {
    fn drop(&mut self) {
        log::debug!("[Syphon] Sender '{}' dropped", self.name);
    }
}

/// Convert RGBA data to BGRA (Syphon's preferred format on macOS)
pub fn rgba_to_bgra(rgba_data: &[u8]) -> Vec<u8> {
    let pixel_count = rgba_data.len() / 4;
    let mut bgra_data = vec![0u8; rgba_data.len()];
    
    for i in 0..pixel_count {
        let src_idx = i * 4;
        let dst_idx = i * 4;
        
        // Swap R and B
        bgra_data[dst_idx] = rgba_data[src_idx + 2];     // B <- R
        bgra_data[dst_idx + 1] = rgba_data[src_idx + 1]; // G <- G
        bgra_data[dst_idx + 2] = rgba_data[src_idx];     // R <- B
        bgra_data[dst_idx + 3] = rgba_data[src_idx + 3]; // A <- A
    }
    
    bgra_data
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rgba_to_bgra_conversion() {
        // Red pixel in RGBA
        let rgba = vec![255, 0, 0, 255];
        let bgra = rgba_to_bgra(&rgba);
        
        assert_eq!(bgra[0], 0);      // B
        assert_eq!(bgra[1], 0);      // G
        assert_eq!(bgra[2], 255);    // R
        assert_eq!(bgra[3], 255);    // A
    }

    #[test]
    fn test_syphon_sender_creation() {
        // This will fail on non-macOS, that's expected
        if !SyphonSender::is_syphon_available() {
            println!("Syphon not available (expected on non-macOS)");
            return;
        }
        
        // Can't actually test without macOS runtime
    }
}
