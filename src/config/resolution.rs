//! # Resolution Configuration
//!
//! Common resolution presets for input, internal, and output resolutions.
//! Supports dropdown selection with custom option.

use serde::{Deserialize, Serialize};

/// Common resolution presets
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResolutionPreset {
    /// 640x480 (SD)
    #[serde(rename = "640x480")]
    R640x480,
    /// 800x600 (SVGA)
    #[serde(rename = "800x600")]
    R800x600,
    /// 1024x768 (XGA)
    #[serde(rename = "1024x768")]
    R1024x768,
    /// 1280x720 (HD 720p)
    #[serde(rename = "1280x720")]
    R1280x720,
    /// 1280x1024 (SXGA)
    #[serde(rename = "1280x1024")]
    R1280x1024,
    /// 1920x1080 (Full HD 1080p)
    #[serde(rename = "1920x1080")]
    R1920x1080,
    /// 2560x1440 (QHD 1440p)
    #[serde(rename = "2560x1440")]
    R2560x1440,
    /// 3840x2160 (4K UHD)
    #[serde(rename = "3840x2160")]
    R3840x2160,
    /// 4096x2160 (DCI 4K)
    #[serde(rename = "4096x2160")]
    R4096x2160,
    /// 7680x4320 (8K UHD)
    #[serde(rename = "7680x4320")]
    R7680x4320,
    /// Square 1:1 - 1080x1080 (Instagram)
    #[serde(rename = "1080x1080")]
    R1080x1080,
    /// Vertical 9:16 - 1080x1920 (TikTok/Reels)
    #[serde(rename = "1080x1920")]
    R1080x1920,
    /// Custom resolution
    #[serde(rename = "custom")]
    Custom,
}

impl ResolutionPreset {
    /// Get all available presets
    pub fn all() -> &'static [ResolutionPreset] {
        &[
            ResolutionPreset::R640x480,
            ResolutionPreset::R800x600,
            ResolutionPreset::R1024x768,
            ResolutionPreset::R1280x720,
            ResolutionPreset::R1280x1024,
            ResolutionPreset::R1920x1080,
            ResolutionPreset::R2560x1440,
            ResolutionPreset::R3840x2160,
            ResolutionPreset::R4096x2160,
            ResolutionPreset::R7680x4320,
            ResolutionPreset::R1080x1080,
            ResolutionPreset::R1080x1920,
            ResolutionPreset::Custom,
        ]
    }

    /// Get display name for the preset
    pub fn name(&self) -> &'static str {
        match self {
            ResolutionPreset::R640x480 => "640x480 (SD)",
            ResolutionPreset::R800x600 => "800x600 (SVGA)",
            ResolutionPreset::R1024x768 => "1024x768 (XGA)",
            ResolutionPreset::R1280x720 => "1280x720 (HD 720p)",
            ResolutionPreset::R1280x1024 => "1280x1024 (SXGA)",
            ResolutionPreset::R1920x1080 => "1920x1080 (Full HD 1080p)",
            ResolutionPreset::R2560x1440 => "2560x1440 (QHD 1440p)",
            ResolutionPreset::R3840x2160 => "3840x2160 (4K UHD)",
            ResolutionPreset::R4096x2160 => "4096x2160 (DCI 4K)",
            ResolutionPreset::R7680x4320 => "7680x4320 (8K UHD)",
            ResolutionPreset::R1080x1080 => "1080x1080 (Square 1:1)",
            ResolutionPreset::R1080x1920 => "1080x1920 (Vertical 9:16)",
            ResolutionPreset::Custom => "Custom...",
        }
    }

    /// Get width and height for the preset
    pub fn dimensions(&self) -> Option<(u32, u32)> {
        match self {
            ResolutionPreset::R640x480 => Some((640, 480)),
            ResolutionPreset::R800x600 => Some((800, 600)),
            ResolutionPreset::R1024x768 => Some((1024, 768)),
            ResolutionPreset::R1280x720 => Some((1280, 720)),
            ResolutionPreset::R1280x1024 => Some((1280, 1024)),
            ResolutionPreset::R1920x1080 => Some((1920, 1080)),
            ResolutionPreset::R2560x1440 => Some((2560, 1440)),
            ResolutionPreset::R3840x2160 => Some((3840, 2160)),
            ResolutionPreset::R4096x2160 => Some((4096, 2160)),
            ResolutionPreset::R7680x4320 => Some((7680, 4320)),
            ResolutionPreset::R1080x1080 => Some((1080, 1080)),
            ResolutionPreset::R1080x1920 => Some((1080, 1920)),
            ResolutionPreset::Custom => None,
        }
    }

    /// Find preset from dimensions, or return Custom
    pub fn from_dimensions(width: u32, height: u32) -> Self {
        for preset in Self::all() {
            if let Some((w, h)) = preset.dimensions() {
                if w == width && h == height {
                    return *preset;
                }
            }
        }
        ResolutionPreset::Custom
    }

    /// Check if this is the Custom preset
    pub fn is_custom(&self) -> bool {
        matches!(self, ResolutionPreset::Custom)
    }
}

impl Default for ResolutionPreset {
    fn default() -> Self {
        ResolutionPreset::R1280x720
    }
}

/// Resolution configuration with preset and optional custom values
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolutionConfig {
    /// Selected preset
    pub preset: ResolutionPreset,
    /// Custom width (only used when preset is Custom)
    pub custom_width: u32,
    /// Custom height (only used when preset is Custom)
    pub custom_height: u32,
}

impl ResolutionConfig {
    /// Create new resolution config with preset
    pub fn new_preset(preset: ResolutionPreset) -> Self {
        let (w, h) = preset.dimensions().unwrap_or((1280, 720));
        Self {
            preset,
            custom_width: w,
            custom_height: h,
        }
    }

    /// Create new resolution config with custom dimensions
    pub fn new_custom(width: u32, height: u32) -> Self {
        Self {
            preset: ResolutionPreset::Custom,
            custom_width: width,
            custom_height: height,
        }
    }

    /// Get actual width
    pub fn width(&self) -> u32 {
        match self.preset.dimensions() {
            Some((w, _)) => w,
            None => self.custom_width,
        }
    }

    /// Get actual height
    pub fn height(&self) -> u32 {
        match self.preset.dimensions() {
            Some((_, h)) => h,
            None => self.custom_height,
        }
    }

    /// Get dimensions as tuple
    pub fn dimensions(&self) -> (u32, u32) {
        (self.width(), self.height())
    }

    /// Set dimensions (updates to Custom preset)
    pub fn set_dimensions(&mut self, width: u32, height: u32) {
        self.preset = ResolutionPreset::Custom;
        self.custom_width = width;
        self.custom_height = height;
    }

    /// Update preset and dimensions
    pub fn set_preset(&mut self, preset: ResolutionPreset) {
        self.preset = preset;
        if let Some((w, h)) = preset.dimensions() {
            self.custom_width = w;
            self.custom_height = h;
        }
    }

    /// Get display string for current resolution
    pub fn display_string(&self) -> String {
        let (w, h) = self.dimensions();
        if self.preset.is_custom() {
            format!("{}x{} (Custom)", w, h)
        } else {
            format!("{}x{}", w, h)
        }
    }
}

impl Default for ResolutionConfig {
    fn default() -> Self {
        Self::new_preset(ResolutionPreset::R1280x720)
    }
}

/// Complete resolution settings for the application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolutionSettings {
    /// Input resolution (camera/video input)
    pub input: ResolutionConfig,
    /// Internal processing resolution
    pub internal: ResolutionConfig,
    /// Output window resolution
    pub output: ResolutionConfig,
}

impl Default for ResolutionSettings {
    fn default() -> Self {
        Self {
            input: ResolutionConfig::new_preset(ResolutionPreset::R1280x720),
            internal: ResolutionConfig::new_preset(ResolutionPreset::R1280x720),
            output: ResolutionConfig::new_preset(ResolutionPreset::R1920x1080),
        }
    }
}
