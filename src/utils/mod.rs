//! # Utilities Module
//!
//! Helper functions and types used throughout the application.

use std::time::{Duration, Instant};

/// Frame rate limiter
pub struct FrameLimiter {
    target_interval: Duration,
    last_frame: Instant,
}

impl FrameLimiter {
    /// Create a new frame limiter with target FPS
    pub fn new(target_fps: u32) -> Self {
        Self {
            target_interval: Duration::from_secs_f32(1.0 / target_fps as f32),
            last_frame: Instant::now(),
        }
    }
    
    /// Set target FPS
    pub fn set_target_fps(&mut self, target_fps: u32) {
        self.target_interval = Duration::from_secs_f32(1.0 / target_fps as f32);
    }
    
    /// Wait until next frame should be rendered
    pub fn wait(&mut self) {
        let elapsed = self.last_frame.elapsed();
        if elapsed < self.target_interval {
            std::thread::sleep(self.target_interval - elapsed);
        }
        self.last_frame = Instant::now();
    }
    
    /// Check if a frame should be rendered (non-blocking)
    pub fn should_render(&self) -> bool {
        self.last_frame.elapsed() >= self.target_interval
    }
    
    /// Mark frame as rendered
    pub fn mark_rendered(&mut self) {
        self.last_frame = Instant::now();
    }
}

/// Simple moving average for smoothing values
pub struct MovingAverage {
    values: Vec<f32>,
    index: usize,
    sum: f32,
}

impl MovingAverage {
    /// Create a new moving average with given window size
    pub fn new(window_size: usize) -> Self {
        Self {
            values: vec![0.0; window_size],
            index: 0,
            sum: 0.0,
        }
    }
    
    /// Add a new value and get the current average
    pub fn add(&mut self, value: f32) -> f32 {
        self.sum -= self.values[self.index];
        self.sum += value;
        self.values[self.index] = value;
        self.index = (self.index + 1) % self.values.len();
        self.sum / self.values.len() as f32
    }
    
    /// Get current average without adding
    pub fn average(&self) -> f32 {
        self.sum / self.values.len() as f32
    }
    
    /// Reset to zeros
    pub fn reset(&mut self) {
        self.values.fill(0.0);
        self.sum = 0.0;
        self.index = 0;
    }
}

/// LFO (Low Frequency Oscillator) generator
pub struct Lfo {
    phase: f32,
    rate: f32,
    amplitude: f32,
    waveform: Waveform,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Waveform {
    Sine,
    Triangle,
    Saw,
    Square,
    Random,
}

impl Lfo {
    /// Create a new LFO
    pub fn new(rate: f32, amplitude: f32, waveform: Waveform) -> Self {
        Self {
            phase: 0.0,
            rate,
            amplitude,
            waveform,
        }
    }
    
    /// Update LFO and get current value
    pub fn update(&mut self, delta_time: f32) -> f32 {
        self.phase += self.rate * delta_time;
        self.phase = self.phase.fract();
        
        let raw = match self.waveform {
            Waveform::Sine => (self.phase * std::f32::consts::TAU).sin(),
            Waveform::Triangle => {
                let p = self.phase * 4.0;
                if p < 1.0 { p }
                else if p < 3.0 { 2.0 - p }
                else { p - 4.0 }
            }
            Waveform::Saw => self.phase * 2.0 - 1.0,
            Waveform::Square => if self.phase < 0.5 { 1.0 } else { -1.0 }
            Waveform::Random => {
                // Simple pseudo-random based on phase
                let seed = (self.phase * 1000.0) as u32;
                (((seed.wrapping_mul(1103515245).wrapping_add(12345)) % 65536) as f32 / 32768.0) - 1.0
            }
        };
        
        raw * self.amplitude
    }
    
    /// Set phase directly
    pub fn set_phase(&mut self, phase: f32) {
        self.phase = phase.fract();
    }
    
    /// Reset phase to zero
    pub fn reset(&mut self) {
        self.phase = 0.0;
    }
}

/// Color conversion utilities
pub mod color {
    /// Convert RGB to HSB
    pub fn rgb_to_hsb(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let delta = max - min;
        
        let h = if delta == 0.0 {
            0.0
        } else if max == r {
            60.0 * (((g - b) / delta) % 6.0)
        } else if max == g {
            60.0 * ((b - r) / delta + 2.0)
        } else {
            60.0 * ((r - g) / delta + 4.0)
        };
        
        let s = if max == 0.0 { 0.0 } else { delta / max };
        let v = max;
        
        (h / 360.0, s, v)
    }
    
    /// Convert HSB to RGB
    pub fn hsb_to_rgb(h: f32, s: f32, v: f32) -> (f32, f32, f32) {
        let h = h * 360.0;
        let c = v * s;
        let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
        let m = v - c;
        
        let (r, g, b) = if h < 60.0 {
            (c, x, 0.0)
        } else if h < 120.0 {
            (x, c, 0.0)
        } else if h < 180.0 {
            (0.0, c, x)
        } else if h < 240.0 {
            (0.0, x, c)
        } else if h < 300.0 {
            (x, 0.0, c)
        } else {
            (c, 0.0, x)
        };
        
        (r + m, g + m, b + m)
    }
}

/// Math utilities
pub mod math {
    /// Wrap value to [0, 1] range
    pub fn wrap01(v: f32) -> f32 {
        v.fract().abs()
    }
    
    /// Fold value to [0, 1] range
    pub fn fold01(v: f32) -> f32 {
        let v = v.abs();
        let int_part = v as i32;
        let frac = v.fract();
        if int_part % 2 == 0 { frac } else { 1.0 - frac }
    }
    
    /// Mirror value around 0
    pub fn mirror(v: f32) -> f32 {
        if v > 0.0 { v } else { -(1.0 + v) }
    }
    
    /// Linear interpolation
    pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
        a + (b - a) * t.clamp(0.0, 1.0)
    }
    
    /// Smooth step interpolation
    pub fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
        let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    }
}
