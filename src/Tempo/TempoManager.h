#pragma once

#include "ofMain.h"
#include "../Core/SettingsManager.h"

namespace dragonwaves {

//==============================================================================
// Beat divisions
//==============================================================================
enum class BeatDivision {
    SIXTEENTH = 0,      // 1/16 beat
    EIGHTH = 1,         // 1/8 beat
    QUARTER = 2,        // 1/4 beat (default)
    HALF = 3,           // 1/2 beat
    WHOLE = 4,          // 1 beat
    DOUBLE = 5,         // 2 beats
    QUADRUPLE = 6,      // 4 beats
    OCTUPLE = 7,        // 8 beats
    COUNT = 8
};

static const char* BeatDivisionNames[8] = {
    "1/16",
    "1/8",
    "1/4",
    "1/2",
    "1",
    "2",
    "4",
    "8"
};

static const float BeatDivisionValues[8] = {
    0.0625f,    // 1/16
    0.125f,     // 1/8
    0.25f,      // 1/4
    0.5f,       // 1/2
    1.0f,       // 1
    2.0f,       // 2
    4.0f,       // 4
    8.0f        // 8
};

//==============================================================================
// BPM Waveform types
//==============================================================================
enum class BpmWaveform {
    SINE = 0,
    TRIANGLE = 1,
    SAW = 2,
    SQUARE = 3,
    RANDOM = 4,
    COUNT = 5
};

static const char* BpmWaveformNames[5] = {
    "Sine",
    "Triangle",
    "Saw",
    "Square",
    "Random"
};

// TempoSettings is now defined in Core/SettingsManager.h

//==============================================================================
// BPM modulation for a single parameter
//==============================================================================
struct BpmModulation {
    bool enabled = false;
    int divisionIndex = 2;              // Default to 1/4 beat
    float phase = 0.0f;                 // Phase offset (0-1)
    int waveform = 0;                   // BpmWaveform
    float minValue = 0.0f;
    float maxValue = 1.0f;
    bool bipolar = false;               // If true, oscillates around center
    
    // Runtime state (not saved)
    float currentPhase = 0.0f;
    float lastRandom = 0.0f;
    
    float process(float beatPhase, float bpm);
    void loadFromJson(const ofJson& json);
    void saveToJson(ofJson& json) const;
    
private:
    float getWaveformValue(float phase);
};

//==============================================================================
// Tempo manager - BPM and beat synchronization
//==============================================================================
class TempoManager {
public:
    TempoManager();
    
    void setup(const TempoSettings& settings);
    void update(float deltaTime);  // Call every frame
    
    // Tap tempo
    void tap();
    void resetTap();
    
    // Getters
    float getBpm() const { return settings.bpm; }
    float getBeatPeriod() const { return 60.0f / settings.bpm; }
    float getBeatPhase() const { return currentBeatPhase; }
    float getBarPhase() const { return currentBarPhase; }
    int getCurrentBeat() const { return currentBeat; }
    int getCurrentBar() const { return currentBar; }
    bool isEnabled() const { return settings.enabled; }
    bool isPlaying() const { return playing; }
    
    // Setters
    void setBpm(float bpm);
    void setEnabled(bool enabled) { settings.enabled = enabled; }
    void setPlaying(bool play) { playing = play; }
    void nudgeBpm(float delta);
    
    // Tap tempo state
    int getTapCount() const { return tapTimes.size(); }
    float getCalculatedBpm() const;
    bool isTapPending() const;
    
    // Utility
    float getPhaseForDivision(BeatDivision division) const;
    float getPhaseForDivision(int divisionIndex) const;
    float getDivisionPeriod(BeatDivision division) const;
    
    // Settings
    void loadSettings(const TempoSettings& newSettings) { settings = newSettings; }
    const TempoSettings& getSettings() const { return settings; }
    
    // Reset
    void resetPhase();
    
    // Settings - public for OSC parameter access
    TempoSettings settings;
    
private:
    
    // Timing
    float currentBeatPhase = 0.0f;      // 0-1 within current beat
    float currentBarPhase = 0.0f;       // 0-1 within current bar (4 beats)
    int currentBeat = 0;                // 0-3 within bar
    int currentBar = 0;
    
    // State
    bool playing = true;
    float lastUpdateTime = 0.0f;
    
    // Tap tempo
    std::vector<float> tapTimes;        // Relative times between taps
    float lastTapTime = 0.0f;
    float tapStartTime = 0.0f;
    
    // Beat callback (for visual flash, etc.)
    bool beatTriggered = false;
    bool barTriggered = false;
    
    void calculateTapBpm();
    void advancePhase(float deltaTime);
};

} // namespace dragonwaves

// Backwards compatibility
typedef dragonwaves::TempoManager TempoManager;
typedef dragonwaves::TempoSettings TempoSettings;
typedef dragonwaves::BpmModulation BpmModulation;
typedef dragonwaves::BeatDivision BeatDivision;
typedef dragonwaves::BpmWaveform BpmWaveform;
