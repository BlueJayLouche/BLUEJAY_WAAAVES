//! # Core Module
//!
//! Contains shared state and core types used throughout the application.

use crate::config::AppConfig;
use crate::params::{Block1Params, Block2Params, Block3Params, LfoBank, ParamModulationData};
use std::collections::HashMap;

pub mod lfo_engine;

/// Preview source selection (defined here to avoid circular deps)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PreviewSource {
    #[default]
    Block1,
    Block2,
    Block3, // Final output
    Input1,
    Input2,
}

impl PreviewSource {
    pub fn display_name(&self) -> &'static str {
        match self {
            PreviewSource::Block1 => "Block 1",
            PreviewSource::Block2 => "Block 2",
            PreviewSource::Block3 => "Block 3 (Final)",
            PreviewSource::Input1 => "Input 1",
            PreviewSource::Input2 => "Input 2",
        }
    }
}

/// Input change request
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputChangeRequest {
    None,
    StartWebcam { input_id: u8, device_index: usize, width: u32, height: u32, fps: u32 },
    StopInput { input_id: u8 },
}

/// Audio change request
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AudioChangeRequest {
    None,
    ChangeDevice { device_index: i32 },
}

/// Debug visualization modes for modular Block 1
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Block1DebugView {
    /// Normal final output
    #[default]
    Normal,
    /// Stage 1: Input sampling output
    Stage1Input,
    /// Stage 2: Effects output (if enabled)
    Stage2Effects,
    /// Stage 3: Mixing output
    Stage3Mix,
    /// Feedback buffer contents
    FeedbackBuffer,
}

/// Output display mode - which block to show
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum OutputMode {
    Block1,
    Block2,
    #[default]
    Block3,
    PreviewInput1,
    PreviewInput2,
}

/// LFO assignment for a single parameter
#[derive(Debug, Clone, Copy, Default)]
pub struct LfoAssignment {
    /// Which LFO bank (0-15) to use, or -1 for none
    pub bank_index: i32,
    /// Amplitude/scaling of the modulation
    pub amplitude: f32,
    /// Whether this assignment is active
    pub enabled: bool,
}

/// LFO parameter mappings for a block
pub type LfoParameterMap = HashMap<String, LfoAssignment>;

/// Shared application state between windows
/// 
/// This struct is wrapped in an Arc<Mutex<>> and shared between
/// the output window (engine) and control window (GUI).
#[derive(Debug)]
pub struct SharedState {
    /// Block 1 parameters (channel mixing)
    pub block1: Block1Params,
    /// Block 2 parameters (secondary processing)
    pub block2: Block2Params,
    /// Block 3 parameters (final mixing)
    pub block3: Block3Params,
    /// LFO banks for modulation
    pub lfo_banks: Vec<LfoBank>,
    /// LFO to parameter mappings for Block 1
    pub block1_lfo_map: LfoParameterMap,
    /// LFO to parameter mappings for Block 2
    pub block2_lfo_map: LfoParameterMap,
    /// LFO to parameter mappings for Block 3
    pub block3_lfo_map: LfoParameterMap,
    /// Audio analysis data
    pub audio: AudioState,
    /// Current frame number
    pub frame_count: u64,
    /// Whether to clear feedback buffers
    pub clear_feedback: bool,
    /// Output resolution
    pub output_size: (u32, u32),
    /// Internal processing resolution
    pub internal_size: (u32, u32),
    /// Recording state
    pub is_recording: bool,
    /// Input 1 change request (GUI -> Engine)
    pub input1_change_request: InputChangeRequest,
    /// Input 2 change request (GUI -> Engine)
    pub input2_change_request: InputChangeRequest,
    /// Audio change request (GUI -> Engine)
    pub audio_change_request: AudioChangeRequest,
    /// Output display mode (which block to show)
    pub output_mode: OutputMode,
    /// Global BPM for tempo-synced LFOs
    pub bpm: f32,
    /// Debug view for modular Block 1
    pub block1_debug_view: Block1DebugView,
    /// Active audio/BPM modulations for Block 1
    pub block1_modulations: HashMap<String, ParamModulationData>,
    /// Active audio/BPM modulations for Block 2
    pub block2_modulations: HashMap<String, ParamModulationData>,
    /// Active audio/BPM modulations for Block 3
    pub block3_modulations: HashMap<String, ParamModulationData>,
    /// UI scale for ImGui (1.0 = 100%, 2.0 = 200%, etc.)
    /// This allows GUI to request scale changes that the engine applies
    pub ui_scale: f32,
    /// Preview window state - sampled color from preview (RGB 0-1)
    pub preview_sampled_color: [f32; 3],
    /// Preview source selection (which block/input to preview)
    pub preview_source: PreviewSource,
    /// Mouse pick UV coordinates [u, v] where user clicked on preview (0-1 range)
    pub preview_pick_uv: [f32; 2],
    /// Flag set by GUI when a color pick is requested
    pub preview_pick_requested: bool,
    /// Whether preview window is open (if false, engine skips preview computation)
    pub preview_enabled: bool,
}

/// Audio analysis state
#[derive(Debug, Clone)]
pub struct AudioState {
    /// FFT data (frequency bands)
    pub fft: Vec<f32>,
    /// Overall volume
    pub volume: f32,
    /// Beat detection
    pub beat: bool,
    /// BPM estimate
    pub bpm: f32,
    /// Beat phase (0-1)
    pub beat_phase: f32,
}

impl SharedState {
    /// Create new shared state from configuration
    pub fn new(config: &AppConfig) -> Self {
        Self {
            block1: Block1Params::default(),
            block2: Block2Params::default(),
            block3: Block3Params::default(),
            lfo_banks: vec![LfoBank::default(); 16], // 16 macro banks
            block1_lfo_map: HashMap::new(),
            block2_lfo_map: HashMap::new(),
            block3_lfo_map: HashMap::new(),
            audio: AudioState::default(),
            frame_count: 0,
            clear_feedback: false,
            output_size: (config.output_window.width, config.output_window.height),
            internal_size: (config.pipeline.internal_width, config.pipeline.internal_height),
            is_recording: false,
            input1_change_request: InputChangeRequest::None,
            input2_change_request: InputChangeRequest::None,
            audio_change_request: AudioChangeRequest::None,
            output_mode: OutputMode::default(),
            bpm: 120.0,
            block1_debug_view: Block1DebugView::default(),
            block1_modulations: HashMap::new(),
            block2_modulations: HashMap::new(),
            block3_modulations: HashMap::new(),
            ui_scale: config.ui_scale,
            preview_sampled_color: [1.0, 1.0, 1.0], // Default white
            preview_source: PreviewSource::Block3,   // Default to final output
            preview_pick_uv: [0.5, 0.5],             // Default to center
            preview_pick_requested: false,
            preview_enabled: true,                   // Preview starts enabled
        }
    }
}

impl Default for AudioState {
    fn default() -> Self {
        Self {
            fft: vec![0.0; 16], // 16 frequency bands
            volume: 0.0,
            beat: false,
            bpm: 120.0,
            beat_phase: 0.0,
        }
    }
}

/// Vertex data for full-screen quad
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 2],
    pub texcoord: [f32; 2],
}

impl Vertex {
    /// Create a vertex buffer for a full-screen quad
    pub fn quad_vertices() -> Vec<Vertex> {
        vec![
            // Triangle 1
            Vertex { position: [-1.0, -1.0], texcoord: [0.0, 1.0] },  // Bottom-left (V=1)
            Vertex { position: [ 1.0, -1.0], texcoord: [1.0, 1.0] },  // Bottom-right (V=1)
            Vertex { position: [-1.0,  1.0], texcoord: [0.0, 0.0] },  // Top-left (V=0)
            // Triangle 2
            Vertex { position: [-1.0,  1.0], texcoord: [0.0, 0.0] },  // Top-left (V=0)
            Vertex { position: [ 1.0, -1.0], texcoord: [1.0, 1.0] },  // Bottom-right (V=1)
            Vertex { position: [ 1.0,  1.0], texcoord: [1.0, 0.0] },  // Top-right (V=0)
        ]
    }
    
    /// Vertex buffer layout for wgpu
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // Position at location 0
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // Texcoord at location 1
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}
