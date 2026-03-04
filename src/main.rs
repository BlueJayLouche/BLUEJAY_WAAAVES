//! # RustJay Waaaves
//! 
//! A high-performance VJ application written in Rust, ported from the original
//! OpenFrameworks-based "BLUEJAY_WAAAVES" project.
//!
//! ## Architecture
//!
//! The application uses a dual-window setup:
//! - **Output Window**: Full-screen OpenGL rendering of the visual output
//! - **Control Window**: ImGui-based interface for real-time parameter control
//!
//! ## Shader Pipeline
//!
//! The rendering pipeline consists of three shader blocks:
//! - **Block 1**: Channel mixing and feedback processing
//! - **Block 2**: Secondary input processing and feedback  
//! - **Block 3**: Final mixing, colorization, and output
//!
//! ## Performance Features
//!
//! - Zero-copy texture transfers where possible
//! - Persistent PBOs for async GPU→CPU readback (NDI output)
//! - Early-exit shader optimizations
//! - Efficient framebuffer management

use env_logger;
use log::info;
use std::env;
use std::sync::{Arc, Mutex};

mod audio;
mod config;
mod core;
mod engine;
mod gui;
mod input;
mod midi;
mod params;
mod recorder;
mod utils;

// Simple mode module
mod simple_main;

use config::AppConfig;
use core::SharedState;

/// Application entry point
///
/// Creates the event loop and initializes both the output window
/// and the control GUI window.
/// 
/// Use `--simple` flag to run the simplified single-shader feedback mode.
fn main() -> anyhow::Result<()> {
    env_logger::init();
    
    info!("Starting RustJay Waaaves v{}", env!("CARGO_PKG_VERSION"));
    
    // Check for simple mode
    let args: Vec<String> = env::args().collect();
    let simple_mode = args.contains(&"--simple".to_string());
    
    // Load configuration
    let config = AppConfig::load_or_default();
    info!("Configuration loaded: {:?}", config);
    
    // Create shared state for inter-window communication
    let shared_state = Arc::new(Mutex::new(SharedState::new(&config)));
    
    if simple_mode {
        info!("Running in SIMPLE mode - single feedback shader");
        simple_main::run_simple_app(config, shared_state)?;
    } else {
        info!("Running in FULL mode - three-block pipeline");
        engine::run_app(config, shared_state)?;
    }
    
    Ok(())
}
