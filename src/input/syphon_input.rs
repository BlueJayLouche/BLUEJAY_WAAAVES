//! # Syphon Input Receiver (macOS)
//!
//! Receives video frames from Syphon servers on the local machine.
//!
//! ## Architecture
//!
//! - Background thread polls for new frames from SyphonClient
//! - Frames are queued for the main thread to consume
//! - Supports both GPU (IOSurface) and CPU (buffer) modes

use std::sync::{mpsc, Arc, atomic::{AtomicBool, Ordering}};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

/// Information about a discovered Syphon server
#[derive(Debug, Clone)]
pub struct SyphonServerInfo {
    pub name: String,
    pub app_name: String,
    pub dimensions: Option<(u32, u32)>,
    /// UUID for connection
    pub uuid: String,
}

/// A received Syphon frame
pub struct SyphonFrame {
    pub width: u32,
    pub height: u32,
    /// RGBA pixel data
    pub data: Vec<u8>,
    pub timestamp: Instant,
}

/// Syphon input receiver
///
/// Connects to a Syphon server and receives frames in a background thread.
pub struct SyphonInputReceiver {
    server_name: Option<String>,
    receiver_thread: Option<JoinHandle<()>>,
    frame_tx: mpsc::Sender<SyphonFrame>,
    frame_rx: mpsc::Receiver<SyphonFrame>,
    running: Arc<AtomicBool>,
    resolution: (u32, u32),
}

impl SyphonInputReceiver {
    /// Create a new Syphon input receiver
    pub fn new() -> Self {
        let (frame_tx, frame_rx) = mpsc::channel();
        
        Self {
            server_name: None,
            receiver_thread: None,
            frame_tx,
            frame_rx,
            running: Arc::new(AtomicBool::new(false)),
            resolution: (1920, 1080),
        }
    }
    
    /// Check if Syphon is available
    pub fn is_available() -> bool {
        cfg!(target_os = "macos")
    }
    
    /// Connect to a Syphon server by name
    pub fn connect(&mut self, server_name: impl Into<String>) -> anyhow::Result<()> {
        let server_name = server_name.into();
        
        if self.is_connected() {
            self.disconnect();
        }
        
        log::info!("[Syphon Input] Connecting to server: {}", server_name);
        
        // TODO: Initialize Objective-C runtime
        // 1. Look up server in SyphonServerDirectory
        // 2. Create SyphonClient for the server
        // 3. Start receive thread
        
        let running = Arc::clone(&self.running);
        running.store(true, Ordering::SeqCst);
        
        let frame_tx = self.frame_tx.clone();
        let name_clone = server_name.clone();
        
        let thread_handle = thread::spawn(move || {
            Self::receive_thread(name_clone, frame_tx, running);
        });
        
        self.server_name = Some(server_name);
        self.receiver_thread = Some(thread_handle);
        
        Ok(())
    }
    
    /// Receive thread that polls SyphonClient
    #[allow(unused_variables)]
    fn receive_thread(
        server_name: String,
        frame_tx: mpsc::Sender<SyphonFrame>,
        running: Arc<AtomicBool>,
    ) {
        log::info!("[Syphon Input] Receive thread started for '{}'", server_name);
        
        // TODO: Initialize SyphonClient
        // let client = create_syphon_client(&server_name);
        
        while running.load(Ordering::SeqCst) {
            // TODO: Check for new frame
            // if client.has_new_frame() {
            //     let iosurface = client.new_frame_image();
            //     let (width, height) = get_dimensions(&iosurface);
            //     
            //     // Read pixels from IOSurface
            //     let data = read_iosurface_pixels(iosurface);
            //     
            //     let frame = SyphonFrame {
            //         width,
            //         height,
            //         data,
            //         timestamp: Instant::now(),
            //     };
            //     
            //     // Send to main thread (non-blocking)
            //     if frame_tx.try_send(frame).is_err() {
            //         // Queue full, drop frame
            //     }
            // }
            
            thread::sleep(Duration::from_millis(1));
        }
        
        log::info!("[Syphon Input] Receive thread stopped for '{}'", server_name);
    }
    
    /// Disconnect from current server
    pub fn disconnect(&mut self) {
        if !self.is_connected() {
            return;
        }
        
        log::info!("[Syphon Input] Disconnecting from: {:?}", self.server_name);
        
        self.running.store(false, Ordering::SeqCst);
        
        if let Some(handle) = self.receiver_thread.take() {
            let _ = handle.join();
        }
        
        // Clear any pending frames
        while self.frame_rx.try_recv().is_ok() {}
        
        self.server_name = None;
    }
    
    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.server_name.is_some()
    }
    
    /// Get the latest frame (non-blocking, consumes the frame)
    pub fn get_latest_frame(&mut self) -> Option<SyphonFrame> {
        // Drain all frames and return only the most recent
        let mut latest: Option<SyphonFrame> = None;
        
        while let Ok(frame) = self.frame_rx.try_recv() {
            self.resolution = (frame.width, frame.height);
            latest = Some(frame);
        }
        
        latest
    }
    
    /// Check if a new frame is available (approximate)
    /// 
    /// Note: This consumes and re-queues the frame if available.
    /// For precise checking, use get_latest_frame() and check the result.
    pub fn has_frame(&self) -> bool {
        // mpsc doesn't have is_empty, so we track this differently
        // In a real implementation, use crossbeam or track state
        false // Placeholder - implementation will use proper channel
    }
    
    /// Get current resolution
    pub fn resolution(&self) -> (u32, u32) {
        self.resolution
    }
    
    /// Get connected server name
    pub fn server_name(&self) -> Option<&str> {
        self.server_name.as_deref()
    }
}

impl Default for SyphonInputReceiver {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for SyphonInputReceiver {
    fn drop(&mut self) {
        self.disconnect();
    }
}

/// Syphon server discovery
///
/// Scans for available Syphon servers on the local machine.
pub struct SyphonDiscovery;

impl SyphonDiscovery {
    /// Create new discovery
    pub fn new() -> Self {
        Self
    }
    
    /// Discover available Syphon servers
    ///
    /// Returns a list of available servers.
    pub fn discover_servers(&self) -> Vec<SyphonServerInfo> {
        log::debug!("[Syphon] Discovering servers...");
        
        let mut servers = Vec::new();
        
        // TODO: Use SyphonServerDirectory to get list of servers
        // 
        // Pseudocode:
        // let directory = SyphonServerDirectory::shared_directory();
        // for server_description in directory.servers() {
        //     servers.push(SyphonServerInfo {
        //         name: server_description.name(),
        //         app_name: server_description.app_name(),
        //         dimensions: Some((server_description.width(), server_description.height())),
        //         uuid: server_description.uuid(),
        //     });
        // }
        
        servers
    }
    
    /// Check if a specific server is still available
    pub fn is_server_available(&self, name: &str) -> bool {
        let servers = self.discover_servers();
        servers.iter().any(|s| s.name == name)
    }
}

impl Default for SyphonDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

/// Integration with the input system
///
/// Wraps SyphonInputReceiver with convenience methods for the engine.
pub struct SyphonInputIntegration {
    receiver: Option<SyphonInputReceiver>,
    discovery: SyphonDiscovery,
    cached_servers: Vec<SyphonServerInfo>,
    last_discovery: Option<Instant>,
}

impl SyphonInputIntegration {
    /// Create new integration
    pub fn new() -> Self {
        Self {
            receiver: None,
            discovery: SyphonDiscovery::new(),
            cached_servers: Vec::new(),
            last_discovery: None,
        }
    }
    
    /// Check if Syphon is available
    pub fn is_available() -> bool {
        SyphonInputReceiver::is_available()
    }
    
    /// Refresh the list of available servers
    pub fn refresh_servers(&mut self) {
        self.cached_servers = self.discovery.discover_servers();
        self.last_discovery = Some(Instant::now());
        log::info!("[Syphon] Discovered {} servers", self.cached_servers.len());
    }
    
    /// Get cached server list (may be stale)
    pub fn servers(&self) -> &[SyphonServerInfo] {
        &self.cached_servers
    }
    
    /// Connect to a server by name
    pub fn connect(&mut self, server_name: &str) -> anyhow::Result<()> {
        if self.receiver.is_some() {
            self.disconnect();
        }
        
        let mut receiver = SyphonInputReceiver::new();
        receiver.connect(server_name)?;
        self.receiver = Some(receiver);
        
        Ok(())
    }
    
    /// Disconnect from current server
    pub fn disconnect(&mut self) {
        self.receiver = None;
    }
    
    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.receiver.as_ref().map_or(false, |r| r.is_connected())
    }
    
    /// Get latest frame data for GPU upload
    pub fn get_frame_data(&mut self) -> Option<(u32, u32, Vec<u8>)> {
        let receiver = self.receiver.as_mut()?;
        let frame = receiver.get_latest_frame()?;
        Some((frame.width, frame.height, frame.data))
    }
    
    /// Update (called each frame by engine)
    pub fn update(&mut self) {
        // Auto-refresh discovery every 5 seconds
        if self.last_discovery.map_or(true, |t| t.elapsed().as_secs() > 5) {
            self.refresh_servers();
        }
    }
    
    /// Get connected server name
    pub fn connected_server(&self) -> Option<&str> {
        self.receiver.as_ref()?.server_name()
    }
}

impl Default for SyphonInputIntegration {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_receiver_creation() {
        let receiver = SyphonInputReceiver::new();
        assert!(!receiver.is_connected());
    }

    #[test]
    fn test_discovery_creation() {
        let discovery = SyphonDiscovery::new();
        let servers = discovery.discover_servers();
        // Should return empty list (no implementation yet)
        assert!(servers.is_empty());
    }

    #[test]
    fn test_integration_creation() {
        let integration = SyphonInputIntegration::new();
        assert!(!integration.is_connected());
        assert!(integration.servers().is_empty());
    }
}
