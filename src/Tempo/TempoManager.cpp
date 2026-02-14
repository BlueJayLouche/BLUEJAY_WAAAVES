#include "TempoManager.h"
#include "../Core/SettingsManager.h"

namespace dragonwaves {

//==============================================================================
// BpmModulation
//==============================================================================
float BpmModulation::process(float beatPhase, float bpm) {
    if (!enabled) {
        return 0.0f;
    }
    
    // Calculate period for this division
    float divisionPeriod = BeatDivisionValues[divisionIndex];
    
    // Calculate phase for this division
    float divPhase = fmod(beatPhase + phase, 1.0f);
    divPhase = fmod(divPhase * divisionPeriod, 1.0f);
    
    // Get waveform value (-1 to 1 or 0 to 1)
    float wave = getWaveformValue(divPhase);
    
    if (bipolar) {
        // Bipolar: -1 to 1 maps to min to max
        return ofMap(wave, -1.0f, 1.0f, minValue, maxValue, true);
    } else {
        // Unipolar: 0 to 1 maps to min to max
        return ofMap(wave, 0.0f, 1.0f, minValue, maxValue, true);
    }
}

float BpmModulation::getWaveformValue(float phase) {
    switch (waveform) {
        case 0: // Sine
            return sin(phase * TWO_PI);
            
        case 1: // Triangle
            if (phase < 0.5f) {
                return phase * 4.0f - 1.0f;
            } else {
                return 3.0f - phase * 4.0f;
            }
            
        case 2: // Saw
            return phase * 2.0f - 1.0f;
            
        case 3: // Square
            return (phase < 0.5f) ? 1.0f : -1.0f;
            
        case 4: // Random (sample and hold)
            // Change value at phase wraparound
            if (phase < currentPhase) {
                lastRandom = ofRandom(-1.0f, 1.0f);
            }
            currentPhase = phase;
            return lastRandom;
            
        default:
            return sin(phase * TWO_PI);
    }
}

void BpmModulation::loadFromJson(const ofJson& json) {
    if (json.contains("enabled")) enabled = json["enabled"].get<bool>();
    if (json.contains("divisionIndex")) divisionIndex = json["divisionIndex"].get<int>();
    if (json.contains("phase")) phase = json["phase"].get<float>();
    if (json.contains("waveform")) waveform = json["waveform"].get<int>();
    if (json.contains("minValue")) minValue = json["minValue"].get<float>();
    if (json.contains("maxValue")) maxValue = json["maxValue"].get<float>();
    if (json.contains("bipolar")) bipolar = json["bipolar"].get<bool>();
}

void BpmModulation::saveToJson(ofJson& json) const {
    json["enabled"] = enabled;
    json["divisionIndex"] = divisionIndex;
    json["phase"] = phase;
    json["waveform"] = waveform;
    json["minValue"] = minValue;
    json["maxValue"] = maxValue;
    json["bipolar"] = bipolar;
}

//==============================================================================
// TempoManager
//==============================================================================
TempoManager::TempoManager() {
    tapTimes.reserve(16);
}

void TempoManager::setup(const TempoSettings& newSettings) {
    settings = newSettings;
    settings.bpm = ofClamp(settings.bpm, settings.minBpm, settings.maxBpm);
    lastUpdateTime = ofGetElapsedTimef();
}

void TempoManager::update(float deltaTime) {
    if (!settings.enabled || !playing) {
        return;
    }
    
    // Check for tap timeout
    if (settings.autoResetTap && tapTimes.size() > 0) {
        float timeSinceLastTap = ofGetElapsedTimef() - lastTapTime;
        if (timeSinceLastTap > settings.tapTimeout) {
            resetTap();
        }
    }
    
    // Advance phase
    advancePhase(deltaTime);
}

void TempoManager::advancePhase(float deltaTime) {
    float beatPeriod = getBeatPeriod();
    if (beatPeriod <= 0.0f) return;
    
    // Advance beat phase
    float phaseIncrement = deltaTime / beatPeriod;
    currentBeatPhase += phaseIncrement;
    
    // Check for beat wraparound
    if (currentBeatPhase >= 1.0f) {
        currentBeatPhase -= 1.0f;
        currentBeat++;
        beatTriggered = true;
        
        // Check for bar wraparound (4 beats per bar)
        if (currentBeat >= 4) {
            currentBeat = 0;
            currentBar++;
            barTriggered = true;
        }
    }
    
    // Update bar phase
    currentBarPhase = (currentBeat + currentBeatPhase) / 4.0f;
}

void TempoManager::tap() {
    float currentTime = ofGetElapsedTimef();
    
    // Reset if this is the first tap or timeout occurred
    if (tapTimes.empty()) {
        tapStartTime = currentTime;
        tapTimes.push_back(0.0f);
    } else {
        float interval = currentTime - lastTapTime;
        
        // Validate interval (must be reasonable for BPM range)
        float minInterval = 60.0f / settings.maxBpm;
        float maxInterval = 60.0f / settings.minBpm;
        
        if (interval >= minInterval && interval <= maxInterval) {
            tapTimes.push_back(interval);
            
            // Keep only recent taps
            while (tapTimes.size() > (size_t)settings.tapHistorySize) {
                tapTimes.erase(tapTimes.begin());
            }
            
            // Calculate new BPM
            calculateTapBpm();
        } else if (interval > settings.tapTimeout) {
            // Reset if too long
            resetTap();
            tapStartTime = currentTime;
            tapTimes.push_back(0.0f);
        }
    }
    
    lastTapTime = currentTime;
}

void TempoManager::calculateTapBpm() {
    if (tapTimes.size() < 2) return;
    
    // Skip first entry (which is 0.0f for the initial tap)
    float sum = 0.0f;
    int count = 0;
    
    for (size_t i = 1; i < tapTimes.size(); i++) {
        sum += tapTimes[i];
        count++;
    }
    
    if (count > 0) {
        float avgInterval = sum / count;
        float newBpm = 60.0f / avgInterval;
        setBpm(newBpm);
    }
}

void TempoManager::resetTap() {
    tapTimes.clear();
    lastTapTime = 0.0f;
    tapStartTime = 0.0f;
}

void TempoManager::setBpm(float bpm) {
    settings.bpm = ofClamp(bpm, settings.minBpm, settings.maxBpm);
}

void TempoManager::nudgeBpm(float delta) {
    setBpm(settings.bpm + delta);
}

float TempoManager::getCalculatedBpm() const {
    if (tapTimes.size() < 2) return settings.bpm;
    
    float sum = 0.0f;
    int count = 0;
    
    for (size_t i = 1; i < tapTimes.size(); i++) {
        sum += tapTimes[i];
        count++;
    }
    
    if (count > 0) {
        return 60.0f / (sum / count);
    }
    return settings.bpm;
}

bool TempoManager::isTapPending() const {
    if (tapTimes.empty()) return false;
    float timeSinceLastTap = ofGetElapsedTimef() - lastTapTime;
    return timeSinceLastTap < settings.tapTimeout;
}

float TempoManager::getPhaseForDivision(BeatDivision division) const {
    return getPhaseForDivision(static_cast<int>(division));
}

float TempoManager::getPhaseForDivision(int divisionIndex) const {
    if (divisionIndex < 0 || divisionIndex >= 8) return currentBeatPhase;
    
    float divisionValue = BeatDivisionValues[divisionIndex];
    return fmod(currentBeatPhase * divisionValue, 1.0f);
}

float TempoManager::getDivisionPeriod(BeatDivision division) const {
    float beatPeriod = getBeatPeriod();
    float divisionValue = BeatDivisionValues[static_cast<int>(division)];
    return beatPeriod * divisionValue;
}

void TempoManager::resetPhase() {
    currentBeatPhase = 0.0f;
    currentBarPhase = 0.0f;
    currentBeat = 0;
}

} // namespace dragonwaves
