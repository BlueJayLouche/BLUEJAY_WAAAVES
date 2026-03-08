//! # Output Module
//!
//! Handles video output to external systems including:
//! - NDI output
//! - Syphon output (macOS)
//! - Spout output (Windows)
//! - Video recording (via FFmpeg)

pub mod ndi_sender;
pub use ndi_sender::{NdiOutputSender, is_ndi_output_available};

pub mod ndi_async;
pub use ndi_async::AsyncNdiOutput;

// Platform-specific IPC outputs
#[cfg(target_os = "macos")]
pub mod syphon_sender;
#[cfg(target_os = "macos")]
pub use syphon_sender::{SyphonSender, SyphonWgpuSender};

#[cfg(target_os = "macos")]
pub mod syphon_async;
#[cfg(target_os = "macos")]
pub use syphon_async::{AsyncSyphonOutput, SyphonOutputIntegration};
