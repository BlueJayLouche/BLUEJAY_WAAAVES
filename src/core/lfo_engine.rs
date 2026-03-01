//! # LFO Engine
//!
//! Calculates LFO values and applies them to parameters.
//! Supports multiple waveforms, tempo sync, and per-parameter modulation.

use crate::core::{LfoAssignment, LfoParameterMap, SharedState};
use crate::params::{Block1Params, Block2Params, Block3Params, LfoBank};
use std::f32::consts::PI;

/// Beat division multipliers (relative to quarter note) for LFO rates
/// These represent the cycle duration in beats (smaller = faster)
const BEAT_DIVISIONS: [f32; 8] = [
    0.0625, // 1/16 - fastest (1/16 of a beat)
    0.125,  // 1/8 (1/8 of a beat)
    0.25,   // 1/4 - default (1/4 of a beat)
    0.5,    // 1/2 (1/2 of a beat)
    1.0,    // 1 beat
    2.0,    // 2 beats
    4.0,    // 4 beats
    8.0,    // 8 beats - slowest
];

/// Beat division names for delay time tempo sync
pub const DELAY_BEAT_DIVISION_NAMES: [&str; 8] = [
    "1/16", "1/8", "1/4", "1/2", "1", "2", "4", "8"
];

/// Beat multipliers for delay time calculation (in beats)
/// These represent: 1/16, 1/8, 1/4, 1/2, 1, 2, 4, 8 beats
const DELAY_BEAT_MULTIPLIERS: [f32; 8] = [
    0.25,  // 1/16 beat
    0.5,   // 1/8 beat
    1.0,   // 1/4 beat (default)
    2.0,   // 1/2 beat
    4.0,   // 1 beat
    8.0,   // 2 beats
    16.0,  // 4 beats
    32.0,  // 8 beats
];

/// Calculate delay frames from BPM and beat division
/// 
/// Formula: delayFrames = (60 / BPM) * beatMultiplier * FPS
/// 
/// # Arguments
/// * `bpm` - Beats per minute
/// * `division` - Beat division index (0-7)
/// * `target_fps` - Target frame rate (usually 60.0)
/// 
/// # Returns
/// Number of frames to delay (clamped to 1-119)
pub fn calculate_delay_frames_from_tempo(bpm: f32, division: i32, target_fps: f32) -> i32 {
    let bpm = bpm.max(1.0);  // Prevent division by zero
    let division = division.clamp(0, DELAY_BEAT_MULTIPLIERS.len() as i32 - 1) as usize;
    
    let beat_duration = 60.0 / bpm;  // seconds per beat
    let delay_seconds = beat_duration * DELAY_BEAT_MULTIPLIERS[division];
    let delay_frames = (delay_seconds * target_fps) as i32;
    
    // Clamp to valid range (1-119, since 0 is immediate feedback)
    delay_frames.clamp(1, 119)
}

/// LFO waveforms
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Waveform {
    Sine = 0,
    Triangle = 1,
    Ramp = 2,
    Saw = 3,
    Square = 4,
}

impl From<i32> for Waveform {
    fn from(val: i32) -> Self {
        match val {
            0 => Waveform::Sine,
            1 => Waveform::Triangle,
            2 => Waveform::Ramp,
            3 => Waveform::Saw,
            4 => Waveform::Square,
            _ => Waveform::Sine,
        }
    }
}

/// Calculate LFO value at a given phase
pub fn calculate_lfo_value(phase: f32, waveform: Waveform) -> f32 {
    let phase = phase % 1.0;
    
    match waveform {
        Waveform::Sine => (phase * 2.0 * PI).sin(),
        Waveform::Triangle => {
            if phase < 0.25 {
                4.0 * phase
            } else if phase < 0.75 {
                2.0 - 4.0 * phase
            } else {
                4.0 * phase - 4.0
            }
        }
        Waveform::Ramp => 2.0 * phase - 1.0,
        Waveform::Saw => 1.0 - 2.0 * phase,
        Waveform::Square => {
            if phase < 0.5 {
                1.0
            } else {
                -1.0
            }
        }
    }
}

/// Update LFO bank phases based on time/BPM
pub fn update_lfo_phases(
    lfo_banks: &mut [LfoBank],
    bpm: f32,
    delta_time: f32,
) {
    for lfo in lfo_banks.iter_mut() {
        let rate = if lfo.tempo_sync {
            // Calculate rate based on BPM and beat division
            let division = lfo.division.clamp(0, BEAT_DIVISIONS.len() as i32 - 1) as usize;
            let beat_duration = 60.0 / bpm.max(1.0); // Duration of one beat in seconds
            let cycle_duration = beat_duration * BEAT_DIVISIONS[division];
            1.0 / cycle_duration
        } else {
            // Free rate (0-10 Hz typical range, mapped from 0-1)
            lfo.rate * 5.0 // Scale to reasonable range
        };
        
        // Update phase
        lfo.phase += rate * delta_time;
        lfo.phase = lfo.phase % 1.0;
    }
}

/// Get LFO value for a bank
pub fn get_lfo_output(lfo_banks: &[LfoBank], bank_index: i32) -> f32 {
    if bank_index < 0 || bank_index >= lfo_banks.len() as i32 {
        return 0.0;
    }
    
    let lfo = &lfo_banks[bank_index as usize];
    let waveform = Waveform::from(lfo.waveform);
    calculate_lfo_value(lfo.phase, waveform) * lfo.amplitude
}

/// Apply LFO modulation to a parameter value
pub fn apply_lfo_modulation(
    base_value: f32,
    lfo_value: f32,
    assignment: &LfoAssignment,
) -> f32 {
    if !assignment.enabled || assignment.bank_index < 0 {
        return base_value;
    }
    
    // Apply modulation: base + (LFO * amplitude)
    // LFO outputs -1 to 1, so this creates bipolar modulation
    base_value + (lfo_value * assignment.amplitude)
}

/// Apply LFOs to Block 1 parameters
pub fn apply_lfos_to_block1(
    params: &mut Block1Params,
    lfo_banks: &[LfoBank],
    lfo_map: &LfoParameterMap,
) {
    for (param_name, assignment) in lfo_map.iter() {
        if !assignment.enabled || assignment.bank_index < 0 {
            continue;
        }
        
        let lfo_value = get_lfo_output(lfo_banks, assignment.bank_index);
        let modulated = apply_lfo_modulation(0.0, lfo_value, assignment);
        
        // Apply to specific parameter
        match param_name.as_str() {
            // === CH1 Adjust LFOs ===
            "ch1_x_displace" => params.ch1_x_displace += modulated,
            "ch1_y_displace" => params.ch1_y_displace += modulated,
            "ch1_z_displace" => params.ch1_z_displace += modulated,
            "ch1_rotate" => params.ch1_rotate += modulated,
            "ch1_hsb_attenuate_x" => { params.ch1_hsb_attenuate_x += modulated; params.ch1_hsb_attenuate.x = params.ch1_hsb_attenuate_x; }
            "ch1_hsb_attenuate_y" => { params.ch1_hsb_attenuate_y += modulated; params.ch1_hsb_attenuate.y = params.ch1_hsb_attenuate_y; }
            "ch1_hsb_attenuate_z" => { params.ch1_hsb_attenuate_z += modulated; params.ch1_hsb_attenuate.z = params.ch1_hsb_attenuate_z; }
            "ch1_kaleidoscope_slice" => params.ch1_kaleidoscope_slice += modulated,
            "ch1_blur_amount" => params.ch1_blur_amount += modulated,
            "ch1_blur_radius" => params.ch1_blur_radius += modulated,
            "ch1_sharpen_amount" => params.ch1_sharpen_amount += modulated,
            "ch1_sharpen_radius" => params.ch1_sharpen_radius += modulated,
            
            // === CH2 Mix & Key LFOs ===
            "ch2_mix_amount" => params.ch2_mix_amount += modulated,
            "ch2_key_threshold" => params.ch2_key_threshold += modulated,
            "ch2_key_soft" => params.ch2_key_soft += modulated,
            
            // === CH2 Adjust LFOs ===
            "ch2_x_displace" => params.ch2_x_displace += modulated,
            "ch2_y_displace" => params.ch2_y_displace += modulated,
            "ch2_z_displace" => params.ch2_z_displace += modulated,
            "ch2_rotate" => params.ch2_rotate += modulated,
            "ch2_hsb_attenuate_x" => { params.ch2_hsb_attenuate_x += modulated; params.ch2_hsb_attenuate.x = params.ch2_hsb_attenuate_x; }
            "ch2_hsb_attenuate_y" => { params.ch2_hsb_attenuate_y += modulated; params.ch2_hsb_attenuate.y = params.ch2_hsb_attenuate_y; }
            "ch2_hsb_attenuate_z" => { params.ch2_hsb_attenuate_z += modulated; params.ch2_hsb_attenuate.z = params.ch2_hsb_attenuate_z; }
            "ch2_kaleidoscope_slice" => params.ch2_kaleidoscope_slice += modulated,
            "ch2_blur_amount" => params.ch2_blur_amount += modulated,
            "ch2_blur_radius" => params.ch2_blur_radius += modulated,
            "ch2_sharpen_amount" => params.ch2_sharpen_amount += modulated,
            "ch2_sharpen_radius" => params.ch2_sharpen_radius += modulated,
            
            // === FB1 Mix & Key LFOs ===
            "fb1_mix_amount" => params.fb1_mix_amount += modulated,
            "fb1_key_threshold" => params.fb1_key_threshold += modulated,
            "fb1_key_soft" => params.fb1_key_soft += modulated,
            
            // === FB1 Geo1 LFOs ===
            "fb1_x_displace" => params.fb1_x_displace += modulated,
            "fb1_y_displace" => params.fb1_y_displace += modulated,
            "fb1_z_displace" => params.fb1_z_displace += modulated,
            "fb1_rotate" => params.fb1_rotate += modulated,
            
            // === FB1 Geo2 LFOs (Shear Matrix) ===
            "fb1_shear_matrix_x" => { params.fb1_shear_matrix_x += modulated; params.fb1_shear_matrix.x = params.fb1_shear_matrix_x; }
            "fb1_shear_matrix_y" => { params.fb1_shear_matrix_y += modulated; params.fb1_shear_matrix.y = params.fb1_shear_matrix_y; }
            "fb1_shear_matrix_z" => { params.fb1_shear_matrix_z += modulated; params.fb1_shear_matrix.z = params.fb1_shear_matrix_z; }
            "fb1_shear_matrix_w" => { params.fb1_shear_matrix_w += modulated; params.fb1_shear_matrix.w = params.fb1_shear_matrix_w; }
            "fb1_kaleidoscope_slice" => params.fb1_kaleidoscope_slice += modulated,
            
            // === FB1 Color LFOs ===
            "fb1_hsb_offset_x" => { params.fb1_hsb_offset_x += modulated; params.fb1_hsb_offset.x = params.fb1_hsb_offset_x; }
            "fb1_hsb_offset_y" => { params.fb1_hsb_offset_y += modulated; params.fb1_hsb_offset.y = params.fb1_hsb_offset_y; }
            "fb1_hsb_offset_z" => { params.fb1_hsb_offset_z += modulated; params.fb1_hsb_offset.z = params.fb1_hsb_offset_z; }
            "fb1_hsb_attenuate_x" => { params.fb1_hsb_attenuate_x += modulated; params.fb1_hsb_attenuate.x = params.fb1_hsb_attenuate_x; }
            "fb1_hsb_attenuate_y" => { params.fb1_hsb_attenuate_y += modulated; params.fb1_hsb_attenuate.y = params.fb1_hsb_attenuate_y; }
            "fb1_hsb_attenuate_z" => { params.fb1_hsb_attenuate_z += modulated; params.fb1_hsb_attenuate.z = params.fb1_hsb_attenuate_z; }
            
            // === FB1 Delay LFO ===
            "fb1_delay_time" => params.fb1_delay_time += modulated as i32,
            
            _ => {}
        }
    }
}

/// Apply LFOs to Block 2 parameters
pub fn apply_lfos_to_block2(
    params: &mut Block2Params,
    lfo_banks: &[LfoBank],
    lfo_map: &LfoParameterMap,
) {
    for (param_name, assignment) in lfo_map.iter() {
        if !assignment.enabled || assignment.bank_index < 0 {
            continue;
        }
        
        let lfo_value = get_lfo_output(lfo_banks, assignment.bank_index);
        let modulated = apply_lfo_modulation(0.0, lfo_value, assignment);
        
        match param_name.as_str() {
            // === Block2 Input Adjust LFOs ===
            "block2_input_x_displace" => params.block2_input_x_displace += modulated,
            "block2_input_y_displace" => params.block2_input_y_displace += modulated,
            "block2_input_z_displace" => params.block2_input_z_displace += modulated,
            "block2_input_rotate" => params.block2_input_rotate += modulated,
            "block2_input_hsb_attenuate_x" => { params.block2_input_hsb_attenuate_x += modulated; params.block2_input_hsb_attenuate.x = params.block2_input_hsb_attenuate_x; }
            "block2_input_hsb_attenuate_y" => { params.block2_input_hsb_attenuate_y += modulated; params.block2_input_hsb_attenuate.y = params.block2_input_hsb_attenuate_y; }
            "block2_input_hsb_attenuate_z" => { params.block2_input_hsb_attenuate_z += modulated; params.block2_input_hsb_attenuate.z = params.block2_input_hsb_attenuate_z; }
            "block2_input_kaleidoscope_slice" => params.block2_input_kaleidoscope_slice += modulated,
            "block2_input_blur_amount" => params.block2_input_blur_amount += modulated,
            "block2_input_blur_radius" => params.block2_input_blur_radius += modulated,
            "block2_input_sharpen_amount" => params.block2_input_sharpen_amount += modulated,
            "block2_input_sharpen_radius" => params.block2_input_sharpen_radius += modulated,
            
            // === FB2 Mix & Key LFOs ===
            "fb2_mix_amount" => params.fb2_mix_amount += modulated,
            "fb2_key_threshold" => params.fb2_key_threshold += modulated,
            "fb2_key_soft" => params.fb2_key_soft += modulated,
            
            // === FB2 Geo1 LFOs ===
            "fb2_x_displace" => params.fb2_x_displace += modulated,
            "fb2_y_displace" => params.fb2_y_displace += modulated,
            "fb2_z_displace" => params.fb2_z_displace += modulated,
            "fb2_rotate" => params.fb2_rotate += modulated,
            
            // === FB2 Geo2 LFOs (Shear Matrix) ===
            "fb2_shear_matrix_x" => { params.fb2_shear_matrix_x += modulated; params.fb2_shear_matrix.x = params.fb2_shear_matrix_x; }
            "fb2_shear_matrix_y" => { params.fb2_shear_matrix_y += modulated; params.fb2_shear_matrix.y = params.fb2_shear_matrix_y; }
            "fb2_shear_matrix_z" => { params.fb2_shear_matrix_z += modulated; params.fb2_shear_matrix.z = params.fb2_shear_matrix_z; }
            "fb2_shear_matrix_w" => { params.fb2_shear_matrix_w += modulated; params.fb2_shear_matrix.w = params.fb2_shear_matrix_w; }
            "fb2_kaleidoscope_slice" => params.fb2_kaleidoscope_slice += modulated,
            
            // === FB2 Color LFOs ===
            "fb2_hsb_offset_x" => { params.fb2_hsb_offset_x += modulated; params.fb2_hsb_offset.x = params.fb2_hsb_offset_x; }
            "fb2_hsb_offset_y" => { params.fb2_hsb_offset_y += modulated; params.fb2_hsb_offset.y = params.fb2_hsb_offset_y; }
            "fb2_hsb_offset_z" => { params.fb2_hsb_offset_z += modulated; params.fb2_hsb_offset.z = params.fb2_hsb_offset_z; }
            "fb2_hsb_attenuate_x" => { params.fb2_hsb_attenuate_x += modulated; params.fb2_hsb_attenuate.x = params.fb2_hsb_attenuate_x; }
            "fb2_hsb_attenuate_y" => { params.fb2_hsb_attenuate_y += modulated; params.fb2_hsb_attenuate.y = params.fb2_hsb_attenuate_y; }
            "fb2_hsb_attenuate_z" => { params.fb2_hsb_attenuate_z += modulated; params.fb2_hsb_attenuate.z = params.fb2_hsb_attenuate_z; }
            
            // === FB2 Delay LFO ===
            "fb2_delay_time" => params.fb2_delay_time += modulated as i32,
            
            _ => {}
        }
    }
}

/// Apply LFOs to Block 3 parameters
pub fn apply_lfos_to_block3(
    params: &mut Block3Params,
    lfo_banks: &[LfoBank],
    lfo_map: &LfoParameterMap,
) {
    for (param_name, assignment) in lfo_map.iter() {
        if !assignment.enabled || assignment.bank_index < 0 {
            continue;
        }
        
        let lfo_value = get_lfo_output(lfo_banks, assignment.bank_index);
        let modulated = apply_lfo_modulation(0.0, lfo_value, assignment);
        
        match param_name.as_str() {
            // === Block 1 Geo1 LFOs ===
            "block1_x_displace" => params.block1_x_displace += modulated,
            "block1_y_displace" => params.block1_y_displace += modulated,
            "block1_z_displace" => params.block1_z_displace += modulated,
            "block1_rotate" => params.block1_rotate += modulated,
            
            // === Block 1 Geo2 LFOs (Shear Matrix) ===
            "block1_shear_matrix_x" => { params.block1_shear_matrix_x += modulated; params.block1_shear_matrix.x = params.block1_shear_matrix_x; }
            "block1_shear_matrix_y" => { params.block1_shear_matrix_y += modulated; params.block1_shear_matrix.y = params.block1_shear_matrix_y; }
            "block1_shear_matrix_z" => { params.block1_shear_matrix_z += modulated; params.block1_shear_matrix.z = params.block1_shear_matrix_z; }
            "block1_shear_matrix_w" => { params.block1_shear_matrix_w += modulated; params.block1_shear_matrix.w = params.block1_shear_matrix_w; }
            "block1_kaleidoscope_slice" => params.block1_kaleidoscope_slice += modulated,
            
            // === Block 1 Colorize LFOs (Bands 1-5) ===
            "block1_colorize_band1_x" => { params.block1_colorize_band1_x += modulated; params.block1_colorize_band1.x = params.block1_colorize_band1_x; }
            "block1_colorize_band1_y" => { params.block1_colorize_band1_y += modulated; params.block1_colorize_band1.y = params.block1_colorize_band1_y; }
            "block1_colorize_band1_z" => { params.block1_colorize_band1_z += modulated; params.block1_colorize_band1.z = params.block1_colorize_band1_z; }
            "block1_colorize_band2_x" => { params.block1_colorize_band2_x += modulated; params.block1_colorize_band2.x = params.block1_colorize_band2_x; }
            "block1_colorize_band2_y" => { params.block1_colorize_band2_y += modulated; params.block1_colorize_band2.y = params.block1_colorize_band2_y; }
            "block1_colorize_band2_z" => { params.block1_colorize_band2_z += modulated; params.block1_colorize_band2.z = params.block1_colorize_band2_z; }
            "block1_colorize_band3_x" => { params.block1_colorize_band3_x += modulated; params.block1_colorize_band3.x = params.block1_colorize_band3_x; }
            "block1_colorize_band3_y" => { params.block1_colorize_band3_y += modulated; params.block1_colorize_band3.y = params.block1_colorize_band3_y; }
            "block1_colorize_band3_z" => { params.block1_colorize_band3_z += modulated; params.block1_colorize_band3.z = params.block1_colorize_band3_z; }
            "block1_colorize_band4_x" => { params.block1_colorize_band4_x += modulated; params.block1_colorize_band4.x = params.block1_colorize_band4_x; }
            "block1_colorize_band4_y" => { params.block1_colorize_band4_y += modulated; params.block1_colorize_band4.y = params.block1_colorize_band4_y; }
            "block1_colorize_band4_z" => { params.block1_colorize_band4_z += modulated; params.block1_colorize_band4.z = params.block1_colorize_band4_z; }
            "block1_colorize_band5_x" => { params.block1_colorize_band5_x += modulated; params.block1_colorize_band5.x = params.block1_colorize_band5_x; }
            "block1_colorize_band5_y" => { params.block1_colorize_band5_y += modulated; params.block1_colorize_band5.y = params.block1_colorize_band5_y; }
            "block1_colorize_band5_z" => { params.block1_colorize_band5_z += modulated; params.block1_colorize_band5.z = params.block1_colorize_band5_z; }
            
            // === Block 2 Geo1 LFOs ===
            "block2_x_displace" => params.block2_x_displace += modulated,
            "block2_y_displace" => params.block2_y_displace += modulated,
            "block2_z_displace" => params.block2_z_displace += modulated,
            "block2_rotate" => params.block2_rotate += modulated,
            
            // === Block 2 Geo2 LFOs (Shear Matrix) ===
            "block2_shear_matrix_x" => { params.block2_shear_matrix_x += modulated; params.block2_shear_matrix.x = params.block2_shear_matrix_x; }
            "block2_shear_matrix_y" => { params.block2_shear_matrix_y += modulated; params.block2_shear_matrix.y = params.block2_shear_matrix_y; }
            "block2_shear_matrix_z" => { params.block2_shear_matrix_z += modulated; params.block2_shear_matrix.z = params.block2_shear_matrix_z; }
            "block2_shear_matrix_w" => { params.block2_shear_matrix_w += modulated; params.block2_shear_matrix.w = params.block2_shear_matrix_w; }
            "block2_kaleidoscope_slice" => params.block2_kaleidoscope_slice += modulated,
            
            // === Block 2 Colorize LFOs (Bands 1-5) ===
            "block2_colorize_band1_x" => { params.block2_colorize_band1_x += modulated; params.block2_colorize_band1.x = params.block2_colorize_band1_x; }
            "block2_colorize_band1_y" => { params.block2_colorize_band1_y += modulated; params.block2_colorize_band1.y = params.block2_colorize_band1_y; }
            "block2_colorize_band1_z" => { params.block2_colorize_band1_z += modulated; params.block2_colorize_band1.z = params.block2_colorize_band1_z; }
            "block2_colorize_band2_x" => { params.block2_colorize_band2_x += modulated; params.block2_colorize_band2.x = params.block2_colorize_band2_x; }
            "block2_colorize_band2_y" => { params.block2_colorize_band2_y += modulated; params.block2_colorize_band2.y = params.block2_colorize_band2_y; }
            "block2_colorize_band2_z" => { params.block2_colorize_band2_z += modulated; params.block2_colorize_band2.z = params.block2_colorize_band2_z; }
            "block2_colorize_band3_x" => { params.block2_colorize_band3_x += modulated; params.block2_colorize_band3.x = params.block2_colorize_band3_x; }
            "block2_colorize_band3_y" => { params.block2_colorize_band3_y += modulated; params.block2_colorize_band3.y = params.block2_colorize_band3_y; }
            "block2_colorize_band3_z" => { params.block2_colorize_band3_z += modulated; params.block2_colorize_band3.z = params.block2_colorize_band3_z; }
            "block2_colorize_band4_x" => { params.block2_colorize_band4_x += modulated; params.block2_colorize_band4.x = params.block2_colorize_band4_x; }
            "block2_colorize_band4_y" => { params.block2_colorize_band4_y += modulated; params.block2_colorize_band4.y = params.block2_colorize_band4_y; }
            "block2_colorize_band4_z" => { params.block2_colorize_band4_z += modulated; params.block2_colorize_band4.z = params.block2_colorize_band4_z; }
            "block2_colorize_band5_x" => { params.block2_colorize_band5_x += modulated; params.block2_colorize_band5.x = params.block2_colorize_band5_x; }
            "block2_colorize_band5_y" => { params.block2_colorize_band5_y += modulated; params.block2_colorize_band5.y = params.block2_colorize_band5_y; }
            "block2_colorize_band5_z" => { params.block2_colorize_band5_z += modulated; params.block2_colorize_band5.z = params.block2_colorize_band5_z; }
            
            // === Matrix Mix LFOs ===
            "matrix_mix_r_to_r" => { params.matrix_mix_r_to_r += modulated; params.bg_rgb_into_fg_red.x = params.matrix_mix_r_to_r; }
            "matrix_mix_r_to_g" => { params.matrix_mix_r_to_g += modulated; params.bg_rgb_into_fg_green.x = params.matrix_mix_r_to_g; }
            "matrix_mix_r_to_b" => { params.matrix_mix_r_to_b += modulated; params.bg_rgb_into_fg_blue.x = params.matrix_mix_r_to_b; }
            "matrix_mix_g_to_r" => { params.matrix_mix_g_to_r += modulated; params.bg_rgb_into_fg_red.y = params.matrix_mix_g_to_r; }
            "matrix_mix_g_to_g" => { params.matrix_mix_g_to_g += modulated; params.bg_rgb_into_fg_green.y = params.matrix_mix_g_to_g; }
            "matrix_mix_g_to_b" => { params.matrix_mix_g_to_b += modulated; params.bg_rgb_into_fg_blue.y = params.matrix_mix_g_to_b; }
            "matrix_mix_b_to_r" => { params.matrix_mix_b_to_r += modulated; params.bg_rgb_into_fg_red.z = params.matrix_mix_b_to_r; }
            "matrix_mix_b_to_g" => { params.matrix_mix_b_to_g += modulated; params.bg_rgb_into_fg_green.z = params.matrix_mix_b_to_g; }
            "matrix_mix_b_to_b" => { params.matrix_mix_b_to_b += modulated; params.bg_rgb_into_fg_blue.z = params.matrix_mix_b_to_b; }
            
            // === Final Mix & Key LFOs ===
            "final_mix_amount" => params.final_mix_amount += modulated,
            "final_key_threshold" => params.final_key_threshold += modulated,
            "final_key_soft" => params.final_key_soft += modulated,
            
            _ => {}
        }
    }
}

/// Main LFO update function - updates phases and applies modulation
pub fn update_lfo_modulation(
    state: &mut SharedState,
    bpm: f32,
    delta_time: f32,
) {
    // Update LFO phases
    update_lfo_phases(&mut state.lfo_banks, bpm, delta_time);
    
    // Apply LFO modulation to parameters
    // Note: We create copies and modify them, then the engine uses the modified values
    apply_lfos_to_block1(&mut state.block1, &state.lfo_banks, &state.block1_lfo_map);
    apply_lfos_to_block2(&mut state.block2, &state.lfo_banks, &state.block2_lfo_map);
    apply_lfos_to_block3(&mut state.block3, &state.lfo_banks, &state.block3_lfo_map);
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sine_waveform() {
        assert!((calculate_lfo_value(0.0, Waveform::Sine) - 0.0).abs() < 0.001);
        assert!((calculate_lfo_value(0.25, Waveform::Sine) - 1.0).abs() < 0.001);
        assert!((calculate_lfo_value(0.5, Waveform::Sine) - 0.0).abs() < 0.001);
        assert!((calculate_lfo_value(0.75, Waveform::Sine) - (-1.0)).abs() < 0.001);
    }
    
    #[test]
    fn test_square_waveform() {
        assert_eq!(calculate_lfo_value(0.0, Waveform::Square), 1.0);
        assert_eq!(calculate_lfo_value(0.25, Waveform::Square), 1.0);
        assert_eq!(calculate_lfo_value(0.5, Waveform::Square), -1.0);
        assert_eq!(calculate_lfo_value(0.75, Waveform::Square), -1.0);
    }
    
    #[test]
    fn test_triangle_waveform() {
        assert!((calculate_lfo_value(0.0, Waveform::Triangle) - 0.0).abs() < 0.001);
        assert!((calculate_lfo_value(0.25, Waveform::Triangle) - 1.0).abs() < 0.001);
        assert!((calculate_lfo_value(0.5, Waveform::Triangle) - 0.0).abs() < 0.001);
        assert!((calculate_lfo_value(0.75, Waveform::Triangle) - (-1.0)).abs() < 0.001);
    }
}
