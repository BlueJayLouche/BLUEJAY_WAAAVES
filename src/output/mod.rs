//! # Output Module
//!
//! Handles video output to external systems including:
//! - NDI output
//! - Syphon output (macOS)
//! - Spout output (Windows)
//! - Video recording (via FFmpeg)

// NDI output (requires ndi feature)
#[cfg(feature = "ndi")]
pub mod ndi_sender;
#[cfg(feature = "ndi")]
pub use ndi_sender::{NdiOutputSender, is_ndi_output_available};

#[cfg(feature = "ndi")]
pub mod ndi_async;
#[cfg(feature = "ndi")]
pub use ndi_async::AsyncNdiOutput;

// Platform-specific IPC outputs (macOS only, requires syphon feature)
#[cfg(all(target_os = "macos", feature = "syphon"))]
pub mod syphon_sender;
#[cfg(all(target_os = "macos", feature = "syphon"))]
pub use syphon_sender::{SyphonSender, SyphonWgpuSender};

#[cfg(all(target_os = "macos", feature = "syphon"))]
pub mod syphon_async;
#[cfg(all(target_os = "macos", feature = "syphon"))]
pub use syphon_async::{AsyncSyphonOutput, SyphonOutputIntegration};
