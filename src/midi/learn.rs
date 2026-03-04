//! # MIDI Learn System
//!
//! Implements the "MIDI Learn" functionality where users can click a parameter
//! and then move a MIDI control to automatically map it.

use std::time::{Duration, Instant};

use super::{MidiChannel, MidiController, MidiEvent, MidiMessage, MidiMessageType};
use super::mapping::ParamRange;

/// State of the MIDI learn system
#[derive(Debug, Clone)]
pub struct LearnState {
    /// Whether learn mode is active
    pub active: bool,
    /// Parameter ID being learned
    pub param_id: String,
    /// Parameter minimum value
    pub param_min: f32,
    /// Parameter maximum value
    pub param_max: f32,
    /// When learning started (for timeout)
    pub start_time: Option<Instant>,
    /// Timeout duration
    pub timeout: Duration,
    /// Visual feedback: flash counter
    pub flash_count: u32,
    /// Last flash time
    pub last_flash: Instant,
}

impl LearnState {
    /// Create a new learn state (inactive)
    pub fn new() -> Self {
        Self {
            active: false,
            param_id: String::new(),
            param_min: 0.0,
            param_max: 1.0,
            start_time: None,
            timeout: Duration::from_secs(10),
            flash_count: 0,
            last_flash: Instant::now(),
        }
    }

    /// Check if learn mode is currently active
    pub fn is_active(&self) -> bool {
        if !self.active {
            return false;
        }

        // Check for timeout
        if let Some(start) = self.start_time {
            if start.elapsed() > self.timeout {
                return false;
            }
        }

        true
    }

    /// Start learning for a parameter
    pub fn start(&mut self, param_id: String, param_min: f32, param_max: f32) {
        self.active = true;
        self.param_id = param_id;
        self.param_min = param_min;
        self.param_max = param_max;
        self.start_time = Some(Instant::now());
        self.flash_count = 0;
        self.last_flash = Instant::now();
        log::info!(
            "MIDI learn started for '{}', range [{}-{}]",
            self.param_id, param_min, param_max
        );
    }

    /// Cancel learning
    pub fn cancel(&mut self) {
        self.active = false;
        self.param_id.clear();
        self.start_time = None;
        log::info!("MIDI learn cancelled");
    }

    /// Handle a MIDI message during learn mode
    /// Returns Some(param_id) if a mapping was successfully learned
    pub fn handle_midi_message(&mut self, event: &MidiEvent) -> Option<String> {
        if !self.is_active() {
            return None;
        }

        // Only learn from certain message types
        let should_learn = match &event.message {
            MidiMessage::ControlChange { .. } => true,
            MidiMessage::NoteOn { velocity, .. } => *velocity > 0,
            MidiMessage::PitchBend { .. } => true,
            MidiMessage::PolyAftertouch { .. } => true,
            MidiMessage::ChannelAftertouch { .. } => true,
            _ => false,
        };

        if should_learn {
            let param_id = self.param_id.clone();
            self.active = false;
            self.start_time = None;
            log::info!(
                "MIDI learn completed: '{}' mapped to {:?} from device '{}'",
                param_id, event.message, event.device_id
            );
            Some(param_id)
        } else {
            None
        }
    }

    /// Get the current flash state for visual feedback (0.0 - 1.0)
    pub fn flash_intensity(&self) -> f32 {
        if !self.is_active() {
            return 0.0;
        }

        let elapsed = self.last_flash.elapsed().as_secs_f32();
        let period = 0.5; // Flash every 500ms
        
        // Simple sine wave flash
        ((elapsed / period) * std::f32::consts::PI).sin().abs()
    }

    /// Check if the parameter should show "learning" highlight
    pub fn is_learning(&self, param_id: &str) -> bool {
        self.is_active() && self.param_id == param_id
    }

    /// Scale a MIDI value to the parameter range
    pub fn scale_value(&self, midi_value: u16) -> f32 {
        let normalized = midi_value as f32 / 127.0;
        self.param_min + normalized * (self.param_max - self.param_min)
    }
}

impl Default for LearnState {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for GUI elements that support MIDI learn
pub trait MidiLearn {
    /// Get the parameter ID for this control
    fn param_id(&self) -> &str;

    /// Get the parameter range
    fn param_range(&self) -> ParamRange;

    /// Check if this control is currently being learned
    fn is_learning(&self, learn_state: &LearnState) -> bool;

    /// Get display name for the parameter
    fn display_name(&self) -> String;
}

/// Helper struct for learnable parameters
#[derive(Debug, Clone)]
pub struct LearnableParam {
    pub param_id: String,
    pub display_name: String,
    pub min_value: f32,
    pub max_value: f32,
}

impl LearnableParam {
    /// Create a new learnable parameter
    pub fn new(param_id: &str, display_name: &str, min: f32, max: f32) -> Self {
        Self {
            param_id: param_id.to_string(),
            display_name: display_name.to_string(),
            min_value: min,
            max_value: max,
        }
    }

    /// Create from a parameter ID, auto-detecting range
    pub fn from_id(param_id: &str) -> Self {
        let range = ParamRange::for_param(param_id);
        let display_name = super::mapping::param_display_name(param_id);
        
        Self {
            param_id: param_id.to_string(),
            display_name,
            min_value: range.min,
            max_value: range.max,
        }
    }
}

impl MidiLearn for LearnableParam {
    fn param_id(&self) -> &str {
        &self.param_id
    }

    fn param_range(&self) -> ParamRange {
        ParamRange {
            min: self.min_value,
            max: self.max_value,
        }
    }

    fn is_learning(&self, learn_state: &LearnState) -> bool {
        learn_state.is_learning(&self.param_id)
    }

    fn display_name(&self) -> String {
        self.display_name.clone()
    }
}

/// Collection of all learnable parameters in the application
pub struct LearnableParams {
    params: Vec<LearnableParam>,
}

impl LearnableParams {
    /// Create the complete list of learnable parameters
    pub fn all() -> Self {
        let mut params = Vec::new();

        // Block 1 - Channel 1
        params.push(LearnableParam::from_id("block1.ch1_x_displace"));
        params.push(LearnableParam::from_id("block1.ch1_y_displace"));
        params.push(LearnableParam::from_id("block1.ch1_z_displace"));
        params.push(LearnableParam::from_id("block1.ch1_rotate"));
        params.push(LearnableParam::from_id("block1.ch1_blur_amount"));
        params.push(LearnableParam::from_id("block1.ch1_sharpen_amount"));
        params.push(LearnableParam::from_id("block1.ch1_kaleidoscope_amount"));

        // Block 1 - Channel 2
        params.push(LearnableParam::from_id("block1.ch2_mix_amount"));
        params.push(LearnableParam::from_id("block1.ch2_x_displace"));
        params.push(LearnableParam::from_id("block1.ch2_y_displace"));
        params.push(LearnableParam::from_id("block1.ch2_z_displace"));
        params.push(LearnableParam::from_id("block1.ch2_rotate"));
        params.push(LearnableParam::from_id("block1.ch2_blur_amount"));
        params.push(LearnableParam::from_id("block1.ch2_sharpen_amount"));
        params.push(LearnableParam::from_id("block1.ch2_key_threshold"));
        params.push(LearnableParam::from_id("block1.ch2_key_soft"));

        // Block 1 - FB1
        params.push(LearnableParam::from_id("block1.fb1_mix_amount"));
        params.push(LearnableParam::from_id("block1.fb1_x_displace"));
        params.push(LearnableParam::from_id("block1.fb1_y_displace"));
        params.push(LearnableParam::from_id("block1.fb1_z_displace"));
        params.push(LearnableParam::from_id("block1.fb1_rotate"));
        params.push(LearnableParam::from_id("block1.fb1_blur_amount"));
        params.push(LearnableParam::from_id("block1.fb1_sharpen_amount"));
        params.push(LearnableParam::from_id("block1.fb1_delay_time"));

        // Block 2 - Input
        params.push(LearnableParam::from_id("block2.input_x_displace"));
        params.push(LearnableParam::from_id("block2.input_y_displace"));
        params.push(LearnableParam::from_id("block2.input_z_displace"));
        params.push(LearnableParam::from_id("block2.input_rotate"));
        params.push(LearnableParam::from_id("block2.input_blur_amount"));
        params.push(LearnableParam::from_id("block2.input_sharpen_amount"));

        // Block 2 - FB2
        params.push(LearnableParam::from_id("block2.fb2_mix_amount"));
        params.push(LearnableParam::from_id("block2.fb2_x_displace"));
        params.push(LearnableParam::from_id("block2.fb2_y_displace"));
        params.push(LearnableParam::from_id("block2.fb2_z_displace"));
        params.push(LearnableParam::from_id("block2.fb2_rotate"));

        // Block 3 - Block 1 Re-process
        params.push(LearnableParam::from_id("block3.block1_x_displace"));
        params.push(LearnableParam::from_id("block3.block1_y_displace"));
        params.push(LearnableParam::from_id("block3.block1_z_displace"));
        params.push(LearnableParam::from_id("block3.block1_rotate"));

        // Block 3 - Block 2 Re-process
        params.push(LearnableParam::from_id("block3.block2_x_displace"));
        params.push(LearnableParam::from_id("block3.block2_y_displace"));
        params.push(LearnableParam::from_id("block3.block2_z_displace"));
        params.push(LearnableParam::from_id("block3.block2_rotate"));

        // Block 3 - Matrix Mixer
        params.push(LearnableParam::from_id("block3.matrix_mix_r_to_r"));
        params.push(LearnableParam::from_id("block3.matrix_mix_r_to_g"));
        params.push(LearnableParam::from_id("block3.matrix_mix_r_to_b"));
        params.push(LearnableParam::from_id("block3.matrix_mix_g_to_r"));
        params.push(LearnableParam::from_id("block3.matrix_mix_g_to_g"));
        params.push(LearnableParam::from_id("block3.matrix_mix_g_to_b"));
        params.push(LearnableParam::from_id("block3.matrix_mix_b_to_r"));
        params.push(LearnableParam::from_id("block3.matrix_mix_b_to_g"));
        params.push(LearnableParam::from_id("block3.matrix_mix_b_to_b"));

        // Block 3 - Final Mix
        params.push(LearnableParam::from_id("block3.final_mix_amount"));
        params.push(LearnableParam::from_id("block3.final_key_threshold"));
        params.push(LearnableParam::from_id("block3.final_key_soft"));

        // Global
        params.push(LearnableParam::from_id("global.bpm"));

        Self { params }
    }

    /// Get all parameters
    pub fn params(&self) -> &[LearnableParam] {
        &self.params
    }

    /// Find a parameter by ID
    pub fn find(&self, param_id: &str) -> Option<&LearnableParam> {
        self.params.iter().find(|p| p.param_id == param_id)
    }

    /// Get parameters for a specific block
    pub fn for_block(&self, block: &str) -> Vec<&LearnableParam> {
        self.params
            .iter()
            .filter(|p| p.param_id.starts_with(block))
            .collect()
    }
}

impl Default for LearnableParams {
    fn default() -> Self {
        Self::all()
    }
}
