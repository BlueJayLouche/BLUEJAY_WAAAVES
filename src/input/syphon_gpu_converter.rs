//! GPU-based BGRA to RGBA conversion for Syphon input
//!
//! This module is only compiled when the `syphon` feature is enabled on macOS.

#![cfg(all(target_os = "macos", feature = "syphon"))]

//!
//! Uses wgpu compute shaders for high-performance color format conversion.
//! This avoids CPU-bound pixel manipulation and keeps video processing on the GPU.

use wgpu::util::DeviceExt;

/// GPU converter for BGRA to RGBA conversion
pub struct BgraToRgbaConverter {
    /// Compute pipeline
    pipeline: wgpu::ComputePipeline,
    /// Bind group layout
    bind_group_layout: wgpu::BindGroupLayout,
    /// Device reference
    device: Arc<wgpu::Device>,
    /// Queue reference
    queue: Arc<wgpu::Queue>,
    /// Current input buffer (BGRA)
    input_buffer: Option<wgpu::Buffer>,
    /// Current output buffer (RGBA)
    output_buffer: Option<wgpu::Buffer>,
    /// Current resolution
    resolution: (u32, u32),
}

use std::sync::Arc;

impl BgraToRgbaConverter {
    /// Create a new GPU converter
    pub fn new(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) -> Self {
        // Create bind group layout
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("BGRA to RGBA Bind Group Layout"),
            entries: &[
                // Input buffer (BGRA)
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Output buffer (RGBA)
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Uniforms (width, height, stride)
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        // Create pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("BGRA to RGBA Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        // Create compute shader
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("BGRA to RGBA Compute Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("bgra_to_rgba.wgsl").into()),
        });

        // Create compute pipeline
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("BGRA to RGBA Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });

        Self {
            pipeline,
            bind_group_layout,
            device,
            queue,
            input_buffer: None,
            output_buffer: None,
            resolution: (0, 0),
        }
    }

    /// Convert BGRA data to RGBA using GPU
    /// 
    /// Returns a wgpu texture containing the RGBA data
    pub fn convert_to_texture(
        &mut self,
        bgra_data: &[u8],
        width: u32,
        height: u32,
    ) -> Option<wgpu::Texture> {
        // Calculate stride (may be larger than width * 4 due to alignment)
        let stride = if height > 0 { bgra_data.len() / height as usize } else { width as usize * 4 };
        
        // Recreate buffers if resolution changed
        if self.resolution != (width, height) || self.input_buffer.is_none() {
            self.create_buffers(width, height, stride);
            self.resolution = (width, height);
        }

        // Upload BGRA data to input buffer
        self.queue.write_buffer(
            self.input_buffer.as_ref()?,
            0,
            bgra_data,
        );

        // Create uniform buffer with conversion params
        let uniforms = [width, height, stride as u32, 0u32]; // padding
        let uniform_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("BGRA Conversion Uniforms"),
            contents: bytemuck::cast_slice(&uniforms),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Create bind group
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("BGRA to RGBA Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.input_buffer.as_ref()?.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self.output_buffer.as_ref()?.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: uniform_buffer.as_entire_binding(),
                },
            ],
        });

        // Encode compute pass
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("BGRA to RGBA Encoder"),
        });

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("BGRA to RGBA Pass"),
                timestamp_writes: None,
            });

            compute_pass.set_pipeline(&self.pipeline);
            compute_pass.set_bind_group(0, &bind_group, &[]);
            
            // Dispatch workgroups (8x8 threads per workgroup)
            let workgroups_x = (width + 7) / 8;
            let workgroups_y = (height + 7) / 8;
            compute_pass.dispatch_workgroups(workgroups_x, workgroups_y, 1);
        }

        // Submit work
        self.queue.submit(std::iter::once(encoder.finish()));

        // Create output texture from buffer
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Syphon Input Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        // Copy output buffer to texture
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Copy to Texture Encoder"),
        });

        encoder.copy_buffer_to_texture(
            wgpu::TexelCopyBufferInfo {
                buffer: self.output_buffer.as_ref()?,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(width * 4),
                    rows_per_image: Some(height),
                },
            },
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        self.queue.submit(std::iter::once(encoder.finish()));

        Some(texture)
    }

    fn create_buffers(&mut self, width: u32, height: u32, stride: usize) {
        let input_size = (stride * height as usize) as u64;
        let output_size = (width * height * 4) as u64;

        self.input_buffer = Some(self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("BGRA Input Buffer"),
            size: input_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }));

        self.output_buffer = Some(self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("RGBA Output Buffer"),
            size: output_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        }));
    }
}
