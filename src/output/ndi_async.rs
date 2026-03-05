//! # Async NDI Output
//!
//! High-performance NDI output with concurrent buffer processing.
//!
//! Architecture:
//! - Main thread: Record copy command, submit, start map_async for each buffer independently
//! - Callback thread pool: When map completes, read data, send to NDI, unmap, mark free
//! - No waiting - each buffer flows through independently

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

/// NDI buffer with state tracking
struct TrackedBuffer {
    buffer: Arc<wgpu::Buffer>,
    state: Arc<Mutex<BufferState>>,
}

/// Async NDI output processor
/// 
/// Each buffer is processed independently in its own thread when map completes.
pub struct AsyncNdiOutput {
    /// Triple buffered wgpu buffers with state tracking
    buffers: Vec<TrackedBuffer>,
    /// NDI sender (cloneable)
    ndi_sender: crate::output::NdiOutputSender,
    /// Device for polling
    _device: Arc<wgpu::Device>,
    /// Frame dimensions
    width: u32,
    height: u32,
    /// Background thread handle for polling
    _poll_thread: Option<JoinHandle<()>>,
    /// Shutdown signal
    running: Arc<AtomicBool>,
}

impl AsyncNdiOutput {
    /// Create new async NDI output
    pub fn new(
        device: &wgpu::Device,
        ndi_sender: crate::output::NdiOutputSender,
        width: u32,
        height: u32,
    ) -> Self {
        // Create triple-buffered readback buffers
        let buffer_size = (width * height * 4) as u64;
        let mut buffers = Vec::with_capacity(3);
        
        for i in 0..3 {
            let buffer = Arc::new(device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(&format!("NDI Readback Buffer {}", i)),
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
        
        // Start a poll thread to keep wgpu callbacks processing
        let device_arc = Arc::new(device.clone());
        let running_poll = Arc::clone(&running);
        let poll_thread = thread::spawn(move || {
            while running_poll.load(Ordering::Relaxed) {
                device_arc.poll(wgpu::PollType::Poll);
                thread::sleep(std::time::Duration::from_micros(100));
            }
        });
        
        Self {
            buffers,
            ndi_sender,
            _device: Arc::new(device.clone()),
            width,
            height,
            _poll_thread: Some(poll_thread),
            running,
        }
    }
    
    /// Find a free buffer for copying
    pub fn acquire_buffer(&self) -> Option<(usize, &wgpu::Buffer)> {
        for (idx, tracked) in self.buffers.iter().enumerate() {
            let mut state = tracked.state.lock().unwrap();
            if *state == BufferState::Free {
                *state = BufferState::InFlight;
                // Return the buffer reference
                let buffer_ptr = tracked.buffer.as_ref() as *const wgpu::Buffer;
                return Some((idx, unsafe { &*buffer_ptr }));
            }
        }
        // All buffers in flight - log this occasionally
        static LAST_WARN: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let last = LAST_WARN.load(std::sync::atomic::Ordering::Relaxed);
        if now > last + 5 {  // Warn at most every 5 seconds
            LAST_WARN.store(now, std::sync::atomic::Ordering::Relaxed);
            log::warn!("[NDI] All buffers in flight - frame dropped");
        }
        None
    }
    
    /// Start async processing of a buffer (call AFTER queue.submit)
    /// This spawns a thread that will handle the map completion and NDI sending
    pub fn process_buffer_async(&self, buffer_idx: usize) {
        if buffer_idx >= self.buffers.len() {
            return;
        }
        
        let buffer = Arc::clone(&self.buffers[buffer_idx].buffer);
        let state = Arc::clone(&self.buffers[buffer_idx].state);
        let ndi_sender = self.ndi_sender.clone();
        let width = self.width;
        let height = self.height;
        
        // Spawn a dedicated thread for this buffer's processing
        // This allows multiple buffers to be processed concurrently
        thread::spawn(move || {
            let slice = buffer.slice(..);
            
            // Channel for map completion
            let (tx, rx) = std::sync::mpsc::channel::<bool>();
            
            // Start async map
            slice.map_async(wgpu::MapMode::Read, move |result| {
                let _ = tx.send(result.is_ok());
            });
            
            // Wait for map with short polling loop (allows concurrent processing)
            let mut mapped = false;
            for _ in 0..10000 { // Max ~1 second wait
                match rx.try_recv() {
                    Ok(true) => {
                        mapped = true;
                        break;
                    }
                    Ok(false) => {
                        log::warn!("[NDI] Buffer {} map failed", buffer_idx);
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
                // Read data efficiently
                let data = slice.get_mapped_range();
                // Direct slice copy instead of iterator
                let frame_data: Vec<u8> = data.to_vec();
                drop(data);
                buffer.unmap();
                
                // Send to NDI
                ndi_sender.submit_frame(&frame_data, width, height);
            } else {
                buffer.unmap();
                log::warn!("[NDI] Buffer {} map timeout or failed", buffer_idx);
            }
            
            // Mark buffer as free
            let mut state = state.lock().unwrap();
            *state = BufferState::Free;
        });
    }
    
    /// Poll for state changes (optional, mainly for logging)
    pub fn poll(&self) {
        // State changes happen in background threads, just check counts for logging
        let free_count = self.buffers.iter()
            .filter(|b| *b.state.lock().unwrap() == BufferState::Free)
            .count();
        
        if free_count == 0 {
            log::debug!("[NDI] All buffers in flight");
        }
    }
}

impl Drop for AsyncNdiOutput {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        if let Some(handle) = self._poll_thread.take() {
            let _ = handle.join();
        }
    }
}
