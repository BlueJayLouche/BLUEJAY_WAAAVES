#pragma once

#include "ofMain.h"
#include "ofxXmlSettings.h"

// Cross-platform macros
#if defined(TARGET_WIN32)
    #define OFAPP_HAS_SPOUT 1
#else
    #define OFAPP_HAS_SPOUT 0
#endif

namespace dragonwaves {

//==============================================================================
// Display Settings - Configurable post-compilation via XML
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
    
    // Getters/Setters for XML binding
    void loadFromXml(ofxXmlSettings& xml);
    void saveToXml(ofxXmlSettings& xml);
};

//==============================================================================
// OSC Settings
//==============================================================================
struct OscSettings {
    bool enabled = false;
    int receivePort = 7000;
    std::string sendIP = "127.0.0.1";
    int sendPort = 7001;
    
    void loadFromXml(ofxXmlSettings& xml);
    void saveToXml(ofxXmlSettings& xml);
};

//==============================================================================
// MIDI Settings
//==============================================================================
struct MidiSettings {
    int selectedPort = -1;
    std::string deviceName = "";
    bool enabled = false;
    
    void loadFromXml(ofxXmlSettings& xml);
    void saveToXml(ofxXmlSettings& xml);
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
    
    void loadFromXml(ofxXmlSettings& xml);
    void saveToXml(ofxXmlSettings& xml);
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
    
    // Load all settings from XML
    void load();
    
    // Save all settings to XML
    void save();
    
    // Getters
    DisplaySettings& getDisplay() { return display; }
    OscSettings& getOsc() { return osc; }
    MidiSettings& getMidi() { return midi; }
    InputSourceSettings& getInputSources() { return inputSources; }
    
    // Paths
    std::string getPresetsPath() { return "presets/"; }
    std::string getSettingsFile() { return "settings.xml"; }
    
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
};

} // namespace dragonwaves

// Backwards compatibility typedef
using SettingsManager = dragonwaves::SettingsManager;
