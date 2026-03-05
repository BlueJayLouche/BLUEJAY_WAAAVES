//! # Async Syphon Output (macOS)
//!
//! High-performance Syphon output with concurrent buffer processing.
//!
//! Mirrors AsyncNdiOutput architecture:
//! - Main thread: Record copy command, submit, start map_async
//! - Callback thread: When map completes, read data, send to Syphon, unmap
//! - Triple buffering prevents frame drops

use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use std::thread::{self, JoinHandle};

/// Buffer state
#[derive(Clone, Copy, Debug, PartialEq)]
enum BufferState {
    /// Buffer is free and ready for copy
    Free,
    /// Copy submitted to GPU, map_async started
    InFlight,
}

/// Syphon buffer with state tracking
struct TrackedBuffer {
    buffer: Arc<wgpu::Buffer>,
    state: Arc<Mutex<BufferState>>,
}

/// Async Syphon output processor
///
/// Triple-buffered GPU readback with concurrent Syphon publishing.
pub struct AsyncSyphonOutput {
    /// Triple buffered wgpu buffers
    buffers: Vec<TrackedBuffer>,
    /// Syphon sender (cloneable)
    syphon_sender: crate::output::SyphonSender,
    /// Device for polling
    _device: Arc<wgpu::Device>,
    /// Frame dimensions
    width: u32,
    height: u32,
    /// Background poll thread
    _poll_thread: Option<JoinHandle<()>>,
    /// Shutdown signal
    running: Arc<AtomicBool>,
}

impl AsyncSyphonOutput {
    /// Create new async Syphon output
    ///
    /// # Arguments
    /// * `device` - wgpu device
    /// * `syphon_sender` - Syphon sender instance
    /// * `width` - Frame width
    /// * `height` - Frame height
    pub fn new(
        device: &wgpu::Device,
        syphon_sender: crate::output::SyphonSender,
        width: u32,
        height: u32,
    ) -> Self {
        // Create triple-buffered readback buffers
        let buffer_size = (width * height * 4) as u64;
        let mut buffers = Vec::with_capacity(3);
        
        for i in 0..3 {
            let buffer = Arc::new(device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(&format!("Syphon Readback Buffer {}", i)),
                size: buffer_size,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            }));
            buffers.push(TrackedBuffer {
                buffer,
                state: Arc::new(Mutex::new(BufferState::Free)),
            });
        }
        
        let running = Arc::new(AtomicBool::new(true));
        
        // Start a poll thread
        let device_arc = Arc::new(device.clone());
        let running_poll = Arc::clone(&running);
        let poll_thread = thread::spawn(move || {
            while running_poll.load(Ordering::Relaxed) {
                let _ = device_arc.poll(wgpu::PollType::Poll);
                thread::sleep(std::time::Duration::from_micros(100));
            }
        });
        
        log::info!("[Syphon] Async output created: {}x{}", width, height);
        
        Self {
            buffers,
            syphon_sender,
            _device: Arc::new(device.clone()),
            width,
            height,
            _poll_thread: Some(poll_thread),
            running,
        }
    }
    
    /// Find a free buffer for copying
    ///
    /// Returns the buffer index and reference if available.
    pub fn acquire_buffer(&self) -> Option<(usize, &wgpu::Buffer)> {
        for (idx, tracked) in self.buffers.iter().enumerate() {
            let mut state = tracked.state.lock().unwrap();
            if *state == BufferState::Free {
                *state = BufferState::InFlight;
                // Return buffer reference
                let buffer_ptr = tracked.buffer.as_ref() as *const wgpu::Buffer;
                return Some((idx, unsafe { &*buffer_ptr }));
            }
        }
        
        // All buffers in flight - log occasionally
        static LAST_WARN: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let last = LAST_WARN.load(std::sync::atomic::Ordering::Relaxed);
        if now > last + 5 {
            LAST_WARN.store(now, std::sync::atomic::Ordering::Relaxed);
            log::warn!("[Syphon] All buffers in flight - frame dropped");
        }
        
        None
    }
    
    /// Start async processing of a buffer (call AFTER queue.submit)
    ///
    /// This spawns a thread that handles map completion and Syphon publishing.
    pub fn process_buffer_async(&self, buffer_idx: usize) {
        if buffer_idx >= self.buffers.len() {
            return;
        }
        
        let buffer = Arc::clone(&self.buffers[buffer_idx].buffer);
        let state = Arc::clone(&self.buffers[buffer_idx].state);
        let syphon_sender = self.syphon_sender.clone();
        let width = self.width;
        let height = self.height;
        
        // Spawn thread for this buffer's processing
        thread::spawn(move || {
            let slice = buffer.slice(..);
            
            // Channel for map completion
            let (tx, rx) = std::sync::mpsc::channel::<bool>();
            
            // Start async map
            slice.map_async(wgpu::MapMode::Read, move |result| {
                let _ = tx.send(result.is_ok());
            });
            
            // Wait for map with polling
            let mut mapped = false;
            for _ in 0..10000 {
                match rx.try_recv() {
                    Ok(true) => {
                        mapped = true;
                        break;
                    }
                    Ok(false) => {
                        log::warn!("[Syphon] Buffer {} map failed", buffer_idx);
                        break;
                    }
                    Err(std::sync::mpsc::TryRecvError::Empty) => {
                        thread::sleep(std::time::Duration::from_micros(100));
                    }
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                        break;
                    }
                }
            }
            
            if mapped {
                // Read data
                let data = slice.get_mapped_range();
                let frame_data: Vec<u8> = data.to_vec();
                drop(data);
                buffer.unmap();
                
                // Send to Syphon
                syphon_sender.submit_frame(&frame_data, width, height);
            } else {
                buffer.unmap();
                log::warn!("[Syphon] Buffer {} map timeout", buffer_idx);
            }
            
            // Mark buffer as free
            let mut state = state.lock().unwrap();
            *state = BufferState::Free;
        });
    }
    
    /// Check if all buffers are busy (for monitoring)
    pub fn is_overloaded(&self) -> bool {
        self.buffers.iter().all(|b| {
            *b.state.lock().unwrap() == BufferState::InFlight
        })
    }
    
    /// Get number of free buffers
    pub fn free_buffer_count(&self) -> usize {
        self.buffers.iter().filter(|b| {
            *b.state.lock().unwrap() == BufferState::Free
        }).count()
    }
}

impl Drop for AsyncSyphonOutput {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        if let Some(handle) = self._poll_thread.take() {
            let _ = handle.join();
        }
    }
}

/// Helper to integrate with the render loop
///
/// Usage in engine:
/// ```ignore
/// // In render loop:
/// if let Some((idx, buffer)) = syphon_output.acquire_buffer() {
///     encoder.copy_texture_to_buffer(output_texture, buffer, ...);
///     queue.submit([encoder.finish()]);
///     syphon_output.process_buffer_async(idx);
/// }
/// ```
pub struct SyphonOutputIntegration {
    async_output: Option<AsyncSyphonOutput>,
    enabled: bool,
}

impl SyphonOutputIntegration {
    /// Create new integration (disabled by default)
    pub fn new() -> Self {
        Self {
            async_output: None,
            enabled: false,
        }
    }
    
    /// Enable Syphon output
    pub fn enable(&mut self, device: &wgpu::Device, name: &str, width: u32, height: u32) -> anyhow::Result<()> {
        let sender = crate::output::SyphonSender::new(name, width, height)?;
        self.async_output = Some(AsyncSyphonOutput::new(device, sender, width, height));
        self.enabled = true;
        log::info!("[Syphon] Output enabled: {} at {}x{}", name, width, height);
        Ok(())
    }
    
    /// Disable output
    pub fn disable(&mut self) {
        self.async_output = None;
        self.enabled = false;
        log::info!("[Syphon] Output disabled");
    }
    
    /// Check if enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled && self.async_output.is_some()
    }
    
    /// Submit a frame (convenience method)
    pub fn submit_frame(&self, texture: &wgpu::Texture, device: &wgpu::Device, queue: &wgpu::Queue, encoder: &mut wgpu::CommandEncoder) {
        let Some(ref output) = self.async_output else {
            return;
        };
        
        let Some((idx, buffer)) = output.acquire_buffer() else {
            return;
        };
        
        // Copy texture to buffer
        encoder.copy_texture_to_buffer(
            texture.as_image_copy(),
            wgpu::ImageCopyBuffer {
                buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(self.async_output.as_ref().unwrap().width * 4),
                    rows_per_image: Some(self.async_output.as_ref().unwrap().height),
                },
            },
            wgpu::Extent3d {
                width: self.async_output.as_ref().unwrap().width,
                height: self.async_output.as_ref().unwrap().height,
                depth_or_array_layers: 1,
            },
        );
        
        // Note: caller must submit and call process_buffer_async
    }
    
    /// Get the async output for manual control
    pub fn async_output(&self) -> Option<&AsyncSyphonOutput> {
        self.async_output.as_ref()
    }
}

impl Default for SyphonOutputIntegration {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_state() {
        assert_ne!(BufferState::Free, BufferState::InFlight);
    }

    #[test]
    fn test_integration_creation() {
        let integration = SyphonOutputIntegration::new();
        assert!(!integration.is_enabled());
    }
}
