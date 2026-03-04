//! # MIDI Module
//!
//! Comprehensive MIDI learn/mapping system for real-time parameter control.
//!
//! ## Features
//! - MIDI Learn: Click a parameter, then move a MIDI control to map it
//! - Support for CC, Note On, Pitch Bend, and other message types
//! - 14-bit CC support for high-resolution control
//! - Multiple device support with hot-plugging
//! - Persistent mappings saved to `midi_mappings.toml`
//! - Range scaling (MIDI 0-127 maps to parameter min-max)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod input;
pub mod learn;
pub mod mapping;

pub use input::{MidiInputHandler, MidiMessage};
pub use learn::{LearnState, MidiLearn};
pub use mapping::{MidiMapping, MidiMappingConfig, MidiMessageType};

/// MIDI event sent from input thread to main thread
#[derive(Debug, Clone)]
pub struct MidiEvent {
    /// Device name/ID that sent the message
    pub device_id: String,
    /// The MIDI message
    pub message: MidiMessage,
    /// Timestamp (for debugging)
    pub timestamp: u64,
}

/// MIDI channel type (1-16, or 0 for Omni)
pub type MidiChannel = u8;

/// Controller/note number (0-127)
pub type MidiController = u8;

/// MIDI value (0-127 for 7-bit, 0-16383 for 14-bit)
pub type MidiValue = u16;

/// Shared MIDI state between GUI and engine
#[derive(Debug)]
pub struct MidiState {
    /// Currently active mappings (param_id -> mapping)
    pub mappings: HashMap<String, MidiMapping>,
    /// MIDI learn state
    pub learn: LearnState,
    /// Connected input devices
    pub connected_devices: Vec<String>,
    /// Currently selected device for learning (None = all devices)
    pub selected_device: Option<String>,
    /// Enable/disable MIDI input globally
    pub enabled: bool,
    /// Last received MIDI message (for display/debugging)
    pub last_message: Option<MidiEvent>,
    /// Channel filter (0 = Omni, 1-16 = specific channel)
    pub channel_filter: MidiChannel,
    /// Enable 14-bit CC support
    pub high_resolution_cc: bool,
    /// Latch mode for buttons (toggle vs momentary)
    pub latch_mode: bool,
}

impl MidiState {
    /// Create new MIDI state
    pub fn new() -> Self {
        Self {
            mappings: HashMap::new(),
            learn: LearnState::default(),
            connected_devices: Vec::new(),
            selected_device: None,
            enabled: true,
            last_message: None,
            channel_filter: 0, // Omni
            high_resolution_cc: true,
            latch_mode: false,
        }
    }

    /// Load mappings from config file
    pub fn load_mappings(&mut self, path: &str) -> anyhow::Result<()> {
        if !std::path::Path::new(path).exists() {
            log::info!("MIDI mappings file not found, starting with empty mappings");
            return Ok(());
        }

        let contents = std::fs::read_to_string(path)?;
        let config: MidiMappingConfig = toml::from_str(&contents)?;
        
        self.mappings.clear();
        let mut skipped = 0;
        for mapping in config.mappings {
            // Skip mappings with empty param_id (bug from older versions)
            if mapping.param_id.is_empty() {
                skipped += 1;
                continue;
            }
            self.mappings.insert(mapping.param_id.clone(), mapping);
        }
        
        if skipped > 0 {
            log::warn!("Skipped {} invalid mappings with empty param_id", skipped);
        }
        
        log::info!("Loaded {} MIDI mappings from {}", self.mappings.len(), path);
        Ok(())
    }

    /// Save mappings to config file
    pub fn save_mappings(&self, path: &str) -> anyhow::Result<()> {
        let config = MidiMappingConfig {
            mappings: self.mappings.values().cloned().collect(),
        };
        
        let toml = toml::to_string_pretty(&config)?;
        std::fs::write(path, toml)?;
        
        log::info!("Saved {} MIDI mappings to {}", self.mappings.len(), path);
        Ok(())
    }

    /// Find mapping for a given MIDI message
    pub fn find_mapping(&self, device_id: &str, message: &MidiMessage) -> Option<&MidiMapping> {
        log::trace!("MIDI: find_mapping called with device='{}', msg_type={:?}", device_id, message.message_type());
        for (param_id, mapping) in &self.mappings {
            log::trace!("MIDI: Checking mapping for '{}' (device='{}', type={:?}, ch={}, controller={})", 
                param_id, mapping.device, mapping.message_type, mapping.channel, mapping.controller);
            if mapping.matches(device_id, message) {
                log::trace!("MIDI: Mapping matches!");
                return Some(mapping);
            }
        }
        None
    }

    /// Add or update a mapping
    pub fn add_mapping(&mut self, param_id: String, mapping: MidiMapping) {
        log::info!("Adding MIDI mapping: {} -> {:?}", param_id, mapping);
        self.mappings.insert(param_id, mapping);
    }

    /// Remove a mapping by parameter ID
    pub fn remove_mapping(&mut self, param_id: &str) -> bool {
        if self.mappings.remove(param_id).is_some() {
            log::info!("Removed MIDI mapping for {}", param_id);
            true
        } else {
            false
        }
    }

    /// Clear all mappings
    pub fn clear_mappings(&mut self) {
        log::info!("Clearing all MIDI mappings");
        self.mappings.clear();
    }

    /// Start learning mode for a parameter
    pub fn start_learning(&mut self, param_id: String, param_min: f32, param_max: f32) {
        log::info!("Starting MIDI learn for parameter: {}", param_id);
        self.learn.start(param_id, param_min, param_max);
    }

    /// Cancel learning mode
    pub fn cancel_learning(&mut self) {
        if self.learn.is_active() {
            log::info!("Cancelling MIDI learn");
            self.learn.cancel();
        }
    }

    /// Process an incoming MIDI event (called from main thread)
    /// Returns the parameter ID and scaled value if a mapping was triggered
    pub fn process_midi_event(&mut self, event: MidiEvent) -> Option<(String, f32)> {
        self.last_message = Some(event.clone());

        // Check if we're in learn mode
        if self.learn.is_active() {
            log::trace!("MIDI: Learn mode active, checking message");
            if let Some(param_id) = self.learn.handle_midi_message(&event) {
                // Create mapping from the learned message
                let mut mapping = MidiMapping::from_event(&event, self.learn.param_min, self.learn.param_max);
                mapping.param_id = param_id.clone();  // Set the param_id!
                log::debug!("MIDI: Learned mapping for '{}' from device '{}'", param_id, event.device_id);
                self.add_mapping(param_id.clone(), mapping);
                return Some((param_id, self.learn.scale_value(event.message.value())));
            }
            return None;
        }

        // Check channel filter (0 = Omni, accept all)
        if self.channel_filter > 0 {
            let msg_channel = event.message.channel();
            if msg_channel != 0 && msg_channel != self.channel_filter {
                log::trace!("MIDI: Channel filter rejected message (ch {} != filter {})", msg_channel, self.channel_filter);
                return None;
            }
        }

        // Look up mapping
        log::trace!("MIDI: Looking up mapping for device '{}' with {} mappings", event.device_id, self.mappings.len());
        if let Some(mapping) = self.find_mapping(&event.device_id, &event.message) {
            let value = mapping.scale_value(event.message.value());
            log::trace!("MIDI: Found mapping for param '{}' -> value {:.3}", mapping.param_id, value);
            return Some((mapping.param_id.clone(), value));
        }

        log::trace!("MIDI: No mapping found");
        None
    }
}

impl Default for MidiState {
    fn default() -> Self {
        Self::new()
    }
}

/// Thread-safe MIDI state wrapper
pub type SharedMidiState = Arc<Mutex<MidiState>>;

/// Create a new shared MIDI state
pub fn create_shared_midi_state() -> SharedMidiState {
    Arc::new(Mutex::new(MidiState::new()))
}

use std::sync::{Arc, Mutex};

/// MIDI control change constants
pub mod cc {
    pub const MODULATION_WHEEL: u8 = 1;
    pub const BREATH_CONTROLLER: u8 = 2;
    pub const FOOT_CONTROLLER: u8 = 4;
    pub const PORTAMENTO_TIME: u8 = 5;
    pub const DATA_ENTRY_MSB: u8 = 6;
    pub const CHANNEL_VOLUME: u8 = 7;
    pub const BALANCE: u8 = 8;
    pub const PAN: u8 = 10;
    pub const EXPRESSION: u8 = 11;
    pub const EFFECT_CONTROL_1: u8 = 12;
    pub const EFFECT_CONTROL_2: u8 = 13;
    pub const GENERAL_PURPOSE_1: u8 = 16;
    pub const GENERAL_PURPOSE_2: u8 = 17;
    pub const GENERAL_PURPOSE_3: u8 = 18;
    pub const GENERAL_PURPOSE_4: u8 = 19;
    pub const BANK_SELECT_LSB: u8 = 32;
    pub const MODULATION_WHEEL_LSB: u8 = 33;
    pub const BREATH_CONTROLLER_LSB: u8 = 34;
    pub const FOOT_CONTROLLER_LSB: u8 = 36;
    pub const PORTAMENTO_TIME_LSB: u8 = 37;
    pub const DATA_ENTRY_LSB: u8 = 38;
    pub const CHANNEL_VOLUME_LSB: u8 = 39;
    pub const BALANCE_LSB: u8 = 40;
    pub const PAN_LSB: u8 = 42;
    pub const EXPRESSION_LSB: u8 = 43;
    pub const SUSTAIN_PEDAL: u8 = 64;
    pub const PORTAMENTO_ON_OFF: u8 = 65;
    pub const SOSTENUTO: u8 = 66;
    pub const SOFT_PEDAL: u8 = 67;
    pub const LEGATO_FOOTSWITCH: u8 = 68;
    pub const HOLD_2: u8 = 69;
    pub const SOUND_CONTROLLER_1: u8 = 70; // Sound Variation
    pub const SOUND_CONTROLLER_2: u8 = 71; // Timbre/Harmonic Intensity
    pub const SOUND_CONTROLLER_3: u8 = 72; // Release Time
    pub const SOUND_CONTROLLER_4: u8 = 73; // Attack Time
    pub const SOUND_CONTROLLER_5: u8 = 74; // Brightness
    pub const SOUND_CONTROLLER_6: u8 = 75; // Decay Time
    pub const SOUND_CONTROLLER_7: u8 = 76; // Vibrato Rate
    pub const SOUND_CONTROLLER_8: u8 = 77; // Vibrato Depth
    pub const SOUND_CONTROLLER_9: u8 = 78; // Vibrato Delay
    pub const SOUND_CONTROLLER_10: u8 = 79; // undefined
    pub const GENERAL_PURPOSE_5: u8 = 80;
    pub const GENERAL_PURPOSE_6: u8 = 81;
    pub const GENERAL_PURPOSE_7: u8 = 82;
    pub const GENERAL_PURPOSE_8: u8 = 83;
    pub const PORTAMENTO_CONTROL: u8 = 84;
    pub const EFFECTS_1_DEPTH: u8 = 91; // Reverb Send Level
    pub const EFFECTS_2_DEPTH: u8 = 92; // Tremolo Depth
    pub const EFFECTS_3_DEPTH: u8 = 93; // Chorus Send Level
    pub const EFFECTS_4_DEPTH: u8 = 94; // Celeste Depth
    pub const EFFECTS_5_DEPTH: u8 = 95; // Phaser Depth
    pub const DATA_INCREMENT: u8 = 96;
    pub const DATA_DECREMENT: u8 = 97;
    pub const NRPN_LSB: u8 = 98;
    pub const NRPN_MSB: u8 = 99;
    pub const RPN_LSB: u8 = 100;
    pub const RPN_MSB: u8 = 101;
    pub const ALL_SOUND_OFF: u8 = 120;
    pub const RESET_ALL_CONTROLLERS: u8 = 121;
    pub const LOCAL_CONTROL_ON_OFF: u8 = 122;
    pub const ALL_NOTES_OFF: u8 = 123;
    pub const OMNI_MODE_OFF: u8 = 124;
    pub const OMNI_MODE_ON: u8 = 125;
    pub const MONO_MODE_ON: u8 = 126;
    pub const POLY_MODE_ON: u8 = 127;
}

/// Get human-readable name for a CC number
pub fn cc_name(cc: u8) -> &'static str {
    match cc {
        0 => "Bank Select",
        1 => "Modulation Wheel",
        2 => "Breath Controller",
        4 => "Foot Controller",
        5 => "Portamento Time",
        6 => "Data Entry MSB",
        7 => "Channel Volume",
        8 => "Balance",
        10 => "Pan",
        11 => "Expression",
        12 => "Effect Control 1",
        13 => "Effect Control 2",
        16..=19 => "General Purpose",
        32..=63 => "LSB (14-bit)",
        64 => "Sustain Pedal",
        65 => "Portamento",
        66 => "Sostenuto",
        67 => "Soft Pedal",
        68 => "Legato Footswitch",
        69 => "Hold 2",
        70 => "Sound Variation",
        71 => "Timbre/Harmonic",
        72 => "Release Time",
        73 => "Attack Time",
        74 => "Brightness",
        75 => "Decay Time",
        76 => "Vibrato Rate",
        77 => "Vibrato Depth",
        78 => "Vibrato Delay",
        79 => "Sound Controller 10",
        80..=83 => "General Purpose",
        84 => "Portamento Control",
        91 => "Reverb Send",
        92 => "Tremolo Depth",
        93 => "Chorus Send",
        94 => "Celeste Depth",
        95 => "Phaser Depth",
        96 => "Data Increment",
        97 => "Data Decrement",
        98 => "NRPN LSB",
        99 => "NRPN MSB",
        100 => "RPN LSB",
        101 => "RPN MSB",
        120 => "All Sound Off",
        121 => "Reset All Controllers",
        122 => "Local Control",
        123 => "All Notes Off",
        124 => "Omni Mode Off",
        125 => "Omni Mode On",
        126 => "Mono Mode On",
        127 => "Poly Mode On",
        _ => "Undefined",
    }
}
