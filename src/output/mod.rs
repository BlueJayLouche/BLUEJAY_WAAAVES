//! # Output Module
//!
//! Handles video output to external systems including:
//! - NDI output
//! - Video recording (via FFmpeg)
//! - Future: Syphon/Spout, etc.

pub mod ndi_sender;
pub use ndi_sender::{NdiOutputSender, is_ndi_output_available};

pub mod ndi_async;
pub use ndi_async::AsyncNdiOutput;
