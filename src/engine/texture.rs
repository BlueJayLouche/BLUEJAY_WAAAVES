//! # Texture Module
//!
//! Texture management for wgpu.

/// Render target texture wrapper
pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub width: u32,
    pub height: u32,
}

impl Texture {
    /// Create a new render target texture (defaults to Rgba8Unorm)
    pub fn create_render_target(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        label: &str,
    ) -> Self {
        Self::create_render_target_with_format(device, width, height, label, wgpu::TextureFormat::Rgba8Unorm)
    }
    
    /// Create a new render target texture with specific format
    pub fn create_render_target_with_format(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        label: &str,
        format: wgpu::TextureFormat,
    ) -> Self {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT 
                | wgpu::TextureUsages::TEXTURE_BINDING 
                | wgpu::TextureUsages::COPY_SRC 
                | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        
        Self {
            texture,
            view,
            sampler,
            width,
            height,
        }
    }
    
    /// Clear texture to black (for feedback texture initialization)
    pub fn clear_to_black(&self, queue: &wgpu::Queue) {
        let black_data = vec![0u8; (self.width * self.height * 4) as usize];
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &black_data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * self.width),
                rows_per_image: Some(self.height),
            },
            wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
        );
    }
    
    /// Create a texture from raw bytes (for input images)
    pub fn from_bytes(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes: &[u8],
        width: u32,
        height: u32,
        label: &str,
    ) -> Self {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            bytes,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            size,
        );
        
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        
        Self {
            texture,
            view,
            sampler,
            width,
            height,
        }
    }
}
