//! # Tempo-Synced LFO System
//! 
//! Simple sine LFOs synchronized to BPM

/// Tempo-synced LFO
#[derive(Debug, Clone, Copy)]
pub struct TempoLfo {
    /// LFO phase (0.0 to 1.0)
    pub phase: f32,
    /// Frequency in cycles per beat
    pub freq: f32,
    /// Current output value (0.0 to 1.0 for sine)
    pub value: f32,
}

impl TempoLfo {
    /// Create new LFO
    /// freq: cycles per beat (0.25 = 1/4, 0.5 = 1/2, 1.0 = 1, 2.0 = 2x, etc.)
    pub fn new(freq: f32) -> Self {
        Self {
            phase: 0.0,
            freq,
            value: 0.5, // Start at mid-point
        }
    }
    
    /// Update LFO phase based on time delta
    /// bpm: beats per minute
    /// delta_time: seconds since last frame
    pub fn update(&mut self, bpm: f32, delta_time: f32) {
        // Calculate phase increment
        // beats per second = bpm / 60.0
        // cycles per second = beats_per_second * freq
        // phase increment = cycles_per_second * delta_time
        let beats_per_second = bpm / 60.0;
        let cycles_per_second = beats_per_second * self.freq;
        let phase_increment = cycles_per_second * delta_time;
        
        self.phase = (self.phase + phase_increment) % 1.0;
        
        // Calculate sine wave output (0.0 to 1.0)
        // sin(0) = 0, sin(PI/2) = 1, sin(PI) = 0, sin(3PI/2) = -1, sin(2PI) = 0
        // Map -1..1 to 0..1
        let sine = (self.phase * 2.0 * std::f32::consts::PI).sin();
        self.value = sine * 0.5 + 0.5;
    }
    
    /// Reset phase to 0
    pub fn reset(&mut self) {
        self.phase = 0.0;
        self.value = 0.5;
    }
    
    /// Set frequency
    pub fn set_freq(&mut self, freq: f32) {
        self.freq = freq;
    }
}

/// Collection of tempo-synced LFOs
#[derive(Debug, Clone)]
pub struct TempoLfoBank {
    /// Hue shift LFO (1/4 note default)
    pub hue: TempoLfo,
    /// Rotation LFO (1/2 note default)
    pub rotate: TempoLfo,
    /// Zoom LFO (1/1 note default)
    pub zoom: TempoLfo,
    /// BPM
    pub bpm: f32,
}

impl Default for TempoLfoBank {
    fn default() -> Self {
        Self {
            hue: TempoLfo::new(1.0),      // 1 cycle per beat
            rotate: TempoLfo::new(0.5),   // 1/2 cycle per beat  
            zoom: TempoLfo::new(0.25),    // 1/4 cycle per beat
            bpm: 120.0,
        }
    }
}

impl TempoLfoBank {
    /// Create with specific BPM
    pub fn with_bpm(bpm: f32) -> Self {
        Self {
            hue: TempoLfo::new(1.0),
            rotate: TempoLfo::new(0.5),
            zoom: TempoLfo::new(0.25),
            bpm,
        }
    }
    
    /// Update all LFOs
    pub fn update(&mut self, delta_time: f32) {
        self.hue.update(self.bpm, delta_time);
        self.rotate.update(self.bpm, delta_time);
        self.zoom.update(self.bpm, delta_time);
    }
    
    /// Get current values as tuple
    pub fn values(&self) -> (f32, f32, f32) {
        (self.hue.value, self.rotate.value, self.zoom.value)
    }
    
    /// Reset all LFOs
    pub fn reset_all(&mut self) {
        self.hue.reset();
        self.rotate.reset();
        self.zoom.reset();
    }
    
    /// Set BPM
    pub fn set_bpm(&mut self, bpm: f32) {
        self.bpm = bpm;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_lfo_update() {
        let mut lfo = TempoLfo::new(1.0); // 1 cycle per beat
        // At 60 BPM, 1 beat per second
        // After 0.5 seconds, phase should be 0.5
        lfo.update(60.0, 0.5);
        assert!((lfo.phase - 0.5).abs() < 0.01);
        // Sine at PI should be 0, mapped to 0.5
        assert!((lfo.value - 0.5).abs() < 0.01);
    }
    
    #[test]
    fn test_lfo_bank() {
        let mut bank = TempoLfoBank::with_bpm(120.0);
        // At 120 BPM, 2 beats per second
        // After 0.5 seconds, 1 beat elapsed
        bank.update(0.5);
        assert!((bank.hue.phase - 1.0).abs() < 0.01 || bank.hue.phase < 0.01); // Should wrap
    }
}
