//! # Video/Audio Recorder Module
//!
//! Handles recording of output video and audio to file using FFmpeg.
//! Supports multiple codecs and quality presets.

use crate::core::{RecordingSettings, VideoCodec, RecordingQuality};
use std::io::Write;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::Instant;

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
    /// Recording start time
    start_time: Option<Instant>,
    /// Last frame time for measuring actual FPS
    last_frame_time: Option<Instant>,
    /// Accumulated frame time deltas for average FPS calculation
    frame_time_sum: f64,
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
            start_time: None,
            last_frame_time: None,
            frame_time_sum: 0.0,
        }
    }
    
    /// Get measured FPS based on actual frame timing
    pub fn measured_fps(&self) -> f64 {
        if self.frame_count > 1 && self.frame_time_sum > 0.0 {
            (self.frame_count as f64 - 1.0) / self.frame_time_sum
        } else {
            self.settings.fps as f64
        }
    }

    /// Start recording
    pub fn start(&mut self) -> anyhow::Result<()> {
        if self.is_recording {
            return Err(anyhow::anyhow!("Already recording"));
        }

        // Check if ffmpeg is installed
        match Command::new("ffmpeg").arg("-version").output() {
            Ok(_) => log::info!("FFmpeg found"),
            Err(e) => {
                return Err(anyhow::anyhow!(
                    "FFmpeg not found. Please install ffmpeg. Error: {}",
                    e
                ));
            }
        }

        let (width, height) = self.resolution;
        let _fps = self.settings.fps; // Kept for reference, but we use variable frame rate
        
        // Generate unique filename if file exists
        let base_filename = &self.settings.filename;
        let mut filename = format!("{}.mp4", base_filename);
        let mut counter = 1;
        while std::path::Path::new(&filename).exists() {
            filename = format!("{}_{:03}.mp4", base_filename, counter);
            counter += 1;
        }

        // Build FFmpeg command - simplified for compatibility
        let mut cmd = Command::new("ffmpeg");
        
        // Video input (raw RGBA from GPU)
        // Note: We don't specify -r (framerate) here to allow variable frame rate
        // The actual playback speed will match the capture speed
        cmd.arg("-hide_banner")
            .arg("-loglevel").arg("error") // Only show errors
            .arg("-f").arg("rawvideo")
            .arg("-pix_fmt").arg("rgba")
            .arg("-s").arg(format!("{}x{}", width, height))
            .arg("-thread_queue_size").arg("512") // Prevent buffer overruns
            .arg("-i").arg("-"); // stdin

        // Video codec settings - simplified to just H.264 for now
        let preset = match self.settings.quality {
            RecordingQuality::Lossless => "ultrafast",
            RecordingQuality::High => "fast",
            RecordingQuality::Medium => "medium",
            RecordingQuality::Low => "veryfast",
        };
        
        let crf = match self.settings.quality {
            RecordingQuality::Lossless => "0",
            RecordingQuality::High => "18",
            RecordingQuality::Medium => "23",
            RecordingQuality::Low => "28",
        };
        
        cmd.arg("-c:v").arg("libx264")
            .arg("-preset").arg(preset)
            .arg("-crf").arg(crf)
            .arg("-pix_fmt").arg("yuv420p")
            .arg("-vsync").arg("vfr") // Variable frame rate - use actual frame timing
            .arg("-movflags").arg("+faststart"); // Web optimization

        // Output file
        cmd.arg(&filename);

        log::info!("Starting recording: {}x{} -> {} (variable frame rate)", width, height, filename);

        // Spawn process with stderr capture for debugging
        let mut child = cmd
            .stdin(Stdio::piped())
            .stderr(Stdio::piped()) // Capture stderr for error reporting
            .spawn()
            .map_err(|e| anyhow::anyhow!("Failed to start ffmpeg: {}. Is ffmpeg installed?", e))?;

        self.video_stdin = child.stdin.take();
        self.process = Some(child);
        self.is_recording = true;
        self.frame_count = 0;
        self.start_time = Some(Instant::now());
        self.last_frame_time = None;
        self.frame_time_sum = 0.0;

        log::info!("Recording started to {}", filename);
        Ok(())
    }

    /// Write a video frame (RGBA data)
    /// Expected data size: width * height * 4 bytes (RGBA)
    pub fn write_frame(&mut self, data: &[u8]) -> anyhow::Result<()> {
        if let Some(ref mut stdin) = self.video_stdin {
            let expected_size = (self.resolution.0 * self.resolution.1 * 4) as usize;
            if data.len() != expected_size {
                return Err(anyhow::anyhow!(
                    "Frame size mismatch: got {} bytes, expected {} bytes ({}x{}x4)",
                    data.len(), expected_size, self.resolution.0, self.resolution.1
                ));
            }
            
            stdin.write_all(data)?;
            
            // Track frame timing
            let now = Instant::now();
            if let Some(last) = self.last_frame_time {
                let delta = now.duration_since(last).as_secs_f64();
                self.frame_time_sum += delta;
            }
            self.last_frame_time = Some(now);
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

        // Calculate final timing stats
        let measured_fps = self.measured_fps();
        let expected_fps = self.settings.fps as f64;
        let fps_ratio = measured_fps / expected_fps;
        
        log::info!(
            "Recording stats: {} frames, measured {:.2} fps (expected {:.0} fps, ratio: {:.2}x)",
            self.frame_count, measured_fps, expected_fps, fps_ratio
        );
        
        if (fps_ratio - 1.0).abs() > 0.1 {
            log::warn!(
                "Framerate mismatch! Recorded at {:.2} fps, expected {:.0} fps. Fix: ffmpeg -i input.mp4 -filter:v 'setpts={:.4}*PTS' output.mp4",
                measured_fps, expected_fps, fps_ratio
            );
        }

        // Close stdin to signal EOF to ffmpeg
        drop(self.video_stdin.take());

        // Wait for process to finish
        if let Some(mut child) = self.process.take() {
            match child.wait() {
                Ok(status) => {
                    if status.success() {
                        log::info!("Recording saved successfully");
                    } else {
                        log::warn!("FFmpeg exited with status: {}", status);
                        
                        // Try to read stderr for error details
                        if let Some(mut stderr) = child.stderr.take() {
                            let mut error_msg = String::new();
                            if std::io::Read::read_to_string(&mut stderr, &mut error_msg).is_ok() && !error_msg.is_empty() {
                                log::error!("FFmpeg error: {}", error_msg);
                            }
                        }
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
