//! # MIDI Mapping Types
//!
//! Defines the data structures for MIDI parameter mappings.

use serde::{Deserialize, Serialize};
use super::{MidiChannel, MidiController, MidiEvent, MidiValue};
use crate::midi::MidiMessage;

/// Type of MIDI message that can be mapped
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MidiMessageType {
    /// Control Change (CC)
    #[serde(rename = "CC")]
    ControlChange,
    /// Note On (velocity used as value)
    #[serde(rename = "NoteOn")]
    NoteOn,
    /// Note Off
    #[serde(rename = "NoteOff")]
    NoteOff,
    /// Polyphonic Aftertouch
    #[serde(rename = "PolyAftertouch")]
    PolyAftertouch,
    /// Channel Aftertouch
    #[serde(rename = "ChannelAftertouch")]
    ChannelAftertouch,
    /// Pitch Bend (-8192 to 8191, mapped to 0-16383)
    #[serde(rename = "PitchBend")]
    PitchBend,
    /// Program Change
    #[serde(rename = "ProgramChange")]
    ProgramChange,
}

impl MidiMessageType {
    /// Get human-readable name
    pub fn name(&self) -> &'static str {
        match self {
            MidiMessageType::ControlChange => "CC",
            MidiMessageType::NoteOn => "Note On",
            MidiMessageType::NoteOff => "Note Off",
            MidiMessageType::PolyAftertouch => "Poly Aftertouch",
            MidiMessageType::ChannelAftertouch => "Channel Aftertouch",
            MidiMessageType::PitchBend => "Pitch Bend",
            MidiMessageType::ProgramChange => "Program Change",
        }
    }

    /// Check if this message type uses a controller/note number
    pub fn uses_controller(&self) -> bool {
        matches!(self, 
            MidiMessageType::ControlChange |
            MidiMessageType::NoteOn |
            MidiMessageType::NoteOff |
            MidiMessageType::PolyAftertouch |
            MidiMessageType::ProgramChange
        )
    }
}

impl Default for MidiMessageType {
    fn default() -> Self {
        MidiMessageType::ControlChange
    }
}

/// A single MIDI-to-parameter mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MidiMapping {
    /// Parameter ID (e.g., "block1.fb1_mix_amount")
    pub param_id: String,
    /// Device name/ID (empty string = match any device)
    #[serde(default)]
    pub device: String,
    /// Type of MIDI message
    pub message_type: MidiMessageType,
    /// MIDI channel (0 = Omni, 1-16 = specific channel)
    #[serde(default)]
    pub channel: MidiChannel,
    /// Controller number or note number (for CC/Note messages)
    #[serde(default)]
    pub controller: MidiController,
    /// Minimum parameter value
    #[serde(default = "default_min_value")]
    pub min_value: f32,
    /// Maximum parameter value
    #[serde(default = "default_max_value")]
    pub max_value: f32,
    /// Whether this is a 14-bit high-resolution mapping
    #[serde(default)]
    pub high_resolution: bool,
    /// Invert the MIDI value (127-value)
    #[serde(default)]
    pub invert: bool,
    /// Use toggle mode (for buttons)
    #[serde(default)]
    pub toggle: bool,
    /// Curve type: "linear", "log", "exp"
    #[serde(default = "default_curve")]
    pub curve: String,
}

fn default_min_value() -> f32 {
    0.0
}

fn default_max_value() -> f32 {
    1.0
}

fn default_curve() -> String {
    "linear".to_string()
}

impl MidiMapping {
    /// Create a new mapping
    pub fn new(
        param_id: String,
        message_type: MidiMessageType,
        channel: MidiChannel,
        controller: MidiController,
    ) -> Self {
        Self {
            param_id,
            device: String::new(),
            message_type,
            channel,
            controller,
            min_value: 0.0,
            max_value: 1.0,
            high_resolution: false,
            invert: false,
            toggle: false,
            curve: "linear".to_string(),
        }
    }

    /// Create a mapping from a MIDI event
    pub fn from_event(event: &MidiEvent, min_value: f32, max_value: f32) -> Self {
        let (msg_type, controller) = match &event.message {
            MidiMessage::ControlChange { controller, .. } => {
                (MidiMessageType::ControlChange, *controller)
            }
            MidiMessage::NoteOn { note, .. } => {
                (MidiMessageType::NoteOn, *note)
            }
            MidiMessage::NoteOff { note, .. } => {
                (MidiMessageType::NoteOff, *note)
            }
            MidiMessage::PolyAftertouch { note, .. } => {
                (MidiMessageType::PolyAftertouch, *note)
            }
            MidiMessage::ChannelAftertouch { .. } => {
                (MidiMessageType::ChannelAftertouch, 0)
            }
            MidiMessage::PitchBend { .. } => {
                (MidiMessageType::PitchBend, 0)
            }
            MidiMessage::ProgramChange { program, .. } => {
                (MidiMessageType::ProgramChange, *program)
            }
        };

        Self {
            param_id: String::new(), // Will be set later
            device: event.device_id.clone(),
            message_type: msg_type,
            channel: event.message.channel(),
            controller,
            min_value,
            max_value,
            high_resolution: false,
            invert: false,
            toggle: false,
            curve: "linear".to_string(),
        }
    }

    /// Check if a MIDI message matches this mapping
    pub fn matches(&self, device_id: &str, message: &MidiMessage) -> bool {
        // Check device (empty string = match any)
        if !self.device.is_empty() && self.device != device_id {
            log::trace!("MIDI: Device mismatch - self='{}', event='{}'", self.device, device_id);
            return false;
        }

        // Check message type
        let msg_type = message.message_type();
        if msg_type != self.message_type {
            log::trace!("MIDI: Message type mismatch - self={:?}, event={:?}", self.message_type, msg_type);
            return false;
        }

        // Check channel (0 = Omni)
        let channel = message.channel();
        if self.channel != 0 && channel != 0 && channel != self.channel {
            log::trace!("MIDI: Channel mismatch - self={}, event={}", self.channel, channel);
            return false;
        }

        // Check controller/note number (if applicable)
        if self.message_type.uses_controller() {
            let msg_controller = match message {
                MidiMessage::ControlChange { controller, .. } => *controller,
                MidiMessage::NoteOn { note, .. } => *note,
                MidiMessage::NoteOff { note, .. } => *note,
                MidiMessage::PolyAftertouch { note, .. } => *note,
                MidiMessage::ProgramChange { program, .. } => *program,
                _ => 0,
            };
            if msg_controller != self.controller {
                log::trace!("MIDI: Controller mismatch - self={}, event={}", self.controller, msg_controller);
                return false;
            }
        }

        true
    }

    /// Scale a MIDI value to the parameter range
    pub fn scale_value(&self, midi_value: MidiValue) -> f32 {
        let max_midi = if self.high_resolution { 16383.0 } else { 127.0 };
        
        // Normalize to 0-1
        let mut normalized = midi_value as f32 / max_midi;
        
        // Apply inversion if enabled
        if self.invert {
            normalized = 1.0 - normalized;
        }

        // Apply curve
        normalized = match self.curve.as_str() {
            "log" => normalized.powf(2.0),         // Logarithmic
            "exp" => normalized.sqrt(),            // Exponential
            "cubic" => normalized.powf(3.0),       // Cubic
            _ => normalized,                       // Linear
        };

        // Map to parameter range
        self.min_value + normalized * (self.max_value - self.min_value)
    }

    /// Get a human-readable description
    pub fn description(&self) -> String {
        let device_str = if self.device.is_empty() {
            "Any Device".to_string()
        } else {
            self.device.clone()
        };

        let channel_str = if self.channel == 0 {
            "Omni".to_string()
        } else {
            format!("Ch{}", self.channel)
        };

        match self.message_type {
            MidiMessageType::ControlChange => {
                format!(
                    "{} {} CC{} [{}-{}]",
                    device_str, channel_str, self.controller,
                    self.min_value, self.max_value
                )
            }
            MidiMessageType::NoteOn | MidiMessageType::NoteOff => {
                format!(
                    "{} {} Note{} [{}-{}]",
                    device_str, channel_str, self.controller,
                    self.min_value, self.max_value
                )
            }
            MidiMessageType::PitchBend => {
                format!(
                    "{} {} PitchBend [{}-{}]",
                    device_str, channel_str,
                    self.min_value, self.max_value
                )
            }
            _ => {
                format!(
                    "{} {} {:?} [{}-{}]",
                    device_str, channel_str, self.message_type,
                    self.min_value, self.max_value
                )
            }
        }
    }
}

impl Default for MidiMapping {
    fn default() -> Self {
        Self {
            param_id: String::new(),
            device: String::new(),
            message_type: MidiMessageType::ControlChange,
            channel: 0,
            controller: 0,
            min_value: 0.0,
            max_value: 1.0,
            high_resolution: false,
            invert: false,
            toggle: false,
            curve: "linear".to_string(),
        }
    }
}

/// Configuration file structure for MIDI mappings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MidiMappingConfig {
    #[serde(default)]
    pub mappings: Vec<MidiMapping>,
}

impl Default for MidiMappingConfig {
    fn default() -> Self {
        Self {
            mappings: Vec::new(),
        }
    }
}

/// Parameter range info for MIDI learn
#[derive(Debug, Clone, Copy)]
pub struct ParamRange {
    pub min: f32,
    pub max: f32,
}

impl ParamRange {
    /// Get range for a known parameter
    pub fn for_param(param_id: &str) -> Self {
        // Check for boolean parameters
        if param_id.contains("_mirror") 
            || param_id.contains("_flip") 
            || param_id.contains("_invert")
            || param_id.contains("_switch")
            || param_id.contains("_on") {
            return Self { min: 0.0, max: 1.0 };
        }

        // Check for specific parameter patterns
        if param_id.contains("_mix_amount") 
            || param_id.contains("_amount") && !param_id.contains("_x_displace") && !param_id.contains("_y_displace") {
            return Self { min: 0.0, max: 1.0 };
        }

        if param_id.contains("_x_displace") || param_id.contains("_y_displace") {
            return Self { min: -2.0, max: 2.0 };
        }

        if param_id.contains("_z_displace") {
            return Self { min: 0.0, max: 10.0 };
        }

        if param_id.contains("_rotate") {
            return Self { min: -360.0, max: 360.0 };
        }

        if param_id.contains("_blur_") || param_id.contains("_sharpen_") {
            return Self { min: 0.0, max: 1.0 };
        }

        if param_id.contains("_key_threshold") || param_id.contains("_key_soft") {
            return Self { min: -1.0, max: 1.0 };
        }

        // Default range
        Self { min: 0.0, max: 1.0 }
    }
}

/// Get a human-readable name for a parameter ID
pub fn param_display_name(param_id: &str) -> String {
    // Convert snake_case to Title Case
    param_id
        .split('.')
        .last()
        .unwrap_or(param_id)
        .split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
