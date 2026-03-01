//! # Engine Module
//!
//! The core rendering engine using wgpu for cross-platform GPU acceleration.
//! Supports dual-window architecture: output window + control window with imgui.

use crate::audio::AudioInput;
use crate::config::AppConfig;
use crate::core::lfo_engine::{update_lfo_phases, apply_lfos_to_block1, apply_lfos_to_block2, apply_lfos_to_block3};
use crate::params::preset::{apply_audio_modulations, ParamModulationData};
use std::collections::HashMap;
use crate::core::{OutputMode, SharedState, Vertex};
use crate::engine::imgui_renderer::ImGuiRenderer;
use crate::engine::pipelines::Block1Pipeline;
use crate::engine::blocks::{ModularBlock1, ModularBlock2, ModularBlock3};

use crate::engine::texture::Texture;
use crate::gui::ControlGui;
use crate::input::{InputManager, InputTextureManager};
use anyhow::Result;
use std::sync::Arc;
use wgpu::util::DeviceExt;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

pub mod imgui_renderer;
pub mod pipelines;
pub mod preview;
pub mod texture;
pub mod simple_feedback;
pub mod lfo_tempo;
pub mod simple_engine;
pub mod stages;
pub mod blocks;

/// Main application runner
pub fn run_app(
    config: AppConfig,
    shared_state: Arc<std::sync::Mutex<SharedState>>,
) -> Result<()> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);
    
    let mut app = App::new(config, shared_state);
    event_loop.run_app(&mut app)?;
    
    Ok(())
}

/// Application state managing all windows
struct App {
    config: AppConfig,
    shared_state: Arc<std::sync::Mutex<SharedState>>,
    
    // Shared wgpu resources
    wgpu_instance: Option<wgpu::Instance>,
    wgpu_adapter: Option<wgpu::Adapter>,
    wgpu_device: Option<Arc<wgpu::Device>>,
    wgpu_queue: Option<Arc<wgpu::Queue>>,
    
    // Output window (main display)
    output_window: Option<Arc<Window>>,
    output_engine: Option<WgpuEngine>,
    output_fullscreen: bool,  // Track fullscreen state
    
    // Control window (imgui)
    control_window: Option<Arc<Window>>,
    control_gui: Option<ControlGui>,
    imgui_renderer: Option<ImGuiRenderer>,
    
    // Audio input
    audio_input: Option<AudioInput>,
    
    // Video input
    video_input: Option<InputManager>,
    
    // Keyboard modifier state tracking
    shift_pressed: bool,
}

impl App {
    fn new(config: AppConfig, shared_state: Arc<std::sync::Mutex<SharedState>>) -> Self {
        // Initialize audio input (wrapped in catch_unwind to prevent panic)
        let audio_input = if config.audio.enable_input {
            match std::panic::catch_unwind(|| {
                let mut audio = AudioInput::new(config.audio.fft_size);
                if let Err(e) = audio.initialize() {
                    log::error!("Failed to initialize audio input: {:?}", e);
                    None
                } else {
                    log::info!("Audio input initialized successfully");
                    Some(audio)
                }
            }) {
                Ok(result) => result,
                Err(_) => {
                    log::error!("Audio initialization panicked");
                    None
                }
            }
        } else {
            log::info!("Audio input disabled in config");
            None
        };
        
        // Video input will be initialized after wgpu device is created
        let video_input: Option<InputManager> = None;
        
        Self {
            config,
            shared_state,
            wgpu_instance: None,
            wgpu_adapter: None,
            wgpu_device: None,
            wgpu_queue: None,
            output_window: None,
            output_engine: None,
            output_fullscreen: false,
            control_window: None,
            control_gui: None,
            imgui_renderer: None,
            audio_input,
            video_input,
            shift_pressed: false,
        }
    }
    
    /// Render the preview window with modular Block 1 debug output
    /// Toggle fullscreen mode for the output window
    fn toggle_fullscreen(&mut self) {
        if let Some(ref output_window) = self.output_window {
            self.output_fullscreen = !self.output_fullscreen;
            
            let fullscreen_mode = if self.output_fullscreen {
                // Use borderless fullscreen on the current monitor
                Some(winit::window::Fullscreen::Borderless(None))
            } else {
                None
            };
            
            output_window.set_fullscreen(fullscreen_mode);
            log::info!("Output window fullscreen: {}", self.output_fullscreen);
            
            // Show a brief notification (could add on-screen display here)
            if let Some(ref mut gui) = self.control_gui {
                let msg = if self.output_fullscreen { "Fullscreen ON" } else { "Fullscreen OFF" };
                gui.show_status(msg);
            }
        }
    }
    
    /// Trigger tap tempo from keyboard shortcut
    fn trigger_tap_tempo(&mut self) {
        if let Some(ref mut gui) = self.control_gui {
            // Call the existing tap tempo handler
            gui.trigger_tap_tempo();
            log::debug!("Tap tempo triggered from keyboard");
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Create shared wgpu instance first
        if self.wgpu_instance.is_none() {
            self.wgpu_instance = Some(wgpu::Instance::new(&wgpu::InstanceDescriptor {
                backends: wgpu::Backends::all(),
                ..Default::default()
            }));
        }
        let instance = self.wgpu_instance.as_ref().unwrap();
        
        // Create output window first
        if self.output_window.is_none() {
            let window_attrs = winit::window::WindowAttributes::default()
                .with_title(&self.config.output_window.title)
                .with_inner_size(winit::dpi::LogicalSize::new(
                    self.config.output_window.width,
                    self.config.output_window.height,
                ))
                .with_resizable(self.config.output_window.resizable)
                .with_decorations(self.config.output_window.decorated);
            
            let window = Arc::new(event_loop.create_window(window_attrs).unwrap());
            self.output_window = Some(Arc::clone(&window));
            
            // Initialize output engine using shared instance
            let shared_state = Arc::clone(&self.shared_state);
            let config = self.config.clone();
            match pollster::block_on(WgpuEngine::new(
                instance, 
                window, 
                &config, 
                shared_state,
                &mut self.wgpu_adapter,
                &mut self.wgpu_device,
                &mut self.wgpu_queue,
            )) {
                Ok(engine) => {
                    // Initialize video input manager now that we have device/queue
                    if self.video_input.is_none() {
                        let mut input_manager = InputManager::new();
                        if let (Some(device), Some(queue)) = (&self.wgpu_device, &self.wgpu_queue) {
                            if let Err(e) = input_manager.initialize(Arc::clone(device), Arc::clone(queue)) {
                                log::error!("Failed to initialize video input: {:?}", e);
                            } else {
                                log::info!("Video input manager initialized with GPU device");
                            }
                        }
                        self.video_input = Some(input_manager);
                    }
                    self.output_engine = Some(engine);
                }
                Err(err) => {
                    eprintln!("Failed to create output engine: {}", err);
                    event_loop.exit();
                    return;
                }
            }
        }
        
        // Create control window after output window is ready (shares device/queue)
        if self.control_window.is_none() {
            if let (Some(device), Some(queue)) = (&self.wgpu_device, &self.wgpu_queue) {
                let window_attrs = winit::window::WindowAttributes::default()
                    .with_title("RUSTJAY WAAAVES - Control")
                    .with_inner_size(winit::dpi::LogicalSize::new(
                        self.config.control_window.width,
                        self.config.control_window.height,
                    ))
                    .with_resizable(true)
                    .with_decorations(true);
                
                let window = Arc::new(event_loop.create_window(window_attrs).unwrap());
                self.control_window = Some(Arc::clone(&window));
                
                // Initialize ImGui renderer using shared device/queue
                let ui_scale = self.config.ui_scale;
                match pollster::block_on(ImGuiRenderer::new(
                    instance, 
                    Arc::clone(device), 
                    Arc::clone(queue), 
                    Arc::clone(&window),
                    ui_scale
                )) {
                    Ok(mut renderer) => {
                        // Initialize ControlGui with the imgui context
                        match ControlGui::new(&self.config, Arc::clone(&self.shared_state)) {
                            Ok(mut gui) => {
                                // Auto-start webcams based on saved config
                                gui.auto_start_webcams();
                                
                                // Register preview texture with imgui renderer if preview is available
                                if let Some(ref output_engine) = self.output_engine {
                                    if let Some(ref preview) = output_engine.preview_renderer {
                                        let texture_id = renderer.register_external_texture(
                                            preview.get_texture_arc(),
                                            preview.get_texture_view_arc(),
                                            320, 180
                                        );
                                        gui.set_preview_texture_id(texture_id);
                                        log::info!("Registered preview texture with ImGui: {:?}", texture_id);
                                    }
                                }
                                
                                self.control_gui = Some(gui);
                                self.imgui_renderer = Some(renderer);
                            }
                            Err(err) => {
                                eprintln!("Failed to create control GUI: {}", err);
                            }
                        }
                    }
                    Err(err) => {
                        eprintln!("Failed to create ImGui renderer: {}", err);
                    }
                }
            }
        }
    }
    
    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        // Handle output window events
        if let Some(ref output_window) = self.output_window {
            if window_id == output_window.id() {
                match event {
                    WindowEvent::CloseRequested => {
                        event_loop.exit();
                    }
                    WindowEvent::KeyboardInput { event, .. } => {
                        // Track modifier state
                        match &event.logical_key {
                            winit::keyboard::Key::Named(winit::keyboard::NamedKey::Shift) => {
                                self.shift_pressed = event.state == winit::event::ElementState::Pressed;
                            }
                            _ => {}
                        }
                        
                        if event.state == winit::event::ElementState::Pressed {
                            match &event.logical_key {
                                winit::keyboard::Key::Named(winit::keyboard::NamedKey::Escape) => {
                                    event_loop.exit();
                                }
                                winit::keyboard::Key::Character(ch) => {
                                    let key = ch.to_lowercase();
                                    
                                    if self.shift_pressed && key == "f" {
                                        // Shift+F: Toggle fullscreen
                                        self.toggle_fullscreen();
                                    } else if self.shift_pressed && key == "t" {
                                        // Shift+T: Tap tempo
                                        self.trigger_tap_tempo();
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    WindowEvent::Resized(size) => {
                        if let Some(ref mut engine) = self.output_engine {
                            engine.resize(size.width, size.height);
                        }
                    }
                    WindowEvent::RedrawRequested => {
                        if let Some(ref mut engine) = self.output_engine {
                            engine.render();
                        }
                    }
                    _ => {}
                }
                return;
            }
        }
        
        // Handle control window events
        if let Some(ref control_window) = self.control_window {
            if window_id == control_window.id() {
                // Pass events to ImGui
                if let Some(ref mut renderer) = self.imgui_renderer {
                    renderer.handle_event(&event, control_window);
                }
                
                match event {
                    WindowEvent::CloseRequested => {
                        // Just close the control window, keep output running
                        self.control_window = None;
                        self.control_gui = None;
                        self.imgui_renderer = None;
                    }
                    WindowEvent::KeyboardInput { event, .. } => {
                        // Track modifier state
                        match &event.logical_key {
                            winit::keyboard::Key::Named(winit::keyboard::NamedKey::Shift) => {
                                self.shift_pressed = event.state == winit::event::ElementState::Pressed;
                            }
                            _ => {}
                        }
                        
                        if event.state == winit::event::ElementState::Pressed {
                            if let winit::keyboard::Key::Character(ch) = &event.logical_key {
                                let key = ch.to_lowercase();
                                
                                if self.shift_pressed && key == "t" {
                                    // Shift+T: Tap tempo (also works in control window)
                                    self.trigger_tap_tempo();
                                }
                            }
                        }
                    }
                    WindowEvent::Resized(size) => {
                        if let Some(ref mut renderer) = self.imgui_renderer {
                            renderer.resize(size.width, size.height);
                        }
                    }
                    WindowEvent::RedrawRequested => {
                        // Render imgui
                        if let (Some(ref mut renderer), Some(ref mut gui)) = 
                            (self.imgui_renderer.as_mut(), self.control_gui.as_mut()) 
                        {
                            // Check for UI scale changes from shared state
                            if let Ok(state) = self.shared_state.lock() {
                                let desired_scale = state.ui_scale;
                                if (renderer.ui_scale() - desired_scale).abs() > 0.01 {
                                    renderer.set_ui_scale(desired_scale);
                                }
                            }
                            
                            // Update display size and render
                            let window_size = control_window.inner_size();
                            renderer.set_display_size(window_size.width as f32, window_size.height as f32);
                            
                            if let Err(err) = renderer.render_frame(|ui| gui.build_ui(ui)) {
                                eprintln!("ImGui render error: {}", err);
                            }
                        }
                    }
                    _ => {}
                }
                return;
            }
        }
    }
    
    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        // Update audio input
        if let Some(ref mut audio) = self.audio_input {
            // Get processed 8-band FFT (with amplitude/smoothing/normalization applied)
            let fft_8band = audio.get_8band_fft();
            let beat_state = audio.get_beat_state();
            
            // Update shared state with audio data
            if let Ok(mut state) = self.shared_state.lock() {
                // Copy 8-band FFT to first 8 slots of shared state
                for i in 0..8.min(state.audio.fft.len()) {
                    state.audio.fft[i] = fft_8band[i];
                }
                // Fill remaining slots with zeros if needed
                for i in 8..state.audio.fft.len() {
                    state.audio.fft[i] = 0.0;
                }
                
                state.audio.volume = beat_state.energy;
                state.audio.beat = beat_state.beat;
                state.audio.bpm = beat_state.bpm;
                state.audio.beat_phase = beat_state.phase;
                
                // Sync audio processing settings from SharedState to AudioInput
                let amplitude = state.audio.amplitude;
                let smoothing = state.audio.smoothing;
                let normalization = state.audio.normalization;
                let pink_compensation = state.audio.pink_compensation;
                audio.set_amplitude(amplitude);
                audio.set_smoothing(smoothing);
                audio.set_normalization(normalization);
                audio.set_pink_compensation(pink_compensation);
            }
        }
        
        // Handle input change requests from GUI
        {
            let mut state = self.shared_state.lock().unwrap();
            
            // Handle input 1 change request
            let input1_request = std::mem::replace(&mut state.input1_change_request, crate::core::InputChangeRequest::None);
            drop(state); // Drop lock before calling into video input
            
            match input1_request {
                crate::core::InputChangeRequest::StartWebcam { device_index, width, height, fps, .. } => {
                    log::info!("[INPUT] Received StartWebcam request for input 1, device {}", device_index);
                    if let Some(ref mut video) = self.video_input {
                        log::info!("[INPUT] Processing webcam start request for input 1, device {}", device_index);
                        match video.start_input1_webcam(device_index, width, height, fps) {
                            Ok(_) => log::info!("[INPUT] Successfully started webcam input 1"),
                            Err(e) => log::error!("[INPUT] Failed to start webcam input 1: {:?}", e),
                        }
                    } else {
                        log::error!("[INPUT] Video input manager not initialized! Cannot start webcam.");
                    }
                }
                crate::core::InputChangeRequest::StopInput { .. } => {
                    if let Some(ref mut video) = self.video_input {
                        log::info!("[INPUT] Processing stop request for input 1");
                        video.stop_input1();
                        log::info!("[INPUT] Input 1 stopped");
                    }
                }
                _ => {}
            }
            
            // Handle input 2 change request
            let mut state = self.shared_state.lock().unwrap();
            let input2_request = std::mem::replace(&mut state.input2_change_request, crate::core::InputChangeRequest::None);
            drop(state);
            
            match input2_request {
                crate::core::InputChangeRequest::StartWebcam { device_index, width, height, fps, .. } => {
                    if let Some(ref mut video) = self.video_input {
                        log::info!("[INPUT] Processing webcam start request for input 2, device {}", device_index);
                        match video.start_input2_webcam(device_index, width, height, fps) {
                            Ok(_) => log::info!("[INPUT] Successfully started webcam input 2"),
                            Err(e) => log::error!("[INPUT] Failed to start webcam input 2: {:?}", e),
                        }
                    } else {
                        log::error!("[INPUT] Video input manager not initialized!");
                    }
                }
                crate::core::InputChangeRequest::StopInput { .. } => {
                    if let Some(ref mut video) = self.video_input {
                        log::info!("[INPUT] Processing stop request for input 2");
                        video.stop_input2();
                        log::info!("[INPUT] Input 2 stopped");
                    }
                }
                _ => {}
            }
            
            // Handle audio change request
            let mut state = self.shared_state.lock().unwrap();
            let audio_request = std::mem::replace(&mut state.audio_change_request, crate::core::AudioChangeRequest::None);
            drop(state);
            
            match audio_request {
                crate::core::AudioChangeRequest::ChangeDevice { device_index } => {
                    log::info!("Processing audio device change request to index {}", device_index);
                    // Stop current audio
                    if let Some(ref mut audio) = self.audio_input {
                        let _ = audio.stop();
                    }
                    // Create new audio input with specific device
                    let mut new_audio = AudioInput::new(self.config.audio.fft_size);
                    match new_audio.initialize_with_device(device_index) {
                        Ok(_) => {
                            log::info!("Audio reinitialized successfully with device {}", device_index);
                            self.audio_input = Some(new_audio);
                        }
                        Err(e) => {
                            log::error!("Failed to reinitialize audio: {:?}", e);
                        }
                    }
                }
                _ => {}
            }
        }
        
        // Update video input and upload frames to GPU
        if let (Some(ref mut video), Some(ref mut engine)) = (self.video_input.as_mut(), self.output_engine.as_mut()) {
            video.update();
            
            // Upload input 1 frame if available
            if video.input1_has_new_frame() {
                if let Some(frame_data) = video.take_input1_frame() {
                    let (width, height) = video.get_input1_resolution();
                    log::info!("Uploading input 1 frame to GPU: {}x{} ({} bytes)", width, height, frame_data.len());
                    engine.input_texture_manager.update_input1(&frame_data, width, height);
                }
            }
            
            // Upload input 2 frame if available
            if video.input2_has_new_frame() {
                if let Some(frame_data) = video.take_input2_frame() {
                    let (width, height) = video.get_input2_resolution();
                    log::info!("Uploading input 2 frame to GPU: {}x{} ({} bytes)", width, height, frame_data.len());
                    engine.input_texture_manager.update_input2(&frame_data, width, height);
                }
            }
        }
        
        // Request redraws for both windows
        if let Some(ref window) = self.output_window {
            window.request_redraw();
        }
        if let Some(ref window) = self.control_window {
            window.request_redraw();
        }
    }
}

/// wgpu-based rendering engine for output
pub struct WgpuEngine {
    #[allow(dead_code)]
    instance: wgpu::Instance,
    #[allow(dead_code)]
    adapter: wgpu::Adapter,
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    
    shared_state: Arc<std::sync::Mutex<SharedState>>,
    
    // Modular blocks (new 3-stage implementation)
    modular_block1: ModularBlock1,
    modular_block2: ModularBlock2,
    modular_block3: ModularBlock3,
    
    // Legacy pipeline (to be replaced)
    block1_pipeline: Block1Pipeline,
    
    // Simple blit pipeline for output
    blit_pipeline: wgpu::RenderPipeline,
    blit_bind_group_layout: wgpu::BindGroupLayout,
    
    // Render targets
    block1_texture: Texture,
    block2_texture: Texture,
    block3_texture: Texture,
    
    // Feedback textures (separate from render targets to avoid usage conflicts)
    fb1_texture: Texture,
    fb2_texture: Texture,
    
    // Temporal filter textures
    temporal1_texture: Texture,
    temporal2_texture: Texture,
    
    /// Input texture manager for video sources
    pub input_texture_manager: InputTextureManager,
    
    vertex_buffer: wgpu::Buffer,
    
    frame_count: u64,
    
    /// Preview renderer for color picker
    preview_renderer: Option<crate::engine::preview::PreviewRenderer>,
}

impl WgpuEngine {
    pub async fn new(
        instance: &wgpu::Instance,
        window: Arc<Window>,
        app_config: &AppConfig,
        shared_state: Arc<std::sync::Mutex<SharedState>>,
        adapter_out: &mut Option<wgpu::Adapter>,
        device_out: &mut Option<Arc<wgpu::Device>>,
        queue_out: &mut Option<Arc<wgpu::Queue>>,
    ) -> Result<Self> {
        let size = window.inner_size();
        
        let surface = instance.create_surface(window)?;
        
        // Get or create adapter
        let adapter = if let Some(adapter) = adapter_out.take() {
            adapter
        } else {
            instance
                .request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::HighPerformance,
                    compatible_surface: Some(&surface),
                    force_fallback_adapter: false,
                })
                .await?
        };
        
        // Get or create device/queue
        let (device, queue) = if let (Some(device), Some(queue)) = (device_out.clone(), queue_out.clone()) {
            (device, queue)
        } else {
            let (device, queue) = adapter
                .request_device(
                    &wgpu::DeviceDescriptor {
                        required_features: wgpu::Features::empty(),
                        required_limits: wgpu::Limits::default(),
                        label: Some("Device"),
                        memory_hints: wgpu::MemoryHints::default(),
                        trace: wgpu::Trace::Off,
                    },
                )
                .await?;
            let device = Arc::new(device);
            let queue = Arc::new(queue);
            *device_out = Some(Arc::clone(&device));
            *queue_out = Some(Arc::clone(&queue));
            (device, queue)
        };
        
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);
        
        let internal_width = app_config.pipeline.internal_width;
        let internal_height = app_config.pipeline.internal_height;
        
        // Render targets
        let block1_texture = Texture::create_render_target(
            &device,
            internal_width,
            internal_height,
            "Block1 Texture",
        );
        let block2_texture = Texture::create_render_target(
            &device,
            internal_width,
            internal_height,
            "Block2 Texture",
        );
        let block3_texture = Texture::create_render_target(
            &device,
            config.width,
            config.height,
            "Block3 Texture",
        );
        
        // Feedback textures (for sampling in shaders)
        let fb1_texture = Texture::create_render_target(
            &device,
            internal_width,
            internal_height,
            "FB1 Texture",
        );
        fb1_texture.clear_to_black(&queue);
        
        let fb2_texture = Texture::create_render_target(
            &device,
            internal_width,
            internal_height,
            "FB2 Texture",
        );
        fb2_texture.clear_to_black(&queue);
        
        // Temporal filter textures
        let temporal1_texture = Texture::create_render_target(
            &device,
            internal_width,
            internal_height,
            "Temporal1 Texture",
        );
        temporal1_texture.clear_to_black(&queue);
        
        let temporal2_texture = Texture::create_render_target(
            &device,
            internal_width,
            internal_height,
            "Temporal2 Texture",
        );
        temporal2_texture.clear_to_black(&queue);
        
        let block1_pipeline = Block1Pipeline::new(&device, internal_width, internal_height);
        
        // Create modular blocks (new 3-stage implementation)
        let modular_block1 = ModularBlock1::new(&device, &queue, internal_width, internal_height);
        let modular_block2 = ModularBlock2::new(&device, &queue, internal_width, internal_height);
        // Block 3 renders to output surface size (not internal resolution)
        let modular_block3 = ModularBlock3::new(&device, &queue, config.width, config.height);
        
        // Create simple blit pipeline for final output
        let blit_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Blit Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        
        let blit_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Blit Shader"),
            source: wgpu::ShaderSource::Wgsl(r#"
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) texcoord: vec2<f32>,
};

@vertex
fn vs_main(@location(0) position: vec2<f32>, @location(1) texcoord: vec2<f32>) -> VertexOutput {
    var out: VertexOutput;
    out.position = vec4<f32>(position, 0.0, 1.0);
    out.texcoord = texcoord;
    return out;
}

@group(0) @binding(0)
var source_tex: texture_2d<f32>;
@group(0) @binding(1)
var source_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(source_tex, source_sampler, in.texcoord);
}
"#.into()),
        });
        
        let blit_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Blit Pipeline Layout"),
            bind_group_layouts: &[&blit_bind_group_layout],
            push_constant_ranges: &[],
        });
        
        let blit_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Blit Pipeline"),
            layout: Some(&blit_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &blit_shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            offset: 0,
                            shader_location: 0,
                            format: wgpu::VertexFormat::Float32x2,
                        },
                        wgpu::VertexAttribute {
                            offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                            shader_location: 1,
                            format: wgpu::VertexFormat::Float32x2,
                        },
                    ],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &blit_shader,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });
        
        let vertices = Vertex::quad_vertices();
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Quad Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        
        // Create input texture manager
        let input_texture_manager = InputTextureManager::new(
            Arc::clone(&device),
            Arc::clone(&queue),
        );
        
        // Store adapter for potential reuse
        *adapter_out = Some(adapter.clone());
        
        Ok(Self {
            instance: instance.clone(),
            adapter,
            device: Arc::clone(&device),
            queue: Arc::clone(&queue),
            surface,
            config,

            shared_state,
            modular_block1,
            modular_block2,
            modular_block3,
            block1_pipeline,
            blit_pipeline,
            blit_bind_group_layout,
            block1_texture,
            block2_texture,
            block3_texture,
            fb1_texture,
            fb2_texture,
            temporal1_texture,
            temporal2_texture,
            input_texture_manager,
            vertex_buffer,
            frame_count: 0,
            
            // Initialize preview renderer (320x180 = 16:9 aspect)
            preview_renderer: Some(crate::engine::preview::PreviewRenderer::new(
                &device, 320, 180
            )),
        })
    }
    
    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            // Check if surface capabilities have changed (e.g., moved to different monitor)
            let surface_caps = self.surface.get_capabilities(&self.adapter);
            let preferred_format = surface_caps
                .formats
                .iter()
                .copied()
                .find(|f| f.is_srgb())
                .unwrap_or(surface_caps.formats[0]);
            
            // Update format if it changed (e.g., moving between displays)
            if preferred_format != self.config.format {
                println!("Surface format changed from {:?} to {:?}", self.config.format, preferred_format);
                self.config.format = preferred_format;
            }
            
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
            
            // Resize Block3 texture and modular Block3 to match surface size
            self.block3_texture = Texture::create_render_target(
                &self.device,
                width,
                height,
                "Block3 Texture",
            );
            self.modular_block3.resize(&self.device, &self.queue, width, height);
        }
    }
    
    /// Resize internal textures (called when pipeline config changes)
    pub fn resize_internal(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            // Recreate render targets
            self.block1_texture = Texture::create_render_target(
                &self.device,
                width,
                height,
                "Block1 Texture",
            );
            self.block2_texture = Texture::create_render_target(
                &self.device,
                width,
                height,
                "Block2 Texture",
            );
            
            // Recreate feedback textures
            self.fb1_texture = Texture::create_render_target(
                &self.device,
                width,
                height,
                "FB1 Texture",
            );
            self.fb2_texture = Texture::create_render_target(
                &self.device,
                width,
                height,
                "FB2 Texture",
            );
            
            // Recreate temporal textures
            self.temporal1_texture = Texture::create_render_target(
                &self.device,
                width,
                height,
                "Temporal1 Texture",
            );
            self.temporal2_texture = Texture::create_render_target(
                &self.device,
                width,
                height,
                "Temporal2 Texture",
            );
            
            // Update shared state
            if let Ok(mut state) = self.shared_state.lock() {
                state.internal_size = (width, height);
            }
        }
    }
    
    pub fn render(&mut self) {
        let mut state = self.shared_state.lock().unwrap();
        
        let output_mode = state.output_mode;
        let block2_input_select = state.block2.block2_input_select;
        
        // Update LFO phases using BPM from GUI
        let bpm = state.bpm;
        update_lfo_phases(&mut state.lfo_banks, bpm, 1.0 / 60.0); // Assuming 60fps
        
        // Clone base parameters and apply LFO modulation
        let mut modulated_block1 = state.block1.clone();
        let mut modulated_block2 = state.block2.clone();
        let mut modulated_block3 = state.block3.clone();
        
        apply_lfos_to_block1(&mut modulated_block1, &state.lfo_banks, &state.block1_lfo_map);
        apply_lfos_to_block2(&mut modulated_block2, &state.lfo_banks, &state.block2_lfo_map);
        apply_lfos_to_block3(&mut modulated_block3, &state.lfo_banks, &state.block3_lfo_map);
        
        // Apply audio modulations - get FFT values from shared state (already processed)
        let fft_values: [f32; 8] = std::array::from_fn(|i| state.audio.fft.get(i).copied().unwrap_or(0.0));
        
        // Clone modulations to avoid borrowing issues
        let block1_mods = state.block1_modulations.clone();
        let block2_mods = state.block2_modulations.clone();
        let block3_mods = state.block3_modulations.clone();
        
        apply_audio_modulations_to_block1(&mut modulated_block1, &block1_mods, &fft_values);
        apply_audio_modulations_to_block2(&mut modulated_block2, &block2_mods, &fft_values);
        apply_audio_modulations_to_block3(&mut modulated_block3, &block3_mods, &fft_values);
        
        // Apply delay time tempo sync
        // If sync is enabled, calculate delay frames from BPM
        if modulated_block1.fb1_delay_time_sync {
            modulated_block1.fb1_delay_time = crate::core::lfo_engine::calculate_delay_frames_from_tempo(
                bpm, modulated_block1.fb1_delay_time_division, 60.0
            );
        }
        if modulated_block2.fb2_delay_time_sync {
            modulated_block2.fb2_delay_time = crate::core::lfo_engine::calculate_delay_frames_from_tempo(
                bpm, modulated_block2.fb2_delay_time_division, 60.0
            );
        }
        
        // Log output mode and params periodically (every 60 frames)
        if self.frame_count % 60 == 0 {
            log::info!("=== Frame {} ===", self.frame_count);
            log::info!("Output mode: {:?}, Block2 input select: {}", output_mode, block2_input_select);
            // Check if inputs have data
            let has_input1 = self.input_texture_manager.input1.is_some();
            let has_input2 = self.input_texture_manager.input2.is_some();
            let input1_has_data = self.input_texture_manager.input1_has_data();
            let input2_has_data = self.input_texture_manager.input2_has_data();
            log::info!("Input textures: input1_exists={}, input2_exists={}, input1_has_data={}, input2_has_data={}", 
                has_input1, has_input2, input1_has_data, input2_has_data);
            // Log block1 input selection
            log::info!("Block1 params: ch1_input_select={}, ch2_input_select={}", 
                state.block1.ch1_input_select,
                state.block1.ch2_input_select);
            // Log key parameters that affect visibility
            log::info!("Block1: ch1_z_displace={}, ch1_hsb_attenuate={:?}", 
                state.block1.ch1_z_displace, state.block1.ch1_hsb_attenuate);
            log::info!("Block1: fb1_mix_amount={}, ch2_mix_amount={}",
                state.block1.fb1_mix_amount, state.block1.ch2_mix_amount);
        }
        
        // Get input texture sizes for proper UV scaling
        let input1_size = self.input_texture_manager.get_input1_resolution();
        let input2_size = self.input_texture_manager.get_input2_resolution();
        
        // Use modulated parameters for rendering
        self.block1_pipeline.update_params(&self.queue, &modulated_block1, input1_size, input2_size);
        self.modular_block2.update_params(&self.queue, &modulated_block2);
        // Block 3 params are passed to render method
        
        // Get preview source for modular Block 1 before dropping state
        let preview_source = state.preview_source;
        
        drop(state);
        
        // Update input textures in the pipeline
        let input1_view = self.input_texture_manager.get_input1_view();
        let input2_view = self.input_texture_manager.get_input2_view();
        let input1_has_data = self.input_texture_manager.input1_has_data();
        let input2_has_data = self.input_texture_manager.input2_has_data();
        
        // Log texture binding periodically
        if self.frame_count % 60 == 0 {
            log::info!("[BIND] Updating textures - input1_has_data: {}, input2_has_data: {}",
                input1_has_data, input2_has_data);
        }
        
        self.block1_pipeline.update_textures(
            &self.device,
            input1_view,
            input2_view,
            &self.fb1_texture.view,
            &self.temporal1_texture.view,
        );
        
        let surface_texture = match self.surface.get_current_texture() {
            Ok(t) => t,
            Err(_) => {
                self.surface.configure(&self.device, &self.config);
                return;
            }
        };
        
        let surface_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });
        
        // Render Block 1
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Block1 Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.block1_texture.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            
            render_pass.set_pipeline(&self.block1_pipeline.pipeline);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_bind_group(0, &self.block1_pipeline.bind_group, &[]);
            render_pass.draw(0..6, 0..1);
        }
        
        // Also render modular Block 1 (parallel implementation)
        // This renders to modular_block1's internal buffers
        {
            // Render modular Block 1
            self.modular_block1.render(
                &mut encoder,
                &self.device,
                &self.queue,
                input1_view,
                input2_view,
                &modulated_block1,
            );
            
            // Copy modular Block 1 output to block1_texture so it gets used downstream
            // This also applies the debug view selection
            // Use internal resolution (same as modular Block 1 textures)
            encoder.copy_texture_to_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &self.modular_block1.resources.get_output_texture(),
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::TexelCopyTextureInfo {
                    texture: &self.block1_texture.texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::Extent3d {
                    width: self.block1_texture.width,
                    height: self.block1_texture.height,
                    depth_or_array_layers: 1,
                },
            );
            
            // Update feedback for next frame
            self.modular_block1.update_feedback(&mut encoder);
        }
        
        // Determine Block2 input based on block2_input_select
        // 0 = block1, 1 = input1, 2 = input2
        let input1_has_data = self.input_texture_manager.input1_has_data();
        let input2_has_data = self.input_texture_manager.input2_has_data();
        
        let block2_input_view: &wgpu::TextureView = match block2_input_select {
            1 => {
                if input1_has_data {
                    log::info!("Block2 using Input 1 (has data)");
                } else {
                    log::info!("Block2 using Input 1 (NO DATA - using black texture)");
                }
                input1_view
            }
            2 => {
                if input2_has_data {
                    log::info!("Block2 using Input 2 (has data)");
                } else {
                    log::info!("Block2 using Input 2 (NO DATA - using black texture)");
                }
                input2_view
            }
            _ => {
                log::info!("Block2 using Block 1 output (default)");
                &self.block1_texture.view
            }
        };
        
        // Render Block 2 using modular implementation
        self.modular_block2.render(
            &mut encoder,
            &self.device,
            &self.queue,
            &self.block1_texture.view,
            input1_view,
            input2_view,
            &self.block2_texture.view,
            &modulated_block2,
        );
        
        // Update FB2 feedback for next frame
        // Block 2 renders to block2_texture (not ping-pong buffers), so copy from there
        self.modular_block2.resources.update_feedback_from_external(&mut encoder, &self.block2_texture.texture);
        self.modular_block2.resources.update_delay_buffer_from_external(&mut encoder, &self.block2_texture.texture);
        
        // Determine which block to display based on output_mode
        let (output_texture, _output_view): (&wgpu::Texture, &wgpu::TextureView) = match output_mode {
            OutputMode::Block1 => (&self.block1_texture.texture, &self.block1_texture.view),
            OutputMode::Block2 => (&self.block2_texture.texture, &self.block2_texture.view),
            OutputMode::Block3 => (&self.block3_texture.texture, &self.block3_texture.view),
            OutputMode::PreviewInput1 => {
                if let Some(ref input) = self.input_texture_manager.input1 {
                    (&input.texture.texture, &input.texture.view)
                } else {
                    (&self.block1_texture.texture, &self.block1_texture.view)
                }
            },
            OutputMode::PreviewInput2 => {
                if let Some(ref input) = self.input_texture_manager.input2 {
                    (&input.texture.texture, &input.texture.view)
                } else {
                    (&self.block1_texture.texture, &self.block1_texture.view)
                }
            },
        };
        
        // Render Block 3 using modular implementation
        let block3_output_view = self.modular_block3.render(
            &mut encoder,
            &self.device,
            &self.queue,
            &self.block1_texture.view,
            &self.block2_texture.view,
            &modulated_block3,
        );
        
        // Copy modular Block 3 output to block3_texture for display
        encoder.copy_texture_to_texture(
            wgpu::TexelCopyTextureInfo {
                texture: self.modular_block3.get_output_texture(),
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyTextureInfo {
                texture: &self.block3_texture.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::Extent3d {
                width: self.block3_texture.width,
                height: self.block3_texture.height,
                depth_or_array_layers: 1,
            },
        );
        
        // Blit the selected output to the surface using a render pass
        // This handles format conversion (Rgba8Unorm -> Bgra8UnormSrgb)
        {
            // Create a temporary bind group for blitting
            let blit_sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                ..Default::default()
            });
            
            let blit_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Blit Bind Group"),
                layout: &self.blit_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(_output_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&blit_sampler),
                    },
                ],
            });
            
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Blit Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &surface_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            
            render_pass.set_pipeline(&self.blit_pipeline);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_bind_group(0, &blit_bind_group, &[]);
            render_pass.draw(0..6, 0..1);
        }
        
        // Copy render targets to feedback textures for next frame
        // This must happen after all render passes are done
        encoder.copy_texture_to_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.block1_texture.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyTextureInfo {
                texture: &self.fb1_texture.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::Extent3d {
                width: self.block1_texture.width,
                height: self.block1_texture.height,
                depth_or_array_layers: 1,
            },
        );
        
        // Note: Block 2 feedback is managed by modular_block2.resources
        // The update_feedback() call above handles copying output to feedback
        
        // Check if preview is enabled before computing
        let preview_enabled = if let Ok(state) = self.shared_state.lock() {
            state.preview_enabled
        } else {
            true // Default to enabled if can't read state
        };
        
        // Update preview renderer only if enabled (saves GPU when window closed)
        if preview_enabled {
            if let Some(ref mut preview) = self.preview_renderer {
                // Sync preview source (captured before dropping state)
                preview.set_source(preview_source);
                
                // Get source texture based on preview source
                let source_view: &wgpu::TextureView = match preview_source {
                    crate::engine::preview::PreviewSource::Block1 => &self.block1_texture.view,
                    crate::engine::preview::PreviewSource::Block2 => &self.block2_texture.view,
                    crate::engine::preview::PreviewSource::Block3 => &self.block3_texture.view,
                    crate::engine::preview::PreviewSource::Input1 => input1_view,
                    crate::engine::preview::PreviewSource::Input2 => input2_view,
                };
                
                preview.update(&self.device, &self.queue, source_view);
                preview.process_readback();
                
                // Handle color picking - only sample when mouse click is requested
                let pick_request = if let Ok(state) = self.shared_state.lock() {
                    if state.preview_pick_requested {
                        Some(state.preview_pick_uv)
                    } else {
                        None
                    }
                } else {
                    None
                };
                
                // Only sample color when a pick request is active
                if let Some(pick_uv) = pick_request {
                    if let Some(color) = preview.pick_color_uv(pick_uv[0], pick_uv[1]) {
                        // Convert u8 RGB to f32 0-1 range
                        let color_f32 = [
                            color[0] as f32 / 255.0,
                            color[1] as f32 / 255.0,
                            color[2] as f32 / 255.0,
                        ];
                        // Write back to shared state and clear the request
                        if let Ok(mut state) = self.shared_state.lock() {
                            state.preview_sampled_color = color_f32;
                            state.preview_pick_requested = false;
                            log::debug!("Color picked at UV [{:.3}, {:.3}]: [{:.3}, {:.3}, {:.3}]", 
                                pick_uv[0], pick_uv[1], color_f32[0], color_f32[1], color_f32[2]);
                        }
                    }
                }
            }
        }
        
        self.queue.submit(std::iter::once(encoder.finish()));
        surface_texture.present();
        
        self.frame_count += 1;
    }
    
    /// Get the modular Block 1 output view for preview window
    pub fn get_modular_block1_output_view(&self) -> &wgpu::TextureView {
        self.modular_block1.get_output_view()
    }
    
    /// Get the device for creating preview resources
    pub fn get_device(&self) -> &wgpu::Device {
        &self.device
    }
    
    /// Get the queue for preview rendering
    pub fn get_queue(&self) -> &wgpu::Queue {
        &self.queue
    }
    
    /// Get the preview renderer
    pub fn get_preview_renderer(&self) -> Option<&crate::engine::preview::PreviewRenderer> {
        self.preview_renderer.as_ref()
    }
    
    /// Get the preview renderer (mutable)
    pub fn get_preview_renderer_mut(&mut self) -> Option<&mut crate::engine::preview::PreviewRenderer> {
        self.preview_renderer.as_mut()
    }
}

/// Apply audio modulations to Block 1 parameters
fn apply_audio_modulations_to_block1(
    params: &mut crate::params::Block1Params,
    modulations: &HashMap<String, ParamModulationData>,
    fft_values: &[f32; 8],
) {
    for (param_name, modulation) in modulations {
        if !modulation.audio_enabled {
            continue;
        }
        
        let fft_band = modulation.audio_fft_band.clamp(0, 7) as usize;
        let fft_value = fft_values[fft_band];
        let modulated = apply_audio_modulations(0.0, modulation, fft_value, 0.0);
        
        // Apply modulation to the parameter
        match param_name.as_str() {
            "ch1_x_displace" => params.ch1_x_displace += modulated,
            "ch1_y_displace" => params.ch1_y_displace += modulated,
            "ch1_z_displace" => params.ch1_z_displace += modulated,
            "ch1_rotate" => params.ch1_rotate += modulated,
            "ch1_hsb_attenuate.x" => params.ch1_hsb_attenuate.x += modulated,
            "ch1_hsb_attenuate.y" => params.ch1_hsb_attenuate.y += modulated,
            "ch1_hsb_attenuate.z" => params.ch1_hsb_attenuate.z += modulated,
            "ch1_kaleidoscope_amount" => params.ch1_kaleidoscope_amount += modulated,
            "ch1_blur_amount" => params.ch1_blur_amount += modulated,
            "ch2_mix_amount" => params.ch2_mix_amount += modulated,
            "ch2_x_displace" => params.ch2_x_displace += modulated,
            "ch2_y_displace" => params.ch2_y_displace += modulated,
            "ch2_rotate" => params.ch2_rotate += modulated,
            "fb1_mix_amount" => params.fb1_mix_amount += modulated,
            "fb1_x_displace" => params.fb1_x_displace += modulated,
            "fb1_y_displace" => params.fb1_y_displace += modulated,
            "fb1_rotate" => params.fb1_rotate += modulated,
            _ => {} // Unknown parameter
        }
    }
}

/// Apply audio modulations to Block 2 parameters
fn apply_audio_modulations_to_block2(
    params: &mut crate::params::Block2Params,
    modulations: &HashMap<String, ParamModulationData>,
    fft_values: &[f32; 8],
) {
    for (param_name, modulation) in modulations {
        if !modulation.audio_enabled {
            continue;
        }
        
        let fft_band = modulation.audio_fft_band.clamp(0, 7) as usize;
        let fft_value = fft_values[fft_band];
        let modulated = apply_audio_modulations(0.0, modulation, fft_value, 0.0);
        
        match param_name.as_str() {
            "block2_input_x_displace" => params.block2_input_x_displace += modulated,
            "block2_input_y_displace" => params.block2_input_y_displace += modulated,
            "block2_input_rotate" => params.block2_input_rotate += modulated,
            "block2_input_blur_amount" => params.block2_input_blur_amount += modulated,
            "fb2_mix_amount" => params.fb2_mix_amount += modulated,
            "fb2_x_displace" => params.fb2_x_displace += modulated,
            "fb2_y_displace" => params.fb2_y_displace += modulated,
            "fb2_rotate" => params.fb2_rotate += modulated,
            _ => {}
        }
    }
}

/// Apply audio modulations to Block 3 parameters
fn apply_audio_modulations_to_block3(
    params: &mut crate::params::Block3Params,
    modulations: &HashMap<String, ParamModulationData>,
    fft_values: &[f32; 8],
) {
    for (param_name, modulation) in modulations {
        if !modulation.audio_enabled {
            continue;
        }
        
        let fft_band = modulation.audio_fft_band.clamp(0, 7) as usize;
        let fft_value = fft_values[fft_band];
        let modulated = apply_audio_modulations(0.0, modulation, fft_value, 0.0);
        
        match param_name.as_str() {
            "block1_reprocess_x_displace" => params.block1_x_displace += modulated,
            "block1_reprocess_y_displace" => params.block1_y_displace += modulated,
            "block1_reprocess_rotate" => params.block1_rotate += modulated,
            "block2_reprocess_x_displace" => params.block2_x_displace += modulated,
            "block2_reprocess_y_displace" => params.block2_y_displace += modulated,
            "block2_reprocess_rotate" => params.block2_rotate += modulated,
            "final_mix_amount" => params.final_mix_amount += modulated,
            _ => {}
        }
    }
}
