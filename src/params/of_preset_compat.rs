//! # OpenFrameworks Preset Compatibility
//!
//! This module provides compatibility with the original BLUEJAY_WAAAVES
//! OpenFrameworks preset format (gwSaveStateXXX.json files).
//!
//! OF presets use a flat array-based structure with 16-element arrays
//! for each parameter group, while the Rust version uses structured types.

use crate::params::{Block1Params, Block2Params, Block3Params};
use glam::{Vec3, Vec4};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// OF-style parameter group (16-element array)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OfParamGroup {
    #[serde(default)]
    pub values: Vec<f32>,
    #[serde(default)]
    pub discrete: Vec<serde_json::Value>, // Can be bool, int, etc.
    #[serde(default)]
    pub lfo: Vec<f32>,
}

/// OF Block 1 data structure
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OfBlock1Data {
    #[serde(rename = "ch1Adjust", default)]
    pub ch1_adjust: Vec<f32>,
    #[serde(rename = "ch1AdjustDiscrete", default)]
    pub ch1_adjust_discrete: Vec<serde_json::Value>,
    #[serde(rename = "ch1AdjustLfo", default)]
    pub ch1_adjust_lfo: Vec<f32>,
    
    #[serde(rename = "ch2MixAndKey", default)]
    pub ch2_mix_and_key: Vec<f32>,
    #[serde(rename = "ch2MixAndKeyDiscrete", default)]
    pub ch2_mix_and_key_discrete: Vec<serde_json::Value>,
    #[serde(rename = "ch2MixAndKeyLfo", default)]
    pub ch2_mix_and_key_lfo: Vec<f32>,
    
    #[serde(rename = "ch2Adjust", default)]
    pub ch2_adjust: Vec<f32>,
    #[serde(rename = "ch2AdjustDiscrete", default)]
    pub ch2_adjust_discrete: Vec<serde_json::Value>,
    #[serde(rename = "ch2AdjustLfo", default)]
    pub ch2_adjust_lfo: Vec<f32>,
    
    #[serde(rename = "fb1MixAndKey", default)]
    pub fb1_mix_and_key: Vec<f32>,
    #[serde(rename = "fb1MixAndKeyDiscrete", default)]
    pub fb1_mix_and_key_discrete: Vec<serde_json::Value>,
    #[serde(rename = "fb1MixAndKeyLfo", default)]
    pub fb1_mix_and_key_lfo: Vec<f32>,
    
    #[serde(rename = "fb1Geo1", default)]
    pub fb1_geo1: Vec<f32>,
    #[serde(rename = "fb1Geo1Discrete", default)]
    pub fb1_geo1_discrete: Vec<serde_json::Value>,
    #[serde(rename = "fb1Geo1Lfo1", default)]
    pub fb1_geo1_lfo1: Vec<f32>,
    #[serde(rename = "fb1Geo1Lfo2", default)]
    pub fb1_geo1_lfo2: Vec<f32>,
    
    #[serde(rename = "fb1Color1", default)]
    pub fb1_color1: Vec<f32>,
    #[serde(rename = "fb1Color1Discrete", default)]
    pub fb1_color1_discrete: Vec<serde_json::Value>,
    #[serde(rename = "fb1Color1Lfo1", default)]
    pub fb1_color1_lfo1: Vec<f32>,
    
    #[serde(rename = "fb1Filters", default)]
    pub fb1_filters: Vec<f32>,
    #[serde(rename = "fb1FiltersDiscrete", default)]
    pub fb1_filters_discrete: Vec<serde_json::Value>,
    
    #[serde(rename = "b1GeometricalAnimations", default)]
    pub b1_geo_animations: Vec<serde_json::Value>,
    #[serde(rename = "b1_extraWhatever", default)]
    pub b1_extra: Vec<serde_json::Value>,
}

/// OF Block 2 data structure
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OfBlock2Data {
    #[serde(rename = "block2InputAdjust", default)]
    pub input_adjust: Vec<f32>,
    #[serde(rename = "block2InputAdjustDiscrete", default)]
    pub input_adjust_discrete: Vec<serde_json::Value>,
    #[serde(rename = "block2InputAdjustLfo", default)]
    pub input_adjust_lfo: Vec<f32>,
    #[serde(rename = "block2InputAdjustLfoDiscrete", default)]
    pub input_adjust_lfo_discrete: Vec<serde_json::Value>,
    
    #[serde(rename = "fb2MixAndKey", default)]
    pub fb2_mix_and_key: Vec<f32>,
    #[serde(rename = "fb2MixAndKeyDiscrete", default)]
    pub fb2_mix_and_key_discrete: Vec<serde_json::Value>,
    #[serde(rename = "fb2MixAndKeyLfo", default)]
    pub fb2_mix_and_key_lfo: Vec<f32>,
    
    #[serde(rename = "fb2Geo1", default)]
    pub fb2_geo1: Vec<f32>,
    #[serde(rename = "fb2Geo1Discrete", default)]
    pub fb2_geo1_discrete: Vec<serde_json::Value>,
    #[serde(rename = "fb2Geo1Lfo1", default)]
    pub fb2_geo1_lfo1: Vec<f32>,
    #[serde(rename = "fb2Geo1Lfo2", default)]
    pub fb2_geo1_lfo2: Vec<f32>,
    
    #[serde(rename = "fb2Color1", default)]
    pub fb2_color1: Vec<f32>,
    #[serde(rename = "fb2Color1Discrete", default)]
    pub fb2_color1_discrete: Vec<serde_json::Value>,
    #[serde(rename = "fb2Color1Lfo1", default)]
    pub fb2_color1_lfo1: Vec<f32>,
    
    #[serde(rename = "fb2Filters", default)]
    pub fb2_filters: Vec<f32>,
    #[serde(rename = "fb2FiltersDiscrete", default)]
    pub fb2_filters_discrete: Vec<serde_json::Value>,
    
    #[serde(rename = "b2GeometricalAnimations", default)]
    pub b2_geo_animations: Vec<serde_json::Value>,
    #[serde(rename = "b2_extraWhatever", default)]
    pub b2_extra: Vec<serde_json::Value>,
}

/// OF Block 3 data structure
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OfBlock3Data {
    #[serde(rename = "block1Geo", default)]
    pub block1_geo: Vec<f32>,
    #[serde(rename = "block1GeoDiscrete", default)]
    pub block1_geo_discrete: Vec<serde_json::Value>,
    #[serde(rename = "block1Geo1Lfo1", default)]
    pub block1_geo_lfo1: Vec<f32>,
    #[serde(rename = "block1Geo1Lfo2", default)]
    pub block1_geo_lfo2: Vec<f32>,
    
    #[serde(rename = "block1Colorize", default)]
    pub block1_colorize: Vec<f32>,
    #[serde(rename = "block1ColorizeDiscrete", default)]
    pub block1_colorize_discrete: Vec<serde_json::Value>,
    #[serde(rename = "block1ColorizeLfo1", default)]
    pub block1_colorize_lfo1: Vec<f32>,
    #[serde(rename = "block1ColorizeLfo2", default)]
    pub block1_colorize_lfo2: Vec<f32>,
    #[serde(rename = "block1ColorizeLfo3", default)]
    pub block1_colorize_lfo3: Vec<f32>,
    
    #[serde(rename = "block1Filters", default)]
    pub block1_filters: Vec<f32>,
    #[serde(rename = "block1FiltersDiscrete", default)]
    pub block1_filters_discrete: Vec<serde_json::Value>,
    
    #[serde(rename = "block2Geo", default)]
    pub block2_geo: Vec<f32>,
    #[serde(rename = "block2GeoDiscrete", default)]
    pub block2_geo_discrete: Vec<serde_json::Value>,
    #[serde(rename = "block2Geo1Lfo1", default)]
    pub block2_geo_lfo1: Vec<f32>,
    #[serde(rename = "block2Geo1Lfo2", default)]
    pub block2_geo_lfo2: Vec<f32>,
    
    #[serde(rename = "block2Colorize", default)]
    pub block2_colorize: Vec<f32>,
    #[serde(rename = "block2ColorizeDiscrete", default)]
    pub block2_colorize_discrete: Vec<serde_json::Value>,
    #[serde(rename = "block2ColorizeLfo1", default)]
    pub block2_colorize_lfo1: Vec<f32>,
    #[serde(rename = "block2ColorizeLfo2", default)]
    pub block2_colorize_lfo2: Vec<f32>,
    #[serde(rename = "block2ColorizeLfo3", default)]
    pub block2_colorize_lfo3: Vec<f32>,
    
    #[serde(rename = "block2Filters", default)]
    pub block2_filters: Vec<f32>,
    #[serde(rename = "block2FiltersDiscrete", default)]
    pub block2_filters_discrete: Vec<serde_json::Value>,
    
    #[serde(rename = "finalMixAndKey", default)]
    pub final_mix_and_key: Vec<f32>,
    #[serde(rename = "finalMixAndKeyDiscrete", default)]
    pub final_mix_and_key_discrete: Vec<serde_json::Value>,
    #[serde(rename = "finalMixAndKeyLfo", default)]
    pub final_mix_and_key_lfo: Vec<f32>,
    
    #[serde(rename = "matrixMix", default)]
    pub matrix_mix: Vec<f32>,
    #[serde(rename = "matrixMixDiscrete", default)]
    pub matrix_mix_discrete: Vec<serde_json::Value>,
    #[serde(rename = "matrixMixLfo1", default)]
    pub matrix_mix_lfo1: Vec<f32>,
    #[serde(rename = "matrixMixLfo2", default)]
    pub matrix_mix_lfo2: Vec<f32>,
    
    #[serde(rename = "b3_extraWhatever", default)]
    pub b3_extra: Vec<serde_json::Value>,
}

/// OF Macro assignment
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OfMacroBlock {
    #[serde(flatten)]
    pub assignments: HashMap<String, i32>,
}

/// Complete OF preset structure
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OfPreset {
    #[serde(rename = "BLOCK_1")]
    pub block1: OfBlock1Data,
    #[serde(rename = "BLOCK_2")]
    pub block2: OfBlock2Data,
    #[serde(rename = "BLOCK_3")]
    pub block3: OfBlock3Data,
    
    // Macros 0-15
    #[serde(rename = "MACRO0", default)]
    pub macro0: Option<OfMacroBlock>,
    #[serde(rename = "MACRO1", default)]
    pub macro1: Option<OfMacroBlock>,
    #[serde(rename = "MACRO2", default)]
    pub macro2: Option<OfMacroBlock>,
    #[serde(rename = "MACRO3", default)]
    pub macro3: Option<OfMacroBlock>,
    #[serde(rename = "MACRO4", default)]
    pub macro4: Option<OfMacroBlock>,
    #[serde(rename = "MACRO5", default)]
    pub macro5: Option<OfMacroBlock>,
    #[serde(rename = "MACRO6", default)]
    pub macro6: Option<OfMacroBlock>,
    #[serde(rename = "MACRO7", default)]
    pub macro7: Option<OfMacroBlock>,
    #[serde(rename = "MACRO8", default)]
    pub macro8: Option<OfMacroBlock>,
    #[serde(rename = "MACRO9", default)]
    pub macro9: Option<OfMacroBlock>,
    #[serde(rename = "MACRO10", default)]
    pub macro10: Option<OfMacroBlock>,
    #[serde(rename = "MACRO11", default)]
    pub macro11: Option<OfMacroBlock>,
    #[serde(rename = "MACRO12", default)]
    pub macro12: Option<OfMacroBlock>,
    #[serde(rename = "MACRO13", default)]
    pub macro13: Option<OfMacroBlock>,
    #[serde(rename = "MACRO14", default)]
    pub macro14: Option<OfMacroBlock>,
    #[serde(rename = "MACRO15", default)]
    pub macro15: Option<OfMacroBlock>,
}

impl OfPreset {
    /// Load an OF preset from JSON string
    pub fn from_json(json: &str) -> anyhow::Result<Self> {
        let preset: OfPreset = serde_json::from_str(json)?;
        Ok(preset)
    }
    
    /// Load an OF preset from a file
    pub fn from_file(path: &std::path::Path) -> anyhow::Result<Self> {
        let json = std::fs::read_to_string(path)?;
        Self::from_json(&json)
    }
    
    /// Convert OF preset to Rust Block1Params
    pub fn to_block1_params(&self) -> Block1Params {
        let mut p = Block1Params::default();
        let b = &self.block1;
        
        // CH1 Adjust: [x_displace, y_displace, z_displace, rotate, kaleidoscope_amount, 
        //               kaleidoscope_slice, h_mirror, v_mirror, ...]
        if b.ch1_adjust.len() >= 4 {
            p.ch1_x_displace = b.ch1_adjust[0];
            p.ch1_y_displace = b.ch1_adjust[1];
            p.ch1_z_displace = b.ch1_adjust[2];
            p.ch1_rotate = b.ch1_adjust[3];
        }
        if b.ch1_adjust.len() >= 6 {
            p.ch1_kaleidoscope_amount = b.ch1_adjust[4];
            p.ch1_kaleidoscope_slice = b.ch1_adjust[5];
        }
        
        // CH2 Mix and Key
        if b.ch2_mix_and_key.len() >= 3 {
            p.ch2_mix_amount = b.ch2_mix_and_key[0];
            // Index 1-2 might be key values
        }
        
        // CH2 Adjust
        if b.ch2_adjust.len() >= 4 {
            p.ch2_x_displace = b.ch2_adjust[0];
            p.ch2_y_displace = b.ch2_adjust[1];
            // Index 2 might be z_displace
            p.ch2_rotate = b.ch2_adjust[3];
        }
        
        // FB1 Mix and Key
        if b.fb1_mix_and_key.len() >= 6 {
            p.fb1_mix_amount = b.fb1_mix_and_key[0];
            // Index 1-5: mix type, overflow, etc.
        }
        
        // FB1 Geo
        if b.fb1_geo1.len() >= 6 {
            p.fb1_x_displace = b.fb1_geo1[0];
            p.fb1_y_displace = b.fb1_geo1[1];
            p.fb1_z_displace = b.fb1_geo1[2];
            // Index 3 might be another rotate
            p.fb1_rotate = b.fb1_geo1[4];
            // Index 5: kaleidoscope
        }
        
        // FB1 Color - HSB attenuate
        if b.fb1_color1.len() >= 5 {
            p.fb1_hsb_attenuate = Vec3::new(
                b.fb1_color1[0],
                b.fb1_color1[1],
                b.fb1_color1[2],
            );
            // Index 3-4: HSB offset
        }
        
        // FB1 Filters
        if b.fb1_filters.len() >= 10 {
            p.fb1_blur_amount = b.fb1_filters[0];
            p.fb1_blur_radius = b.fb1_filters[1];
            p.fb1_sharpen_amount = b.fb1_filters[2];
            p.fb1_sharpen_radius = b.fb1_filters[3];
            p.fb1_filters_boost = b.fb1_filters[4];
            // Index 5-9: deconvolves and other filters
        }
        
        // Handle discrete values (booleans, enums)
        if let Some(val) = get_discrete_bool(&b.ch1_adjust_discrete, 1) {
            p.ch1_h_mirror = val;
        }
        if let Some(val) = get_discrete_bool(&b.ch1_adjust_discrete, 2) {
            p.ch1_v_mirror = val;
        }
        
        p
    }
    
    /// Convert OF preset to Rust Block2Params
    pub fn to_block2_params(&self) -> Block2Params {
        let mut p = Block2Params::default();
        let b = &self.block2;
        
        // Input Adjust
        if b.input_adjust.len() >= 4 {
            p.block2_input_x_displace = b.input_adjust[0];
            p.block2_input_y_displace = b.input_adjust[1];
            p.block2_input_z_displace = b.input_adjust[2];
            p.block2_input_rotate = b.input_adjust[3];
        }
        
        // FB2 Mix and Key
        if b.fb2_mix_and_key.len() >= 6 {
            p.fb2_mix_amount = b.fb2_mix_and_key[0];
            // Index 1-5: mix type, key values, etc.
        }
        
        // FB2 Geo
        if b.fb2_geo1.len() >= 6 {
            p.fb2_x_displace = b.fb2_geo1[0];
            p.fb2_y_displace = b.fb2_geo1[1];
            p.fb2_z_displace = b.fb2_geo1[2];
            p.fb2_rotate = b.fb2_geo1[4];
        }
        
        // FB2 Color
        if b.fb2_color1.len() >= 5 {
            p.fb2_hsb_attenuate = Vec3::new(
                b.fb2_color1[0],
                b.fb2_color1[1],
                b.fb2_color1[2],
            );
        }
        
        // FB2 Filters
        if b.fb2_filters.len() >= 10 {
            p.fb2_blur_amount = b.fb2_filters[0];
            p.fb2_blur_radius = b.fb2_filters[1];
            p.fb2_sharpen_amount = b.fb2_filters[2];
            p.fb2_sharpen_radius = b.fb2_filters[3];
            p.fb2_filters_boost = b.fb2_filters[4];
        }
        
        p
    }
    
    /// Convert OF preset to Rust Block3Params
    pub fn to_block3_params(&self) -> Block3Params {
        let mut p = Block3Params::default();
        let b = &self.block3;
        
        // Block 1 Geo (re-process)
        if b.block1_geo.len() >= 6 {
            p.block1_x_displace = b.block1_geo[0];
            p.block1_y_displace = b.block1_geo[1];
            p.block1_z_displace = b.block1_geo[2];
            p.block1_rotate = b.block1_geo[4];
        }
        
        // Block 2 Geo (re-process)
        if b.block2_geo.len() >= 6 {
            p.block2_x_displace = b.block2_geo[0];
            p.block2_y_displace = b.block2_geo[1];
            p.block2_z_displace = b.block2_geo[2];
            p.block2_rotate = b.block2_geo[4];
        }
        
        // Block 1 Filters
        if b.block1_filters.len() >= 10 {
            p.block1_blur_amount = b.block1_filters[0];
            p.block1_blur_radius = b.block1_filters[1];
            p.block1_sharpen_amount = b.block1_filters[2];
            p.block1_sharpen_radius = b.block1_filters[3];
            p.block1_filters_boost = b.block1_filters[4];
        }
        
        // Block 2 Filters
        if b.block2_filters.len() >= 10 {
            p.block2_blur_amount = b.block2_filters[0];
            p.block2_blur_radius = b.block2_filters[1];
            p.block2_sharpen_amount = b.block2_filters[2];
            p.block2_sharpen_radius = b.block2_filters[3];
            p.block2_filters_boost = b.block2_filters[4];
        }
        
        // Final Mix
        if b.final_mix_and_key.len() >= 1 {
            p.final_mix_amount = b.final_mix_and_key[0];
        }
        
        // Matrix Mix
        if b.matrix_mix.len() >= 4 {
            // Matrix values for mixing
        }
        
        p
    }
}

/// Helper to get a boolean from discrete values
fn get_discrete_bool(values: &[serde_json::Value], index: usize) -> Option<bool> {
    values.get(index).and_then(|v| v.as_bool())
}

/// Helper to get an integer from discrete values
fn get_discrete_int(values: &[serde_json::Value], index: usize) -> Option<i32> {
    values.get(index).and_then(|v| {
        v.as_i64().map(|i| i as i32)
            .or_else(|| v.as_f64().map(|f| f as i32))
    })
}

/// Convert an OF preset file to Rust preset format
pub fn convert_of_preset_to_rust(
    of_path: &std::path::Path,
    name: &str,
) -> anyhow::Result<super::PresetData> {
    let of_preset = OfPreset::from_file(of_path)?;
    
    Ok(super::PresetData {
        block1: of_preset.to_block1_params(),
        block2: of_preset.to_block2_params(),
        block3: of_preset.to_block3_params(),
        block1_modulations: HashMap::new(), // TODO: Extract LFO data
        block2_modulations: HashMap::new(),
        block3_modulations: HashMap::new(),
        tempo: super::PresetTempoData::default(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        name: name.to_string(),
    })
}

/// Batch convert all OF presets in a directory
pub fn batch_convert_of_presets(
    of_dir: &std::path::Path,
    output_bank: &str,
) -> anyhow::Result<usize> {
    let mut count = 0;
    
    if !of_dir.exists() {
        anyhow::bail!("Source directory does not exist: {:?}", of_dir);
    }
    
    // Create output bank directory
    let output_dir = std::path::PathBuf::from("presets").join(output_bank);
    std::fs::create_dir_all(&output_dir)?;
    
    // Process all JSON files
    for entry in std::fs::read_dir(of_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.extension().and_then(|e| e.to_str()) == Some("json") {
            let filename = path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("converted");
            
            // Clean up the name
            let clean_name = filename
                .trim_start_matches("gwSaveState")
                .trim_start_matches(|c: char| c.is_ascii_digit())
                .to_string();
            
            let clean_name = if clean_name.is_empty() {
                filename.to_string()
            } else {
                clean_name
            };
            
            match convert_of_preset_to_rust(&path, &clean_name) {
                Ok(preset_data) => {
                    let output_path = output_dir.join(format!("{}.json", clean_name));
                    let json = serde_json::to_string_pretty(&preset_data)?;
                    std::fs::write(&output_path, json)?;
                    log::info!("Converted: {} -> {:?}", filename, output_path);
                    count += 1;
                }
                Err(e) => {
                    log::warn!("Failed to convert {}: {}", filename, e);
                }
            }
        }
    }
    
    log::info!("Batch conversion complete: {} presets converted", count);
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_of_preset() {
        let json = r#"{
            "BLOCK_1": {
                "ch1Adjust": [0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
                "ch1AdjustDiscrete": [0, false, false, false, false, false, false, false, false, false, 0, 0, 0, 0, 0, 0],
                "ch1AdjustLfo": [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]
            },
            "BLOCK_2": {
                "block2InputAdjust": [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]
            },
            "BLOCK_3": {
                "block1Geo": [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]
            }
        }"#;
        
        let preset = OfPreset::from_json(json).unwrap();
        assert_eq!(preset.block1.ch1_adjust[0], 0.5);
        
        let params = preset.to_block1_params();
        assert_eq!(params.ch1_x_displace, 0.5);
    }
}
