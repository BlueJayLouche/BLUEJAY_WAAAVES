#include "SettingsManager.h"
#include <sys/stat.h>

namespace dragonwaves {

//==============================================================================
// File Watching Helpers
//==============================================================================
std::time_t SettingsManager::getFileModificationTime(const std::string& path) {
    struct stat result;
    if (stat(path.c_str(), &result) == 0) {
        return result.st_mtime;
    }
    return 0;
}

void SettingsManager::updateLastModificationTime() {
    lastSettingsPath = getSettingsFile();
    lastFileModificationTime = getFileModificationTime(lastSettingsPath);
}

//==============================================================================
// Migration from XML
//==============================================================================
bool SettingsManager::migrateFromXml() {
    ofxXmlSettings xml;
    std::string legacyPath = getLegacySettingsFile();
    
    if (!xml.loadFile(legacyPath)) {
        return false;  // No legacy file to migrate
    }
    
    ofLogNotice("SettingsManager") << "Migrating legacy settings.xml to settings.json...";
    
    // Load from XML format
    if (xml.tagExists("display")) {
        xml.pushTag("display");
        display.input1Width = xml.getValue("input1Width", 640);
        display.input1Height = xml.getValue("input1Height", 480);
        display.input2Width = xml.getValue("input2Width", 640);
        display.input2Height = xml.getValue("input2Height", 480);
        display.internalWidth = xml.getValue("internalWidth", 1280);
        display.internalHeight = xml.getValue("internalHeight", 720);
        display.outputWidth = xml.getValue("outputWidth", 1280);
        display.outputHeight = xml.getValue("outputHeight", 720);
        display.ndiSendWidth = xml.getValue("ndiSendWidth", 1280);
        display.ndiSendHeight = xml.getValue("ndiSendHeight", 720);
#if OFAPP_HAS_SPOUT
        display.spoutSendWidth = xml.getValue("spoutSendWidth", 1280);
        display.spoutSendHeight = xml.getValue("spoutSendHeight", 720);
#endif
        display.targetFPS = xml.getValue("targetFPS", 30);
        xml.popTag();
    }
    
    if (xml.tagExists("osc")) {
        xml.pushTag("osc");
        osc.enabled = xml.getValue("enabled", 0) == 1;
        osc.receivePort = xml.getValue("receivePort", 7000);
        osc.sendIP = xml.getValue("sendIP", "127.0.0.1");
        osc.sendPort = xml.getValue("sendPort", 7001);
        xml.popTag();
    }
    
    if (xml.tagExists("midi")) {
        xml.pushTag("midi");
        midi.selectedPort = xml.getValue("selectedPort", -1);
        midi.deviceName = xml.getValue("deviceName", "");
        midi.enabled = xml.getValue("enabled", 0) == 1;
        xml.popTag();
    }
    
    if (xml.tagExists("inputSources")) {
        xml.pushTag("inputSources");
        inputSources.input1SourceType = xml.getValue("input1SourceType", 1);
        inputSources.input2SourceType = xml.getValue("input2SourceType", 1);
        inputSources.input1DeviceID = xml.getValue("input1DeviceID", 0);
        inputSources.input2DeviceID = xml.getValue("input2DeviceID", 1);
        inputSources.input1NdiSourceIndex = xml.getValue("input1NdiSourceIndex", 0);
        inputSources.input2NdiSourceIndex = xml.getValue("input2NdiSourceIndex", 0);
#if OFAPP_HAS_SPOUT
        inputSources.input1SpoutSourceIndex = xml.getValue("input1SpoutSourceIndex", 0);
        inputSources.input2SpoutSourceIndex = xml.getValue("input2SpoutSourceIndex", 0);
#endif
        xml.popTag();
    }
    
    uiScaleIndex = xml.getValue("uiScaleIndex", 0);
    
    // Save to JSON format
    save();
    
    ofLogNotice("SettingsManager") << "Migration complete! settings.json created.";
    return true;
}

//==============================================================================
// DisplaySettings
//==============================================================================
void DisplaySettings::loadFromJson(const ofJson& json) {
    if (json.contains("display") && json["display"].is_object()) {
        const auto& display = json["display"];
        input1Width = display.value("input1Width", 640);
        input1Height = display.value("input1Height", 480);
        input2Width = display.value("input2Width", 640);
        input2Height = display.value("input2Height", 480);
        internalWidth = display.value("internalWidth", 1280);
        internalHeight = display.value("internalHeight", 720);
        outputWidth = display.value("outputWidth", 1280);
        outputHeight = display.value("outputHeight", 720);
        ndiSendWidth = display.value("ndiSendWidth", 1280);
        ndiSendHeight = display.value("ndiSendHeight", 720);
#if OFAPP_HAS_SPOUT
        spoutSendWidth = display.value("spoutSendWidth", 1280);
        spoutSendHeight = display.value("spoutSendHeight", 720);
#endif
        targetFPS = display.value("targetFPS", 30);
    }
}

void DisplaySettings::saveToJson(ofJson& json) const {
    json["display"]["input1Width"] = input1Width;
    json["display"]["input1Height"] = input1Height;
    json["display"]["input2Width"] = input2Width;
    json["display"]["input2Height"] = input2Height;
    json["display"]["internalWidth"] = internalWidth;
    json["display"]["internalHeight"] = internalHeight;
    json["display"]["outputWidth"] = outputWidth;
    json["display"]["outputHeight"] = outputHeight;
    json["display"]["ndiSendWidth"] = ndiSendWidth;
    json["display"]["ndiSendHeight"] = ndiSendHeight;
#if OFAPP_HAS_SPOUT
    json["display"]["spoutSendWidth"] = spoutSendWidth;
    json["display"]["spoutSendHeight"] = spoutSendHeight;
#endif
    json["display"]["targetFPS"] = targetFPS;
}

//==============================================================================
// OscSettings
//==============================================================================
void OscSettings::loadFromJson(const ofJson& json) {
    if (json.contains("osc") && json["osc"].is_object()) {
        const auto& osc = json["osc"];
        enabled = osc.value("enabled", false);
        receivePort = osc.value("receivePort", 7000);
        sendIP = osc.value("sendIP", "127.0.0.1");
        sendPort = osc.value("sendPort", 7001);
    }
}

void OscSettings::saveToJson(ofJson& json) const {
    json["osc"]["enabled"] = enabled;
    json["osc"]["receivePort"] = receivePort;
    json["osc"]["sendIP"] = sendIP;
    json["osc"]["sendPort"] = sendPort;
}

//==============================================================================
// MidiSettings
//==============================================================================
void MidiSettings::loadFromJson(const ofJson& json) {
    if (json.contains("midi") && json["midi"].is_object()) {
        const auto& midi = json["midi"];
        selectedPort = midi.value("selectedPort", -1);
        deviceName = midi.value("deviceName", "");
        enabled = midi.value("enabled", false);
    }
}

void MidiSettings::saveToJson(ofJson& json) const {
    json["midi"]["selectedPort"] = selectedPort;
    json["midi"]["deviceName"] = deviceName;
    json["midi"]["enabled"] = enabled;
}

//==============================================================================
// InputSourceSettings
//==============================================================================
void InputSourceSettings::loadFromJson(const ofJson& json) {
    if (json.contains("inputSources") && json["inputSources"].is_object()) {
        const auto& sources = json["inputSources"];
        input1SourceType = sources.value("input1SourceType", 1);
        input2SourceType = sources.value("input2SourceType", 1);
        input1DeviceID = sources.value("input1DeviceID", 0);
        input2DeviceID = sources.value("input2DeviceID", 1);
        input1NdiSourceIndex = sources.value("input1NdiSourceIndex", 0);
        input2NdiSourceIndex = sources.value("input2NdiSourceIndex", 0);
#if OFAPP_HAS_SPOUT
        input1SpoutSourceIndex = sources.value("input1SpoutSourceIndex", 0);
        input2SpoutSourceIndex = sources.value("input2SpoutSourceIndex", 0);
#endif
    }
}

void InputSourceSettings::saveToJson(ofJson& json) const {
    json["inputSources"]["input1SourceType"] = input1SourceType;
    json["inputSources"]["input2SourceType"] = input2SourceType;
    json["inputSources"]["input1DeviceID"] = input1DeviceID;
    json["inputSources"]["input2DeviceID"] = input2DeviceID;
    json["inputSources"]["input1NdiSourceIndex"] = input1NdiSourceIndex;
    json["inputSources"]["input2NdiSourceIndex"] = input2NdiSourceIndex;
#if OFAPP_HAS_SPOUT
    json["inputSources"]["input1SpoutSourceIndex"] = input1SpoutSourceIndex;
    json["inputSources"]["input2SpoutSourceIndex"] = input2SpoutSourceIndex;
#endif
}

//==============================================================================
// SettingsManager
//==============================================================================
void SettingsManager::load() {
    std::string settingsPath = getSettingsFile();
    
    // Try to load from JSON first
    ofFile jsonFile(settingsPath);
    if (jsonFile.exists()) {
        ofJson json;
        try {
            jsonFile >> json;
            jsonFile.close();
            
            // Load all settings sections
            display.loadFromJson(json);
            osc.loadFromJson(json);
            midi.loadFromJson(json);
            inputSources.loadFromJson(json);
            audio.loadFromJson(json);
            tempo.loadFromJson(json);
            
            // UI settings
            if (json.contains("uiScaleIndex")) {
                uiScaleIndex = json.value("uiScaleIndex", 0);
            }
            
            // Update file modification tracking
            updateLastModificationTime();
            
            ofLogNotice("SettingsManager") << "Settings loaded from " << settingsPath;
            return;
        } catch (const std::exception& e) {
            ofLogError("SettingsManager") << "Error loading JSON: " << e.what() << ", trying migration...";
        }
    }
    
    // Try to migrate from legacy XML format
    if (migrateFromXml()) {
        updateLastModificationTime();
        return;
    }
    
    // No settings file found, use defaults and save
    ofLogNotice("SettingsManager") << "No settings file found, using defaults";
    save();
    updateLastModificationTime();
}

void SettingsManager::reload() {
    ofLogNotice("SettingsManager") << "Reloading settings from disk...";
    
    // Check if file has actually changed
    std::time_t currentModTime = getFileModificationTime(getSettingsFile());
    if (currentModTime == lastFileModificationTime) {
        ofLogNotice("SettingsManager") << "File unchanged, skipping reload";
        return;
    }
    
    // Store old values to detect changes
    DisplaySettings oldDisplay = display;
    OscSettings oldOsc = osc;
    MidiSettings oldMidi = midi;
    InputSourceSettings oldInputSources = inputSources;
    AudioSettings oldAudio = audio;
    TempoSettings oldTempo = tempo;
    int oldUiScaleIndex = uiScaleIndex;
    
    // Reload from disk
    load();
    
    // Check what changed
    bool displayChanged = (memcmp(&oldDisplay, &display, sizeof(DisplaySettings)) != 0);
    bool oscChanged = (memcmp(&oldOsc, &osc, sizeof(OscSettings)) != 0);
    bool midiChanged = (memcmp(&oldMidi, &midi, sizeof(MidiSettings)) != 0);
    bool inputSourcesChanged = (memcmp(&oldInputSources, &inputSources, sizeof(InputSourceSettings)) != 0);
    bool audioChanged = (memcmp(&oldAudio, &audio, sizeof(AudioSettings)) != 0);
    bool tempoChanged = (memcmp(&oldTempo, &tempo, sizeof(TempoSettings)) != 0);
    bool uiScaleChanged = (oldUiScaleIndex != uiScaleIndex);
    
    // Check for resolution/FPS changes
    if (oldDisplay.internalWidth != display.internalWidth ||
        oldDisplay.internalHeight != display.internalHeight ||
        oldDisplay.outputWidth != display.outputWidth ||
        oldDisplay.outputHeight != display.outputHeight ||
        oldDisplay.input1Width != display.input1Width ||
        oldDisplay.input1Height != display.input1Height ||
        oldDisplay.input2Width != display.input2Width ||
        oldDisplay.input2Height != display.input2Height) {
        resolutionChanged = true;
    }
    
    if (oldDisplay.targetFPS != display.targetFPS) {
        fpsChanged = true;
    }
    
    if (displayChanged || oscChanged || midiChanged || inputSourcesChanged || audioChanged || tempoChanged || uiScaleChanged) {
        ofLogNotice("SettingsManager") << "Settings reloaded. Changes detected:";
        if (displayChanged) ofLogNotice("SettingsManager") << "  - Display settings changed";
        if (oscChanged) ofLogNotice("SettingsManager") << "  - OSC settings changed";
        if (midiChanged) ofLogNotice("SettingsManager") << "  - MIDI settings changed";
        if (inputSourcesChanged) ofLogNotice("SettingsManager") << "  - Input sources changed";
        if (audioChanged) ofLogNotice("SettingsManager") << "  - Audio settings changed";
        if (tempoChanged) ofLogNotice("SettingsManager") << "  - Tempo settings changed";
        if (uiScaleChanged) ofLogNotice("SettingsManager") << "  - UI scale changed";
        
        // Notify listeners
        if (settingsChangedCallback) {
            settingsChangedCallback();
        }
    } else {
        ofLogNotice("SettingsManager") << "Settings reloaded (no changes detected)";
    }
}

void SettingsManager::update() {
    if (!fileWatchingEnabled) return;
    
    // Check if it's time to check the file
    timeSinceLastCheck += ofGetLastFrameTime();
    if (timeSinceLastCheck < fileCheckInterval) return;
    timeSinceLastCheck = 0.0f;
    
    // Check if file has been modified
    std::time_t currentModTime = getFileModificationTime(getSettingsFile());
    if (currentModTime != lastFileModificationTime && currentModTime != 0) {
        ofLogNotice("SettingsManager") << "Detected settings.json change, reloading...";
        reload();
    }
}

void SettingsManager::save() {
    ofJson json;
    
    // Save all settings sections
    display.saveToJson(json);
    osc.saveToJson(json);
    midi.saveToJson(json);
    inputSources.saveToJson(json);
    audio.saveToJson(json);
    tempo.saveToJson(json);
    
    // UI settings
    json["uiScaleIndex"] = uiScaleIndex;
    
    std::string settingsPath = getSettingsFile();
    try {
        ofFile file(settingsPath, ofFile::WriteOnly);
        file << json.dump(4);  // Pretty print with 4-space indentation
        file.close();
        ofLogNotice("SettingsManager") << "Settings saved to " << settingsPath;
        // Update modification time to avoid self-triggering reload
        updateLastModificationTime();
    } catch (const std::exception& e) {
        ofLogError("SettingsManager") << "Failed to save settings to " << settingsPath << ": " << e.what();
    }
}

void SettingsManager::applyDisplaySettings(const DisplaySettings& newSettings) {
    // Check if resolution changed
    if (display.internalWidth != newSettings.internalWidth ||
        display.internalHeight != newSettings.internalHeight ||
        display.outputWidth != newSettings.outputWidth ||
        display.outputHeight != newSettings.outputHeight ||
        display.input1Width != newSettings.input1Width ||
        display.input1Height != newSettings.input1Height ||
        display.input2Width != newSettings.input2Width ||
        display.input2Height != newSettings.input2Height) {
        resolutionChanged = true;
    }
    
    // Check if FPS changed
    if (display.targetFPS != newSettings.targetFPS) {
        fpsChanged = true;
    }
    
    display = newSettings;
}

//==============================================================================
// Legacy XML Methods (for migration only)
//==============================================================================
void DisplaySettings::loadFromXml(ofxXmlSettings& xml) {
    xml.pushTag("display");
    input1Width = xml.getValue("input1Width", 640);
    input1Height = xml.getValue("input1Height", 480);
    input2Width = xml.getValue("input2Width", 640);
    input2Height = xml.getValue("input2Height", 480);
    internalWidth = xml.getValue("internalWidth", 1280);
    internalHeight = xml.getValue("internalHeight", 720);
    outputWidth = xml.getValue("outputWidth", 1280);
    outputHeight = xml.getValue("outputHeight", 720);
    ndiSendWidth = xml.getValue("ndiSendWidth", 1280);
    ndiSendHeight = xml.getValue("ndiSendHeight", 720);
#if OFAPP_HAS_SPOUT
    spoutSendWidth = xml.getValue("spoutSendWidth", 1280);
    spoutSendHeight = xml.getValue("spoutSendHeight", 720);
#endif
    targetFPS = xml.getValue("targetFPS", 30);
    xml.popTag();
}

void DisplaySettings::saveToXml(ofxXmlSettings& xml) const {
    xml.addTag("display");
    xml.pushTag("display");
    xml.setValue("input1Width", input1Width);
    xml.setValue("input1Height", input1Height);
    xml.setValue("input2Width", input2Width);
    xml.setValue("input2Height", input2Height);
    xml.setValue("internalWidth", internalWidth);
    xml.setValue("internalHeight", internalHeight);
    xml.setValue("outputWidth", outputWidth);
    xml.setValue("outputHeight", outputHeight);
    xml.setValue("ndiSendWidth", ndiSendWidth);
    xml.setValue("ndiSendHeight", ndiSendHeight);
#if OFAPP_HAS_SPOUT
    xml.setValue("spoutSendWidth", spoutSendWidth);
    xml.setValue("spoutSendHeight", spoutSendHeight);
#endif
    xml.setValue("targetFPS", targetFPS);
    xml.popTag();
}

void OscSettings::loadFromXml(ofxXmlSettings& xml) {
    xml.pushTag("osc");
    enabled = xml.getValue("enabled", 0) == 1;
    receivePort = xml.getValue("receivePort", 7000);
    sendIP = xml.getValue("sendIP", "127.0.0.1");
    sendPort = xml.getValue("sendPort", 7001);
    xml.popTag();
}

void OscSettings::saveToXml(ofxXmlSettings& xml) const {
    xml.addTag("osc");
    xml.pushTag("osc");
    xml.setValue("enabled", enabled ? 1 : 0);
    xml.setValue("receivePort", receivePort);
    xml.setValue("sendIP", sendIP);
    xml.setValue("sendPort", sendPort);
    xml.popTag();
}

void MidiSettings::loadFromXml(ofxXmlSettings& xml) {
    xml.pushTag("midi");
    selectedPort = xml.getValue("selectedPort", -1);
    deviceName = xml.getValue("deviceName", "");
    enabled = xml.getValue("enabled", 0) == 1;
    xml.popTag();
}

void MidiSettings::saveToXml(ofxXmlSettings& xml) const {
    xml.addTag("midi");
    xml.pushTag("midi");
    xml.setValue("selectedPort", selectedPort);
    xml.setValue("deviceName", deviceName);
    xml.setValue("enabled", enabled ? 1 : 0);
    xml.popTag();
}

void InputSourceSettings::loadFromXml(ofxXmlSettings& xml) {
    xml.pushTag("inputSources");
    input1SourceType = xml.getValue("input1SourceType", 1);
    input2SourceType = xml.getValue("input2SourceType", 1);
    input1DeviceID = xml.getValue("input1DeviceID", 0);
    input2DeviceID = xml.getValue("input2DeviceID", 1);
    input1NdiSourceIndex = xml.getValue("input1NdiSourceIndex", 0);
    input2NdiSourceIndex = xml.getValue("input2NdiSourceIndex", 0);
#if OFAPP_HAS_SPOUT
    input1SpoutSourceIndex = xml.getValue("input1SpoutSourceIndex", 0);
    input2SpoutSourceIndex = xml.getValue("input2SpoutSourceIndex", 0);
#endif
    xml.popTag();
}

void InputSourceSettings::saveToXml(ofxXmlSettings& xml) const {
    xml.addTag("inputSources");
    xml.pushTag("inputSources");
    xml.setValue("input1SourceType", input1SourceType);
    xml.setValue("input2SourceType", input2SourceType);
    xml.setValue("input1DeviceID", input1DeviceID);
    xml.setValue("input2DeviceID", input2DeviceID);
    xml.setValue("input1NdiSourceIndex", input1NdiSourceIndex);
    xml.setValue("input2NdiSourceIndex", input2NdiSourceIndex);
#if OFAPP_HAS_SPOUT
    xml.setValue("input1SpoutSourceIndex", input1SpoutSourceIndex);
    xml.setValue("input2SpoutSourceIndex", input2SpoutSourceIndex);
#endif
    xml.popTag();
}

//==============================================================================
// AudioSettings
//==============================================================================
void AudioSettings::loadFromJson(const ofJson& json) {
    if (json.contains("audio") && json["audio"].is_object()) {
        const auto& audio = json["audio"];
        enabled = audio.value("enabled", false);
        inputDevice = audio.value("inputDevice", 0);
        sampleRate = audio.value("sampleRate", 44100);
        bufferSize = audio.value("bufferSize", 512);
        fftSize = audio.value("fftSize", 1024);
        numBins = audio.value("numBins", 128);
        smoothing = audio.value("smoothing", 0.5f);
        normalization = audio.value("normalization", true);
        amplitude = audio.value("amplitude", 1.0f);
        peakDecay = audio.value("peakDecay", 0.95f);
    }
}

void AudioSettings::saveToJson(ofJson& json) const {
    json["audio"]["enabled"] = enabled;
    json["audio"]["inputDevice"] = inputDevice;
    json["audio"]["sampleRate"] = sampleRate;
    json["audio"]["bufferSize"] = bufferSize;
    json["audio"]["fftSize"] = fftSize;
    json["audio"]["numBins"] = numBins;
    json["audio"]["smoothing"] = smoothing;
    json["audio"]["normalization"] = normalization;
    json["audio"]["amplitude"] = amplitude;
    json["audio"]["peakDecay"] = peakDecay;
}

void AudioSettings::loadFromXml(ofxXmlSettings& xml) {
    xml.pushTag("audio");
    enabled = xml.getValue("enabled", 0) == 1;
    inputDevice = xml.getValue("inputDevice", 0);
    sampleRate = xml.getValue("sampleRate", 44100);
    bufferSize = xml.getValue("bufferSize", 512);
    fftSize = xml.getValue("fftSize", 1024);
    numBins = xml.getValue("numBins", 128);
    smoothing = xml.getValue("smoothing", 0.5f);
    normalization = xml.getValue("normalization", 1) == 1;
    amplitude = xml.getValue("amplitude", 1.0f);
    peakDecay = xml.getValue("peakDecay", 0.95f);
    xml.popTag();
}

void AudioSettings::saveToXml(ofxXmlSettings& xml) const {
    xml.addTag("audio");
    xml.pushTag("audio");
    xml.setValue("enabled", enabled ? 1 : 0);
    xml.setValue("inputDevice", inputDevice);
    xml.setValue("sampleRate", sampleRate);
    xml.setValue("bufferSize", bufferSize);
    xml.setValue("numBins", numBins);
    xml.setValue("fftSize", fftSize);
    xml.setValue("smoothing", smoothing);
    xml.setValue("normalization", normalization ? 1 : 0);
    xml.setValue("amplitude", amplitude);
    xml.setValue("peakDecay", peakDecay);
    xml.popTag();
}

//==============================================================================
// TempoSettings
//==============================================================================
void TempoSettings::loadFromJson(const ofJson& json) {
    if (json.contains("tempo") && json["tempo"].is_object()) {
        const auto& tempo = json["tempo"];
        bpm = tempo.value("bpm", 120.0f);
        enabled = tempo.value("enabled", true);
        tapHistorySize = tempo.value("tapHistorySize", 8);
        minBpm = tempo.value("minBpm", 20.0f);
        maxBpm = tempo.value("maxBpm", 300.0f);
        autoResetTap = tempo.value("autoResetTap", true);
        tapTimeout = tempo.value("tapTimeout", 2.0f);
    }
}

void TempoSettings::saveToJson(ofJson& json) const {
    json["tempo"]["bpm"] = bpm;
    json["tempo"]["enabled"] = enabled;
    json["tempo"]["tapHistorySize"] = tapHistorySize;
    json["tempo"]["minBpm"] = minBpm;
    json["tempo"]["maxBpm"] = maxBpm;
    json["tempo"]["autoResetTap"] = autoResetTap;
    json["tempo"]["tapTimeout"] = tapTimeout;
}

void TempoSettings::loadFromXml(ofxXmlSettings& xml) {
    xml.pushTag("tempo");
    bpm = xml.getValue("bpm", 120.0f);
    enabled = xml.getValue("enabled", 1) == 1;
    tapHistorySize = xml.getValue("tapHistorySize", 8);
    minBpm = xml.getValue("minBpm", 20.0f);
    maxBpm = xml.getValue("maxBpm", 300.0f);
    autoResetTap = xml.getValue("autoResetTap", 1) == 1;
    tapTimeout = xml.getValue("tapTimeout", 2.0f);
    xml.popTag();
}

void TempoSettings::saveToXml(ofxXmlSettings& xml) const {
    xml.addTag("tempo");
    xml.pushTag("tempo");
    xml.setValue("bpm", bpm);
    xml.setValue("enabled", enabled ? 1 : 0);
    xml.setValue("tapHistorySize", tapHistorySize);
    xml.setValue("minBpm", minBpm);
    xml.setValue("maxBpm", maxBpm);
    xml.setValue("autoResetTap", autoResetTap ? 1 : 0);
    xml.setValue("tapTimeout", tapTimeout);
    xml.popTag();
}

} // namespace dragonwaves
