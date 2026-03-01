//! # GUI Module
//!
//! ImGui-based control interface for the VJ application.
//! Provides real-time parameter control, preset management, and input configuration.

use crate::config::{AppConfig, LayoutConfig, TabId};
use crate::core::{InputChangeRequest, OutputMode, PreviewSource, SharedState};
use crate::input::InputType;
use crate::params::preset::{PresetData, PresetManager};
use crate::params::{Block1Params, Block2Params, Block3Params};
use glam::{Vec3, Vec4};
use imgui::{CollapsingHeader, ComboBox, Condition, Drag, Ui};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// =============================================================================
// CONSTANTS
// =============================================================================

/// Waveform names for LFO selection
pub const WAVEFORM_NAMES: &[&str] = &["Sine", "Triangle", "Ramp", "Saw", "Square"];

/// Beat divisions for tempo-synced LFOs
pub const BEAT_DIVISIONS: &[&str] = &["1/16", "1/8", "1/4", "1/2", "1", "2", "4", "8"];

/// Geometric overflow modes
pub const GEO_OVERFLOW_MODES: &[&str] = &["Clamp", "Toroid", "Mirror"];

/// Mix/blend types
pub const MIX_TYPES: &[&str] = &["Linear", "Additive", "Difference", "Multiplicative", "Dodge"];

/// Keying modes (OF-style: 0=Lumakey, 1=Chromakey)
pub const KEY_MODES: &[&str] = &["Lumakey", "Chromakey"];

// =============================================================================
// TYPE DEFINITIONS
// =============================================================================

/// Audio modulation state for a parameter
#[derive(Debug, Clone, Default)]
pub struct ParamAudioModulation {
    pub enabled: bool,
    pub fft_band: i32,
    pub amount: f32,
}

/// LFO state for a parameter
#[derive(Debug, Clone)]
pub struct LfoState {
    pub enabled: bool,
    pub amplitude: f32,
    pub rate: f32,
    pub waveform: i32,       // 0=Sine, 1=Triangle, 2=Ramp, 3=Saw, 4=Square
    pub tempo_sync: bool,
    pub division: i32,       // 0=1/16, 1=1/8, 2=1/4, 3=1/2, 4=1, 5=2, 6=4, 7=8
    pub bank_index: i32,     // Which LFO bank (0-15) to use
}

impl Default for LfoState {
    fn default() -> Self {
        Self {
            enabled: false,
            amplitude: 0.0,
            rate: 0.5,
            waveform: 0,
            tempo_sync: false,
            division: 2, // 1/4 note default
            bank_index: 0, // Default to bank 0
        }
    }
}

/// Block 1 sub-tabs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Block1Tab {
    Ch1Adjust,
    Ch2MixAndKey,
    Ch2Adjust,
    Fb1Parameters,
    Lfo,
}

/// Block 2 sub-tabs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Block2Tab {
    InputAdjust,
    Fb2Parameters,
    Lfo,
}

/// Block 3 sub-tabs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Block3Tab {
    Block1Reprocess,
    Block2Reprocess,
    MatrixMixer,
    FinalMix,
    Lfo,
}



/// Main tab selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MainTab {
    Block1,
    Block2,
    Block3,
    Macros,
    Inputs,
    Settings,
}

// =============================================================================
// CONTROL GUI STRUCT
// =============================================================================

/// Main GUI controller for the application
pub struct ControlGui {
    /// Shared state with the engine
    pub shared_state: Arc<Mutex<SharedState>>,
    /// Application configuration (for saving input settings)
    config: crate::config::AppConfig,
    /// Layout configuration for popped-out tabs
    layout_config: LayoutConfig,
    
    /// Show ImGui demo window
    pub show_demo: bool,
    
    /// Currently selected main tab
    pub selected_tab: MainTab,
    
    /// Block 1 sub-tab
    pub block1_tab: Block1Tab,
    /// Block 2 sub-tab
    pub block2_tab: Block2Tab,
    /// Block 3 sub-tab
    pub block3_tab: Block3Tab,
    
    // Preset management
    preset_manager: PresetManager,
    preset_name_input: String,
    selected_bank: String,
    selected_preset_index: i32,
    preset_status_message: String,
    preset_status_timer: f32,
    
    // Parameter copies for editing (to reduce lock contention)
    pub block1_edit: Block1Params,
    pub block2_edit: Block2Params,
    pub block3_edit: Block3Params,
    
    // Input selection
    pub input1_type: InputType,
    pub input2_type: InputType,
    pub selected_webcam1: i32,
    pub selected_webcam2: i32,
    pub webcam_devices: Vec<String>,
    
    // Audio modulation UI state
    show_audio_panel: bool,
    selected_block1_param: i32,
    selected_block2_param: i32,
    selected_block3_param: i32,
    audio_mod_fft_band: i32,
    audio_mod_amount: f32,
    
    // LFO editor state
    selected_lfo_bank: i32,
    
    // Tempo / Tap Tempo state
    bpm: f32,
    bpm_enabled: bool,
    bpm_playing: bool,
    tap_times: Vec<f64>,
    last_tap_time: f64,
    beat_flash: f32,
    
    // LFO parameter states (amplitude, rate, waveform, sync)
    // Block 1 LFOs
    ch1_lfo_params: LfoParamGroup,      // CH1 Adjust LFOs
    ch2_mix_lfo_params: LfoParamGroup,  // CH2 Mix LFOs
    ch2_adj_lfo_params: LfoParamGroup,  // CH2 Adjust LFOs
    fb1_lfo_params: LfoParamGroup,      // FB1 LFOs
    
    // Block 2 LFOs
    b2_input_lfo_params: LfoParamGroup,
    fb2_lfo_params: LfoParamGroup,
    
    // Block 3 LFOs
    b3_b1_lfo_params: LfoParamGroup,
    b3_b2_lfo_params: LfoParamGroup,
    
    // Per-parameter LFO states for detailed LFO control
    block1_lfos: HashMap<String, LfoState>,
    block2_lfos: HashMap<String, LfoState>,
    block3_lfos: HashMap<String, LfoState>,
    
    // Status message
    status_message: String,
    status_timer: f32,
    
    // Main FPS counter (displayed in Settings tab)
    main_fps: f32,
    frame_times: [f32; 60], // Ring buffer for last 60 frames
    frame_time_index: usize, // Current index in ring buffer
    frame_time_count: usize, // Number of valid entries
    last_frame_time: std::time::Instant,
    
    // Preview window state
    show_preview_window: bool,
    preview_source: PreviewSource,
    preview_texture_id: Option<imgui::TextureId>,
    preview_sampled_color: [f32; 3],
    preview_crosshair_uv: [f32; 2], // Crosshair position in UV coordinates (0-1)
    preview_image_pos: [f32; 2],   // Position of preview image in window (for mouse picking)
    preview_image_size: [f32; 2],  // Size of preview image
    selected_key_target: i32,       // Selected key target for "Apply to Key"
    preview_fps: f32,              // FPS counter for preview
    preview_last_frame_time: std::time::Instant, // For FPS calculation
}

/// LFO parameters for a group of controls
#[derive(Debug, Clone)]
pub struct LfoParamGroup {
    pub amplitude: f32,
    pub rate: f32,
    pub waveform: i32,
    pub tempo_sync: bool,
    pub division: i32,
}

impl Default for LfoParamGroup {
    fn default() -> Self {
        Self {
            amplitude: 0.0,
            rate: 0.15,
            waveform: 0,
            tempo_sync: false,
            division: 2, // 1/4 note default
        }
    }
}

// =============================================================================
// IMPLEMENTATION
// =============================================================================

impl ControlGui {
    /// Create a new ControlGui instance
    pub fn new(config: &crate::config::AppConfig, shared_state: Arc<Mutex<SharedState>>) -> anyhow::Result<Self> {
        let preset_manager = PresetManager::new();
        let selected_bank = preset_manager.get_current_bank().to_string();
        
        // Get initial state from shared state
        let (block1_edit, block2_edit, block3_edit) = {
            let state = shared_state.lock().unwrap();
            (state.block1, state.block2, state.block3)
        };
        
        // Load webcam devices
        let webcam_devices = crate::input::webcam::list_cameras();
        log::info!("Found {} webcam device(s)", webcam_devices.len());
        
        // Load input settings from config
        let input1_type = match config.inputs.input1_type {
            0 => InputType::None,
            1 => InputType::Webcam,
            2 => InputType::Ndi,
            3 => InputType::Spout,
            4 => InputType::VideoFile,
            _ => InputType::None,
        };
        let input2_type = match config.inputs.input2_type {
            0 => InputType::None,
            1 => InputType::Webcam,
            2 => InputType::Ndi,
            3 => InputType::Spout,
            4 => InputType::VideoFile,
            _ => InputType::None,
        };
        
        // Validate device indices
        let selected_webcam1 = if config.inputs.input1_device >= 0 
            && (config.inputs.input1_device as usize) < webcam_devices.len() {
            config.inputs.input1_device
        } else {
            -1
        };
        let selected_webcam2 = if config.inputs.input2_device >= 0 
            && (config.inputs.input2_device as usize) < webcam_devices.len() {
            config.inputs.input2_device
        } else {
            -1
        };
        
        log::info!("Config: Input1={:?} (device {}), Input2={:?} (device {}), AutoStart={}",
            input1_type, selected_webcam1, input2_type, selected_webcam2, config.inputs.auto_start_webcams);
        
        Ok(Self {
            shared_state,
            config: config.clone(),
            show_demo: false,
            selected_tab: MainTab::Block1,
            block1_tab: Block1Tab::Ch1Adjust,
            block2_tab: Block2Tab::InputAdjust,
            block3_tab: Block3Tab::FinalMix,
            preset_manager,
            preset_name_input: String::new(),
            selected_bank,
            selected_preset_index: -1,
            preset_status_message: String::new(),
            preset_status_timer: 0.0,
            block1_edit,
            block2_edit,
            block3_edit,
            input1_type,
            input2_type,
            selected_webcam1,
            selected_webcam2,
            webcam_devices,
            show_audio_panel: false,
            selected_block1_param: 0,
            selected_block2_param: 0,
            selected_block3_param: 0,
            audio_mod_fft_band: 0,
            audio_mod_amount: 0.5,
            selected_lfo_bank: 0,
            
            // Tempo state
            bpm: 120.0,
            bpm_enabled: true,
            bpm_playing: true,
            tap_times: Vec::new(),
            last_tap_time: 0.0,
            beat_flash: 0.0,
            
            // Block 1 LFOs
            ch1_lfo_params: LfoParamGroup::default(),
            ch2_mix_lfo_params: LfoParamGroup::default(),
            ch2_adj_lfo_params: LfoParamGroup::default(),
            fb1_lfo_params: LfoParamGroup::default(),
            
            // Block 2 LFOs
            b2_input_lfo_params: LfoParamGroup::default(),
            fb2_lfo_params: LfoParamGroup::default(),
            
            // Block 3 LFOs
            b3_b1_lfo_params: LfoParamGroup::default(),
            b3_b2_lfo_params: LfoParamGroup::default(),
            
            // Per-parameter LFO states
            block1_lfos: HashMap::new(),
            block2_lfos: HashMap::new(),
            block3_lfos: HashMap::new(),
            
            status_message: String::new(),
            status_timer: 0.0,
            
            // Main FPS counter
            main_fps: 0.0,
            frame_times: [0.0; 60],
            frame_time_index: 0,
            frame_time_count: 0,
            last_frame_time: std::time::Instant::now(),
            
            // Layout config for popped-out tabs
            layout_config: LayoutConfig::load(),
            
            // Preview window state
            show_preview_window: false,
            preview_source: PreviewSource::Block3,
            preview_texture_id: None,
            preview_sampled_color: [1.0, 1.0, 1.0],
            preview_crosshair_uv: [0.5, 0.5], // Start at center
            preview_image_pos: [0.0, 0.0],
            preview_image_size: [320.0, 180.0],
            selected_key_target: 0, // Default to first key target
            preview_fps: 0.0,
            preview_last_frame_time: std::time::Instant::now(),
        })
    }
    
    /// Set the preview texture ID (registered with imgui-wgpu)
    pub fn set_preview_texture_id(&mut self, texture_id: imgui::TextureId) {
        self.preview_texture_id = Some(texture_id);
        log::info!("Preview texture ID set: {:?}", texture_id);
    }
    
    /// Sync parameters from shared state to local copies
    pub fn sync_from_shared_state(&mut self) {
        if let Ok(state) = self.shared_state.lock() {
            self.block1_edit = state.block1;
            self.block2_edit = state.block2;
            self.block3_edit = state.block3;
        }
    }
    
    /// Sync parameters from local copies to shared state
    pub fn sync_to_shared_state(&mut self) {
        if let Ok(mut state) = self.shared_state.lock() {
            state.block1 = self.block1_edit;
            state.block2 = self.block2_edit;
            state.block3 = self.block3_edit;
            
            // Sync LFO assignments
            self.sync_lfo_map(&self.block1_lfos, &mut state.block1_lfo_map);
            self.sync_lfo_map(&self.block2_lfos, &mut state.block2_lfo_map);
            self.sync_lfo_map(&self.block3_lfos, &mut state.block3_lfo_map);
        }
    }
    
    /// Sync GUI LFO states to shared LFO parameter map
    fn sync_lfo_map(
        &self,
        gui_lfos: &HashMap<String, LfoState>,
        shared_map: &mut crate::core::LfoParameterMap,
    ) {
        shared_map.clear();
        for (param_id, lfo_state) in gui_lfos.iter() {
            if lfo_state.enabled {
                shared_map.insert(
                    param_id.clone(),
                    crate::core::LfoAssignment {
                        bank_index: lfo_state.bank_index,
                        amplitude: lfo_state.amplitude,
                        enabled: true,
                    },
                );
            }
        }
    }
    
    /// Show a status message
    pub fn show_status(&mut self, message: &str) {
        self.status_message = message.to_string();
        self.status_timer = 3.0; // Show for 3 seconds
    }
    
    /// Refresh the list of available devices
    fn refresh_devices(&mut self) {
        // Scan for webcam devices
        self.webcam_devices = crate::input::webcam::list_cameras();
        log::info!("Refreshed device list: {} webcam(s) found", self.webcam_devices.len());
        
        // Reset selections if they're now out of bounds
        if self.selected_webcam1 >= 0 && (self.selected_webcam1 as usize) >= self.webcam_devices.len() {
            self.selected_webcam1 = -1;
        }
        if self.selected_webcam2 >= 0 && (self.selected_webcam2 as usize) >= self.webcam_devices.len() {
            self.selected_webcam2 = -1;
        }
    }
    
    /// Save current input settings to config file
    fn save_input_config(&self) {
        // Update config with current values
        let input1_type_int = match self.input1_type {
            InputType::None => 0,
            InputType::Webcam => 1,
            InputType::Ndi => 2,
            InputType::Spout => 3,
            InputType::VideoFile => 4,
        };
        let input2_type_int = match self.input2_type {
            InputType::None => 0,
            InputType::Webcam => 1,
            InputType::Ndi => 2,
            InputType::Spout => 3,
            InputType::VideoFile => 4,
        };
        
        // We can't modify self.config directly since it's used immutably,
        // so we save directly using the current values
        let mut config = crate::config::AppConfig::load_or_default();
        config.inputs.input1_type = input1_type_int;
        config.inputs.input2_type = input2_type_int;
        config.inputs.input1_device = self.selected_webcam1;
        config.inputs.input2_device = self.selected_webcam2;
        config.inputs.auto_start_webcams = true; // Once user sets up, auto-start is enabled
        
        if let Err(e) = config.save() {
            log::warn!("Failed to save input config: {}", e);
        } else {
            log::debug!("Input config saved: Input1={} (dev {}), Input2={} (dev {})",
                input1_type_int, self.selected_webcam1, input2_type_int, self.selected_webcam2);
        }
    }
    
    /// Auto-start webcams based on saved config (called once at startup)
    pub fn auto_start_webcams(&mut self) {
        if !self.config.inputs.auto_start_webcams {
            log::info!("Auto-start webcams disabled in config");
            return;
        }
        
        log::info!("Auto-starting webcams...");
        
        // Auto-start webcam 1 if configured
        if self.input1_type == InputType::Webcam && self.selected_webcam1 >= 0 {
            let device_index = self.selected_webcam1 as usize;
            if device_index < self.webcam_devices.len() {
                log::info!("Auto-starting Webcam 1 (device {}: {})", 
                    device_index, self.webcam_devices[device_index]);
                if let Ok(mut state) = self.shared_state.lock() {
                    state.input1_change_request = crate::core::InputChangeRequest::StartWebcam { 
                        input_id: 1,
                        device_index,
                        width: 1280,
                        height: 720,
                        fps: 30,
                    };
                }
            } else {
                log::warn!("Webcam 1 device index {} out of bounds ({} devices)", 
                    device_index, self.webcam_devices.len());
            }
        }
        
        // Auto-start webcam 2 if configured
        if self.input2_type == InputType::Webcam && self.selected_webcam2 >= 0 {
            let device_index = self.selected_webcam2 as usize;
            if device_index < self.webcam_devices.len() {
                log::info!("Auto-starting Webcam 2 (device {}: {})", 
                    device_index, self.webcam_devices[device_index]);
                if let Ok(mut state) = self.shared_state.lock() {
                    state.input2_change_request = crate::core::InputChangeRequest::StartWebcam { 
                        input_id: 2,
                        device_index,
                        width: 1280,
                        height: 720,
                        fps: 30,
                    };
                }
            } else {
                log::warn!("Webcam 2 device index {} out of bounds ({} devices)", 
                    device_index, self.webcam_devices.len());
            }
        }
    }
    
    /// Build the complete UI
    pub fn build_ui(&mut self, ui: &mut Ui) {
        // Update FPS counter (average over last 60 frames for smooth display)
        let now = std::time::Instant::now();
        let delta = now.duration_since(self.last_frame_time).as_secs_f32();
        self.last_frame_time = now;
        if delta > 0.0 {
            let fps = 1.0 / delta;
            // Ring buffer: store at current index and advance
            self.frame_times[self.frame_time_index] = fps;
            self.frame_time_index = (self.frame_time_index + 1) % 60;
            if self.frame_time_count < 60 {
                self.frame_time_count += 1;
            }
            // Calculate average over valid entries
            if self.frame_time_count > 0 {
                let sum: f32 = self.frame_times[..self.frame_time_count].iter().sum();
                self.main_fps = sum / self.frame_time_count as f32;
            }
        }
        
        // Update status timer
        if self.status_timer > 0.0 {
            self.status_timer -= ui.io().delta_time;
            if self.status_timer < 0.0 {
                self.status_timer = 0.0;
                self.status_message.clear();
            }
        }
        
        // Update beat flash
        if self.beat_flash > 0.0 {
            self.beat_flash -= ui.io().delta_time;
            if self.beat_flash < 0.0 {
                self.beat_flash = 0.0;
            }
        }
        
        // Sync from shared state at start of frame
        self.sync_from_shared_state();
        
        // Build UI components
        self.build_menu_bar(ui);
        self.build_top_bar(ui);
        self.build_main_tabs(ui);
        
        // Show demo window if requested
        if self.show_demo {
            ui.show_demo_window(&mut self.show_demo);
        }
        
        // Sync back to shared state at end of frame
        self.sync_to_shared_state();
    }
    
    /// Build the menu bar
    fn build_menu_bar(&mut self, ui: &Ui) {
        ui.menu_bar(|| {
            ui.menu("File", || {
                if ui.menu_item("Exit") {
                    // Exit handled by main loop
                }
            });
            
            ui.menu("View", || {
                if ui.menu_item_config("Show Demo Window")
                    .build_with_ref(&mut self.show_demo) {}
            });
        });
    }
    
    /// Build the top bar with preset controls and status
    fn build_top_bar(&mut self, ui: &Ui) {
        ui.group(|| {
            // Preset section
            self.build_preset_section(ui);
            
            // Status message
            if !self.status_message.is_empty() {
                ui.same_line_with_pos(600.0);
                ui.text_colored([0.0, 1.0, 0.0, 1.0], &self.status_message);
            }
        });
        
        ui.separator();
    }
    
    /// Build preset management section
    fn build_preset_section(&mut self, ui: &Ui) {
        ui.text("Preset:");
        ui.same_line();
        
        // Bank selector
        let banks = self.preset_manager.get_bank_names();
        let current_bank = self.preset_manager.get_current_bank();
        let mut bank_idx = banks.iter().position(|b| b == current_bank).unwrap_or(0) as i32;
        
        let bank_preview = if banks.is_empty() { "Default".to_string() } else { banks[bank_idx as usize].clone() };
        
        ComboBox::new(ui, "##bank_select")
            .preview_value(&bank_preview)
            .build(|| {
                for (idx, name) in banks.iter().enumerate() {
                    if ui.selectable_config(name)
                        .selected(idx == bank_idx as usize)
                        .build() {
                        bank_idx = idx as i32;
                    }
                }
            });
        
        if bank_idx >= 0 && bank_idx < banks.len() as i32 {
            let new_bank = banks[bank_idx as usize].clone();
            if new_bank != self.selected_bank {
                self.preset_manager.switch_bank(&new_bank);
                self.selected_bank = new_bank;
                self.selected_preset_index = -1;
            }
        }
        
        ui.same_line();
        
        // Preset name input
        ui.set_next_item_width(150.0);
        imgui::InputText::new(ui, "##preset_name", &mut self.preset_name_input)
            .hint("Preset name")
            .build();
        
        ui.same_line();
        
        // Save button
        if ui.button("Save") {
            if !self.preset_name_input.is_empty() {
                let preset_data = PresetData {
                    block1: self.block1_edit,
                    block2: self.block2_edit,
                    block3: self.block3_edit,
                    block1_modulations: HashMap::new(),
                    block2_modulations: HashMap::new(),
                    block3_modulations: HashMap::new(),
                    tempo: crate::params::preset::PresetTempoData::default(),
                    version: env!("CARGO_PKG_VERSION").to_string(),
                    name: self.preset_name_input.clone(),
                };
                
                match self.preset_manager.save_preset(&self.preset_name_input, &preset_data) {
                    Ok(_) => self.show_status(&format!("Saved preset '{}'", self.preset_name_input)),
                    Err(e) => self.show_status(&format!("Save failed: {}", e)),
                }
            }
        }
        
        ui.same_line();
        ui.separator();
        ui.same_line();
        
        // Load preset selector
        let preset_names = self.preset_manager.get_preset_names();
        
        let load_preview = if self.selected_preset_index >= 0 && 
                              (self.selected_preset_index as usize) < preset_names.len() {
            preset_names[self.selected_preset_index as usize].clone()
        } else {
            "Select preset...".to_string()
        };
        
        let mut selected_idx = self.selected_preset_index;
        ComboBox::new(ui, "##load_preset")
            .preview_value(&load_preview)
            .build(|| {
                for (idx, name) in preset_names.iter().enumerate() {
                    if ui.selectable_config(name)
                        .selected(idx == selected_idx as usize)
                        .build() {
                        selected_idx = idx as i32;
                    }
                }
            });
        self.selected_preset_index = selected_idx;
        
        ui.same_line();
        
        // Load button
        if ui.button("Load") && self.selected_preset_index >= 0 {
            let idx = self.selected_preset_index as usize;
            if idx < preset_names.len() {
                let name = &preset_names[idx];
                match self.preset_manager.load_preset(name) {
                    Ok(data) => {
                        self.block1_edit = data.block1;
                        self.block2_edit = data.block2;
                        self.block3_edit = data.block3;
                        self.sync_to_shared_state();
                        self.show_status(&format!("Loaded preset '{}'", name));
                    }
                    Err(e) => self.show_status(&format!("Load failed: {}", e)),
                }
            }
        }
        
        ui.same_line();
        
        // Delete button
        if ui.button("Delete") && self.selected_preset_index >= 0 {
            let idx = self.selected_preset_index as usize;
            match self.preset_manager.delete_preset(idx) {
                Ok(_) => {
                    self.selected_preset_index = -1;
                    self.show_status("Preset deleted");
                }
                Err(e) => self.show_status(&format!("Delete failed: {}", e)),
            }
        }
        
        ui.same_line();
        ui.separator();
        ui.same_line();
        
        // Import OF preset button (batch import from OF saveStates folder)
        if ui.button("Import OF Dir...") {
            // Look for OF presets in the standard location
            let of_presets_dir = std::path::PathBuf::from(
                "/Users/alpha/Developer/of_v0.12.0_osx_release/apps/vj/BLUEJAY_WAAAVES/bin/data/saveStates"
            );
            
            if of_presets_dir.exists() {
                match self.preset_manager.batch_import_of_presets(&of_presets_dir) {
                    Ok((success, failed)) => {
                        self.show_status(&format!("Imported {} OF presets ({} failed)", success, failed));
                        // Refresh preset list
                        self.preset_manager.scan_banks();
                    }
                    Err(e) => {
                        self.show_status(&format!("Import failed: {}", e));
                    }
                }
            } else {
                self.show_status("OF presets directory not found");
            }
        }
    }
    
    /// Build main tab bar
    fn build_main_tabs(&mut self, ui: &Ui) {
        let tab_labels = ["Block 1", "Block 2", "Block 3", "Macros", "Inputs", "Settings"];
        let tab_ids = [
            TabId::Block1,
            TabId::Block2,
            TabId::Block3,
            TabId::Macros,
            TabId::Inputs,
            TabId::Settings,
        ];
        let mut selected_tab_idx = self.selected_tab as usize;
        let mut context_menu_tab: Option<TabId> = None;
        
        if let Some(_tab_bar) = ui.tab_bar("##main_tabs") {
            for (idx, (label, tab_id)) in tab_labels.iter().zip(tab_ids.iter()).enumerate() {
                let is_selected = idx == selected_tab_idx;
                let is_popped = self.layout_config.is_popped(tab_id);
                
                // Build tab item label with optional pop-out indicator
                let tab_label = if is_popped {
                    format!("{} ⧉", label)  // Show indicator when popped
                } else {
                    label.to_string()
                };
                
                // Build tab item and check if clicked
                if let Some(_tab) = ui.tab_item(&tab_label) {
                    if !is_selected {
                        selected_tab_idx = idx;
                        self.selected_tab = match idx {
                            0 => MainTab::Block1,
                            1 => MainTab::Block2,
                            2 => MainTab::Block3,
                            3 => MainTab::Macros,
                            4 => MainTab::Inputs,
                            5 => MainTab::Settings,
                            _ => MainTab::Block1,
                        };
                    }
                }
                
                // Check for right-click on tab item
                if ui.is_item_hovered() && ui.is_mouse_released(imgui::MouseButton::Right) {
                    ui.open_popup(&format!("##context_{:?}", tab_id));
                }
                
                // Context menu for this tab
                ui.popup(&format!("##context_{:?}", tab_id), || {
                    if ui.menu_item("Pop Out") {
                        if !is_popped {
                            self.layout_config.pop_tab(*tab_id);
                            if let Err(e) = self.layout_config.save() {
                                log::warn!("Failed to save layout: {}", e);
                            }
                        }
                    }
                    if ui.menu_item_config("Dock").enabled(is_popped).build() {
                        if is_popped {
                            self.layout_config.dock_tab(tab_id);
                            if let Err(e) = self.layout_config.save() {
                                log::warn!("Failed to save layout: {}", e);
                            }
                        }
                    }
                });
            }
        }
        
        // Build content based on selected tab
        match self.selected_tab {
            MainTab::Block1 => self.build_block1_panel(ui),
            MainTab::Block2 => self.build_block2_panel(ui),
            MainTab::Block3 => self.build_block3_panel(ui),
            MainTab::Macros => self.build_macros_panel(ui),
            MainTab::Inputs => self.build_inputs_panel(ui),
            MainTab::Settings => self.build_settings_panel(ui),
        }
        
        // Render popped-out tab windows
        self.render_popped_tabs(ui);
        
        // Audio panel button at bottom
        ui.separator();
        
        if ui.button("Audio Reactivity") {
            self.show_audio_panel = !self.show_audio_panel;
        }
        
        ui.same_line();
        
        if ui.button("Preview & Color Picker") {
            self.show_preview_window = !self.show_preview_window;
        }
        
        if self.show_audio_panel {
            match self.selected_tab {
                MainTab::Block1 => self.draw_block1_audio_panel(ui),
                MainTab::Block2 => self.draw_block2_audio_panel(ui),
                MainTab::Block3 => self.draw_block3_audio_panel(ui),
                _ => {}
            }
        }
        
        // Draw preview window if enabled
        if self.show_preview_window {
            // Get frame count from shared state for throttling
            let frame_count = if let Ok(state) = self.shared_state.lock() {
                state.frame_count
            } else {
                0
            };
            self.draw_preview_window(ui, frame_count);
        }
    }
    
    /// Render popped-out tab windows
    fn render_popped_tabs(&mut self, ui: &Ui) {
        // Collect tabs to render (to avoid borrow issues)
        let popped_tabs: Vec<TabId> = self.layout_config.popped_tabs.keys().cloned().collect();
        let mut layout_changed = false;
        
        for tab_id in popped_tabs {
            let window_state = self.layout_config.get_window_state(&tab_id);
            let title = tab_id.window_title();
            let bg_color = tab_id.bg_color();
            let border_color = tab_id.border_color();
            
            let mut opened = true;
            let mut new_pos: Option<[f32; 2]> = None;
            let mut new_size: Option<[f32; 2]> = None;
            
            // Apply color styling by pushing style vars
            let _style_bg = ui.push_style_color(imgui::StyleColor::WindowBg, bg_color);
            let _style_border = ui.push_style_color(imgui::StyleColor::Border, border_color);
            let _style_title_bg = ui.push_style_color(imgui::StyleColor::TitleBg, border_color);
            let _style_title_bg_active = ui.push_style_color(imgui::StyleColor::TitleBgActive, border_color);
            
            // Build window with saved position/size
            // Use FirstUseEver so ImGui remembers position after first frame
            ui.window(&title)
                .size([window_state.width, window_state.height], Condition::FirstUseEver)
                .position([window_state.pos_x, window_state.pos_y], Condition::FirstUseEver)
                .opened(&mut opened)
                .build(|| {
                    // Render tab content based on ID
                    match tab_id {
                        TabId::Block1 => self.build_block1_panel(ui),
                        TabId::Block2 => self.build_block2_panel(ui),
                        TabId::Block3 => self.build_block3_panel(ui),
                        TabId::Macros => self.build_macros_panel(ui),
                        TabId::Inputs => self.build_inputs_panel(ui),
                        TabId::Settings => self.build_settings_panel(ui),
                        // Sub-tabs not supported for pop-out yet
                        _ => {
                            ui.text("This tab cannot be popped out.");
                            ui.text("Pop out the parent tab instead.");
                        }
                    }
                    
                    // Capture window state inside the closure
                    new_pos = Some(ui.window_pos());
                    new_size = Some(ui.window_size());
                });
            
            // Style colors are automatically popped when _style vars go out of scope
            
            // Update window state if we got valid position/size
            if let (Some(pos), Some(size)) = (new_pos, new_size) {
                // Check if position or size changed significantly
                let pos_changed = (pos[0] - window_state.pos_x).abs() > 1.0
                    || (pos[1] - window_state.pos_y).abs() > 1.0;
                let size_changed = (size[0] - window_state.width).abs() > 1.0
                    || (size[1] - window_state.height).abs() > 1.0;
                
                if pos_changed || size_changed {
                    log::debug!("Window {:?} changed: pos={:?}, size={:?}", tab_id, pos, size);
                    self.layout_config.update_from_imgui(
                        &tab_id,
                        pos,
                        size,
                        false
                    );
                    layout_changed = true;
                }
            }
            
            // If window was closed, dock the tab
            if !opened {
                self.layout_config.dock_tab(&tab_id);
                layout_changed = true;
            }
        }
        
        // Save layout if anything changed
        if layout_changed {
            if let Err(e) = self.layout_config.save() {
                log::warn!("Failed to save layout: {}", e);
            } else {
                log::debug!("Layout saved successfully");
            }
        }
    }
    
    /// Build Block 1 panel with sub-tabs
    fn build_block1_panel(&mut self, ui: &Ui) {
        // Sub-tab bar
        let subtab_labels = ["Ch 1 Adjust", "Ch 2 Mix & Key", "Ch 2 Adjust", "FB1", "LFO"];
        let mut subtab_idx = self.block1_tab as usize;
        
        if let Some(_tab_bar) = ui.tab_bar("##block1_tabs") {
            for (idx, label) in subtab_labels.iter().enumerate() {
                let is_selected = idx == subtab_idx;
                
                if let Some(_tab) = ui.tab_item(label) {
                    if !is_selected {
                        subtab_idx = idx;
                        self.block1_tab = match idx {
                            0 => Block1Tab::Ch1Adjust,
                            1 => Block1Tab::Ch2MixAndKey,
                            2 => Block1Tab::Ch2Adjust,
                            3 => Block1Tab::Fb1Parameters,
                            4 => Block1Tab::Lfo,
                            _ => Block1Tab::Ch1Adjust,
                        };
                    }
                }
            }
        }
        
        // Build content based on sub-tab
        match self.block1_tab {
            Block1Tab::Ch1Adjust => self.build_block1_ch1_adjust(ui),
            Block1Tab::Ch2MixAndKey => self.build_block1_ch2_mix_key(ui),
            Block1Tab::Ch2Adjust => self.build_block1_ch2_adjust(ui),
            Block1Tab::Fb1Parameters => self.build_block1_fb1_params(ui),
            Block1Tab::Lfo => self.build_block1_lfo(ui),
        }
    }
    
    /// Build Block 1 Channel 1 Adjust panel
    fn build_block1_ch1_adjust(&mut self, ui: &Ui) {
        // Extract config values before borrowing self mutably
        let show_osc = self.config.show_osc_addresses;
        let shared_state = Arc::clone(&self.shared_state);
        
        let p = &mut self.block1_edit;
        
        // Input selection
        ui.text("Input Source:");
        ui.same_line();
        let input_options = ["Input 1", "Input 2"];
        let mut input_idx = p.ch1_input_select as usize;
        let preview = input_options[input_idx].to_string();
        ComboBox::new(ui, "##ch1_input_select")
            .preview_value(&preview)
            .build(|| {
                for (idx, opt) in input_options.iter().enumerate() {
                    if ui.selectable_config(opt).selected(idx == input_idx).build() {
                        input_idx = idx;
                    }
                }
            });
        p.ch1_input_select = input_idx.clamp(0, 1) as i32;
        
        ui.separator();
        
        // Helper closure for OSC tooltips
        let osc_tooltip = |ui: &Ui, address: &str, value: Option<f32>| {
            if show_osc && ui.is_item_hovered() {
                let mut tooltip = format!("OSC: {}", address);
                if let Some(val) = value {
                    tooltip.push_str(&format!("\nValue: {:.3}", val));
                }
                ui.tooltip_text(tooltip);
            }
        };
        
        // Geometry section
        if CollapsingHeader::new("Geometry").default_open(true).build(ui) {
            Drag::new("X Displace##ch1").speed(0.01).range(-2.0, 2.0).build(ui, &mut p.ch1_x_displace);
            osc_tooltip(ui, "/block1/ch1/x_displace", Some(p.ch1_x_displace));
            
            Drag::new("Y Displace##ch1").speed(0.01).range(-2.0, 2.0).build(ui, &mut p.ch1_y_displace);
            osc_tooltip(ui, "/block1/ch1/y_displace", Some(p.ch1_y_displace));
            
            Drag::new("Z Displace##ch1").speed(0.01).range(0.0, 10.0).build(ui, &mut p.ch1_z_displace);
            osc_tooltip(ui, "/block1/ch1/z_displace", Some(p.ch1_z_displace));
            
            Drag::new("Rotate##ch1").speed(0.1).range(-360.0, 360.0).build(ui, &mut p.ch1_rotate);
            osc_tooltip(ui, "/block1/ch1/rotate", Some(p.ch1_rotate));
            
            // Kaleidoscope
            Drag::new("Kaleidoscope Amount##ch1").speed(0.01).range(0.0, 1.0).build(ui, &mut p.ch1_kaleidoscope_amount);
            osc_tooltip(ui, "/block1/ch1/kaleidoscope_amount", Some(p.ch1_kaleidoscope_amount));
            
            Drag::new("Kaleidoscope Slice##ch1").speed(0.01).range(0.0, 1.0).build(ui, &mut p.ch1_kaleidoscope_slice);
            osc_tooltip(ui, "/block1/ch1/kaleidoscope_slice", Some(p.ch1_kaleidoscope_slice));
            
            // Geo overflow
            let mut overflow_idx = p.ch1_geo_overflow as usize;
            let preview = GEO_OVERFLOW_MODES[overflow_idx].to_string();
            ComboBox::new(ui, "Overflow Mode##ch1")
                .preview_value(&preview)
                .build(|| {
                    for (idx, opt) in GEO_OVERFLOW_MODES.iter().enumerate() {
                        if ui.selectable_config(opt).selected(idx == overflow_idx).build() {
                            overflow_idx = idx;
                        }
                    }
                });
            p.ch1_geo_overflow = overflow_idx.clamp(0, GEO_OVERFLOW_MODES.len() - 1) as i32;
        }
        
        // Color section
        if CollapsingHeader::new("Color").default_open(true).build(ui) {
            let mut hsb = [p.ch1_hsb_attenuate.x, p.ch1_hsb_attenuate.y, p.ch1_hsb_attenuate.z];
            Drag::new("HSB Attenuate##ch1").speed(0.01).range(0.0, 2.0).build_array(ui, &mut hsb);
            p.ch1_hsb_attenuate = Vec3::new(hsb[0], hsb[1], hsb[2]);
            // Sync individual components for LFO modulation
            p.ch1_hsb_attenuate_x = hsb[0];
            p.ch1_hsb_attenuate_y = hsb[1];
            p.ch1_hsb_attenuate_z = hsb[2];
            
            ui.checkbox("Hue Invert##ch1", &mut p.ch1_hue_invert);
            ui.checkbox("Saturation Invert##ch1", &mut p.ch1_saturation_invert);
            ui.checkbox("Brightness Invert##ch1", &mut p.ch1_bright_invert);
            ui.checkbox("RGB Invert##ch1", &mut p.ch1_rgb_invert);
            
            ui.checkbox("Solarize##ch1", &mut p.ch1_solarize);
            ui.checkbox("Posterize##ch1", &mut p.ch1_posterize_switch);
            if p.ch1_posterize_switch {
                Drag::new("Posterize Levels##ch1").speed(0.1).range(2.0, 32.0).build(ui, &mut p.ch1_posterize);
            }
        }
        
        // Filters section
        if CollapsingHeader::new("Filters").default_open(true).build(ui) {
            Drag::new("Blur Amount##ch1").speed(0.01).range(0.0, 1.0).build(ui, &mut p.ch1_blur_amount);
            Drag::new("Blur Radius##ch1").speed(0.01).range(0.0, 5.0).build(ui, &mut p.ch1_blur_radius);
            
            Drag::new("Sharpen Amount##ch1").speed(0.01).range(0.0, 1.0).build(ui, &mut p.ch1_sharpen_amount);
            Drag::new("Sharpen Radius##ch1").speed(0.01).range(0.0, 5.0).build(ui, &mut p.ch1_sharpen_radius);
            
            Drag::new("Filters Boost##ch1").speed(0.01).range(0.0, 2.0).build(ui, &mut p.ch1_filters_boost);
        }
        
        // Switches section
        if CollapsingHeader::new("Switches").default_open(false).build(ui) {
            ui.checkbox("H Mirror##ch1", &mut p.ch1_h_mirror);
            ui.checkbox("V Mirror##ch1", &mut p.ch1_v_mirror);
            ui.checkbox("H Flip##ch1", &mut p.ch1_h_flip);
            ui.checkbox("V Flip##ch1", &mut p.ch1_v_flip);
            ui.checkbox("HD Aspect##ch1", &mut p.ch1_hd_aspect_on);
        }
    }
    
    /// Build Block 1 Channel 2 Mix & Key panel
    fn build_block1_ch2_mix_key(&mut self, ui: &Ui) {
        // Extract config values before borrowing self mutably
        let show_osc = self.config.show_osc_addresses;
        
        let p = &mut self.block1_edit;
        
        // Helper closure for OSC tooltips
        let osc_tooltip = |ui: &Ui, address: &str, value: Option<f32>| {
            if show_osc && ui.is_item_hovered() {
                let mut tooltip = format!("OSC: {}", address);
                if let Some(val) = value {
                    tooltip.push_str(&format!("\nValue: {:.3}", val));
                }
                ui.tooltip_text(tooltip);
            }
        };
        
        // Mix section
        if CollapsingHeader::new("Mix").default_open(true).build(ui) {
            // Mix amount with fine control (slower speed for precision near 0 and 1)
Drag::new("Mix Amount##ch2mix").speed(0.002).range(0.0, 1.0).build(ui, &mut p.ch2_mix_amount);
osc_tooltip(ui, "/block1/ch2/mix_amount", Some(p.ch2_mix_amount));
            
            let mut mix_type = p.ch2_mix_type as usize;
            let preview = MIX_TYPES[mix_type].to_string();
            ComboBox::new(ui, "Mix Type##ch2")
                .preview_value(&preview)
                .build(|| {
                    for (idx, opt) in MIX_TYPES.iter().enumerate() {
                        if ui.selectable_config(opt).selected(idx == mix_type).build() {
                            mix_type = idx;
                        }
                    }
                });
            p.ch2_mix_type = mix_type.clamp(0, MIX_TYPES.len() - 1) as i32;
            
            let mut overflow = p.ch2_mix_overflow as usize;
            let preview = GEO_OVERFLOW_MODES[overflow].to_string();
            ComboBox::new(ui, "Mix Overflow##ch2")
                .preview_value(&preview)
                .build(|| {
                    for (idx, opt) in GEO_OVERFLOW_MODES.iter().enumerate() {
                        if ui.selectable_config(opt).selected(idx == overflow).build() {
                            overflow = idx;
                        }
                    }
                });
            p.ch2_mix_overflow = overflow.clamp(0, GEO_OVERFLOW_MODES.len() - 1) as i32;
        }
        
        // Key section - OF-style (always active)
        if CollapsingHeader::new("Key").default_open(true).build(ui) {
            // Key mode selector (OF: 0=lumakey, 1=chromakey)
            let mut key_mode = (p.ch2_key_mode.clamp(0, 1)) as usize;
            let key_modes = ["Lumakey", "Chromakey"];
            let preview = key_modes[key_mode].to_string();
            ComboBox::new(ui, "Key Mode##ch2")
                .preview_value(&preview)
                .build(|| {
                    for (idx, opt) in key_modes.iter().enumerate() {
                        if ui.selectable_config(*opt).selected(idx == key_mode).build() {
                            key_mode = idx;
                        }
                    }
                });
            p.ch2_key_mode = key_mode as i32;
            
            // Key value (OF uses -1.0 to 1.0 range)
            let mut key_color = [p.ch2_key_value_red, p.ch2_key_value_green, p.ch2_key_value_blue];
            
            if p.ch2_key_mode == 0 {
                // Lumakey mode - single slider controls all channels
                ui.text("Key Value:");
                Drag::new("Key Value##ch2").speed(0.01).range(-1.0, 1.0).build(ui, &mut key_color[0]);
                key_color[1] = key_color[0];
                key_color[2] = key_color[0];
            } else {
                // Chromakey mode - RGB sliders
                ui.text("Key Color (RGB -1 to 1):");
                Drag::new("Red##ch2key").speed(0.01).range(-1.0, 1.0).build(ui, &mut key_color[0]);
                Drag::new("Green##ch2key").speed(0.01).range(-1.0, 1.0).build(ui, &mut key_color[1]);
                Drag::new("Blue##ch2key").speed(0.01).range(-1.0, 1.0).build(ui, &mut key_color[2]);
            }
            
            // Color preview (mapped to 0-1 for display)
            let preview_color = [
                (key_color[0] + 1.0) * 0.5,
                (key_color[1] + 1.0) * 0.5,
                (key_color[2] + 1.0) * 0.5,
                1.0, // Alpha
            ];
            ui.color_button("Key Color##ch2preview", preview_color);
            ui.same_line();
            ui.text("preview");
            
            p.ch2_key_value_red = key_color[0];
            p.ch2_key_value_green = key_color[1];
            p.ch2_key_value_blue = key_color[2];
            
            ui.separator();
            
            // Key Order dropdown
            let key_orders = ["Key First, Then Mix", "Mix First, Then Key"];
            let mut order_idx = p.ch2_key_order as usize;
            let preview = key_orders[order_idx.clamp(0, key_orders.len() - 1)].to_string();
            ComboBox::new(ui, "Key Order##ch2")
                .preview_value(&preview)
                .build(|| {
                    for (idx, opt) in key_orders.iter().enumerate() {
                        if ui.selectable_config(*opt).selected(idx == order_idx).build() {
                            order_idx = idx;
                        }
                    }
                });
            p.ch2_key_order = order_idx.clamp(0, key_orders.len() - 1) as i32;
            
            // Key threshold and soft (OF uses -1.0 to 1.0)
            ui.text("Key Parameters:");
            Drag::new("Threshold##ch2").speed(0.01).range(-1.0, 1.0).build(ui, &mut p.ch2_key_threshold);
            osc_tooltip(ui, "/block1/ch2/key_threshold", Some(p.ch2_key_threshold));
            Drag::new("Soft##ch2").speed(0.01).range(-1.0, 1.0).build(ui, &mut p.ch2_key_soft);
            osc_tooltip(ui, "/block1/ch2/key_soft", Some(p.ch2_key_soft));
        }
    }
    
    /// Build Block 1 Channel 2 Adjust panel
    fn build_block1_ch2_adjust(&mut self, ui: &Ui) {
        // Extract config values before borrowing self mutably
        let show_osc = self.config.show_osc_addresses;
        
        let p = &mut self.block1_edit;
        
        // Helper closure for OSC tooltips
        let osc_tooltip = |ui: &Ui, address: &str, value: Option<f32>| {
            if show_osc && ui.is_item_hovered() {
                let mut tooltip = format!("OSC: {}", address);
                if let Some(val) = value {
                    tooltip.push_str(&format!("\nValue: {:.3}", val));
                }
                ui.tooltip_text(tooltip);
            }
        };
        
        // Input selection (moved here to match CH1 layout)
        ui.text("Input Source:");
        ui.same_line();
        let input_options = ["Input 1", "Input 2"];
        let mut input_idx = p.ch2_input_select as usize;
        let preview = input_options[input_idx].to_string();
        ComboBox::new(ui, "##ch2_input_select")
            .preview_value(&preview)
            .build(|| {
                for (idx, opt) in input_options.iter().enumerate() {
                    if ui.selectable_config(opt).selected(idx == input_idx).build() {
                        input_idx = idx;
                    }
                }
            });
        p.ch2_input_select = input_idx.clamp(0, 1) as i32;
        
        ui.separator();
        
        // Geometry section
        if CollapsingHeader::new("Geometry").default_open(true).build(ui) {
            Drag::new("X Displace##ch2adj").speed(0.01).range(-2.0, 2.0).build(ui, &mut p.ch2_x_displace);
            osc_tooltip(ui, "/block1/ch2/x_displace", Some(p.ch2_x_displace));
            
            Drag::new("Y Displace##ch2adj").speed(0.01).range(-2.0, 2.0).build(ui, &mut p.ch2_y_displace);
            osc_tooltip(ui, "/block1/ch2/y_displace", Some(p.ch2_y_displace));
            
            Drag::new("Z Displace##ch2adj").speed(0.01).range(0.0, 10.0).build(ui, &mut p.ch2_z_displace);
            osc_tooltip(ui, "/block1/ch2/z_displace", Some(p.ch2_z_displace));
            
            Drag::new("Rotate##ch2adj").speed(0.1).range(-360.0, 360.0).build(ui, &mut p.ch2_rotate);
            osc_tooltip(ui, "/block1/ch2/rotate", Some(p.ch2_rotate));
            
            Drag::new("Kaleidoscope Amount##ch2adj").speed(0.01).range(0.0, 1.0).build(ui, &mut p.ch2_kaleidoscope_amount);
            osc_tooltip(ui, "/block1/ch2/kaleidoscope_amount", Some(p.ch2_kaleidoscope_amount));
            
            Drag::new("Kaleidoscope Slice##ch2adj").speed(0.01).range(0.0, 1.0).build(ui, &mut p.ch2_kaleidoscope_slice);
            osc_tooltip(ui, "/block1/ch2/kaleidoscope_slice", Some(p.ch2_kaleidoscope_slice));
            
            let mut overflow_idx = p.ch2_geo_overflow as usize;
            let preview = GEO_OVERFLOW_MODES[overflow_idx].to_string();
            ComboBox::new(ui, "Overflow Mode##ch2adj")
                .preview_value(&preview)
                .build(|| {
                    for (idx, opt) in GEO_OVERFLOW_MODES.iter().enumerate() {
                        if ui.selectable_config(opt).selected(idx == overflow_idx).build() {
                            overflow_idx = idx;
                        }
                    }
                });
            p.ch2_geo_overflow = overflow_idx.clamp(0, GEO_OVERFLOW_MODES.len() - 1) as i32;
        }
        
        // Color section
        if CollapsingHeader::new("Color").default_open(true).build(ui) {
            let mut hsb = [p.ch2_hsb_attenuate.x, p.ch2_hsb_attenuate.y, p.ch2_hsb_attenuate.z];
            Drag::new("HSB Attenuate##ch2adj").speed(0.01).range(0.0, 2.0).build_array(ui, &mut hsb);
            p.ch2_hsb_attenuate = Vec3::new(hsb[0], hsb[1], hsb[2]);
            // Sync individual components for LFO modulation
            p.ch2_hsb_attenuate_x = hsb[0];
            p.ch2_hsb_attenuate_y = hsb[1];
            p.ch2_hsb_attenuate_z = hsb[2];
            osc_tooltip(ui, "/block1/ch2/hsb_attenuate", Some(hsb[0]));
            
            ui.checkbox("Hue Invert##ch2adj", &mut p.ch2_hue_invert);
            ui.checkbox("Saturation Invert##ch2adj", &mut p.ch2_saturation_invert);
            ui.checkbox("Brightness Invert##ch2adj", &mut p.ch2_bright_invert);
            ui.checkbox("RGB Invert##ch2adj", &mut p.ch2_rgb_invert);
            
            ui.checkbox("Solarize##ch2adj", &mut p.ch2_solarize);
            ui.checkbox("Posterize##ch2adj", &mut p.ch2_posterize_switch);
            if p.ch2_posterize_switch {
                Drag::new("Posterize Levels##ch2adj").speed(0.1).range(2.0, 32.0).build(ui, &mut p.ch2_posterize);
                osc_tooltip(ui, "/block1/ch2/posterize", Some(p.ch2_posterize));
            }
        }
        
        // Filters section
        if CollapsingHeader::new("Filters").default_open(true).build(ui) {
            Drag::new("Blur Amount##ch2adj").speed(0.01).range(0.0, 1.0).build(ui, &mut p.ch2_blur_amount);
            osc_tooltip(ui, "/block1/ch2/blur_amount", Some(p.ch2_blur_amount));
            
            Drag::new("Blur Radius##ch2adj").speed(0.01).range(0.0, 5.0).build(ui, &mut p.ch2_blur_radius);
            osc_tooltip(ui, "/block1/ch2/blur_radius", Some(p.ch2_blur_radius));
            
            Drag::new("Sharpen Amount##ch2adj").speed(0.01).range(0.0, 1.0).build(ui, &mut p.ch2_sharpen_amount);
            osc_tooltip(ui, "/block1/ch2/sharpen_amount", Some(p.ch2_sharpen_amount));
            
            Drag::new("Sharpen Radius##ch2adj").speed(0.01).range(0.0, 5.0).build(ui, &mut p.ch2_sharpen_radius);
            osc_tooltip(ui, "/block1/ch2/sharpen_radius", Some(p.ch2_sharpen_radius));
            
            Drag::new("Filters Boost##ch2adj").speed(0.01).range(0.0, 2.0).build(ui, &mut p.ch2_filters_boost);
            osc_tooltip(ui, "/block1/ch2/filters_boost", Some(p.ch2_filters_boost));
        }
    }
    
    /// Build Block 1 FB1 Parameters panel
    fn build_block1_fb1_params(&mut self, ui: &Ui) {
        // Extract config values before borrowing self mutably
        let show_osc = self.config.show_osc_addresses;
        
        let p = &mut self.block1_edit;
        
        // Helper closure for OSC tooltips
        let osc_tooltip = |ui: &Ui, address: &str, value: Option<f32>| {
            if show_osc && ui.is_item_hovered() {
                let mut tooltip = format!("OSC: {}", address);
                if let Some(val) = value {
                    tooltip.push_str(&format!("\nValue: {:.3}", val));
                }
                ui.tooltip_text(tooltip);
            }
        };
        
        // Mix section
        if CollapsingHeader::new("Feedback Mix").default_open(true).build(ui) {
            // Mix amount with fine control
Drag::new("Mix Amount##fb1").speed(0.002).range(0.0, 1.0).build(ui, &mut p.fb1_mix_amount);
            
            let mut mix_type = p.fb1_mix_type as usize;
            let preview = MIX_TYPES[mix_type].to_string();
            ComboBox::new(ui, "Mix Type##fb1")
                .preview_value(&preview)
                .build(|| {
                    for (idx, opt) in MIX_TYPES.iter().enumerate() {
                        if ui.selectable_config(opt).selected(idx == mix_type).build() {
                            mix_type = idx;
                        }
                    }
                });
            p.fb1_mix_type = mix_type.clamp(0, MIX_TYPES.len() - 1) as i32;
            
            let mut overflow = p.fb1_mix_overflow as usize;
            let preview = GEO_OVERFLOW_MODES[overflow].to_string();
            ComboBox::new(ui, "Mix Overflow##fb1")
                .preview_value(&preview)
                .build(|| {
                    for (idx, opt) in GEO_OVERFLOW_MODES.iter().enumerate() {
                        if ui.selectable_config(opt).selected(idx == overflow).build() {
                            overflow = idx;
                        }
                    }
                });
            p.fb1_mix_overflow = overflow.clamp(0, GEO_OVERFLOW_MODES.len() - 1) as i32;
        }
        
        // FB1 Key section - OF-style (always active)
        if CollapsingHeader::new("Feedback Key").default_open(true).build(ui) {
            // Key mode selector (OF: 0=lumakey, 1=chromakey)
            let mut key_mode = (p.fb1_key_mode.clamp(0, 1)) as usize;
            let key_modes = ["Lumakey", "Chromakey"];
            let preview = key_modes[key_mode].to_string();
            ComboBox::new(ui, "Key Mode##fb1")
                .preview_value(&preview)
                .build(|| {
                    for (idx, opt) in key_modes.iter().enumerate() {
                        if ui.selectable_config(*opt).selected(idx == key_mode).build() {
                            key_mode = idx;
                        }
                    }
                });
            p.fb1_key_mode = key_mode as i32;
            
            // Key value (OF uses -1.0 to 1.0 range)
            let mut key_color = [p.fb1_key_value_red, p.fb1_key_value_green, p.fb1_key_value_blue];
            
            if p.fb1_key_mode == 0 {
                // Lumakey mode - single slider controls all channels
                ui.text("Key Value:");
                Drag::new("Key Value##fb1").speed(0.01).range(-1.0, 1.0).build(ui, &mut key_color[0]);
                key_color[1] = key_color[0];
                key_color[2] = key_color[0];
            } else {
                // Chromakey mode - RGB sliders
                ui.text("Key Color (RGB -1 to 1):");
                Drag::new("Red##fb1key").speed(0.01).range(-1.0, 1.0).build(ui, &mut key_color[0]);
                Drag::new("Green##fb1key").speed(0.01).range(-1.0, 1.0).build(ui, &mut key_color[1]);
                Drag::new("Blue##fb1key").speed(0.01).range(-1.0, 1.0).build(ui, &mut key_color[2]);
            }
            
            // Color preview (mapped to 0-1 for display)
            let preview_color = [
                (key_color[0] + 1.0) * 0.5,
                (key_color[1] + 1.0) * 0.5,
                (key_color[2] + 1.0) * 0.5,
                1.0, // Alpha
            ];
            ui.color_button("Key Color##fb1preview", preview_color);
            ui.same_line();
            ui.text("preview");
            
            p.fb1_key_value_red = key_color[0];
            p.fb1_key_value_green = key_color[1];
            p.fb1_key_value_blue = key_color[2];
            
            ui.separator();
            
            // Key Order dropdown
            let key_orders = ["Key First, Then Mix", "Mix First, Then Key"];
            let mut order_idx = p.fb1_key_order as usize;
            let preview = key_orders[order_idx.clamp(0, key_orders.len() - 1)].to_string();
            ComboBox::new(ui, "Key Order##fb1")
                .preview_value(&preview)
                .build(|| {
                    for (idx, opt) in key_orders.iter().enumerate() {
                        if ui.selectable_config(*opt).selected(idx == order_idx).build() {
                            order_idx = idx;
                        }
                    }
                });
            p.fb1_key_order = order_idx.clamp(0, key_orders.len() - 1) as i32;
            
            // Key threshold and soft (OF uses -1.0 to 1.0)
            ui.text("Key Parameters:");
            Drag::new("Threshold##fb1").speed(0.01).range(-1.0, 1.0).build(ui, &mut p.fb1_key_threshold);
            osc_tooltip(ui, "/block1/fb1/key_threshold", Some(p.fb1_key_threshold));
            Drag::new("Soft##fb1").speed(0.01).range(-1.0, 1.0).build(ui, &mut p.fb1_key_soft);
            osc_tooltip(ui, "/block1/fb1/key_soft", Some(p.fb1_key_soft));
        }
        
        // FB1 Geometry section
        if CollapsingHeader::new("Feedback Geometry").default_open(true).build(ui) {
            Drag::new("X Displace##fb1").speed(0.01).range(-2.0, 2.0).build(ui, &mut p.fb1_x_displace);
            osc_tooltip(ui, "/block1/fb1/x_displace", Some(p.fb1_x_displace));
            Drag::new("Y Displace##fb1").speed(0.01).range(-2.0, 2.0).build(ui, &mut p.fb1_y_displace);
            osc_tooltip(ui, "/block1/fb1/y_displace", Some(p.fb1_y_displace));
            Drag::new("Z Displace##fb1").speed(0.01).range(0.0, 10.0).build(ui, &mut p.fb1_z_displace);
            osc_tooltip(ui, "/block1/fb1/z_displace", Some(p.fb1_z_displace));
            Drag::new("Rotate##fb1").speed(0.1).range(-360.0, 360.0).build(ui, &mut p.fb1_rotate);
            osc_tooltip(ui, "/block1/fb1/rotate", Some(p.fb1_rotate));
            
            // Shear matrix
            let mut shear = [p.fb1_shear_matrix.x, p.fb1_shear_matrix.y, 
                            p.fb1_shear_matrix.z, p.fb1_shear_matrix.w];
            Drag::new("Shear Matrix##fb1").speed(0.01).range(-2.0, 2.0).build_array(ui, &mut shear);
            osc_tooltip(ui, "/block1/fb1/shear_matrix", Some(shear[0]));
            p.fb1_shear_matrix = Vec4::new(shear[0], shear[1], shear[2], shear[3]);
            // Sync individual components for LFO modulation
            p.fb1_shear_matrix_x = shear[0];
            p.fb1_shear_matrix_y = shear[1];
            p.fb1_shear_matrix_z = shear[2];
            p.fb1_shear_matrix_w = shear[3];
            
            Drag::new("Kaleidoscope Amount##fb1").speed(0.01).range(0.0, 1.0).build(ui, &mut p.fb1_kaleidoscope_amount);
            osc_tooltip(ui, "/block1/fb1/kaleidoscope_amount", Some(p.fb1_kaleidoscope_amount));
            Drag::new("Kaleidoscope Slice##fb1").speed(0.01).range(0.0, 1.0).build(ui, &mut p.fb1_kaleidoscope_slice);
            osc_tooltip(ui, "/block1/fb1/kaleidoscope_slice", Some(p.fb1_kaleidoscope_slice));
        }
        
        // FB1 Color section
        if CollapsingHeader::new("Feedback Color").default_open(true).build(ui) {
            let mut hsb_offset = [p.fb1_hsb_offset.x, p.fb1_hsb_offset.y, p.fb1_hsb_offset.z];
            Drag::new("HSB Offset##fb1").speed(0.01).range(-1.0, 1.0).build_array(ui, &mut hsb_offset);
            osc_tooltip(ui, "/block1/fb1/hsb_offset", Some(hsb_offset[0]));
            p.fb1_hsb_offset = Vec3::new(hsb_offset[0], hsb_offset[1], hsb_offset[2]);
            // Sync individual components for LFO modulation
            p.fb1_hsb_offset_x = hsb_offset[0];
            p.fb1_hsb_offset_y = hsb_offset[1];
            p.fb1_hsb_offset_z = hsb_offset[2];
            
            let mut hsb_att = [p.fb1_hsb_attenuate.x, p.fb1_hsb_attenuate.y, p.fb1_hsb_attenuate.z];
            Drag::new("HSB Attenuate##fb1").speed(0.01).range(0.0, 2.0).build_array(ui, &mut hsb_att);
            osc_tooltip(ui, "/block1/fb1/hsb_attenuate", Some(hsb_att[0]));
            p.fb1_hsb_attenuate = Vec3::new(hsb_att[0], hsb_att[1], hsb_att[2]);
            // Sync individual components for LFO modulation
            p.fb1_hsb_attenuate_x = hsb_att[0];
            p.fb1_hsb_attenuate_y = hsb_att[1];
            p.fb1_hsb_attenuate_z = hsb_att[2];
            
            let mut hsb_pow = [p.fb1_hsb_powmap.x, p.fb1_hsb_powmap.y, p.fb1_hsb_powmap.z];
            Drag::new("HSB PowMap##fb1").speed(0.01).range(0.0, 5.0).build_array(ui, &mut hsb_pow);
            osc_tooltip(ui, "/block1/fb1/hsb_powmap", Some(hsb_pow[0]));
            p.fb1_hsb_powmap = Vec3::new(hsb_pow[0], hsb_pow[1], hsb_pow[2]);
            
            Drag::new("Hue Shaper##fb1").speed(0.01).range(0.0, 2.0).build(ui, &mut p.fb1_hue_shaper);
            osc_tooltip(ui, "/block1/fb1/hue_shaper", Some(p.fb1_hue_shaper));
            
            ui.checkbox("Hue Invert##fb1", &mut p.fb1_hue_invert);
            ui.checkbox("Saturation Invert##fb1", &mut p.fb1_saturation_invert);
            ui.checkbox("Brightness Invert##fb1", &mut p.fb1_bright_invert);
        }
        
        // FB1 Filters section
        if CollapsingHeader::new("Feedback Filters").default_open(true).build(ui) {
            Drag::new("Blur Amount##fb1").speed(0.01).range(0.0, 1.0).build(ui, &mut p.fb1_blur_amount);
            osc_tooltip(ui, "/block1/fb1/blur_amount", Some(p.fb1_blur_amount));
            Drag::new("Blur Radius##fb1").speed(0.01).range(0.0, 5.0).build(ui, &mut p.fb1_blur_radius);
            osc_tooltip(ui, "/block1/fb1/blur_radius", Some(p.fb1_blur_radius));
            
            Drag::new("Sharpen Amount##fb1").speed(0.01).range(0.0, 1.0).build(ui, &mut p.fb1_sharpen_amount);
            osc_tooltip(ui, "/block1/fb1/sharpen_amount", Some(p.fb1_sharpen_amount));
            Drag::new("Sharpen Radius##fb1").speed(0.01).range(0.0, 5.0).build(ui, &mut p.fb1_sharpen_radius);
            osc_tooltip(ui, "/block1/fb1/sharpen_radius", Some(p.fb1_sharpen_radius));
            
            Drag::new("Temp Filter 1 Amount##fb1").speed(0.01).range(0.0, 1.0).build(ui, &mut p.fb1_temporal_filter1_amount);
            osc_tooltip(ui, "/block1/fb1/temp_filter1_amount", Some(p.fb1_temporal_filter1_amount));
            Drag::new("Temp Filter 1 Res##fb1").speed(0.01).range(0.0, 1.0).build(ui, &mut p.fb1_temporal_filter1_resonance);
            osc_tooltip(ui, "/block1/fb1/temp_filter1_res", Some(p.fb1_temporal_filter1_resonance));
            
            Drag::new("Temp Filter 2 Amount##fb1").speed(0.01).range(0.0, 1.0).build(ui, &mut p.fb1_temporal_filter2_amount);
            osc_tooltip(ui, "/block1/fb1/temp_filter2_amount", Some(p.fb1_temporal_filter2_amount));
            Drag::new("Temp Filter 2 Res##fb1").speed(0.01).range(0.0, 1.0).build(ui, &mut p.fb1_temporal_filter2_resonance);
            osc_tooltip(ui, "/block1/fb1/temp_filter2_res", Some(p.fb1_temporal_filter2_resonance));
            
            Drag::new("Filters Boost##fb1").speed(0.01).range(0.0, 2.0).build(ui, &mut p.fb1_filters_boost);
            osc_tooltip(ui, "/block1/fb1/filters_boost", Some(p.fb1_filters_boost));
            
            ui.separator();
            // Delay section with tempo sync
            ui.text("Feedback Delay");
            
            // Tempo sync toggle
            let mut sync_enabled = p.fb1_delay_time_sync;
            if ui.checkbox("Sync to BPM##fb1delay", &mut sync_enabled) {
                p.fb1_delay_time_sync = sync_enabled;
            }
            if ui.is_item_hovered() {
                ui.tooltip_text("Enable to sync delay time to BPM");
            }
            
            if p.fb1_delay_time_sync {
                // Show beat division dropdown when sync is enabled
                let beat_divisions = ["1/16", "1/8", "1/4", "1/2", "1", "2", "4", "8"];
                let mut div_idx = p.fb1_delay_time_division as usize;
                let preview = beat_divisions[div_idx.min(beat_divisions.len() - 1)].to_string();
                ComboBox::new(ui, "##fb1_delay_division")
                    .preview_value(&preview)
                    .build(|| {
                        for (idx, opt) in beat_divisions.iter().enumerate() {
                            if ui.selectable_config(opt).selected(idx == div_idx).build() {
                                div_idx = idx;
                            }
                        }
                    });
                p.fb1_delay_time_division = div_idx.clamp(0, beat_divisions.len() - 1) as i32;
                
                // Show calculated delay time
                let calculated_frames = crate::core::lfo_engine::calculate_delay_frames_from_tempo(
                    self.bpm, p.fb1_delay_time_division, 60.0
                );
                ui.text_disabled(format!("Calculated: {} frames (≈ {:.2}s at {} BPM)", 
                    calculated_frames, calculated_frames as f32 / 60.0, self.bpm as i32));
            } else {
                // Show frame slider when sync is disabled
                Drag::new("Delay Time (frames)##fb1").speed(1.0).range(0, 120).build(ui, &mut p.fb1_delay_time);
                osc_tooltip(ui, "/block1/fb1/delay_time", Some(p.fb1_delay_time as f32));
                if p.fb1_delay_time > 0 {
                    ui.text_disabled(format!("≈ {:.2} seconds at 60fps", p.fb1_delay_time as f32 / 60.0));
                } else {
                    ui.text_disabled("No delay (use immediate feedback)");
                }
            }
        }
    }
    
    /// Build Block 2 panel with sub-tabs
    fn build_block2_panel(&mut self, ui: &Ui) {
        let subtab_labels = ["Input Adjust", "FB2", "LFO"];
        let mut subtab_idx = self.block2_tab as usize;
        
        if let Some(_tab_bar) = ui.tab_bar("##block2_tabs") {
            for (idx, label) in subtab_labels.iter().enumerate() {
                let is_selected = idx == subtab_idx;
                
                if let Some(_tab) = ui.tab_item(label) {
                    if !is_selected {
                        subtab_idx = idx;
                        self.block2_tab = match idx {
                            0 => Block2Tab::InputAdjust,
                            1 => Block2Tab::Fb2Parameters,
                            2 => Block2Tab::Lfo,
                            _ => Block2Tab::InputAdjust,
                        };
                    }
                }
            }
        }
        
        match self.block2_tab {
            Block2Tab::InputAdjust => self.build_block2_input_adjust(ui),
            Block2Tab::Fb2Parameters => self.build_block2_fb2_params(ui),
            Block2Tab::Lfo => self.build_block2_lfo(ui),
        }
    }
    
    /// Build Block 2 Input Adjust panel
    fn build_block2_input_adjust(&mut self, ui: &Ui) {
        // Extract config values before borrowing self mutably
        let show_osc = self.config.show_osc_addresses;
        
        let p = &mut self.block2_edit;
        
        // Helper closure for OSC tooltips
        let osc_tooltip = |ui: &Ui, address: &str, value: Option<f32>| {
            if show_osc && ui.is_item_hovered() {
                let mut tooltip = format!("OSC: {}", address);
                if let Some(val) = value {
                    tooltip.push_str(&format!("\nValue: {:.3}", val));
                }
                ui.tooltip_text(tooltip);
            }
        };
        
        // Input selection
        ui.text("Input Source:");
        ui.same_line();
        let input_options = ["Block 1", "Input 1", "Input 2"];
        let mut input_idx = p.block2_input_select as usize;
        let preview = input_options[input_idx].to_string();
        ComboBox::new(ui, "##b2_input_select")
            .preview_value(&preview)
            .build(|| {
                for (idx, opt) in input_options.iter().enumerate() {
                    if ui.selectable_config(opt).selected(idx == input_idx).build() {
                        input_idx = idx;
                    }
                }
            });
        p.block2_input_select = input_idx.clamp(0, 2) as i32;
        
        ui.separator();
        
        // Geometry section
        if CollapsingHeader::new("Geometry").default_open(true).build(ui) {
            Drag::new("X Displace##b2in").speed(0.01).range(-2.0, 2.0).build(ui, &mut p.block2_input_x_displace);
            osc_tooltip(ui, "/block2/input/x_displace", Some(p.block2_input_x_displace));
            Drag::new("Y Displace##b2in").speed(0.01).range(-2.0, 2.0).build(ui, &mut p.block2_input_y_displace);
            osc_tooltip(ui, "/block2/input/y_displace", Some(p.block2_input_y_displace));
            Drag::new("Z Displace##b2in").speed(0.01).range(0.0, 10.0).build(ui, &mut p.block2_input_z_displace);
            osc_tooltip(ui, "/block2/input/z_displace", Some(p.block2_input_z_displace));
            Drag::new("Rotate##b2in").speed(0.1).range(-360.0, 360.0).build(ui, &mut p.block2_input_rotate);
            osc_tooltip(ui, "/block2/input/rotate", Some(p.block2_input_rotate));
            
            Drag::new("Kaleidoscope Amount##b2in").speed(0.01).range(0.0, 1.0).build(ui, &mut p.block2_input_kaleidoscope_amount);
            osc_tooltip(ui, "/block2/input/kaleidoscope_amount", Some(p.block2_input_kaleidoscope_amount));
            Drag::new("Kaleidoscope Slice##b2in").speed(0.01).range(0.0, 1.0).build(ui, &mut p.block2_input_kaleidoscope_slice);
            osc_tooltip(ui, "/block2/input/kaleidoscope_slice", Some(p.block2_input_kaleidoscope_slice));
            
            let mut overflow_idx = p.block2_input_geo_overflow as usize;
            let preview = GEO_OVERFLOW_MODES[overflow_idx].to_string();
            ComboBox::new(ui, "Overflow Mode##b2in")
                .preview_value(&preview)
                .build(|| {
                    for (idx, opt) in GEO_OVERFLOW_MODES.iter().enumerate() {
                        if ui.selectable_config(opt).selected(idx == overflow_idx).build() {
                            overflow_idx = idx;
                        }
                    }
                });
            p.block2_input_geo_overflow = overflow_idx.clamp(0, GEO_OVERFLOW_MODES.len() - 1) as i32;
        }
        
        // Color section
        if CollapsingHeader::new("Color").default_open(true).build(ui) {
            let mut hsb = [p.block2_input_hsb_attenuate.x, 
                          p.block2_input_hsb_attenuate.y, 
                          p.block2_input_hsb_attenuate.z];
            Drag::new("HSB Attenuate##b2in").speed(0.01).range(0.0, 2.0).build_array(ui, &mut hsb);
            osc_tooltip(ui, "/block2/input/hsb_attenuate", Some(hsb[0]));
            p.block2_input_hsb_attenuate = Vec3::new(hsb[0], hsb[1], hsb[2]);
            // Sync individual components for LFO modulation
            p.block2_input_hsb_attenuate_x = hsb[0];
            p.block2_input_hsb_attenuate_y = hsb[1];
            p.block2_input_hsb_attenuate_z = hsb[2];
            
            ui.checkbox("Hue Invert##b2in", &mut p.block2_input_hue_invert);
            ui.checkbox("Saturation Invert##b2in", &mut p.block2_input_saturation_invert);
            ui.checkbox("Brightness Invert##b2in", &mut p.block2_input_bright_invert);
            ui.checkbox("RGB Invert##b2in", &mut p.block2_input_rgb_invert);
            
            ui.checkbox("Solarize##b2in", &mut p.block2_input_solarize);
            ui.checkbox("Posterize##b2in", &mut p.block2_input_posterize_switch);
            if p.block2_input_posterize_switch {
                Drag::new("Posterize Levels##b2in").speed(0.1).range(2.0, 32.0).build(ui, &mut p.block2_input_posterize);
                osc_tooltip(ui, "/block2/input/posterize", Some(p.block2_input_posterize));
            }
        }
        
        // Filters section
        if CollapsingHeader::new("Filters").default_open(true).build(ui) {
            Drag::new("Blur Amount##b2in").speed(0.01).range(0.0, 1.0).build(ui, &mut p.block2_input_blur_amount);
            osc_tooltip(ui, "/block2/input/blur_amount", Some(p.block2_input_blur_amount));
            Drag::new("Blur Radius##b2in").speed(0.01).range(0.0, 5.0).build(ui, &mut p.block2_input_blur_radius);
            osc_tooltip(ui, "/block2/input/blur_radius", Some(p.block2_input_blur_radius));
            
            Drag::new("Sharpen Amount##b2in").speed(0.01).range(0.0, 1.0).build(ui, &mut p.block2_input_sharpen_amount);
            osc_tooltip(ui, "/block2/input/sharpen_amount", Some(p.block2_input_sharpen_amount));
            Drag::new("Sharpen Radius##b2in").speed(0.01).range(0.0, 5.0).build(ui, &mut p.block2_input_sharpen_radius);
            osc_tooltip(ui, "/block2/input/sharpen_radius", Some(p.block2_input_sharpen_radius));
            
            Drag::new("Filters Boost##b2in").speed(0.01).range(0.0, 2.0).build(ui, &mut p.block2_input_filters_boost);
            osc_tooltip(ui, "/block2/input/filters_boost", Some(p.block2_input_filters_boost));
        }
    }
    
    /// Build Block 2 FB2 Parameters panel
    fn build_block2_fb2_params(&mut self, ui: &Ui) {
        // Extract config values before borrowing self mutably
        let show_osc = self.config.show_osc_addresses;
        
        let p = &mut self.block2_edit;
        
        // Helper closure for OSC tooltips
        let osc_tooltip = |ui: &Ui, address: &str, value: Option<f32>| {
            if show_osc && ui.is_item_hovered() {
                let mut tooltip = format!("OSC: {}", address);
                if let Some(val) = value {
                    tooltip.push_str(&format!("\nValue: {:.3}", val));
                }
                ui.tooltip_text(tooltip);
            }
        };
        
        // Mix section
        if CollapsingHeader::new("Feedback Mix").default_open(true).build(ui) {
            // Mix amount with fine control
            Drag::new("Mix Amount##fb2").speed(0.002).range(0.0, 1.0).build(ui, &mut p.fb2_mix_amount);
            osc_tooltip(ui, "/block2/fb2/mix_amount", Some(p.fb2_mix_amount));
            
            let mut mix_type = p.fb2_mix_type as usize;
            let preview = MIX_TYPES[mix_type].to_string();
            ComboBox::new(ui, "Mix Type##fb2")
                .preview_value(&preview)
                .build(|| {
                    for (idx, opt) in MIX_TYPES.iter().enumerate() {
                        if ui.selectable_config(opt).selected(idx == mix_type).build() {
                            mix_type = idx;
                        }
                    }
                });
            p.fb2_mix_type = mix_type.clamp(0, MIX_TYPES.len() - 1) as i32;
            
            let mut overflow = p.fb2_mix_overflow as usize;
            let preview = GEO_OVERFLOW_MODES[overflow].to_string();
            ComboBox::new(ui, "Mix Overflow##fb2")
                .preview_value(&preview)
                .build(|| {
                    for (idx, opt) in GEO_OVERFLOW_MODES.iter().enumerate() {
                        if ui.selectable_config(opt).selected(idx == overflow).build() {
                            overflow = idx;
                        }
                    }
                });
            p.fb2_mix_overflow = overflow.clamp(0, GEO_OVERFLOW_MODES.len() - 1) as i32;
        }
        
        // Key section
        if CollapsingHeader::new("Feedback Key").default_open(true).build(ui) {
            // Key mode selector
            let mut key_mode = (p.fb2_key_mode.clamp(0, 1)) as usize;
            let key_modes = ["Lumakey", "Chromakey"];
            let preview = key_modes[key_mode].to_string();
            ComboBox::new(ui, "Key Mode##fb2")
                .preview_value(&preview)
                .build(|| {
                    for (idx, opt) in key_modes.iter().enumerate() {
                        if ui.selectable_config(*opt).selected(idx == key_mode).build() {
                            key_mode = idx;
                        }
                    }
                });
            p.fb2_key_mode = key_mode as i32;
            
            let mut key_color = [p.fb2_key_value.x, p.fb2_key_value.y, p.fb2_key_value.z];
            
            if p.fb2_key_mode == 0 {
                // Lumakey mode - single slider controls all channels
                ui.text("Key Value:");
                Drag::new("Key Value##fb2").speed(0.01).range(-1.0, 1.0).build(ui, &mut key_color[0]);
                key_color[1] = key_color[0];
                key_color[2] = key_color[0];
            } else {
                // Chromakey mode - RGB sliders
                ui.color_edit3("Key Color##fb2", &mut key_color);
            }
            
            // Color preview (mapped to 0-1 for display)
            let preview_color = [
                (key_color[0] + 1.0) * 0.5,
                (key_color[1] + 1.0) * 0.5,
                (key_color[2] + 1.0) * 0.5,
                1.0, // Alpha
            ];
            ui.color_button("Key Color##fb2preview", preview_color);
            ui.same_line();
            ui.text("preview");
            
            p.fb2_key_value = Vec3::new(key_color[0], key_color[1], key_color[2]);
            
            ui.separator();
            
            // Key Order dropdown
            let key_orders = ["Key First, Then Mix", "Mix First, Then Key"];
            let mut order_idx = p.fb2_key_order as usize;
            let preview = key_orders[order_idx.clamp(0, key_orders.len() - 1)].to_string();
            ComboBox::new(ui, "Key Order##fb2")
                .preview_value(&preview)
                .build(|| {
                    for (idx, opt) in key_orders.iter().enumerate() {
                        if ui.selectable_config(*opt).selected(idx == order_idx).build() {
                            order_idx = idx;
                        }
                    }
                });
            p.fb2_key_order = order_idx.clamp(0, key_orders.len() - 1) as i32;
            
            // Key threshold with fine control
Drag::new("Key Threshold##fb2").speed(0.001).range(0.0, 1.0).build(ui, &mut p.fb2_key_threshold);
            Drag::new("Key Soft##fb2").speed(0.002).range(0.0, 1.0).build(ui, &mut p.fb2_key_soft);
        }
        
        // Geometry section
        if CollapsingHeader::new("Feedback Geometry").default_open(true).build(ui) {
            Drag::new("X Displace##fb2").speed(0.01).range(-2.0, 2.0).build(ui, &mut p.fb2_x_displace);
            osc_tooltip(ui, "/block2/fb2/x_displace", Some(p.fb2_x_displace));
            Drag::new("Y Displace##fb2").speed(0.01).range(-2.0, 2.0).build(ui, &mut p.fb2_y_displace);
            osc_tooltip(ui, "/block2/fb2/y_displace", Some(p.fb2_y_displace));
            Drag::new("Z Displace##fb2").speed(0.01).range(0.0, 10.0).build(ui, &mut p.fb2_z_displace);
            osc_tooltip(ui, "/block2/fb2/z_displace", Some(p.fb2_z_displace));
            Drag::new("Rotate##fb2").speed(0.1).range(-360.0, 360.0).build(ui, &mut p.fb2_rotate);
            osc_tooltip(ui, "/block2/fb2/rotate", Some(p.fb2_rotate));
            
            let mut shear = [p.fb2_shear_matrix.x, p.fb2_shear_matrix.y, 
                            p.fb2_shear_matrix.z, p.fb2_shear_matrix.w];
            Drag::new("Shear Matrix##fb2").speed(0.01).range(-2.0, 2.0).build_array(ui, &mut shear);
            osc_tooltip(ui, "/block2/fb2/shear_matrix", Some(shear[0]));
            p.fb2_shear_matrix = Vec4::new(shear[0], shear[1], shear[2], shear[3]);
            // Sync individual components for LFO modulation
            p.fb2_shear_matrix_x = shear[0];
            p.fb2_shear_matrix_y = shear[1];
            p.fb2_shear_matrix_z = shear[2];
            p.fb2_shear_matrix_w = shear[3];
            
            Drag::new("Kaleidoscope Amount##fb2").speed(0.01).range(0.0, 1.0).build(ui, &mut p.fb2_kaleidoscope_amount);
            osc_tooltip(ui, "/block2/fb2/kaleidoscope_amount", Some(p.fb2_kaleidoscope_amount));
            Drag::new("Kaleidoscope Slice##fb2").speed(0.01).range(0.0, 1.0).build(ui, &mut p.fb2_kaleidoscope_slice);
            osc_tooltip(ui, "/block2/fb2/kaleidoscope_slice", Some(p.fb2_kaleidoscope_slice));
            
            ui.checkbox("H Mirror##fb2", &mut p.fb2_h_mirror);
            ui.checkbox("V Mirror##fb2", &mut p.fb2_v_mirror);
            ui.checkbox("H Flip##fb2", &mut p.fb2_h_flip);
            ui.checkbox("V Flip##fb2", &mut p.fb2_v_flip);
        }
        
        // Color section
        if CollapsingHeader::new("Feedback Color").default_open(true).build(ui) {
            let mut hsb_offset = [p.fb2_hsb_offset.x, p.fb2_hsb_offset.y, p.fb2_hsb_offset.z];
            Drag::new("HSB Offset##fb2").speed(0.01).range(-1.0, 1.0).build_array(ui, &mut hsb_offset);
            osc_tooltip(ui, "/block2/fb2/hsb_offset", Some(hsb_offset[0]));
            p.fb2_hsb_offset = Vec3::new(hsb_offset[0], hsb_offset[1], hsb_offset[2]);
            // Sync individual components for LFO modulation
            p.fb2_hsb_offset_x = hsb_offset[0];
            p.fb2_hsb_offset_y = hsb_offset[1];
            p.fb2_hsb_offset_z = hsb_offset[2];
            
            let mut hsb_att = [p.fb2_hsb_attenuate.x, p.fb2_hsb_attenuate.y, p.fb2_hsb_attenuate.z];
            Drag::new("HSB Attenuate##fb2").speed(0.01).range(0.0, 2.0).build_array(ui, &mut hsb_att);
            osc_tooltip(ui, "/block2/fb2/hsb_attenuate", Some(hsb_att[0]));
            p.fb2_hsb_attenuate = Vec3::new(hsb_att[0], hsb_att[1], hsb_att[2]);
            // Sync individual components for LFO modulation
            p.fb2_hsb_attenuate_x = hsb_att[0];
            p.fb2_hsb_attenuate_y = hsb_att[1];
            p.fb2_hsb_attenuate_z = hsb_att[2];
            
            let mut hsb_pow = [p.fb2_hsb_powmap.x, p.fb2_hsb_powmap.y, p.fb2_hsb_powmap.z];
            Drag::new("HSB PowMap##fb2").speed(0.01).range(0.0, 5.0).build_array(ui, &mut hsb_pow);
            osc_tooltip(ui, "/block2/fb2/hsb_powmap", Some(hsb_pow[0]));
            p.fb2_hsb_powmap = Vec3::new(hsb_pow[0], hsb_pow[1], hsb_pow[2]);
            
            Drag::new("Hue Shaper##fb2").speed(0.01).range(0.0, 2.0).build(ui, &mut p.fb2_hue_shaper);
            osc_tooltip(ui, "/block2/fb2/hue_shaper", Some(p.fb2_hue_shaper));
            
            ui.checkbox("Hue Invert##fb2", &mut p.fb2_hue_invert);
            ui.checkbox("Saturation Invert##fb2", &mut p.fb2_saturation_invert);
            ui.checkbox("Brightness Invert##fb2", &mut p.fb2_bright_invert);
            ui.checkbox("RGB Invert##fb2", &mut p.fb2_rgb_invert);
        }
        
        // Filters section
        if CollapsingHeader::new("Feedback Filters").default_open(true).build(ui) {
            Drag::new("Blur Amount##fb2").speed(0.01).range(0.0, 1.0).build(ui, &mut p.fb2_blur_amount);
            osc_tooltip(ui, "/block2/fb2/blur_amount", Some(p.fb2_blur_amount));
            Drag::new("Blur Radius##fb2").speed(0.01).range(0.0, 5.0).build(ui, &mut p.fb2_blur_radius);
            osc_tooltip(ui, "/block2/fb2/blur_radius", Some(p.fb2_blur_radius));
            
            Drag::new("Sharpen Amount##fb2").speed(0.01).range(0.0, 1.0).build(ui, &mut p.fb2_sharpen_amount);
            osc_tooltip(ui, "/block2/fb2/sharpen_amount", Some(p.fb2_sharpen_amount));
            Drag::new("Sharpen Radius##fb2").speed(0.01).range(0.0, 5.0).build(ui, &mut p.fb2_sharpen_radius);
            osc_tooltip(ui, "/block2/fb2/sharpen_radius", Some(p.fb2_sharpen_radius));
            
            Drag::new("Filters Boost##fb2").speed(0.01).range(0.0, 2.0).build(ui, &mut p.fb2_filters_boost);
            osc_tooltip(ui, "/block2/fb2/filters_boost", Some(p.fb2_filters_boost));
            
            ui.separator();
            // Delay section with tempo sync
            ui.text("Feedback Delay");
            
            // Tempo sync toggle
            let mut sync_enabled = p.fb2_delay_time_sync;
            if ui.checkbox("Sync to BPM##fb2delay", &mut sync_enabled) {
                p.fb2_delay_time_sync = sync_enabled;
            }
            if ui.is_item_hovered() {
                ui.tooltip_text("Enable to sync delay time to BPM");
            }
            
            if p.fb2_delay_time_sync {
                // Show beat division dropdown when sync is enabled
                let beat_divisions = ["1/16", "1/8", "1/4", "1/2", "1", "2", "4", "8"];
                let mut div_idx = p.fb2_delay_time_division as usize;
                let preview = beat_divisions[div_idx.min(beat_divisions.len() - 1)].to_string();
                ComboBox::new(ui, "##fb2_delay_division")
                    .preview_value(&preview)
                    .build(|| {
                        for (idx, opt) in beat_divisions.iter().enumerate() {
                            if ui.selectable_config(opt).selected(idx == div_idx).build() {
                                div_idx = idx;
                            }
                        }
                    });
                p.fb2_delay_time_division = div_idx.clamp(0, beat_divisions.len() - 1) as i32;
                
                // Show calculated delay time
                let calculated_frames = crate::core::lfo_engine::calculate_delay_frames_from_tempo(
                    self.bpm, p.fb2_delay_time_division, 60.0
                );
                ui.text_disabled(format!("Calculated: {} frames (≈ {:.2}s at {} BPM)", 
                    calculated_frames, calculated_frames as f32 / 60.0, self.bpm as i32));
            } else {
                // Show frame slider when sync is disabled
                Drag::new("Delay Time (frames)##fb2").speed(1.0).range(0, 120).build(ui, &mut p.fb2_delay_time);
                if p.fb2_delay_time > 0 {
                    ui.text_disabled(format!("≈ {:.2} seconds at 60fps", p.fb2_delay_time as f32 / 60.0));
                } else {
                    ui.text_disabled("No delay (use immediate feedback)");
                }
            }
        }
    }
    
    /// Build Block 3 panel with sub-tabs
    fn build_block3_panel(&mut self, ui: &Ui) {
        let subtab_labels = ["Block 1 Re-process", "Block 2 Re-process", "Matrix Mixer", "Final Mix", "LFO"];
        let mut subtab_idx = self.block3_tab as usize;
        
        if let Some(_tab_bar) = ui.tab_bar("##block3_tabs") {
            for (idx, label) in subtab_labels.iter().enumerate() {
                let is_selected = idx == subtab_idx;
                
                if let Some(_tab) = ui.tab_item(label) {
                    if !is_selected {
                        subtab_idx = idx;
                        self.block3_tab = match idx {
                            0 => Block3Tab::Block1Reprocess,
                            1 => Block3Tab::Block2Reprocess,
                            2 => Block3Tab::MatrixMixer,
                            3 => Block3Tab::FinalMix,
                            4 => Block3Tab::Lfo,
                            _ => Block3Tab::FinalMix,
                        };
                    }
                }
            }
        }
        
        match self.block3_tab {
            Block3Tab::Block1Reprocess => self.build_block3_b1_reprocess(ui),
            Block3Tab::Block2Reprocess => self.build_block3_b2_reprocess(ui),
            Block3Tab::MatrixMixer => self.build_block3_matrix_mixer(ui),
            Block3Tab::FinalMix => self.build_block3_final_mix(ui),
            Block3Tab::Lfo => self.build_block3_lfo(ui),
        }
    }
    
    /// Build Block 3 Block 1 Re-process panel
    fn build_block3_b1_reprocess(&mut self, ui: &Ui) {
        let p = &mut self.block3_edit;
        
        // Geometry section
        if CollapsingHeader::new("Geometry").default_open(true).build(ui) {
            Drag::new("X Displace##b3b1").speed(0.01).range(-2.0, 2.0).build(ui, &mut p.block1_x_displace);
            Drag::new("Y Displace##b3b1").speed(0.01).range(-2.0, 2.0).build(ui, &mut p.block1_y_displace);
            Drag::new("Z Displace##b3b1").speed(0.01).range(0.0, 10.0).build(ui, &mut p.block1_z_displace);
            Drag::new("Rotate##b3b1").speed(0.1).range(-360.0, 360.0).build(ui, &mut p.block1_rotate);
            
            let mut shear = [p.block1_shear_matrix.x, p.block1_shear_matrix.y, 
                            p.block1_shear_matrix.z, p.block1_shear_matrix.w];
            Drag::new("Shear Matrix##b3b1").speed(0.01).range(-2.0, 2.0).build_array(ui, &mut shear);
            p.block1_shear_matrix = Vec4::new(shear[0], shear[1], shear[2], shear[3]);
            // Sync individual components for LFO modulation
            p.block1_shear_matrix_x = shear[0];
            p.block1_shear_matrix_y = shear[1];
            p.block1_shear_matrix_z = shear[2];
            p.block1_shear_matrix_w = shear[3];
            
            Drag::new("Kaleidoscope Amount##b3b1").speed(0.01).range(0.0, 1.0).build(ui, &mut p.block1_kaleidoscope_amount);
            Drag::new("Kaleidoscope Slice##b3b1").speed(0.01).range(0.0, 1.0).build(ui, &mut p.block1_kaleidoscope_slice);
            
            let mut overflow_idx = p.block1_geo_overflow as usize;
            let preview = GEO_OVERFLOW_MODES[overflow_idx].to_string();
            ComboBox::new(ui, "Overflow Mode##b3b1")
                .preview_value(&preview)
                .build(|| {
                    for (idx, opt) in GEO_OVERFLOW_MODES.iter().enumerate() {
                        if ui.selectable_config(opt).selected(idx == overflow_idx).build() {
                            overflow_idx = idx;
                        }
                    }
                });
            p.block1_geo_overflow = overflow_idx.clamp(0, GEO_OVERFLOW_MODES.len() - 1) as i32;
            
            ui.checkbox("H Mirror##b3b1", &mut p.block1_h_mirror);
            ui.checkbox("V Mirror##b3b1", &mut p.block1_v_mirror);
            ui.checkbox("H Flip##b3b1", &mut p.block1_h_flip);
            ui.checkbox("V Flip##b3b1", &mut p.block1_v_flip);
        }
        
        // Colorize section
        if CollapsingHeader::new("Colorize").default_open(true).build(ui) {
            ui.checkbox("Enable Colorize##b3b1", &mut p.block1_colorize_switch);
            
            if p.block1_colorize_switch {
                let colorize_modes = ["HSB", "RGB"];
                let mut mode_idx = p.block1_colorize_hsb_rgb as usize;
                let preview = colorize_modes[mode_idx].to_string();
                ComboBox::new(ui, "Colorize Mode##b3b1")
                    .preview_value(&preview)
                    .build(|| {
                        for (idx, opt) in colorize_modes.iter().enumerate() {
                            if ui.selectable_config(opt).selected(idx == mode_idx).build() {
                                mode_idx = idx;
                            }
                        }
                    });
                p.block1_colorize_hsb_rgb = mode_idx.clamp(0, 1) as i32;
                
                let mut band1 = [p.block1_colorize_band1.x, p.block1_colorize_band1.y, p.block1_colorize_band1.z];
                ui.color_edit3("Band 1##b3b1", &mut band1);
                p.block1_colorize_band1 = Vec3::new(band1[0], band1[1], band1[2]);
                // Sync individual components for LFO modulation
                p.block1_colorize_band1_x = band1[0];
                p.block1_colorize_band1_y = band1[1];
                p.block1_colorize_band1_z = band1[2];
                
                let mut band2 = [p.block1_colorize_band2.x, p.block1_colorize_band2.y, p.block1_colorize_band2.z];
                ui.color_edit3("Band 2##b3b1", &mut band2);
                p.block1_colorize_band2 = Vec3::new(band2[0], band2[1], band2[2]);
                p.block1_colorize_band2_x = band2[0];
                p.block1_colorize_band2_y = band2[1];
                p.block1_colorize_band2_z = band2[2];
                
                let mut band3 = [p.block1_colorize_band3.x, p.block1_colorize_band3.y, p.block1_colorize_band3.z];
                ui.color_edit3("Band 3##b3b1", &mut band3);
                p.block1_colorize_band3 = Vec3::new(band3[0], band3[1], band3[2]);
                p.block1_colorize_band3_x = band3[0];
                p.block1_colorize_band3_y = band3[1];
                p.block1_colorize_band3_z = band3[2];
                
                let mut band4 = [p.block1_colorize_band4.x, p.block1_colorize_band4.y, p.block1_colorize_band4.z];
                ui.color_edit3("Band 4##b3b1", &mut band4);
                p.block1_colorize_band4 = Vec3::new(band4[0], band4[1], band4[2]);
                p.block1_colorize_band4_x = band4[0];
                p.block1_colorize_band4_y = band4[1];
                p.block1_colorize_band4_z = band4[2];
                
                let mut band5 = [p.block1_colorize_band5.x, p.block1_colorize_band5.y, p.block1_colorize_band5.z];
                ui.color_edit3("Band 5##b3b1", &mut band5);
                p.block1_colorize_band5 = Vec3::new(band5[0], band5[1], band5[2]);
                p.block1_colorize_band5_x = band5[0];
                p.block1_colorize_band5_y = band5[1];
                p.block1_colorize_band5_z = band5[2];
            }
        }
        
        // Filters section
        if CollapsingHeader::new("Filters").default_open(true).build(ui) {
            Drag::new("Blur Amount##b3b1").speed(0.01).range(0.0, 1.0).build(ui, &mut p.block1_blur_amount);
            Drag::new("Blur Radius##b3b1").speed(0.01).range(0.0, 5.0).build(ui, &mut p.block1_blur_radius);
            
            Drag::new("Sharpen Amount##b3b1").speed(0.01).range(0.0, 1.0).build(ui, &mut p.block1_sharpen_amount);
            Drag::new("Sharpen Radius##b3b1").speed(0.01).range(0.0, 5.0).build(ui, &mut p.block1_sharpen_radius);
            
            Drag::new("Filters Boost##b3b1").speed(0.01).range(0.0, 2.0).build(ui, &mut p.block1_filters_boost);
            
            ui.checkbox("Dither##b3b1", &mut p.block1_dither_switch);
            if p.block1_dither_switch {
                Drag::new("Dither Amount##b3b1").speed(0.1).range(1.0, 64.0).build(ui, &mut p.block1_dither);
                let dither_types = ["4x4", "8x8"];
                let mut dither_idx = p.block1_dither_type as usize;
                let preview = dither_types[dither_idx].to_string();
                ComboBox::new(ui, "Dither Type##b3b1")
                    .preview_value(&preview)
                    .build(|| {
                        for (idx, opt) in dither_types.iter().enumerate() {
                            if ui.selectable_config(opt).selected(idx == dither_idx).build() {
                                dither_idx = idx;
                            }
                        }
                    });
                p.block1_dither_type = dither_idx.clamp(0, 1) as i32;
            }
        }
    }
    
    /// Build Block 3 Block 2 Re-process panel
    fn build_block3_b2_reprocess(&mut self, ui: &Ui) {
        let p = &mut self.block3_edit;
        
        // Geometry section
        if CollapsingHeader::new("Geometry").default_open(true).build(ui) {
            Drag::new("X Displace##b3b2").speed(0.01).range(-2.0, 2.0).build(ui, &mut p.block2_x_displace);
            Drag::new("Y Displace##b3b2").speed(0.01).range(-2.0, 2.0).build(ui, &mut p.block2_y_displace);
            Drag::new("Z Displace##b3b2").speed(0.01).range(0.0, 10.0).build(ui, &mut p.block2_z_displace);
            Drag::new("Rotate##b3b2").speed(0.1).range(-360.0, 360.0).build(ui, &mut p.block2_rotate);
            
            let mut shear = [p.block2_shear_matrix.x, p.block2_shear_matrix.y, 
                            p.block2_shear_matrix.z, p.block2_shear_matrix.w];
            Drag::new("Shear Matrix##b3b2").speed(0.01).range(-2.0, 2.0).build_array(ui, &mut shear);
            p.block2_shear_matrix = Vec4::new(shear[0], shear[1], shear[2], shear[3]);
            // Sync individual components for LFO modulation
            p.block2_shear_matrix_x = shear[0];
            p.block2_shear_matrix_y = shear[1];
            p.block2_shear_matrix_z = shear[2];
            p.block2_shear_matrix_w = shear[3];
            
            Drag::new("Kaleidoscope Amount##b3b2").speed(0.01).range(0.0, 1.0).build(ui, &mut p.block2_kaleidoscope_amount);
            Drag::new("Kaleidoscope Slice##b3b2").speed(0.01).range(0.0, 1.0).build(ui, &mut p.block2_kaleidoscope_slice);
            
            let mut overflow_idx = p.block2_geo_overflow as usize;
            let preview = GEO_OVERFLOW_MODES[overflow_idx].to_string();
            ComboBox::new(ui, "Overflow Mode##b3b2")
                .preview_value(&preview)
                .build(|| {
                    for (idx, opt) in GEO_OVERFLOW_MODES.iter().enumerate() {
                        if ui.selectable_config(opt).selected(idx == overflow_idx).build() {
                            overflow_idx = idx;
                        }
                    }
                });
            p.block2_geo_overflow = overflow_idx.clamp(0, GEO_OVERFLOW_MODES.len() - 1) as i32;
            
            ui.checkbox("H Mirror##b3b2", &mut p.block2_h_mirror);
            ui.checkbox("V Mirror##b3b2", &mut p.block2_v_mirror);
            ui.checkbox("H Flip##b3b2", &mut p.block2_h_flip);
            ui.checkbox("V Flip##b3b2", &mut p.block2_v_flip);
        }
        
        // Colorize section
        if CollapsingHeader::new("Colorize").default_open(true).build(ui) {
            ui.checkbox("Enable Colorize##b3b2", &mut p.block2_colorize_switch);
            
            if p.block2_colorize_switch {
                let colorize_modes = ["HSB", "RGB"];
                let mut mode_idx = p.block2_colorize_hsb_rgb as usize;
                let preview = colorize_modes[mode_idx].to_string();
                ComboBox::new(ui, "Colorize Mode##b3b2")
                    .preview_value(&preview)
                    .build(|| {
                        for (idx, opt) in colorize_modes.iter().enumerate() {
                            if ui.selectable_config(opt).selected(idx == mode_idx).build() {
                                mode_idx = idx;
                            }
                        }
                    });
                p.block2_colorize_hsb_rgb = mode_idx.clamp(0, 1) as i32;
                
                let mut band1 = [p.block2_colorize_band1.x, p.block2_colorize_band1.y, p.block2_colorize_band1.z];
                ui.color_edit3("Band 1##b3b2", &mut band1);
                p.block2_colorize_band1 = Vec3::new(band1[0], band1[1], band1[2]);
                // Sync individual components for LFO modulation
                p.block2_colorize_band1_x = band1[0];
                p.block2_colorize_band1_y = band1[1];
                p.block2_colorize_band1_z = band1[2];
                
                let mut band2 = [p.block2_colorize_band2.x, p.block2_colorize_band2.y, p.block2_colorize_band2.z];
                ui.color_edit3("Band 2##b3b2", &mut band2);
                p.block2_colorize_band2 = Vec3::new(band2[0], band2[1], band2[2]);
                p.block2_colorize_band2_x = band2[0];
                p.block2_colorize_band2_y = band2[1];
                p.block2_colorize_band2_z = band2[2];
                
                let mut band3 = [p.block2_colorize_band3.x, p.block2_colorize_band3.y, p.block2_colorize_band3.z];
                ui.color_edit3("Band 3##b3b2", &mut band3);
                p.block2_colorize_band3 = Vec3::new(band3[0], band3[1], band3[2]);
                p.block2_colorize_band3_x = band3[0];
                p.block2_colorize_band3_y = band3[1];
                p.block2_colorize_band3_z = band3[2];
                
                let mut band4 = [p.block2_colorize_band4.x, p.block2_colorize_band4.y, p.block2_colorize_band4.z];
                ui.color_edit3("Band 4##b3b2", &mut band4);
                p.block2_colorize_band4 = Vec3::new(band4[0], band4[1], band4[2]);
                p.block2_colorize_band4_x = band4[0];
                p.block2_colorize_band4_y = band4[1];
                p.block2_colorize_band4_z = band4[2];
                
                let mut band5 = [p.block2_colorize_band5.x, p.block2_colorize_band5.y, p.block2_colorize_band5.z];
                ui.color_edit3("Band 5##b3b2", &mut band5);
                p.block2_colorize_band5 = Vec3::new(band5[0], band5[1], band5[2]);
                p.block2_colorize_band5_x = band5[0];
                p.block2_colorize_band5_y = band5[1];
                p.block2_colorize_band5_z = band5[2];
            }
        }
        
        // Filters section
        if CollapsingHeader::new("Filters").default_open(true).build(ui) {
            Drag::new("Blur Amount##b3b2").speed(0.01).range(0.0, 1.0).build(ui, &mut p.block2_blur_amount);
            Drag::new("Blur Radius##b3b2").speed(0.01).range(0.0, 5.0).build(ui, &mut p.block2_blur_radius);
            
            Drag::new("Sharpen Amount##b3b2").speed(0.01).range(0.0, 1.0).build(ui, &mut p.block2_sharpen_amount);
            Drag::new("Sharpen Radius##b3b2").speed(0.01).range(0.0, 5.0).build(ui, &mut p.block2_sharpen_radius);
            
            Drag::new("Filters Boost##b3b2").speed(0.01).range(0.0, 2.0).build(ui, &mut p.block2_filters_boost);
            
            ui.checkbox("Dither##b3b2", &mut p.block2_dither_switch);
            if p.block2_dither_switch {
                Drag::new("Dither Amount##b3b2").speed(0.1).range(1.0, 64.0).build(ui, &mut p.block2_dither);
                let dither_types = ["4x4", "8x8"];
                let mut dither_idx = p.block2_dither_type as usize;
                let preview = dither_types[dither_idx].to_string();
                ComboBox::new(ui, "Dither Type##b3b2")
                    .preview_value(&preview)
                    .build(|| {
                        for (idx, opt) in dither_types.iter().enumerate() {
                            if ui.selectable_config(opt).selected(idx == dither_idx).build() {
                                dither_idx = idx;
                            }
                        }
                    });
                p.block2_dither_type = dither_idx.clamp(0, 1) as i32;
            }
        }
    }
    
    /// Build Block 3 Matrix Mixer panel
    fn build_block3_matrix_mixer(&mut self, ui: &Ui) {
        let p = &mut self.block3_edit;
        
        // Mix Order (which block is foreground/background)
        if CollapsingHeader::new("Mix Order").default_open(true).build(ui) {
            let order_labels = ["Block 1 → Block 2 (B1 FG, B2 BG)", "Block 2 → Block 1 (B2 FG, B1 BG)"];
            let mut order_idx = p.final_key_order as usize;
            
            ui.radio_button(&order_labels[0], &mut order_idx, 0);
            ui.radio_button(&order_labels[1], &mut order_idx, 1);
            
            p.final_key_order = order_idx.clamp(0, 1) as i32;
            
            ui.text_disabled("Changes which block is foreground/background in matrix mix");
        }
        
        ui.separator();
        ui.text("Matrix Mix Type:");
        let mut mix_type = p.matrix_mix_type as usize;
        let preview = MIX_TYPES[mix_type].to_string();
        ComboBox::new(ui, "##matrix_mix_type")
            .preview_value(&preview)
            .build(|| {
                for (idx, opt) in MIX_TYPES.iter().enumerate() {
                    if ui.selectable_config(opt).selected(idx == mix_type).build() {
                        mix_type = idx;
                    }
                }
            });
        p.matrix_mix_type = mix_type.clamp(0, MIX_TYPES.len() - 1) as i32;
        
        let mut overflow = p.matrix_mix_overflow as usize;
        let preview2 = GEO_OVERFLOW_MODES[overflow].to_string();
        ComboBox::new(ui, "##matrix_overflow")
            .preview_value(&preview2)
            .build(|| {
                for (idx, opt) in GEO_OVERFLOW_MODES.iter().enumerate() {
                    if ui.selectable_config(opt).selected(idx == overflow).build() {
                        overflow = idx;
                    }
                }
            });
        p.matrix_mix_overflow = overflow.clamp(0, GEO_OVERFLOW_MODES.len() - 1) as i32;
        
        ui.separator();
        ui.text("RGB Channel Mixing:");
        
        if CollapsingHeader::new("Background RGB into Foreground Red").default_open(true).build(ui) {
            let mut red_mix = [p.bg_rgb_into_fg_red.x, p.bg_rgb_into_fg_red.y, p.bg_rgb_into_fg_red.z];
            Drag::new("Mix##red").speed(0.002).range(-2.0, 2.0).build_array(ui, &mut red_mix);
            p.bg_rgb_into_fg_red = Vec3::new(red_mix[0], red_mix[1], red_mix[2]);
        }
        
        if CollapsingHeader::new("Background RGB into Foreground Green").default_open(true).build(ui) {
            let mut green_mix = [p.bg_rgb_into_fg_green.x, p.bg_rgb_into_fg_green.y, p.bg_rgb_into_fg_green.z];
            Drag::new("Mix##green").speed(0.002).range(-2.0, 2.0).build_array(ui, &mut green_mix);
            p.bg_rgb_into_fg_green = Vec3::new(green_mix[0], green_mix[1], green_mix[2]);
        }
        
        if CollapsingHeader::new("Background RGB into Foreground Blue").default_open(true).build(ui) {
            let mut blue_mix = [p.bg_rgb_into_fg_blue.x, p.bg_rgb_into_fg_blue.y, p.bg_rgb_into_fg_blue.z];
            Drag::new("Mix##blue").speed(0.002).range(-2.0, 2.0).build_array(ui, &mut blue_mix);
            p.bg_rgb_into_fg_blue = Vec3::new(blue_mix[0], blue_mix[1], blue_mix[2]);
        }
    }
    
    /// Build Block 3 Final Mix panel
    fn build_block3_final_mix(&mut self, ui: &Ui) {
        let p = &mut self.block3_edit;
        
        // Final mix section
        if CollapsingHeader::new("Final Mix").default_open(true).build(ui) {
            // Mix amount with fine control
Drag::new("Mix Amount##final").speed(0.002).range(0.0, 1.0).build(ui, &mut p.final_mix_amount);
            
            let mut mix_type = p.final_mix_type as usize;
            let preview = MIX_TYPES[mix_type].to_string();
            ComboBox::new(ui, "Mix Type##final")
                .preview_value(&preview)
                .build(|| {
                    for (idx, opt) in MIX_TYPES.iter().enumerate() {
                        if ui.selectable_config(opt).selected(idx == mix_type).build() {
                            mix_type = idx;
                        }
                    }
                });
            p.final_mix_type = mix_type.clamp(0, MIX_TYPES.len() - 1) as i32;
            
            let mut overflow = p.final_mix_overflow as usize;
            let preview2 = GEO_OVERFLOW_MODES[overflow].to_string();
            ComboBox::new(ui, "Mix Overflow##final")
                .preview_value(&preview2)
                .build(|| {
                    for (idx, opt) in GEO_OVERFLOW_MODES.iter().enumerate() {
                        if ui.selectable_config(opt).selected(idx == overflow).build() {
                            overflow = idx;
                        }
                    }
                });
            p.final_mix_overflow = overflow.clamp(0, GEO_OVERFLOW_MODES.len() - 1) as i32;
            
            Drag::new("Key Order##final").speed(1.0).range(0, 10).build(ui, &mut p.final_key_order);
        }
        
        // Final key section
        if CollapsingHeader::new("Final Key").default_open(true).build(ui) {
            let mut key_color = [p.final_key_value.x, p.final_key_value.y, p.final_key_value.z];
            ui.color_edit3("Key Color##final", &mut key_color);
            p.final_key_value = Vec3::new(key_color[0], key_color[1], key_color[2]);
            
            // Key threshold with fine control
Drag::new("Key Threshold##final").speed(0.001).range(0.0, 1.0).build(ui, &mut p.final_key_threshold);
            Drag::new("Key Soft##final").speed(0.002).range(0.0, 1.0).build(ui, &mut p.final_key_soft);
        }
    }
    
    /// Build Macros panel (LFO controls)
    fn build_macros_panel(&mut self, ui: &Ui) {
        ui.text("LFO Banks (0-15)");
        ui.separator();
        
        // LFO bank selector
        ui.text("Select LFO Bank:");
        for i in 0..16 {
            if i > 0 && i % 8 != 0 {
                ui.same_line();
            }
            let label = format!("{}", i);
            if ui.radio_button_bool(&label, self.selected_lfo_bank == i) {
                self.selected_lfo_bank = i;
            }
        }
        
        ui.separator();
        
        // Edit selected LFO bank
        let bank_idx = self.selected_lfo_bank as usize;
        if let Ok(mut state) = self.shared_state.lock() {
            if bank_idx < state.lfo_banks.len() {
                let lfo = &mut state.lfo_banks[bank_idx];
                
                Drag::new("Rate").speed(0.01).range(-1.0, 1.0).build(ui, &mut lfo.rate);
                Drag::new("Amplitude").speed(0.01).range(0.0, 2.0).build(ui, &mut lfo.amplitude);
                Drag::new("Phase").speed(0.01).range(0.0, 1.0).build(ui, &mut lfo.phase);
                
                let mut waveform = lfo.waveform as usize;
                let preview = WAVEFORM_NAMES[waveform].to_string();
                ComboBox::new(ui, "Waveform")
                    .preview_value(&preview)
                    .build(|| {
                        for (idx, opt) in WAVEFORM_NAMES.iter().enumerate() {
                            if ui.selectable_config(opt).selected(idx == waveform).build() {
                                waveform = idx;
                            }
                        }
                    });
                lfo.waveform = waveform.clamp(0, WAVEFORM_NAMES.len() - 1) as i32;
                
                ui.checkbox("Tempo Sync", &mut lfo.tempo_sync);
                
                if lfo.tempo_sync {
                    let mut division = lfo.division as usize;
                    let preview = BEAT_DIVISIONS[division].to_string();
                    ComboBox::new(ui, "Beat Division")
                        .preview_value(&preview)
                        .build(|| {
                            for (idx, opt) in BEAT_DIVISIONS.iter().enumerate() {
                                if ui.selectable_config(opt).selected(idx == division).build() {
                                    division = idx;
                                }
                            }
                        });
                    lfo.division = division.clamp(0, BEAT_DIVISIONS.len() - 1) as i32;
                }
            }
        }
    }
    
    /// Build Inputs panel
    fn build_inputs_panel(&mut self, ui: &Ui) {
        // Input 1 section
        if CollapsingHeader::new("Input 1").default_open(true).build(ui) {
            let input_types = ["None", "Webcam", "NDI", "Spout", "Video File"];
            let mut type_idx = self.input1_type as usize;
            let old_type = type_idx;
            let preview = input_types[type_idx].to_string();
            
            ComboBox::new(ui, "##input1_type")
                .preview_value(&preview)
                .build(|| {
                    for (idx, opt) in input_types.iter().enumerate() {
                        if ui.selectable_config(opt).selected(idx == type_idx).build() {
                            type_idx = idx;
                        }
                    }
                });
            
            let type_changed = type_idx != old_type;
            self.input1_type = match type_idx {
                0 => InputType::None,
                1 => InputType::Webcam,
                2 => InputType::Ndi,
                3 => InputType::Spout,
                4 => InputType::VideoFile,
                _ => InputType::None,
            };
            
            // Auto-select first webcam if Webcam is chosen but no device selected
            if type_changed && self.input1_type == InputType::Webcam 
                && self.selected_webcam1 < 0 && !self.webcam_devices.is_empty() {
                self.selected_webcam1 = 0;
            }
            
            // Save config when input type changes
            if type_changed {
                self.save_input_config();
            }
            
            // Webcam device selection
            if self.input1_type == InputType::Webcam {
                let devices: Vec<&str> = self.webcam_devices.iter().map(|s| s.as_str()).collect();
                if !devices.is_empty() {
                    let preview = if self.selected_webcam1 >= 0 { 
                        self.webcam_devices[self.selected_webcam1 as usize].clone()
                    } else { "Select device...".to_string() };
                    
                    // Check if selected device is a virtual camera
                    let is_virtual = self.selected_webcam1 >= 0 && 
                        self.webcam_devices[self.selected_webcam1 as usize].to_lowercase().contains("virtual");
                    
                    if is_virtual {
                        ui.text_colored([1.0, 0.5, 0.0, 1.0], 
                            "⚠️ Virtual cameras may not work on macOS.\nUse NDI from OBS instead.");
                    }
                    
                    let mut selected = self.selected_webcam1;
                    ComboBox::new(ui, "##webcam1_select")
                        .preview_value(&preview)
                        .build(|| {
                            for (idx, opt) in devices.iter().enumerate() {
                                if ui.selectable_config(opt).selected(idx == selected as usize).build() {
                                    selected = idx as i32;
                                }
                            }
                        });
                    let device_changed = self.selected_webcam1 != selected;
                    self.selected_webcam1 = selected;
                    
                    // Save config when device selection changes
                    if device_changed {
                        self.save_input_config();
                    }
                    
                    if ui.button("Start Webcam 1") && self.selected_webcam1 >= 0 {
                        if let Ok(mut state) = self.shared_state.lock() {
                            state.input1_change_request = InputChangeRequest::StartWebcam {
                                input_id: 1,
                                device_index: self.selected_webcam1 as usize,
                                width: 1280,
                                height: 720,
                                fps: 30,
                            };
                        }
                    }
                } else {
                    ui.text_disabled("No webcam devices found");
                }
            }
            
            // Stop button
            if ui.button("Stop Input 1") {
                if let Ok(mut state) = self.shared_state.lock() {
                    state.input1_change_request = InputChangeRequest::StopInput { input_id: 1 };
                }
            }
        }
        
        // Input 2 section
        if CollapsingHeader::new("Input 2").default_open(true).build(ui) {
            let input_types = ["None", "Webcam", "NDI", "Spout", "Video File"];
            let mut type_idx = self.input2_type as usize;
            let old_type = type_idx;
            let preview = input_types[type_idx].to_string();
            
            ComboBox::new(ui, "##input2_type")
                .preview_value(&preview)
                .build(|| {
                    for (idx, opt) in input_types.iter().enumerate() {
                        if ui.selectable_config(opt).selected(idx == type_idx).build() {
                            type_idx = idx;
                        }
                    }
                });
            
            let type_changed = type_idx != old_type;
            self.input2_type = match type_idx {
                0 => InputType::None,
                1 => InputType::Webcam,
                2 => InputType::Ndi,
                3 => InputType::Spout,
                4 => InputType::VideoFile,
                _ => InputType::None,
            };
            
            // Auto-select first webcam if Webcam is chosen but no device selected
            if type_changed && self.input2_type == InputType::Webcam 
                && self.selected_webcam2 < 0 && !self.webcam_devices.is_empty() {
                self.selected_webcam2 = 0;
            }
            
            // Save config when input type changes
            if type_changed {
                self.save_input_config();
            }
            
            // Webcam device selection
            if self.input2_type == InputType::Webcam {
                let devices: Vec<&str> = self.webcam_devices.iter().map(|s| s.as_str()).collect();
                if !devices.is_empty() {
                    let preview = if self.selected_webcam2 >= 0 { 
                        self.webcam_devices[self.selected_webcam2 as usize].clone()
                    } else { "Select device...".to_string() };
                    
                    // Check if selected device is a virtual camera
                    let is_virtual = self.selected_webcam2 >= 0 && 
                        self.webcam_devices[self.selected_webcam2 as usize].to_lowercase().contains("virtual");
                    
                    if is_virtual {
                        ui.text_colored([1.0, 0.5, 0.0, 1.0], 
                            "⚠️ Virtual cameras may not work on macOS.\nUse NDI from OBS instead.");
                    }
                    
                    let mut selected = self.selected_webcam2;
                    ComboBox::new(ui, "##webcam2_select")
                        .preview_value(&preview)
                        .build(|| {
                            for (idx, opt) in devices.iter().enumerate() {
                                if ui.selectable_config(opt).selected(idx == selected as usize).build() {
                                    selected = idx as i32;
                                }
                            }
                        });
                    let device_changed = self.selected_webcam2 != selected;
                    self.selected_webcam2 = selected;
                    
                    // Save config when device selection changes
                    if device_changed {
                        self.save_input_config();
                    }
                    
                    if ui.button("Start Webcam 2") && self.selected_webcam2 >= 0 {
                        if let Ok(mut state) = self.shared_state.lock() {
                            state.input2_change_request = InputChangeRequest::StartWebcam {
                                input_id: 2,
                                device_index: self.selected_webcam2 as usize,
                                width: 1280,
                                height: 720,
                                fps: 30,
                            };
                        }
                    }
                } else {
                    ui.text_disabled("No webcam devices found");
                }
            }
            
            // Stop button
            if ui.button("Stop Input 2") {
                if let Ok(mut state) = self.shared_state.lock() {
                    state.input2_change_request = InputChangeRequest::StopInput { input_id: 2 };
                }
            }
        }
        
        // Device refresh button
        ui.separator();
        if ui.button("Refresh Device List") {
            self.refresh_devices();
        }
        ui.same_line();
        ui.text(format!("Found {} webcam(s)", self.webcam_devices.len()));
        
        // Audio input section
        if CollapsingHeader::new("Audio Input").default_open(true).build(ui) {
            if let Ok(state) = self.shared_state.lock() {
                ui.text(format!("Volume: {:.3}", state.audio.volume));
                ui.text(format!("BPM: {:.1}", state.audio.bpm));
                
                // FFT visualization
                ui.text("FFT Bands:");
                for (i, val) in state.audio.fft.iter().enumerate().take(8) {
                    let bar_width = 200.0 * val.min(1.0);
                    ui.text(format!("Band {}: ", i));
                    ui.same_line();
                    let draw_list = ui.get_window_draw_list();
                    let pos = ui.cursor_screen_pos();
                    draw_list.add_rect(
                        [pos[0], pos[1]],
                        [pos[0] + bar_width, pos[1] + 10.0],
                        [0.0, 1.0, 0.0, 1.0],
                    ).filled(true).build();
                    ui.new_line();
                }
            }
        }
    }
    
    /// Build Settings panel
    fn build_settings_panel(&mut self, ui: &Ui) {
        // Performance stats at the top
        ui.text(format!("FPS: {:.1}", self.main_fps));
        ui.separator();
        
        // Output mode selection
        if CollapsingHeader::new("Output Mode").default_open(true).build(ui) {
            if let Ok(mut state) = self.shared_state.lock() {
                let mut selected_mode = match state.output_mode {
                    OutputMode::Block1 => 0,
                    OutputMode::Block2 => 1,
                    OutputMode::Block3 => 2,
                    OutputMode::PreviewInput1 => 3,
                    OutputMode::PreviewInput2 => 4,
                };
                let old_mode = selected_mode;
                
                ui.radio_button("Block 1##out", &mut selected_mode, 0);
                ui.radio_button("Block 2##out", &mut selected_mode, 1);
                ui.radio_button("Block 3##out", &mut selected_mode, 2);
                ui.radio_button("Preview Input 1##out", &mut selected_mode, 3);
                ui.radio_button("Preview Input 2##out", &mut selected_mode, 4);
                
                if selected_mode != old_mode {
                    state.output_mode = match selected_mode {
                        0 => OutputMode::Block1,
                        1 => OutputMode::Block2,
                        2 => OutputMode::Block3,
                        3 => OutputMode::PreviewInput1,
                        4 => OutputMode::PreviewInput2,
                        _ => OutputMode::Block3,
                    };
                }
            }
        }
        
        // Display info
        if CollapsingHeader::new("Display Info").default_open(true).build(ui) {
            if let Ok(state) = self.shared_state.lock() {
                ui.text(format!("Output Size: {}x{}", state.output_size.0, state.output_size.1));
                ui.text(format!("Internal Size: {}x{}", state.internal_size.0, state.internal_size.1));
                ui.text(format!("Frame Count: {}", state.frame_count));
            }
        }
        
        // UI Scale - Match oF version with discrete presets
        if CollapsingHeader::new("UI Scale").default_open(true).build(ui) {
            ui.text("Adjust UI scale for better visibility on high-DPI displays:");
            
            // oF-style scale presets: 100%, 150%, 200%, 250%, 300%
            let scale_presets = [1.0f32, 1.5, 2.0, 2.5, 3.0];
            let scale_labels = ["100%", "150%", "200%", "250%", "300%"];
            
            // Find current preset index (closest match)
            let current_scale = self.config.ui_scale;
            let mut selected_idx = scale_presets.iter()
                .enumerate()
                .min_by(|(_, a), (_, b)| {
                    let diff_a = (**a - current_scale).abs();
                    let diff_b = (**b - current_scale).abs();
                    diff_a.partial_cmp(&diff_b).unwrap()
                })
                .map(|(i, _)| i)
                .unwrap_or(2); // Default to 200%
            
            let preview = scale_labels[selected_idx];
            ComboBox::new(ui, "##ui_scale_combo")
                .preview_value(preview)
                .build(|| {
                    for (idx, label) in scale_labels.iter().enumerate() {
                        if ui.selectable_config(label).selected(idx == selected_idx).build() {
                            selected_idx = idx;
                        }
                    }
                });
            
            // Apply new scale if changed
            let new_scale = scale_presets[selected_idx];
            if new_scale != current_scale {
                self.config.ui_scale = new_scale;
                // Save config immediately
                if let Err(e) = self.config.save() {
                    log::warn!("Failed to save config: {}", e);
                }
                // Update shared state so engine can apply it at runtime
                if let Ok(mut state) = self.shared_state.lock() {
                    state.ui_scale = new_scale;
                }
            }
            
            ui.text_disabled("Scale changes apply immediately");
        }
        
        // OSC Address Display Toggle
        if CollapsingHeader::new("OSC Control").default_open(true).build(ui) {
            let mut show_osc = self.config.show_osc_addresses;
            if ui.checkbox("Show OSC addresses on hover", &mut show_osc) {
                self.config.show_osc_addresses = show_osc;
                // Save config
                if let Err(e) = self.config.save() {
                    log::warn!("Failed to save config: {}", e);
                }
            }
            if ui.is_item_hovered() {
                ui.tooltip_text("When enabled, hover over any parameter to see its OSC address for remote control");
            }
        }
        
        // Preview Window Settings
        if CollapsingHeader::new("Preview & Color Picker").default_open(true).build(ui) {
            ui.text("The Preview window allows you to:");
            ui.bullet_text("View output from any Block or Input");
            ui.bullet_text("Sample colors for keying");
            ui.bullet_text("Copy color values to clipboard");
            
            ui.separator();
            
            if ui.button("Open Preview Window") {
                self.show_preview_window = true;
                // Re-enable preview computation
                if let Ok(mut state) = self.shared_state.lock() {
                    state.preview_enabled = true;
                }
            }
            
            ui.text_disabled("Note: Color sampling requires GPU readback (not yet implemented)");
        }
        
        // Clear feedback button
        if ui.button("Clear Feedback") {
            if let Ok(mut state) = self.shared_state.lock() {
                state.clear_feedback = true;
            }
        }
        
        // Recording toggle
        if let Ok(mut state) = self.shared_state.lock() {
            let mut recording = state.is_recording;
            if ui.checkbox("Recording", &mut recording) {
                state.is_recording = recording;
            }
        }
        
        // Window Layout Management
        if CollapsingHeader::new("Window Layout").default_open(true).build(ui) {
            ui.text("Popped-out tabs: Click 'Pop Out' from tab context menu");
            ui.text("Indicator ⧉ shows which tabs are floating");
            
            if ui.button("Save Layout") {
                if let Err(e) = self.layout_config.save() {
                    self.show_status(&format!("Failed to save layout: {}", e));
                } else {
                    self.show_status("Layout saved!");
                }
            }
            ui.same_line();
            if ui.button("Reset Layout") {
                self.layout_config = LayoutConfig::default();
                if let Err(e) = self.layout_config.save() {
                    self.show_status(&format!("Failed to reset layout: {}", e));
                } else {
                    self.show_status("Layout reset!");
                }
            }
            
            // Show currently popped tabs
            if !self.layout_config.popped_tabs.is_empty() {
                ui.text("Currently floating:");
                for tab_id in self.layout_config.popped_tabs.keys() {
                    ui.bullet_text(tab_id.display_name());
                }
            }
        }
    }
    
    /// Draw Block 1 audio modulation panel
    fn draw_block1_audio_panel(&mut self, ui: &Ui) {
        ui.window("Audio Reactivity - Block 1")
            .size([400.0, 500.0], Condition::FirstUseEver)
            .build(|| {
                let param_names = get_block1_param_names();
                
                ui.text("Select Parameter:");
                let preview = param_names[self.selected_block1_param as usize].clone();
                let mut selected = self.selected_block1_param;
                ComboBox::new(ui, "##b1_param_select")
                    .preview_value(&preview)
                    .build(|| {
                        for (idx, name) in param_names.iter().enumerate() {
                            if ui.selectable_config(name).selected(idx == selected as usize).build() {
                                selected = idx as i32;
                            }
                        }
                    });
                self.selected_block1_param = selected;
                
                ui.separator();
                
                ui.checkbox("Enable Audio Mod", &mut self.audio_mod_enabled());
                Drag::new("FFT Band").speed(1.0).range(0, 15).build(ui, &mut self.audio_mod_fft_band);
                Drag::new("Modulation Amount").speed(0.01).range(0.0, 2.0).build(ui, &mut self.audio_mod_amount);
                
                if ui.button("Apply Modulation") {
                    // Apply to shared state
                }
                
                ui.separator();
                ui.text("Active Modulations:");
                // List active modulations
            });
    }
    
    /// Draw Block 2 audio modulation panel
    fn draw_block2_audio_panel(&mut self, ui: &Ui) {
        ui.window("Audio Reactivity - Block 2")
            .size([400.0, 500.0], Condition::FirstUseEver)
            .build(|| {
                let param_names = get_block2_param_names();
                
                ui.text("Select Parameter:");
                let preview = param_names[self.selected_block2_param as usize].clone();
                let mut selected = self.selected_block2_param;
                ComboBox::new(ui, "##b2_param_select")
                    .preview_value(&preview)
                    .build(|| {
                        for (idx, name) in param_names.iter().enumerate() {
                            if ui.selectable_config(name).selected(idx == selected as usize).build() {
                                selected = idx as i32;
                            }
                        }
                    });
                self.selected_block2_param = selected;
                
                ui.separator();
                
                ui.checkbox("Enable Audio Mod##b2", &mut self.audio_mod_enabled());
                Drag::new("FFT Band##b2").speed(1.0).range(0, 15).build(ui, &mut self.audio_mod_fft_band);
                Drag::new("Modulation Amount##b2").speed(0.01).range(0.0, 2.0).build(ui, &mut self.audio_mod_amount);
            });
    }
    
    /// Draw Block 3 audio modulation panel
    fn draw_block3_audio_panel(&mut self, ui: &Ui) {
        ui.window("Audio Reactivity - Block 3")
            .size([400.0, 500.0], Condition::FirstUseEver)
            .build(|| {
                let param_names = get_block3_param_names();
                
                ui.text("Select Parameter:");
                let preview = param_names[self.selected_block3_param as usize].clone();
                let mut selected = self.selected_block3_param;
                ComboBox::new(ui, "##b3_param_select")
                    .preview_value(&preview)
                    .build(|| {
                        for (idx, name) in param_names.iter().enumerate() {
                            if ui.selectable_config(name).selected(idx == selected as usize).build() {
                                selected = idx as i32;
                            }
                        }
                    });
                self.selected_block3_param = selected;
                
                ui.separator();
                
                ui.checkbox("Enable Audio Mod##b3", &mut self.audio_mod_enabled());
                Drag::new("FFT Band##b3").speed(1.0).range(0, 15).build(ui, &mut self.audio_mod_fft_band);
                Drag::new("Modulation Amount##b3").speed(0.01).range(0.0, 2.0).build(ui, &mut self.audio_mod_amount);
            });
    }
    
    /// Helper for audio mod enabled (placeholder)
    fn audio_mod_enabled(&mut self) -> bool {
        false // Placeholder
    }
    
    /// Draw Preview Window with color picker
    fn draw_preview_window(&mut self, ui: &Ui, frame_count: u64) {
        let sources = [
            PreviewSource::Block1,
            PreviewSource::Block2,
            PreviewSource::Block3,
            PreviewSource::Input1,
            PreviewSource::Input2,
        ];
        
        // Track window open state
        let mut is_open = self.show_preview_window;
        
        ui.window("Preview & Color Picker")
            .size([400.0, 350.0], Condition::FirstUseEver)
            .position([100.0, 100.0], Condition::FirstUseEver)
            .opened(&mut is_open)
            .build(|| {
                // Close button at top right
                if ui.button("X##close_preview") {
                    self.show_preview_window = false;
                    // Sync to shared state so engine stops computing
                    if let Ok(mut state) = self.shared_state.lock() {
                        state.preview_enabled = false;
                    }
                }
                
                ui.separator();
                // Get current preview source from shared state
                let mut preview_source = if let Ok(state) = self.shared_state.lock() {
                    state.preview_source
                } else {
                    crate::core::PreviewSource::Block3
                };
                
                // Source selection dropdown
                let preview_names: Vec<&str> = sources.iter().map(|s| s.display_name()).collect();
                let mut selected_idx = sources.iter().position(|&s| s == preview_source).unwrap_or(2);
                
                ui.text("Preview Source:");
                let preview = preview_names[selected_idx];
                ComboBox::new(ui, "##preview_source")
                    .preview_value(preview)
                    .build(|| {
                        for (idx, name) in preview_names.iter().enumerate() {
                            if ui.selectable_config(*name).selected(idx == selected_idx).build() {
                                selected_idx = idx;
                            }
                        }
                    });
                
                // Update shared state if changed
                let new_source = sources[selected_idx];
                if new_source != preview_source {
                    if let Ok(mut state) = self.shared_state.lock() {
                        state.preview_source = new_source;
                    }
                }
                
                ui.separator();
                
                // Preview image display
                ui.text("Preview Area:");
                
                // Calculate preview size maintaining 16:9 aspect ratio
                let available_width = ui.content_region_avail()[0];
                let preview_width = available_width.min(320.0);
                let preview_height = preview_width * (9.0 / 16.0);
                
                // Store image position and size for mouse picking
                let cursor_pos = ui.cursor_screen_pos();
                self.preview_image_pos = [cursor_pos[0], cursor_pos[1]];
                self.preview_image_size = [preview_width, preview_height];
                
                // Display the preview texture if available
                if let Some(texture_id) = self.preview_texture_id {
                    // Store values locally to avoid borrow issues
                    let image_pos = self.preview_image_pos;
                    let crosshair_uv = self.preview_crosshair_uv;
                    
                    // Draw the preview image
                    imgui::Image::new(texture_id, [preview_width, preview_height])
                        .uv0([0.0, 0.0])  // Top-left
                        .uv1([1.0, 1.0])  // Bottom-right
                        .build(ui);
                    
                    // Handle mouse click on preview for color picking
                    let mouse_pos = ui.io().mouse_pos;
                    let is_hovering = mouse_pos[0] >= image_pos[0] 
                        && mouse_pos[0] < image_pos[0] + preview_width
                        && mouse_pos[1] >= image_pos[1]
                        && mouse_pos[1] < image_pos[1] + preview_height;
                    
                    if is_hovering && ui.is_mouse_clicked(imgui::MouseButton::Left) {
                        // Calculate UV coordinates from mouse position
                        let u = (mouse_pos[0] - image_pos[0]) / preview_width;
                        let v = (mouse_pos[1] - image_pos[1]) / preview_height;
                        
                        // Store the crosshair position
                        self.preview_crosshair_uv = [u, v];
                        
                        // Store the pick position in shared state for the engine to process
                        if let Ok(mut state) = self.shared_state.lock() {
                            state.preview_pick_uv = [u, v];
                            state.preview_pick_requested = true;
                        }
                        
                        log::debug!("Color pick at UV: [{:.3}, {:.3}]", u, v);
                    }
                    
                    // Draw crosshair overlay at stored position
                    let draw_list = ui.get_window_draw_list();
                    let crosshair_x = image_pos[0] + preview_width * crosshair_uv[0];
                    let crosshair_y = image_pos[1] + preview_height * crosshair_uv[1];
                    let crosshair_color = 0xFF00FF00; // Green, ABGR format
                    let crosshair_size = 10.0;
                    
                    // Horizontal line
                    draw_list.add_line(
                        [crosshair_x - crosshair_size, crosshair_y],
                        [crosshair_x + crosshair_size, crosshair_y],
                        crosshair_color,
                    ).build();
                    
                    // Vertical line
                    draw_list.add_line(
                        [crosshair_x, crosshair_y - crosshair_size],
                        [crosshair_x, crosshair_y + crosshair_size],
                        crosshair_color,
                    ).build();
                    
                    // Small circle at center
                    draw_list.add_circle([crosshair_x, crosshair_y], 3.0, crosshair_color)
                        .filled(true)
                        .build();
                } else {
                    // Placeholder when texture not available
                    ui.child_window("PreviewImagePlaceholder")
                        .size([preview_width, preview_height])
                        .border(true)
                        .build(|| {
                            ui.text("Preview texture not available");
                            ui.text("(Restart may be required)");
                        });
                }
                
                ui.separator();
                
                // Color picker section
                ui.text("Color Picker:");
                ui.text_disabled("Click on preview to sample color");
                
                // Get sampled color from shared state
                let sampled_color = if let Ok(state) = self.shared_state.lock() {
                    state.preview_sampled_color
                } else {
                    [1.0, 1.0, 1.0]
                };
                
                // Display sampled color
                ui.color_button("Sampled Color", [sampled_color[0], 
                    sampled_color[1], sampled_color[2], 1.0]);
                ui.same_line();
                ui.text(format!("R: {:.2} G: {:.2} B: {:.2}", 
                    sampled_color[0],
                    sampled_color[1],
                    sampled_color[2]));
                
                // Color edit for manual adjustment
                let mut editable_color = sampled_color;
                if ui.color_edit3("Edit Color", &mut editable_color) {
                    // Update shared state with edited color
                    if let Ok(mut state) = self.shared_state.lock() {
                        state.preview_sampled_color = editable_color;
                    }
                }
                
                // Key target selection and apply
                let key_targets = [
                    "Block 1 - CH2 Key",
                    "Block 1 - FB1 Key",
                    "Block 2 - FB2 Key",
                    "Block 3 - Final Key",
                ];
                
                ui.text("Apply to Key:");
                
                // Target dropdown
                ComboBox::new(ui, "##key_target")
                    .preview_value(key_targets[self.selected_key_target as usize])
                    .build(|| {
                        for (idx, name) in key_targets.iter().enumerate() {
                            if ui.selectable_config(*name).selected(idx == self.selected_key_target as usize).build() {
                                self.selected_key_target = idx as i32;
                            }
                        }
                    });
                
                ui.same_line();
                
                // Apply button
                if ui.button("Apply") {
                    let color = if let Ok(state) = self.shared_state.lock() {
                        state.preview_sampled_color
                    } else {
                        [1.0, 1.0, 1.0]
                    };
                    
                    // Apply color to selected key target (both shared state AND local edit copy)
                    let status_msg = match self.selected_key_target {
                        0 => { // Block 1 - CH2 Key
                            // Update shared state
                            if let Ok(mut state) = self.shared_state.lock() {
                                state.block1.ch2_key_value_red = color[0];
                                state.block1.ch2_key_value_green = color[1];
                                state.block1.ch2_key_value_blue = color[2];
                            }
                            // Also update local edit copy so GUI shows the new value immediately
                            self.block1_edit.ch2_key_value_red = color[0];
                            self.block1_edit.ch2_key_value_green = color[1];
                            self.block1_edit.ch2_key_value_blue = color[2];
                            "Color applied to Block 1 CH2 Key"
                        }
                        1 => { // Block 1 - FB1 Key
                            if let Ok(mut state) = self.shared_state.lock() {
                                state.block1.fb1_key_value_red = color[0];
                                state.block1.fb1_key_value_green = color[1];
                                state.block1.fb1_key_value_blue = color[2];
                            }
                            self.block1_edit.fb1_key_value_red = color[0];
                            self.block1_edit.fb1_key_value_green = color[1];
                            self.block1_edit.fb1_key_value_blue = color[2];
                            "Color applied to Block 1 FB1 Key"
                        }
                        2 => { // Block 2 - FB2 Key
                            use glam::Vec3;
                            if let Ok(mut state) = self.shared_state.lock() {
                                state.block2.fb2_key_value = Vec3::new(color[0], color[1], color[2]);
                            }
                            self.block2_edit.fb2_key_value = Vec3::new(color[0], color[1], color[2]);
                            "Color applied to Block 2 FB2 Key"
                        }
                        3 => { // Block 3 - Final Key
                            use glam::Vec3;
                            if let Ok(mut state) = self.shared_state.lock() {
                                state.block3.final_key_value = Vec3::new(color[0], color[1], color[2]);
                            }
                            self.block3_edit.final_key_value = Vec3::new(color[0], color[1], color[2]);
                            "Color applied to Block 3 Final Key"
                        }
                        _ => ""
                    };
                    
                    if !status_msg.is_empty() {
                        self.show_status(status_msg);
                    }
                }
            });
        
        // Handle window close (user clicked X button or pressed Esc)
        if !is_open && self.show_preview_window {
            self.show_preview_window = false;
            // Disable preview computation to save GPU
            if let Ok(mut state) = self.shared_state.lock() {
                state.preview_enabled = false;
            }
        }
    }
    
    /// Build Block 1 LFO panel - Organized by OF LFO groups
    fn build_block1_lfo(&mut self, ui: &Ui) {
        // Global tempo control
        self.draw_tempo_control(ui);
        ui.separator();
        
        // === CH1 Adjust LFOs (16 params) ===
        if CollapsingHeader::new("CH1 Adjust LFOs").default_open(true).build(ui) {
            let ch1_params = [
                // X/Y/Z Displace <->
                ("ch1_x_displace", "X Displace"),
                ("ch1_y_displace", "Y Displace"),
                ("ch1_z_displace", "Z Displace"),
                // Rotate
                ("ch1_rotate", "Rotate"),
                // HSB Hue ^^
                ("ch1_hsb_attenuate_x", "HSB Hue"),
                // HSB Sat ^^
                ("ch1_hsb_attenuate_y", "HSB Sat"),
                // HSB Bri ^^
                ("ch1_hsb_attenuate_z", "HSB Bri"),
                // Kaleidoscope Slice
                ("ch1_kaleidoscope_slice", "Kaleidoscope Slice"),
                // Blur
                ("ch1_blur_amount", "Blur Amount"),
                ("ch1_blur_radius", "Blur Radius"),
                // Sharpen
                ("ch1_sharpen_amount", "Sharpen Amount"),
                ("ch1_sharpen_radius", "Sharpen Radius"),
            ];
            
            for (param_id, label) in &ch1_params {
                self.draw_lfo_control(ui, param_id, label, 1);
            }
        }
        
        // === CH2 Mix & Key LFOs (6 params) ===
        if CollapsingHeader::new("CH2 Mix & Key LFOs").default_open(true).build(ui) {
            let ch2_mix_params = [
                ("ch2_mix_amount", "Mix Amount"),
                ("ch2_key_threshold", "Key Threshold"),
                ("ch2_key_soft", "Key Soft"),
            ];
            
            for (param_id, label) in &ch2_mix_params {
                self.draw_lfo_control(ui, param_id, label, 1);
            }
        }
        
        // === CH2 Adjust LFOs (16 params) ===
        if CollapsingHeader::new("CH2 Adjust LFOs").default_open(true).build(ui) {
            let ch2_params = [
                // X/Y/Z Displace <->
                ("ch2_x_displace", "X Displace"),
                ("ch2_y_displace", "Y Displace"),
                ("ch2_z_displace", "Z Displace"),
                // Rotate
                ("ch2_rotate", "Rotate"),
                // HSB Hue ^^
                ("ch2_hsb_attenuate_x", "HSB Hue"),
                // HSB Sat ^^
                ("ch2_hsb_attenuate_y", "HSB Sat"),
                // HSB Bri ^^
                ("ch2_hsb_attenuate_z", "HSB Bri"),
                // Kaleidoscope Slice
                ("ch2_kaleidoscope_slice", "Kaleidoscope Slice"),
                // Blur
                ("ch2_blur_amount", "Blur Amount"),
                ("ch2_blur_radius", "Blur Radius"),
                // Sharpen
                ("ch2_sharpen_amount", "Sharpen Amount"),
                ("ch2_sharpen_radius", "Sharpen Radius"),
            ];
            
            for (param_id, label) in &ch2_params {
                self.draw_lfo_control(ui, param_id, label, 1);
            }
        }
        
        // === FB1 Mix & Key LFOs ===
        if CollapsingHeader::new("FB1 Mix & Key LFOs").default_open(true).build(ui) {
            let fb1_mix_params = [
                ("fb1_mix_amount", "Mix Amount"),
                ("fb1_key_threshold", "Key Threshold"),
                ("fb1_key_soft", "Key Soft"),
            ];
            
            for (param_id, label) in &fb1_mix_params {
                self.draw_lfo_control(ui, param_id, label, 1);
            }
        }
        
        // === FB1 Geo1 LFOs (8 params) ===
        if CollapsingHeader::new("FB1 Geo1 LFOs").default_open(true).build(ui) {
            let fb1_geo1_params = [
                ("fb1_x_displace", "X Displace"),
                ("fb1_y_displace", "Y Displace"),
                ("fb1_z_displace", "Z Displace"),
                ("fb1_rotate", "Rotate"),
            ];
            
            for (param_id, label) in &fb1_geo1_params {
                self.draw_lfo_control(ui, param_id, label, 1);
            }
        }
        
        // === FB1 Geo2 LFOs (10 params) ===
        if CollapsingHeader::new("FB1 Geo2 LFOs").default_open(true).build(ui) {
            let fb1_geo2_params = [
                // Stretch (X/Y) - from shear matrix x, w
                ("fb1_shear_matrix_x", "X Stretch"),
                ("fb1_shear_matrix_w", "Y Stretch"),
                // Shear (X/Y) - from shear matrix y, z
                ("fb1_shear_matrix_y", "X Shear"),
                ("fb1_shear_matrix_z", "Y Shear"),
                // Kaleidoscope Slice
                ("fb1_kaleidoscope_slice", "Kaleidoscope Slice"),
            ];
            
            for (param_id, label) in &fb1_geo2_params {
                self.draw_lfo_control(ui, param_id, label, 1);
            }
        }
        
        // === FB1 Color1 LFOs (6 params) ===
        if CollapsingHeader::new("FB1 Color LFOs").default_open(true).build(ui) {
            let fb1_color_params = [
                ("fb1_hsb_offset_x", "HSB Offset Hue"),
                ("fb1_hsb_offset_y", "HSB Offset Sat"),
                ("fb1_hsb_offset_z", "HSB Offset Bri"),
                ("fb1_hsb_attenuate_x", "HSB Attenuate Hue"),
                ("fb1_hsb_attenuate_y", "HSB Attenuate Sat"),
                ("fb1_hsb_attenuate_z", "HSB Attenuate Bri"),
            ];
            
            for (param_id, label) in &fb1_color_params {
                self.draw_lfo_control(ui, param_id, label, 1);
            }
        }
        
        // === FB1 Delay Time LFO ===
        if CollapsingHeader::new("FB1 Delay LFO").default_open(true).build(ui) {
            self.draw_lfo_control(ui, "fb1_delay_time", "Delay Time", 1);
        }
    }
    
    /// Build Block 2 LFO panel - Organized by OF LFO groups
    fn build_block2_lfo(&mut self, ui: &Ui) {
        // Global tempo control
        self.draw_tempo_control(ui);
        ui.separator();
        
        // === Block2 Input Adjust LFOs (16 params) ===
        if CollapsingHeader::new("Input Adjust LFOs").default_open(true).build(ui) {
            let input_params = [
                // X/Y/Z Displace <->
                ("block2_input_x_displace", "X Displace"),
                ("block2_input_y_displace", "Y Displace"),
                ("block2_input_z_displace", "Z Displace"),
                // Rotate
                ("block2_input_rotate", "Rotate"),
                // HSB Hue ^^
                ("block2_input_hsb_attenuate_x", "HSB Hue"),
                // HSB Sat ^^
                ("block2_input_hsb_attenuate_y", "HSB Sat"),
                // HSB Bri ^^
                ("block2_input_hsb_attenuate_z", "HSB Bri"),
                // Kaleidoscope Slice
                ("block2_input_kaleidoscope_slice", "Kaleidoscope Slice"),
                // Blur
                ("block2_input_blur_amount", "Blur Amount"),
                ("block2_input_blur_radius", "Blur Radius"),
                // Sharpen
                ("block2_input_sharpen_amount", "Sharpen Amount"),
                ("block2_input_sharpen_radius", "Sharpen Radius"),
            ];
            
            for (param_id, label) in &input_params {
                self.draw_lfo_control(ui, param_id, label, 2);
            }
        }
        
        // === FB2 Mix & Key LFOs ===
        if CollapsingHeader::new("FB2 Mix & Key LFOs").default_open(true).build(ui) {
            let fb2_mix_params = [
                ("fb2_mix_amount", "Mix Amount"),
                ("fb2_key_threshold", "Key Threshold"),
                ("fb2_key_soft", "Key Soft"),
            ];
            
            for (param_id, label) in &fb2_mix_params {
                self.draw_lfo_control(ui, param_id, label, 2);
            }
        }
        
        // === FB2 Geo1 LFOs (8 params) ===
        if CollapsingHeader::new("FB2 Geo1 LFOs").default_open(true).build(ui) {
            let fb2_geo1_params = [
                ("fb2_x_displace", "X Displace"),
                ("fb2_y_displace", "Y Displace"),
                ("fb2_z_displace", "Z Displace"),
                ("fb2_rotate", "Rotate"),
            ];
            
            for (param_id, label) in &fb2_geo1_params {
                self.draw_lfo_control(ui, param_id, label, 2);
            }
        }
        
        // === FB2 Geo2 LFOs (10 params) ===
        if CollapsingHeader::new("FB2 Geo2 LFOs").default_open(true).build(ui) {
            let fb2_geo2_params = [
                // Stretch (X/Y) - from shear matrix x, w
                ("fb2_shear_matrix_x", "X Stretch"),
                ("fb2_shear_matrix_w", "Y Stretch"),
                // Shear (X/Y) - from shear matrix y, z
                ("fb2_shear_matrix_y", "X Shear"),
                ("fb2_shear_matrix_z", "Y Shear"),
                // Kaleidoscope Slice
                ("fb2_kaleidoscope_slice", "Kaleidoscope Slice"),
            ];
            
            for (param_id, label) in &fb2_geo2_params {
                self.draw_lfo_control(ui, param_id, label, 2);
            }
        }
        
        // === FB2 Color LFOs (6 params) ===
        if CollapsingHeader::new("FB2 Color LFOs").default_open(true).build(ui) {
            let fb2_color_params = [
                ("fb2_hsb_offset_x", "HSB Offset Hue"),
                ("fb2_hsb_offset_y", "HSB Offset Sat"),
                ("fb2_hsb_offset_z", "HSB Offset Bri"),
                ("fb2_hsb_attenuate_x", "HSB Attenuate Hue"),
                ("fb2_hsb_attenuate_y", "HSB Attenuate Sat"),
                ("fb2_hsb_attenuate_z", "HSB Attenuate Bri"),
            ];
            
            for (param_id, label) in &fb2_color_params {
                self.draw_lfo_control(ui, param_id, label, 2);
            }
        }
        
        // === FB2 Delay Time LFO ===
        if CollapsingHeader::new("FB2 Delay LFO").default_open(true).build(ui) {
            self.draw_lfo_control(ui, "fb2_delay_time", "Delay Time", 2);
        }
    }
    
    /// Build Block 3 LFO panel - Organized by OF LFO groups
    fn build_block3_lfo(&mut self, ui: &Ui) {
        // Global tempo control
        self.draw_tempo_control(ui);
        ui.separator();
        
        // === Block 1 Re-process Geo1 LFOs (8 params) ===
        if CollapsingHeader::new("Block1 Geo1 LFOs").default_open(true).build(ui) {
            let b1_geo1_params = [
                ("block1_x_displace", "X Displace"),
                ("block1_y_displace", "Y Displace"),
                ("block1_z_displace", "Z Displace"),
                ("block1_rotate", "Rotate"),
            ];
            
            for (param_id, label) in &b1_geo1_params {
                self.draw_lfo_control(ui, param_id, label, 3);
            }
        }
        
        // === Block 1 Re-process Geo2 LFOs (10 params) ===
        if CollapsingHeader::new("Block1 Geo2 LFOs").default_open(true).build(ui) {
            let b1_geo2_params = [
                // Stretch (X/Y) - from shear matrix x, w
                ("block1_shear_matrix_x", "X Stretch"),
                ("block1_shear_matrix_w", "Y Stretch"),
                // Shear (X/Y) - from shear matrix y, z
                ("block1_shear_matrix_y", "X Shear"),
                ("block1_shear_matrix_z", "Y Shear"),
                // Kaleidoscope Slice
                ("block1_kaleidoscope_slice", "Kaleidoscope Slice"),
            ];
            
            for (param_id, label) in &b1_geo2_params {
                self.draw_lfo_control(ui, param_id, label, 3);
            }
        }
        
        // === Block 1 Colorize LFOs (30 params total) ===
        if CollapsingHeader::new("Block1 Colorize LFOs").default_open(true).build(ui) {
            let b1_color_params = [
                // Band 1
                ("block1_colorize_band1_x", "Band1 Hue/Red"),
                ("block1_colorize_band1_y", "Band1 Sat/Green"),
                ("block1_colorize_band1_z", "Band1 Bri/Blue"),
                // Band 2
                ("block1_colorize_band2_x", "Band2 Hue/Red"),
                ("block1_colorize_band2_y", "Band2 Sat/Green"),
                ("block1_colorize_band2_z", "Band2 Bri/Blue"),
                // Band 3
                ("block1_colorize_band3_x", "Band3 Hue/Red"),
                ("block1_colorize_band3_y", "Band3 Sat/Green"),
                ("block1_colorize_band3_z", "Band3 Bri/Blue"),
                // Band 4
                ("block1_colorize_band4_x", "Band4 Hue/Red"),
                ("block1_colorize_band4_y", "Band4 Sat/Green"),
                ("block1_colorize_band4_z", "Band4 Bri/Blue"),
                // Band 5
                ("block1_colorize_band5_x", "Band5 Hue/Red"),
                ("block1_colorize_band5_y", "Band5 Sat/Green"),
                ("block1_colorize_band5_z", "Band5 Bri/Blue"),
            ];
            
            for (param_id, label) in &b1_color_params {
                self.draw_lfo_control(ui, param_id, label, 3);
            }
        }
        
        // === Block 2 Re-process Geo1 LFOs (8 params) ===
        if CollapsingHeader::new("Block2 Geo1 LFOs").default_open(true).build(ui) {
            let b2_geo1_params = [
                ("block2_x_displace", "X Displace"),
                ("block2_y_displace", "Y Displace"),
                ("block2_z_displace", "Z Displace"),
                ("block2_rotate", "Rotate"),
            ];
            
            for (param_id, label) in &b2_geo1_params {
                self.draw_lfo_control(ui, param_id, label, 3);
            }
        }
        
        // === Block 2 Re-process Geo2 LFOs (10 params) ===
        if CollapsingHeader::new("Block2 Geo2 LFOs").default_open(true).build(ui) {
            let b2_geo2_params = [
                // Stretch (X/Y) - from shear matrix x, w
                ("block2_shear_matrix_x", "X Stretch"),
                ("block2_shear_matrix_w", "Y Stretch"),
                // Shear (X/Y) - from shear matrix y, z
                ("block2_shear_matrix_y", "X Shear"),
                ("block2_shear_matrix_z", "Y Shear"),
                // Kaleidoscope Slice
                ("block2_kaleidoscope_slice", "Kaleidoscope Slice"),
            ];
            
            for (param_id, label) in &b2_geo2_params {
                self.draw_lfo_control(ui, param_id, label, 3);
            }
        }
        
        // === Block 2 Colorize LFOs (30 params total) ===
        if CollapsingHeader::new("Block2 Colorize LFOs").default_open(true).build(ui) {
            let b2_color_params = [
                // Band 1
                ("block2_colorize_band1_x", "Band1 Hue/Red"),
                ("block2_colorize_band1_y", "Band1 Sat/Green"),
                ("block2_colorize_band1_z", "Band1 Bri/Blue"),
                // Band 2
                ("block2_colorize_band2_x", "Band2 Hue/Red"),
                ("block2_colorize_band2_y", "Band2 Sat/Green"),
                ("block2_colorize_band2_z", "Band2 Bri/Blue"),
                // Band 3
                ("block2_colorize_band3_x", "Band3 Hue/Red"),
                ("block2_colorize_band3_y", "Band3 Sat/Green"),
                ("block2_colorize_band3_z", "Band3 Bri/Blue"),
                // Band 4
                ("block2_colorize_band4_x", "Band4 Hue/Red"),
                ("block2_colorize_band4_y", "Band4 Sat/Green"),
                ("block2_colorize_band4_z", "Band4 Bri/Blue"),
                // Band 5
                ("block2_colorize_band5_x", "Band5 Hue/Red"),
                ("block2_colorize_band5_y", "Band5 Sat/Green"),
                ("block2_colorize_band5_z", "Band5 Bri/Blue"),
            ];
            
            for (param_id, label) in &b2_color_params {
                self.draw_lfo_control(ui, param_id, label, 3);
            }
        }
        
        // === Matrix Mix LFOs (18 params) ===
        if CollapsingHeader::new("Matrix Mix LFOs").default_open(true).build(ui) {
            let matrix_params = [
                // B1 R -> B2 R/G/B
                ("matrix_mix_r_to_r", "B1 Red -> B2 Red"),
                ("matrix_mix_r_to_g", "B1 Red -> B2 Green"),
                ("matrix_mix_r_to_b", "B1 Red -> B2 Blue"),
                // B1 G -> B2 R/G/B
                ("matrix_mix_g_to_r", "B1 Green -> B2 Red"),
                ("matrix_mix_g_to_g", "B1 Green -> B2 Green"),
                ("matrix_mix_g_to_b", "B1 Green -> B2 Blue"),
                // B1 B -> B2 R/G/B
                ("matrix_mix_b_to_r", "B1 Blue -> B2 Red"),
                ("matrix_mix_b_to_g", "B1 Blue -> B2 Green"),
                ("matrix_mix_b_to_b", "B1 Blue -> B2 Blue"),
            ];
            
            for (param_id, label) in &matrix_params {
                self.draw_lfo_control(ui, param_id, label, 3);
            }
        }
        
        // === Final Mix & Key LFOs (6 params) ===
        if CollapsingHeader::new("Final Mix & Key LFOs").default_open(true).build(ui) {
            let final_params = [
                ("final_mix_amount", "Mix Amount"),
                ("final_key_threshold", "Key Threshold"),
                ("final_key_soft", "Key Soft"),
            ];
            
            for (param_id, label) in &final_params {
                self.draw_lfo_control(ui, param_id, label, 3);
            }
        }
    }
    
    /// Draw LFO control for a single parameter
    fn draw_lfo_control(&mut self, ui: &Ui, param_id: &str, label: &str, block: i32) {
        let lfo_map = match block {
            1 => &mut self.block1_lfos,
            2 => &mut self.block2_lfos,
            3 => &mut self.block3_lfos,
            _ => &mut self.block1_lfos,
        };
        
        // Get or create LFO state
        let lfo = lfo_map.entry(param_id.to_string()).or_insert_with(|| LfoState {
            enabled: false,
            amplitude: 0.0,
            rate: 0.5,
            waveform: 0,
            tempo_sync: false,
            division: 2, // 1/4 note default
            bank_index: 0,
        });
        
        ui.text(label);
        
        ui.checkbox(&format!("Enable##{}_{}", param_id, block), &mut lfo.enabled);
        
        if lfo.enabled {
            ui.indent();
            
            // LFO Bank selector (0-15)
            let mut bank_idx = lfo.bank_index as usize;
            ui.text("LFO Bank:");
            ui.same_line();
            for i in 0..8 {
                if i > 0 { ui.same_line(); }
                let label = format!("{}##{}_{}_{}", i, param_id, block, i);
                if ui.radio_button_bool(&label, bank_idx == i) {
                    bank_idx = i;
                }
            }
            lfo.bank_index = bank_idx.clamp(0, 15) as i32;
            
            Drag::new(&format!("Amplitude##{}_{}", param_id, block))
                .speed(0.01).range(0.0, 1.0).build(ui, &mut lfo.amplitude);
            
            ui.checkbox(&format!("Tempo Sync##{}_{}", param_id, block), &mut lfo.tempo_sync);
            
            if lfo.tempo_sync {
                // Beat division selector
                let mut div_idx = lfo.division as usize;
                let preview = BEAT_DIVISIONS[div_idx.min(BEAT_DIVISIONS.len() - 1)].to_string();
                ComboBox::new(ui, &format!("Division##{}_{}", param_id, block))
                    .preview_value(&preview)
                    .build(|| {
                        for (idx, opt) in BEAT_DIVISIONS.iter().enumerate() {
                            if ui.selectable_config(opt).selected(idx == div_idx).build() {
                                div_idx = idx;
                            }
                        }
                    });
                lfo.division = div_idx.clamp(0, BEAT_DIVISIONS.len() - 1) as i32;
            } else {
                // Free rate control
                Drag::new(&format!("Rate##{}_{}", param_id, block))
                    .speed(0.01).range(0.0, 10.0).build(ui, &mut lfo.rate);
            }
            
            // Waveform selector
            let mut wave_idx = lfo.waveform as usize;
            let preview = WAVEFORM_NAMES[wave_idx.min(WAVEFORM_NAMES.len() - 1)].to_string();
            ComboBox::new(ui, &format!("Waveform##{}_{}", param_id, block))
                .preview_value(&preview)
                .build(|| {
                    for (idx, opt) in WAVEFORM_NAMES.iter().enumerate() {
                        if ui.selectable_config(opt).selected(idx == wave_idx).build() {
                            wave_idx = idx;
                        }
                    }
                });
            let new_waveform = wave_idx.clamp(0, WAVEFORM_NAMES.len() - 1) as i32;
            if new_waveform != lfo.waveform {
                lfo.waveform = new_waveform;
                // Sync waveform to the LFO bank in shared state
                if lfo.bank_index >= 0 {
                    if let Ok(mut state) = self.shared_state.lock() {
                        if (lfo.bank_index as usize) < state.lfo_banks.len() {
                            state.lfo_banks[lfo.bank_index as usize].waveform = new_waveform;
                        }
                    }
                }
            }
            
            // Sync all LFO parameters to the bank in shared state
            if lfo.bank_index >= 0 {
                if let Ok(mut state) = self.shared_state.lock() {
                    if (lfo.bank_index as usize) < state.lfo_banks.len() {
                        let bank = &mut state.lfo_banks[lfo.bank_index as usize];
                        bank.rate = lfo.rate;
                        bank.tempo_sync = lfo.tempo_sync;
                        bank.division = lfo.division;
                        // waveform is already synced above
                    }
                }
            }
            
            // Show live LFO output visualization
            if lfo.bank_index >= 0 {
                let bank_index = lfo.bank_index;
                let waveform = lfo.waveform;
                let amplitude = lfo.amplitude;
                if let Ok(state) = self.shared_state.lock() {
                    if (bank_index as usize) < state.lfo_banks.len() {
                        let bank = &state.lfo_banks[bank_index as usize];
                        let lfo_value = Self::calculate_lfo_value(bank.phase, waveform);
                        let display_value = lfo_value * amplitude;
                        
                        ui.text("LFO Output:");
                        ui.same_line();
                        
                        // Draw a simple bar showing current LFO value (-1 to 1)
                        let bar_width = 100.0;
                        let center_x = ui.cursor_screen_pos()[0] + bar_width / 2.0;
                        let y = ui.cursor_screen_pos()[1];
                        
                        // Background bar
                        let draw_list = ui.get_window_draw_list();
                        draw_list.add_rect(
                            [center_x - bar_width/2.0, y],
                            [center_x + bar_width/2.0, y + 10.0],
                            [0.3, 0.3, 0.3, 1.0]
                        ).filled(true).build();
                        
                        // Value indicator
                        let value_x = center_x + (display_value * bar_width / 2.0);
                        draw_list.add_rect(
                            [center_x.min(value_x), y],
                            [center_x.max(value_x), y + 10.0],
                            if display_value > 0.0 { [0.0, 1.0, 0.0, 1.0] } else { [1.0, 0.0, 0.0, 1.0] }
                        ).filled(true).build();
                        
                        ui.dummy([bar_width + 10.0, 15.0]);
                        
                        // Show numeric value
                        ui.same_line();
                        ui.text(format!("{:.2}", display_value));
                    }
                }
            }
            
            ui.unindent();
        }
    }
    
    /// Calculate LFO value from phase and waveform
    fn calculate_lfo_value(phase: f32, waveform: i32) -> f32 {
        let phase = phase.fract();
        match waveform {
            0 => (phase * 2.0 * std::f32::consts::PI).sin(), // Sine
            1 => { // Triangle
                if phase < 0.5 {
                    (phase * 4.0) - 1.0
                } else {
                    3.0 - (phase * 4.0)
                }
            }
            2 => phase * 2.0 - 1.0, // Ramp
            3 => 1.0 - phase * 2.0, // Saw
            4 => { // Square
                if phase < 0.5 { 1.0 } else { -1.0 }
            }
            _ => (phase * 2.0 * std::f32::consts::PI).sin(),
        }
    }
    
    /// Draw global tempo control with tap tempo
    fn draw_tempo_control(&mut self, ui: &Ui) {
        // Extract config value first
        let show_osc = self.config.show_osc_addresses;
        
        ui.text("Global Tempo");
        
        // Helper closure for OSC tooltips
        let osc_tooltip = |ui: &Ui, address: &str, value: Option<f32>, show: bool| {
            if show && ui.is_item_hovered() {
                let mut tooltip = format!("OSC: {}", address);
                if let Some(val) = value {
                    tooltip.push_str(&format!("\nValue: {:.3}", val));
                }
                ui.tooltip_text(tooltip);
            }
        };
        
        // BPM display and edit
        ui.same_line_with_pos(120.0);
        let mut bpm = self.bpm;
        if Drag::new("BPM").speed(1.0).range(20.0, 300.0).build(ui, &mut bpm) {
            self.bpm = bpm;
            // Update shared state so engine can use it
            if let Ok(mut state) = self.shared_state.lock() {
                state.bpm = bpm;
            }
        }
        osc_tooltip(ui, "/global/bpm", Some(self.bpm), show_osc);
        
        // Tap tempo button
        ui.same_line();
        let button_label = if self.beat_flash > 0.0 {
            "TAP FLASH!"
        } else {
            "TAP TEMPO"
        };
        
        if ui.button(button_label) {
            self.handle_tap_tempo();
        }
        osc_tooltip(ui, "/global/tap_tempo", None, show_osc);
        
        // Play/Pause button
        ui.same_line();
        let play_label = if self.bpm_playing { "PAUSE" } else { "PLAY" };
        if ui.button(play_label) {
            self.bpm_playing = !self.bpm_playing;
        }
        
        // Sync enable
        ui.same_line();
        ui.checkbox("Sync", &mut self.bpm_enabled);
    }
    
    /// Handle tap tempo button press
    fn handle_tap_tempo(&mut self) {
        use std::time::{SystemTime, UNIX_EPOCH};
        
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64();
        
        // Clear taps if it's been too long since last tap (2 seconds)
        if now - self.last_tap_time > 2.0 {
            self.tap_times.clear();
        }
        
        // Add tap time
        self.tap_times.push(now);
        
        // Keep only last 8 taps (more taps = more accurate average)
        if self.tap_times.len() > 8 {
            self.tap_times.remove(0);
        }
        
        // Update last tap time
        self.last_tap_time = now;
        
        // Reset all LFO phases on every tap (global sync)
        if let Ok(mut state) = self.shared_state.lock() {
            for lfo in &mut state.lfo_banks {
                lfo.phase = 0.0;
            }
        }
        
        // Calculate BPM from tap intervals (need at least 4 taps for accuracy)
        if self.tap_times.len() >= 4 {
            let mut intervals = Vec::new();
            for i in 1..self.tap_times.len() {
                intervals.push(self.tap_times[i] - self.tap_times[i-1]);
            }
            
            // Average interval
            let avg_interval: f64 = intervals.iter().sum::<f64>() / intervals.len() as f64;
            
            if avg_interval > 0.1 && avg_interval < 3.0 { // Reasonable range
                let new_bpm = (60.0 / avg_interval) as f32;
                self.bpm = new_bpm.clamp(40.0, 200.0);
                // Update shared state so engine can use it
                if let Ok(mut state) = self.shared_state.lock() {
                    state.bpm = self.bpm;
                }
            }
        }
        
        // Flash the button
        self.beat_flash = 0.2;
    }
    
    /// Public method to trigger tap tempo from external sources (e.g., keyboard shortcut)
    pub fn trigger_tap_tempo(&mut self) {
        self.handle_tap_tempo();
    }
    
    /// Show OSC address tooltip if the feature is enabled
    /// 
    /// # Arguments
    /// * `ui` - The ImGui UI context
    /// * `address` - The OSC address path (e.g., "/block1/ch1/x_displace")
    /// * `current_value` - Optional current parameter value to display
    pub fn show_osc_tooltip(&self, ui: &Ui, address: &str, current_value: Option<f32>) {
        if !self.config.show_osc_addresses {
            return;
        }
        
        if ui.is_item_hovered() {
            let mut tooltip = format!("OSC: {}", address);
            if let Some(val) = current_value {
                tooltip.push_str(&format!("\nValue: {:.3}", val));
            }
            ui.tooltip_text(tooltip);
        }
    }
    
    /// Generate OSC address for a Block 1 parameter
    pub fn get_osc_address_block1(param_id: &str) -> String {
        format!("/block1/{}", param_id.replace('_', "/").replace(".", "/"))
    }
    
    /// Generate OSC address for a Block 2 parameter
    pub fn get_osc_address_block2(param_id: &str) -> String {
        format!("/block2/{}", param_id.replace('_', "/").replace(".", "/"))
    }
    
    /// Generate OSC address for a Block 3 parameter
    pub fn get_osc_address_block3(param_id: &str) -> String {
        format!("/block3/{}", param_id.replace('_', "/").replace(".", "/"))
    }
}

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Get list of Block 1 parameter names for audio modulation
pub fn get_block1_param_names() -> Vec<String> {
    vec![
        "ch1_x_displace".to_string(),
        "ch1_y_displace".to_string(),
        "ch1_z_displace".to_string(),
        "ch1_rotate".to_string(),
        "ch1_hsb_attenuate.x".to_string(),
        "ch1_hsb_attenuate.y".to_string(),
        "ch1_hsb_attenuate.z".to_string(),
        "ch1_kaleidoscope_amount".to_string(),
        "ch1_blur_amount".to_string(),
        "ch2_mix_amount".to_string(),
        "ch2_x_displace".to_string(),
        "ch2_y_displace".to_string(),
        "ch2_rotate".to_string(),
        "fb1_mix_amount".to_string(),
        "fb1_x_displace".to_string(),
        "fb1_y_displace".to_string(),
        "fb1_rotate".to_string(),
    ]
}

/// Get list of Block 2 parameter names for audio modulation
pub fn get_block2_param_names() -> Vec<String> {
    vec![
        "block2_input_x_displace".to_string(),
        "block2_input_y_displace".to_string(),
        "block2_input_rotate".to_string(),
        "block2_input_blur_amount".to_string(),
        "fb2_mix_amount".to_string(),
        "fb2_x_displace".to_string(),
        "fb2_y_displace".to_string(),
        "fb2_rotate".to_string(),
    ]
}

/// Get list of Block 3 parameter names for audio modulation
pub fn get_block3_param_names() -> Vec<String> {
    vec![
        "block1_x_displace".to_string(),
        "block1_y_displace".to_string(),
        "block1_rotate".to_string(),
        "block2_x_displace".to_string(),
        "block2_y_displace".to_string(),
        "block2_rotate".to_string(),
        "final_mix_amount".to_string(),
    ]
}
