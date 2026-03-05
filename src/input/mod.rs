//! # Input Module
//!
//! Handles video input sources including:
//! - Webcam capture (via nokhwa)
//! - NDI input
//! - Spout (Windows only)
//! - Video file playback

use anyhow::Result;
use std::sync::{mpsc, Arc};
use wgpu::Device;

#[cfg(feature = "webcam")]
pub mod webcam;
#[cfg(feature = "webcam")]
pub use webcam::{WebcamCapture, WebcamFrame};

// NDI input support
pub mod ndi;
pub use ndi::{NdiReceiver, list_ndi_sources, is_ndi_available};

// Syphon input support (macOS)
#[cfg(target_os = "macos")]
pub mod syphon_input;
#[cfg(target_os = "macos")]
pub use syphon_input::{SyphonInputReceiver, SyphonDiscovery, SyphonInputIntegration, SyphonServerInfo};

#[cfg(not(feature = "webcam"))]
pub struct WebcamFrame {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
    pub timestamp: std::time::Instant,
}

#[cfg(not(feature = "webcam"))]
pub struct WebcamCapture;

#[cfg(not(feature = "webcam"))]
impl WebcamCapture {
    pub fn new(_device_index: usize, _width: u32, _height: u32, _fps: u32) -> anyhow::Result<Self> {
        Err(anyhow::anyhow!("Webcam support not compiled. Enable the 'webcam' feature."))
    }
    
    pub fn start(&mut self) -> anyhow::Result<std::sync::mpsc::Receiver<WebcamFrame>> {
        unreachable!()
    }
    
    pub fn stop(&mut self) -> anyhow::Result<()> {
        Ok(())
    }
}

#[cfg(not(feature = "webcam"))]
pub fn list_cameras() -> Vec<String> {
    Vec::new()
}

mod texture_input;
pub use texture_input::InputTextureManager;

/// Input source types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputType {
    None,
    Webcam,
    Ndi,
    Spout,
    VideoFile,
}

/// Input manager handles multiple video input sources
pub struct InputManager {
    /// Input 1 configuration
    pub input1: InputSource,
    /// Input 2 configuration
    pub input2: InputSource,
    /// Available webcam devices
    webcam_devices: Vec<WebcamDeviceInfo>,
    /// Device needs refresh
    devices_dirty: bool,
}

/// Webcam device information
#[derive(Debug, Clone)]
pub struct WebcamDeviceInfo {
    pub index: usize,
    pub name: String,
}

/// Individual input source
pub struct InputSource {
    /// Type of input
    pub input_type: InputType,
    /// Device index (for webcam)
    pub device_index: i32,
    /// NDI source name
    pub ndi_source: Option<String>,
    /// Current texture (if available) - stored as wgpu texture
    /// Note: In the wgpu version, textures are managed by the engine
    pub texture_id: Option<usize>,
    /// Input resolution
    pub resolution: (u32, u32),
    /// Whether input is active
    pub active: bool,
    /// Webcam capture instance
    webcam: Option<WebcamCapture>,
    /// Frame receiver channel
    frame_receiver: Option<mpsc::Receiver<WebcamFrame>>,
    /// Current frame data (CPU side)
    current_frame: Option<Vec<u8>>,
    /// NDI receiver instance
    ndi_receiver: Option<NdiReceiver>,
}

impl InputManager {
    /// Create a new input manager
    pub fn new() -> Self {
        // Scan for webcam devices safely (only if webcam feature enabled)
        #[cfg(feature = "webcam")]
        let device_strings = std::panic::catch_unwind(|| {
            webcam::list_cameras()
        }).unwrap_or_else(|_| {
            log::error!("Webcam enumeration panicked");
            Vec::new()
        });
        
        #[cfg(not(feature = "webcam"))]
        let device_strings: Vec<String> = Vec::new();
        
        log::info!("InputManager found {} webcam devices", device_strings.len());
        
        // Convert to WebcamDeviceInfo
        let webcam_devices: Vec<WebcamDeviceInfo> = device_strings
            .into_iter()
            .enumerate()
            .map(|(idx, name)| WebcamDeviceInfo { index: idx, name })
            .collect();
        
        Self {
            input1: InputSource::new(InputType::None),
            input2: InputSource::new(InputType::None),
            webcam_devices,
            devices_dirty: false,
        }
    }
    
    /// Initialize inputs with wgpu device
    pub fn initialize(&mut self, device: Arc<Device>, queue: Arc<wgpu::Queue>) -> Result<()> {
        // Clone device and queue for input sources
        self.input1.initialize(&device, &queue)?;
        self.input2.initialize(&device, &queue)?;
        Ok(())
    }
    
    /// Update inputs (capture new frames)
    pub fn update(&mut self) {
        self.input1.update();
        self.input2.update();
    }
    
    /// Get input 1 texture id
    pub fn get_input1_texture_id(&self) -> Option<usize> {
        self.input1.texture_id
    }
    
    /// Get input 2 texture id
    pub fn get_input2_texture_id(&self) -> Option<usize> {
        self.input2.texture_id
    }
    
    /// Get input 1 current frame data (for uploading to GPU)
    pub fn get_input1_frame(&self) -> Option<&[u8]> {
        self.input1.current_frame.as_deref()
    }
    
    /// Get input 2 current frame data (for uploading to GPU)
    pub fn get_input2_frame(&self) -> Option<&[u8]> {
        self.input2.current_frame.as_deref()
    }
    
    /// Refresh available webcam devices
    #[cfg(feature = "webcam")]
    pub fn refresh_webcam_devices(&mut self) -> Vec<String> {
        self.webcam_devices = webcam::list_cameras()
            .into_iter()
            .enumerate()
            .map(|(idx, name)| WebcamDeviceInfo { index: idx, name })
            .collect();
        
        self.devices_dirty = false;
        
        self.webcam_devices.iter().map(|d| d.name.clone()).collect()
    }

    /// Refresh available webcam devices (no-op when webcam feature disabled)
    #[cfg(not(feature = "webcam"))]
    pub fn refresh_webcam_devices(&mut self) -> Vec<String> {
        Vec::new()
    }
    
    /// Get list of available webcam devices
    pub fn get_webcam_devices(&self) -> Vec<String> {
        if self.devices_dirty {
            // Return empty if not yet refreshed
            Vec::new()
        } else {
            self.webcam_devices.iter().map(|d| d.name.clone()).collect()
        }
    }
    
    /// Start webcam capture on input 1
    pub fn start_input1_webcam(&mut self, device_index: usize, width: u32, height: u32, fps: u32) -> Result<()> {
        self.input1.start_webcam(device_index, width, height, fps)
    }
    
    /// Start webcam capture on input 2
    pub fn start_input2_webcam(&mut self, device_index: usize, width: u32, height: u32, fps: u32) -> Result<()> {
        self.input2.start_webcam(device_index, width, height, fps)
    }
    
    /// Start NDI on input 1
    pub fn start_input1_ndi(&mut self, source_name: impl Into<String>) -> Result<()> {
        self.input1.start_ndi(source_name)
    }
    
    /// Start NDI on input 2
    pub fn start_input2_ndi(&mut self, source_name: impl Into<String>) -> Result<()> {
        self.input2.start_ndi(source_name)
    }
    
    /// Stop input 1
    pub fn stop_input1(&mut self) {
        self.input1.stop();
    }
    
    /// Stop input 2
    pub fn stop_input2(&mut self) {
        self.input2.stop();
    }
    
    /// Get input 1 resolution
    pub fn get_input1_resolution(&self) -> (u32, u32) {
        self.input1.resolution
    }
    
    /// Get input 2 resolution
    pub fn get_input2_resolution(&self) -> (u32, u32) {
        self.input2.resolution
    }
    
    /// Check if input 1 has a new frame
    pub fn input1_has_new_frame(&self) -> bool {
        self.input1.has_new_frame()
    }
    
    /// Check if input 2 has a new frame
    pub fn input2_has_new_frame(&self) -> bool {
        self.input2.has_new_frame()
    }
    
    /// Take input 1 frame data (consumes the frame)
    pub fn take_input1_frame(&mut self) -> Option<Vec<u8>> {
        self.input1.take_frame()
    }
    
    /// Take input 2 frame data (consumes the frame)
    pub fn take_input2_frame(&mut self) -> Option<Vec<u8>> {
        self.input2.take_frame()
    }
}

impl InputSource {
    /// Create a new input source
    pub fn new(input_type: InputType) -> Self {
        Self {
            input_type,
            device_index: -1,
            ndi_source: None,
            texture_id: None,
            resolution: (640, 480),
            active: false,
            webcam: None,
            frame_receiver: None,
            current_frame: None,
            ndi_receiver: None,
        }
    }
    
    /// Initialize the input source
    pub fn initialize(&mut self, _device: &wgpu::Device, _queue: &wgpu::Queue) -> Result<()> {
        match self.input_type {
            InputType::None => {}
            InputType::Webcam => {
                // Webcam is started via start_webcam()
            }
            InputType::Ndi => {
                // TODO: Initialize NDI receiver
            }
            InputType::Spout => {
                // TODO: Initialize Spout receiver
            }
            InputType::VideoFile => {
                // TODO: Initialize video decoder
            }
        }
        Ok(())
    }
    
    /// Start webcam capture
    pub fn start_webcam(&mut self, device_index: usize, width: u32, height: u32, fps: u32) -> Result<()> {
        // Stop any existing capture
        self.stop();
        
        // Create new webcam capture
        let mut webcam = WebcamCapture::new(device_index, width, height, fps)?;
        let receiver = webcam.start()?;
        
        self.input_type = InputType::Webcam;
        self.device_index = device_index as i32;
        self.resolution = (width, height);
        self.active = true;
        self.webcam = Some(webcam);
        self.frame_receiver = Some(receiver);
        
        Ok(())
    }
    
    /// Start NDI receiver
    pub fn start_ndi(&mut self, source_name: impl Into<String>) -> Result<()> {
        self.stop();
        
        let source_name = source_name.into();
        let mut ndi = NdiReceiver::new(source_name.clone());
        ndi.start()?;
        
        self.input_type = InputType::Ndi;
        self.ndi_source = Some(source_name);
        self.active = true;
        self.ndi_receiver = Some(ndi);
        
        Ok(())
    }
    
    /// Stop the input source
    pub fn stop(&mut self) {
        self.active = false;
        
        if let Some(mut webcam) = self.webcam.take() {
            let _ = webcam.stop();
        }
        
        if let Some(mut ndi) = self.ndi_receiver.take() {
            ndi.stop();
        }
        
        self.frame_receiver = None;
        self.current_frame = None;
        self.input_type = InputType::None;
        self.device_index = -1;
        self.ndi_source = None;
    }
    
    /// Update (check for new frames)
    pub fn update(&mut self) {
        if !self.active {
            return;
        }
        
        // Handle webcam frames
        if let Some(ref receiver) = self.frame_receiver {
            let mut latest_frame: Option<WebcamFrame> = None;
            
            // Drain the channel
            while let Ok(frame) = receiver.try_recv() {
                latest_frame = Some(frame);
            }
            
            // Use the most recent frame if we got any
            if let Some(frame) = latest_frame {
                self.resolution = (frame.width, frame.height);
                self.current_frame = Some(frame.data);
            }
        }
        
        // Handle NDI frames
        if let Some(ref mut ndi) = self.ndi_receiver {
            if let Some(frame) = ndi.get_latest_frame() {
                self.resolution = (frame.width, frame.height);
                self.current_frame = Some(frame.data);
            }
        }
    }
    
    /// Check if there's a new frame available
    pub fn has_new_frame(&self) -> bool {
        self.current_frame.is_some()
    }
    
    /// Take the current frame data (consumes it)
    pub fn take_frame(&mut self) -> Option<Vec<u8>> {
        self.current_frame.take()
    }
    
    /// Get input type
    pub fn get_type(&self) -> InputType {
        self.input_type
    }
    
    /// Check if input is active
    pub fn is_active(&self) -> bool {
        self.active
    }
}

impl Drop for InputSource {
    fn drop(&mut self) {
        self.stop();
    }
}
