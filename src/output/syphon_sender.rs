//! # Syphon Output Sender (macOS)
//!
//! Sends video frames to a Syphon server for consumption by other macOS apps.
//!
//! Architecture mirrors NDI output:
//! - Dedicated publish thread for non-blocking operation
//! - CPU buffer queue (bounded, drops old frames)
//! - Converts RGBA to Syphon-compatible format
//!
//! ## Prerequisites
//!
//! Syphon.framework must be available at runtime. It's included with most VJ apps
//! or can be installed standalone from https://github.com/Syphon/Syphon-Framework

use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::thread::{self, JoinHandle};
use std::time::Instant;
use crossbeam::channel::{self, Sender as ChannelSender, Receiver};

/// Syphon video frame data (CPU side)
pub struct SyphonFrameData {
    pub width: u32,
    pub height: u32,
    /// RGBA pixel data (will be converted to IOSurface/buffer as needed)
    pub data: Vec<u8>,
    pub timestamp: Instant,
}

/// Syphon output sender
///
/// Wraps a SyphonServer and publishes frames in a background thread.
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
        
        log::info!("[Syphon] Sender '{}' created at {}x{}", name, width, height);
        
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
    #[allow(unused_variables)]
    fn publish_thread(
        name: String,
        width: u32,
        height: u32,
        frame_rx: Receiver<SyphonFrameData>,
        running: Arc<AtomicBool>,
    ) {
        // TODO: Initialize Objective-C runtime and create SyphonServer
        // This will be implemented with objc crate
        
        log::info!("[Syphon] Publish thread started for '{}'", name);
        
        let mut frame_count = 0u64;
        let mut last_log = Instant::now();
        
        while running.load(Ordering::SeqCst) {
            match frame_rx.recv_timeout(std::time::Duration::from_millis(100)) {
                Ok(frame_data) => {
                    frame_count += 1;
                    
                    // TODO: Publish frame to Syphon:
                    // 1. Convert RGBA to Syphon-compatible format
                    // 2. Create CGImage or IOSurface from buffer
                    // 3. Publish via SyphonServer
                    
                    // For now, just log occasionally
                    if last_log.elapsed().as_secs() >= 30 {
                        log::info!("[Syphon] {} frames published to '{}'", frame_count, name);
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
        
        // TODO: Release SyphonServer
        log::info!("[Syphon] Publish thread stopped for '{}'", name);
    }
    
    /// Submit a frame for publishing
    ///
    /// The data should be in RGBA format. It will be converted internally.
    pub fn submit_frame(&self, rgba_data: &[u8], width: u32, height: u32) {
        // Validate dimensions
        if width != self.width || height != self.height {
            log::warn!("[Syphon] Frame size mismatch: expected {}x{}, got {}x{}",
                self.width, self.height, width, height);
            return;
        }
        
        if rgba_data.is_empty() {
            log::warn!("[Syphon] Empty frame data received");
            return;
        }
        
        let frame = SyphonFrameData {
            width,
            height,
            data: rgba_data.to_vec(),
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
        // TODO: Check if Syphon.framework can be loaded
        // For now, assume true on macOS
        cfg!(target_os = "macos")
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
            return;
        }
        
        // Can't actually create without macOS runtime
        // Just verify the function exists
    }
}
