//! # Input Texture Manager
//!
//! Manages GPU textures for video input sources and handles frame upload.

use crate::engine::texture::Texture;
use std::sync::Arc;

/// GPU texture for video input
pub struct InputTexture {
    /// The wgpu texture
    pub texture: Texture,
    /// Current resolution
    pub width: u32,
    pub height: u32,
    /// Whether texture has valid data
    pub has_data: bool,
    /// Device index for this texture
    pub device_index: Option<usize>,
}

/// Manager for input textures
pub struct InputTextureManager {
    /// Input 1 texture
    pub input1: Option<InputTexture>,
    /// Input 2 texture  
    pub input2: Option<InputTexture>,
    /// Default/placeholder texture (black)
    default_texture: Texture,
    /// Device for creating textures
    device: Arc<wgpu::Device>,
    /// Queue for uploading data
    queue: Arc<wgpu::Queue>,
}

impl InputTextureManager {
    /// Create new input texture manager
    pub fn new(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) -> Self {
        // Create default black texture at standard resolution
        // This matches the OF app's dummyTexture approach
        let default_width = 1280u32;
        let default_height = 720u32;
        let black_data = vec![0u8; (default_width * default_height * 4) as usize];
        
        let default_texture = Texture::from_bytes(
            &device,
            &queue,
            &black_data,
            default_width,
            default_height,
            "Default Black Texture",
        );
        
        log::info!("Created default black texture: {}x{}", default_width, default_height);
        
        Self {
            input1: None,
            input2: None,
            default_texture,
            device,
            queue,
        }
    }
    
    /// Initialize input 1 texture
    pub fn init_input1(&mut self, width: u32, height: u32, device_index: usize) {
        let texture = Texture::create_render_target(
            &self.device,
            width,
            height,
            "Input 1 Texture",
        );
        
        // Clear to black to avoid garbage data
        texture.clear_to_black(&self.queue);
        
        self.input1 = Some(InputTexture {
            texture,
            width,
            height,
            has_data: false,
            device_index: Some(device_index),
        });
        
        log::info!("Initialized input 1 texture: {}x{}", width, height);
    }
    
    /// Initialize input 2 texture
    pub fn init_input2(&mut self, width: u32, height: u32, device_index: usize) {
        let texture = Texture::create_render_target(
            &self.device,
            width,
            height,
            "Input 2 Texture",
        );
        
        // Clear to black to avoid garbage data
        texture.clear_to_black(&self.queue);
        
        self.input2 = Some(InputTexture {
            texture,
            width,
            height,
            has_data: false,
            device_index: Some(device_index),
        });
        
        log::info!("Initialized input 2 texture: {}x{}", width, height);
    }
    
    /// Update input 1 with new frame data
    pub fn update_input1(&mut self, data: &[u8], width: u32, height: u32) {
        // Validate data size (RGBA = 4 bytes per pixel)
        let expected_size = (width * height * 4) as usize;
        
        // Allow some tolerance for stride/padding differences
        let min_size = ((width - 1) * 4 + width * (height - 1) * 4 + 4) as usize; // At least one row + one pixel
        let max_size = ((width + 1) * 4 * height as u32) as usize; // Allow some padding
        
        if data.len() < min_size || (data.len() > max_size && data.len() != expected_size) {
            log::warn!("Input 1 frame data size mismatch: got {} bytes, expected ~{} for {}x{}", 
                data.len(), expected_size, width, height);
            return;
        }
        
        // If sizes don't match exactly but are close, trim or use as-is
        let data_to_use = if data.len() == expected_size {
            data
        } else if data.len() > expected_size {
            // Trim excess data (likely stride padding at end)
            &data[..expected_size]
        } else {
            // This shouldn't happen with our min_size check, but handle gracefully
            log::warn!("Input 1 frame data too small: {} < {}", data.len(), expected_size);
            return;
        };
        
        self.update_input1_internal(data_to_use, width, height);
    }
    
    fn update_input1_internal(&mut self, data: &[u8], width: u32, height: u32) {
        
        // Recreate texture if size changed
        if let Some(ref mut input) = self.input1 {
            if input.width != width || input.height != height {
                input.texture = Texture::create_render_target(
                    &self.device,
                    width,
                    height,
                    "Input 1 Texture",
                );
                input.width = width;
                input.height = height;
            }
        } else {
            // Create new texture
            self.init_input1(width, height, 0);
        }
        
        // Upload data
        if let Some(ref mut input) = self.input1 {
            self.queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &input.texture.texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                data,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * width),
                    rows_per_image: Some(height),
                },
                wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
            );
            input.has_data = true;
        }
    }
    
    /// Update input 2 with new frame data
    pub fn update_input2(&mut self, data: &[u8], width: u32, height: u32) {
        // Validate data size (RGBA = 4 bytes per pixel)
        let expected_size = (width * height * 4) as usize;
        
        // Allow some tolerance for stride/padding differences
        let min_size = ((width - 1) * 4 + width * (height - 1) * 4 + 4) as usize;
        let max_size = ((width + 1) * 4 * height as u32) as usize;
        
        if data.len() < min_size || (data.len() > max_size && data.len() != expected_size) {
            log::warn!("Input 2 frame data size mismatch: got {} bytes, expected ~{} for {}x{}", 
                data.len(), expected_size, width, height);
            return;
        }
        
        // Trim excess data if needed
        let data_to_use: &[u8];
        if data.len() == expected_size {
            data_to_use = data;
        } else if data.len() > expected_size {
            data_to_use = &data[..expected_size];
        } else {
            log::warn!("Input 2 frame data too small: {} < {}", data.len(), expected_size);
            return;
        }
        
        if let Some(ref mut input) = self.input2 {
            if input.width != width || input.height != height {
                input.texture = Texture::create_render_target(
                    &self.device,
                    width,
                    height,
                    "Input 2 Texture",
                );
                input.width = width;
                input.height = height;
            }
        } else {
            self.init_input2(width, height, 0);
        }
        
        if let Some(ref mut input) = self.input2 {
            self.queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &input.texture.texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                data_to_use,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * width),
                    rows_per_image: Some(height),
                },
                wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
            );
            input.has_data = true;
        }
    }
    
    /// Get input 1 texture view (or default if not available)
    pub fn get_input1_view(&self) -> &wgpu::TextureView {
        self.input1
            .as_ref()
            .map(|i| &i.texture.view)
            .unwrap_or(&self.default_texture.view)
    }
    
    /// Get input 2 texture view (or default if not available)
    pub fn get_input2_view(&self) -> &wgpu::TextureView {
        self.input2
            .as_ref()
            .map(|i| &i.texture.view)
            .unwrap_or(&self.default_texture.view)
    }
    
    /// Get default texture view for unassigned inputs
    pub fn get_default_view(&self) -> &wgpu::TextureView {
        &self.default_texture.view
    }
    
    /// Check if input 1 has valid data
    pub fn input1_has_data(&self) -> bool {
        self.input1.as_ref().map(|i| i.has_data).unwrap_or(false)
    }
    
    /// Check if input 2 has valid data
    pub fn input2_has_data(&self) -> bool {
        self.input2.as_ref().map(|i| i.has_data).unwrap_or(false)
    }
    
    /// Get input 1 resolution
    pub fn get_input1_resolution(&self) -> (u32, u32) {
        self.input1
            .as_ref()
            .map(|i| (i.width, i.height))
            .unwrap_or((640, 480))
    }
    
    /// Get input 2 resolution
    pub fn get_input2_resolution(&self) -> (u32, u32) {
        self.input2
            .as_ref()
            .map(|i| (i.width, i.height))
            .unwrap_or((640, 480))
    }
    
    /// Clear input 1
    pub fn clear_input1(&mut self) {
        self.input1 = None;
        log::info!("Cleared input 1 texture");
    }
    
    /// Clear input 2
    pub fn clear_input2(&mut self) {
        self.input2 = None;
        log::info!("Cleared input 2 texture");
    }
}
