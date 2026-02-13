#pragma once

#include "ofMain.h"
#include "ofJson.h"
#include "ofxXmlSettings.h"  // For migration only
#include <functional>
#include <ctime>

// Cross-platform macros
#if defined(TARGET_WIN32)
    #define OFAPP_HAS_SPOUT 1
#else
    #define OFAPP_HAS_SPOUT 0
#endif

namespace dragonwaves {

//==============================================================================
// Display Settings - Configurable post-compilation via JSON
//==============================================================================
struct DisplaySettings {
    // Input resolutions
    int input1Width = 640;
    int input1Height = 480;
    int input2Width = 640;
    int input2Height = 480;
    
    // Internal processing resolution
    int internalWidth = 1280;
    int internalHeight = 720;
    
    // Output window resolution
    int outputWidth = 1280;
    int outputHeight = 720;
    
    // NDI/Spout send resolution
    int ndiSendWidth = 1280;
    int ndiSendHeight = 720;
    
#if OFAPP_HAS_SPOUT
    int spoutSendWidth = 1280;
    int spoutSendHeight = 720;
#endif
    
    // Performance
    int targetFPS = 30;
    
    // Getters/Setters for JSON binding
    void loadFromJson(const ofJson& json);
    void saveToJson(ofJson& json) const;
    
    // Legacy XML (for migration only)
    void loadFromXml(ofxXmlSettings& xml);
    void saveToXml(ofxXmlSettings& xml) const;
};

//==============================================================================
// OSC Settings
//==============================================================================
struct OscSettings {
    bool enabled = false;
    int receivePort = 7000;
    std::string sendIP = "127.0.0.1";
    int sendPort = 7001;
    
    // JSON binding
    void loadFromJson(const ofJson& json);
    void saveToJson(ofJson& json) const;
    
    // Legacy XML (for migration only)
    void loadFromXml(ofxXmlSettings& xml);
    void saveToXml(ofxXmlSettings& xml) const;
};

//==============================================================================
// MIDI Settings
//==============================================================================
struct MidiSettings {
    int selectedPort = -1;
    std::string deviceName = "";
    bool enabled = false;
    
    // JSON binding
    void loadFromJson(const ofJson& json);
    void saveToJson(ofJson& json) const;
    
    // Legacy XML (for migration only)
    void loadFromXml(ofxXmlSettings& xml);
    void saveToXml(ofxXmlSettings& xml) const;
};

//==============================================================================
// Input Source Settings
//==============================================================================
struct InputSourceSettings {
    int input1SourceType = 1;  // 0=None, 1=Webcam, 2=NDI, 3=Spout (Windows only), 4=Video
    int input2SourceType = 1;
    int input1DeviceID = 0;
    int input2DeviceID = 1;
    int input1NdiSourceIndex = 0;
    int input2NdiSourceIndex = 0;
#if OFAPP_HAS_SPOUT
    int input1SpoutSourceIndex = 0;
    int input2SpoutSourceIndex = 0;
#endif
    
    // JSON binding
    void loadFromJson(const ofJson& json);
    void saveToJson(ofJson& json) const;
    
    // Legacy XML (for migration only)
    void loadFromXml(ofxXmlSettings& xml);
    void saveToXml(ofxXmlSettings& xml) const;
};

//==============================================================================
// Settings Manager - Centralized configuration management
//==============================================================================
class SettingsManager {
public:
    static SettingsManager& getInstance() {
        static SettingsManager instance;
        return instance;
    }
    
    // Load all settings from JSON (with automatic XML migration)
    void load();
    
    // Save all settings to JSON
    void save();
    
    // Getters
    DisplaySettings& getDisplay() { return display; }
    OscSettings& getOsc() { return osc; }
    MidiSettings& getMidi() { return midi; }
    InputSourceSettings& getInputSources() { return inputSources; }
    
    // Paths
    std::string getPresetsPath() { return "presets/"; }
    std::string getSettingsFile() { return "config.json"; }  // Main config file (consolidated JSON)
    std::string getLegacySettingsFile() { return "settings.xml"; }  // For migration
    
    // UI Settings
    int getUIScaleIndex() const { return uiScaleIndex; }
    void setUIScaleIndex(int index) { uiScaleIndex = index; }
    
    // Resolution changed flag
    bool hasResolutionChanged() const { return resolutionChanged; }
    void clearResolutionChanged() { resolutionChanged = false; }
    
    // FPS changed flag
    bool hasFPSChanged() const { return fpsChanged; }
    void clearFPSChanged() { fpsChanged = false; }
    
    // Apply new display settings
    void applyDisplaySettings(const DisplaySettings& newSettings);
    
    // File watching for runtime reload
    void update();  // Call this every frame to check for file changes
    void enableFileWatching(bool enable) { fileWatchingEnabled = enable; }
    bool isFileWatchingEnabled() const { return fileWatchingEnabled; }
    
    // Callbacks for when settings change
    using SettingsChangedCallback = std::function<void()>;
    void onSettingsChanged(SettingsChangedCallback callback) { settingsChangedCallback = callback; }
    
    // Manual reload trigger
    void reload();  // Reload from disk and notify listeners
    
    // Get file modification time
    std::time_t getLastFileModificationTime() const { return lastFileModificationTime; }
    
private:
    SettingsManager() = default;
    ~SettingsManager() = default;
    SettingsManager(const SettingsManager&) = delete;
    SettingsManager& operator=(const SettingsManager&) = delete;
    
    DisplaySettings display;
    OscSettings osc;
    MidiSettings midi;
    InputSourceSettings inputSources;
    
    int uiScaleIndex = 0;  // 0=200%, 1=250%, 2=300%
    
    bool resolutionChanged = false;
    bool fpsChanged = false;
    
    // File watching
    bool fileWatchingEnabled = true;
    std::time_t lastFileModificationTime = 0;
    std::string lastSettingsPath;
    float fileCheckInterval = 1.0f;  // Check every 1 second
    float timeSinceLastCheck = 0.0f;
    
    // Callbacks
    SettingsChangedCallback settingsChangedCallback;
    
    // Helper methods
    std::time_t getFileModificationTime(const std::string& path);
    void updateLastModificationTime();
    bool migrateFromXml();  // Migrate legacy XML settings to JSON
};

} // namespace dragonwaves

// Backwards compatibility typedef
using SettingsManager = dragonwaves::SettingsManager;
