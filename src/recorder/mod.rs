//! # Video/Audio Recorder Module
//!
//! Handles recording of output video and audio to file using FFmpeg.
//! Supports multiple codecs and quality presets.

use crate::core::{RecordingSettings, VideoCodec, RecordingQuality};
use std::io::Write;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};

/// Recording state
#[derive(Debug)]
pub struct Recorder {
    /// FFmpeg process handle
    process: Option<Child>,
    /// Stdin handle for piping video frames
    video_stdin: Option<std::process::ChildStdin>,
    /// Recording settings
    settings: RecordingSettings,
    /// Output resolution
    resolution: (u32, u32),
    /// Frame count
    frame_count: u64,
    /// Whether recording is active
    is_recording: bool,
}

impl Recorder {
    /// Create a new recorder
    pub fn new(settings: RecordingSettings, resolution: (u32, u32)) -> Self {
        Self {
            process: None,
            video_stdin: None,
            settings,
            resolution,
            frame_count: 0,
            is_recording: false,
        }
    }

    /// Start recording
    pub fn start(&mut self) -> anyhow::Result<()> {
        if self.is_recording {
            return Err(anyhow::anyhow!("Already recording"));
        }

        let (width, height) = self.resolution;
        let fps = self.settings.fps;
        let filename = format!("{}.mp4", self.settings.filename);

        // Build FFmpeg command
        let mut cmd = Command::new("ffmpeg");
        
        // Video input (raw RGBA from GPU)
        cmd.arg("-f").arg("rawvideo")
            .arg("-pix_fmt").arg("rgba")
            .arg("-s").arg(format!("{}x{}", width, height))
            .arg("-r").arg(fps.to_string())
            .arg("-i").arg("-"); // stdin

        // Audio input (if enabled)
        if self.settings.include_audio {
            cmd.arg("-f").arg("f32le")
                .arg("-ar").arg("44100")
                .arg("-ac").arg("2")
                .arg("-i").arg("-"); // stdin (would need separate pipe)
        }

        // Video codec settings
        match self.settings.codec {
            VideoCodec::H264 => {
                cmd.arg("-c:v").arg("libx264")
                    .arg("-preset").arg("fast");
                match self.settings.quality {
                    RecordingQuality::Lossless => cmd.arg("-crf").arg("0"),
                    RecordingQuality::High => cmd.arg("-crf").arg("18"),
                    RecordingQuality::Medium => cmd.arg("-crf").arg("23"),
                    RecordingQuality::Low => cmd.arg("-crf").arg("28"),
                };
                cmd.arg("-pix_fmt").arg("yuv420p"); // For compatibility
            }
            VideoCodec::H265 => {
                cmd.arg("-c:v").arg("libx265")
                    .arg("-preset").arg("fast");
                match self.settings.quality {
                    RecordingQuality::Lossless => cmd.arg("-crf").arg("0"),
                    RecordingQuality::High => cmd.arg("-crf").arg("20"),
                    RecordingQuality::Medium => cmd.arg("-crf").arg("28"),
                    RecordingQuality::Low => cmd.arg("-crf").arg("35"),
                };
                cmd.arg("-pix_fmt").arg("yuv420p");
            }
            VideoCodec::ProRes => {
                cmd.arg("-c:v").arg("prores_ks")
                    .arg("-profile:v").arg("3"); // HQ
                match self.settings.quality {
                    RecordingQuality::Lossless => cmd.arg("-qscale:v").arg("0"),
                    RecordingQuality::High => cmd.arg("-qscale:v").arg("5"),
                    RecordingQuality::Medium => cmd.arg("-qscale:v").arg("9"),
                    RecordingQuality::Low => cmd.arg("-qscale:v").arg("13"),
                };
            }
            VideoCodec::VP9 => {
                cmd.arg("-c:v").arg("libvpx-v9")
                    .arg("-deadline").arg("good");
                match self.settings.quality {
                    RecordingQuality::Lossless => cmd.arg("-crf").arg("4"),
                    RecordingQuality::High => cmd.arg("-crf").arg("15"),
                    RecordingQuality::Medium => cmd.arg("-crf").arg("31"),
                    RecordingQuality::Low => cmd.arg("-crf").arg("50"),
                };
            }
            VideoCodec::AV1 => {
                cmd.arg("-c:v").arg("libsvtav1")
                    .arg("-preset").arg("8");
                match self.settings.quality {
                    RecordingQuality::Lossless => cmd.arg("-crf").arg("0"),
                    RecordingQuality::High => cmd.arg("-crf").arg("23"),
                    RecordingQuality::Medium => cmd.arg("-crf").arg("35"),
                    RecordingQuality::Low => cmd.arg("-crf").arg("50"),
                };
            }
        }

        // Audio codec
        if self.settings.include_audio {
            cmd.arg("-c:a").arg("aac")
                .arg("-b:a").arg("192k");
        }

        // Output file (overwrite if exists)
        cmd.arg("-y").arg(&filename);

        log::info!("Starting recording: ffmpeg {:?}", cmd);

        // Spawn process
        let mut child = cmd
            .stdin(Stdio::piped())
            .stderr(Stdio::null()) // Suppress ffmpeg output
            .spawn()
            .map_err(|e| anyhow::anyhow!("Failed to start ffmpeg: {}. Is ffmpeg installed?", e))?;

        self.video_stdin = child.stdin.take();
        self.process = Some(child);
        self.is_recording = true;
        self.frame_count = 0;

        log::info!("Recording started to {}", filename);
        Ok(())
    }

    /// Write a video frame (RGBA data)
    pub fn write_frame(&mut self, data: &[u8]) -> anyhow::Result<()> {
        if let Some(ref mut stdin) = self.video_stdin {
            stdin.write_all(data)?;
            self.frame_count += 1;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Recording not started"))
        }
    }

    /// Stop recording
    pub fn stop(&mut self) -> anyhow::Result<()> {
        if !self.is_recording {
            return Ok(());
        }

        // Close stdin to signal EOF to ffmpeg
        drop(self.video_stdin.take());

        // Wait for process to finish
        if let Some(mut child) = self.process.take() {
            match child.wait() {
                Ok(status) => {
                    if status.success() {
                        log::info!("Recording finished: {} frames", self.frame_count);
                    } else {
                        log::warn!("FFmpeg exited with status: {}", status);
                    }
                }
                Err(e) => log::error!("Failed to wait for ffmpeg: {}", e),
            }
        }

        self.is_recording = false;
        Ok(())
    }

    /// Check if recording is active
    pub fn is_recording(&self) -> bool {
        self.is_recording
    }

    /// Get frame count
    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }
}

impl Drop for Recorder {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

/// Thread-safe recorder handle
pub type RecorderHandle = Arc<Mutex<Option<Recorder>>>;
