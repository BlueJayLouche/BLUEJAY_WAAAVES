//! # Layout Configuration
//!
//! Manages floating window layout for popped-out tabs.
//! Saved to `layout.toml` in the project root.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Unique identifier for a tab
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum TabId {
    /// Main tabs
    Block1,
    Block2,
    Block3,
    Macros,
    Inputs,
    Settings,
    /// Block 1 sub-tabs
    Block1Ch1Adjust,
    Block1Ch2MixKey,
    Block1Ch2Adjust,
    Block1Fb1,
    Block1Lfo,
    /// Block 2 sub-tabs
    Block2InputAdjust,
    Block2Fb2,
    Block2Lfo,
    /// Block 3 sub-tabs
    Block3B1Reprocess,
    Block3B2Reprocess,
    Block3Matrix,
    Block3FinalMix,
    Block3Lfo,
}

impl TabId {
    /// Get display name for the tab
    pub fn display_name(&self) -> &'static str {
        match self {
            TabId::Block1 => "Block 1",
            TabId::Block2 => "Block 2",
            TabId::Block3 => "Block 3",
            TabId::Macros => "Macros",
            TabId::Inputs => "Inputs",
            TabId::Settings => "Settings",
            TabId::Block1Ch1Adjust => "CH1 Adjust",
            TabId::Block1Ch2MixKey => "CH2 Mix & Key",
            TabId::Block1Ch2Adjust => "CH2 Adjust",
            TabId::Block1Fb1 => "FB1 Parameters",
            TabId::Block1Lfo => "Block 1 LFO",
            TabId::Block2InputAdjust => "Input Adjust",
            TabId::Block2Fb2 => "FB2 Parameters",
            TabId::Block2Lfo => "Block 2 LFO",
            TabId::Block3B1Reprocess => "Block 1 Re-process",
            TabId::Block3B2Reprocess => "Block 2 Re-process",
            TabId::Block3Matrix => "Matrix Mixer",
            TabId::Block3FinalMix => "Final Mix",
            TabId::Block3Lfo => "Block 3 LFO",
        }
    }

    /// Get window title for popped-out tab
    pub fn window_title(&self) -> String {
        format!("{}", self.display_name())
    }

    /// Get color for this tab type (RGBA, 0-1 range)
    /// Returns a distinct color for visual identification
    pub fn color(&self) -> [f32; 4] {
        match self {
            // Block 1 - Orange (warm, energetic)
            TabId::Block1 | TabId::Block1Ch1Adjust | TabId::Block1Ch2MixKey | 
            TabId::Block1Ch2Adjust | TabId::Block1Fb1 | TabId::Block1Lfo => {
                [1.0, 0.6, 0.2, 1.0] // Vibrant orange
            }
            // Block 2 - Green (fresh, different)
            TabId::Block2 | TabId::Block2InputAdjust | TabId::Block2Fb2 | TabId::Block2Lfo => {
                [0.3, 0.8, 0.4, 1.0] // Fresh green
            }
            // Block 3 - Pink/Purple (creative, final)
            TabId::Block3 | TabId::Block3B1Reprocess | TabId::Block3B2Reprocess | 
            TabId::Block3Matrix | TabId::Block3FinalMix | TabId::Block3Lfo => {
                [0.9, 0.4, 0.7, 1.0] // Pink/magenta
            }
            // Macros - Purple (mysterious, powerful)
            TabId::Macros => {
                [0.7, 0.4, 0.9, 1.0] // Purple
            }
            // Inputs - Cyan (cool, technical)
            TabId::Inputs => {
                [0.3, 0.7, 0.9, 1.0] // Cyan/blue
            }
            // Settings - Gray (neutral)
            TabId::Settings => {
                [0.6, 0.6, 0.6, 1.0] // Gray
            }
        }
    }

    /// Get background color for window (lighter, semi-transparent)
    pub fn bg_color(&self) -> [f32; 4] {
        let c = self.color();
        // Make it darker and semi-transparent for background
        [c[0] * 0.15, c[1] * 0.15, c[2] * 0.15, 0.85]
    }

    /// Get border color (same as main color)
    pub fn border_color(&self) -> [f32; 4] {
        self.color()
    }
}

/// Window state for a popped-out tab
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct WindowState {
    /// Window position x
    pub pos_x: f32,
    /// Window position y
    pub pos_y: f32,
    /// Window width
    pub width: f32,
    /// Window height
    pub height: f32,
    /// Whether window is collapsed
    pub collapsed: bool,
}

impl Default for WindowState {
    fn default() -> Self {
        Self {
            pos_x: 100.0,
            pos_y: 100.0,
            width: 500.0,
            height: 600.0,
            collapsed: false,
        }
    }
}

/// Layout configuration for all popped-out tabs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutConfig {
    /// Map of tab IDs to their window states
    pub popped_tabs: HashMap<TabId, WindowState>,
    /// Default window size for new pop-outs
    pub default_window_size: [f32; 2],
    /// Auto-save layout on exit
    pub auto_save: bool,
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            popped_tabs: HashMap::new(),
            default_window_size: [500.0, 600.0],
            auto_save: true,
        }
    }
}

impl LayoutConfig {
    /// Load layout from file
    pub fn load() -> Self {
        let path = Self::layout_file_path();
        if !path.exists() {
            return Self::default();
        }

        match std::fs::read_to_string(&path) {
            Ok(content) => {
                match toml::from_str(&content) {
                    Ok(config) => config,
                    Err(e) => {
                        log::warn!("Failed to parse layout.toml: {}", e);
                        Self::default()
                    }
                }
            }
            Err(e) => {
                log::warn!("Failed to read layout.toml: {}", e);
                Self::default()
            }
        }
    }

    /// Save layout to file
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = Self::layout_file_path();
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        log::info!("Layout saved to {:?}", path);
        Ok(())
    }

    /// Get layout file path
    fn layout_file_path() -> PathBuf {
        PathBuf::from("layout.toml")
    }

    /// Check if a tab is popped out
    pub fn is_popped(&self, tab_id: &TabId) -> bool {
        self.popped_tabs.contains_key(tab_id)
    }

    /// Get window state for a tab (creates default if not exists)
    pub fn get_window_state(&self, tab_id: &TabId) -> WindowState {
        self.popped_tabs.get(tab_id).copied().unwrap_or_default()
    }

    /// Set window state for a tab
    pub fn set_window_state(&mut self, tab_id: TabId, state: WindowState) {
        self.popped_tabs.insert(tab_id, state);
    }

    /// Pop out a tab
    pub fn pop_tab(&mut self, tab_id: TabId) {
        if !self.popped_tabs.contains_key(&tab_id) {
            // Calculate cascade position based on number of popped tabs
            let count = self.popped_tabs.len() as f32;
            let mut state = WindowState::default();
            state.pos_x += count * 30.0;
            state.pos_y += count * 30.0;
            self.popped_tabs.insert(tab_id, state);
            log::info!("Popped out tab: {:?}", tab_id);
        }
    }

    /// Dock (close floating window) a tab
    pub fn dock_tab(&mut self, tab_id: &TabId) {
        self.popped_tabs.remove(tab_id);
        log::info!("Docked tab: {:?}", tab_id);
    }

    /// Toggle popped state of a tab
    pub fn toggle_popped(&mut self, tab_id: TabId) {
        if self.is_popped(&tab_id) {
            self.dock_tab(&tab_id);
        } else {
            self.pop_tab(tab_id);
        }
    }

    /// Update window state from ImGui window info
    pub fn update_from_imgui(&mut self, tab_id: &TabId, pos: [f32; 2], size: [f32; 2], collapsed: bool) {
        if let Some(state) = self.popped_tabs.get_mut(tab_id) {
            state.pos_x = pos[0];
            state.pos_y = pos[1];
            state.width = size[0];
            state.height = size[1];
            state.collapsed = collapsed;
        }
    }
}
