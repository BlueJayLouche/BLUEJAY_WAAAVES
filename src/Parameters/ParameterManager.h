#pragma once

#include "ofMain.h"
#include "ofxOsc.h"
#include "ofxMidi.h"
#include "Parameter.h"
#include "../Core/SettingsManager.h"

namespace dragonwaves {

//==============================================================================
// Parameter group for organizing related parameters
//==============================================================================
class ParameterGroup {
public:
    ParameterGroup(const std::string& name, const std::string& oscPrefix)
        : name(name), oscPrefix(oscPrefix) {}
    
    void addParameter(std::shared_ptr<ParameterBase> param);
    std::shared_ptr<ParameterBase> getParameter(const std::string& address) const;
    std::shared_ptr<ParameterBase> getParameterByName(const std::string& name) const;
    
    const std::string& getName() const { return name; }
    const std::string& getOscPrefix() const { return oscPrefix; }
    const std::vector<std::shared_ptr<ParameterBase>>& getParameters() const { return parameters; }
    
private:
    std::string name;
    std::string oscPrefix;
    std::vector<std::shared_ptr<ParameterBase>> parameters;
    std::map<std::string, std::shared_ptr<ParameterBase>> addressMap;
    std::map<std::string, std::shared_ptr<ParameterBase>> nameMap;
};

//==============================================================================
// MIDI mapping entry
//==============================================================================
struct MidiMapping {
    int ccNumber;
    std::string paramAddress;
    float minValue = 0.0f;
    float maxValue = 1.0f;
};

//==============================================================================
// Central parameter manager for OSC/MIDI
//==============================================================================
class ParameterManager : public ofxMidiListener {
public:
    static ParameterManager& getInstance() {
        static ParameterManager instance;
        return instance;
    }
    
    // Setup
    void setup(const OscSettings& oscSettings);
    void close();
    
    // Update (process OSC messages)
    void update();
    
    // Parameter groups
    void registerGroup(std::shared_ptr<ParameterGroup> group);
    std::shared_ptr<ParameterGroup> getGroup(const std::string& name) const;
    
    // Parameter lookup
    std::shared_ptr<ParameterBase> getParameter(const std::string& oscAddress) const;
    
    // OSC Send
    void sendParameter(const std::string& address, float value);
    void sendParameter(const std::string& address, int value);
    void sendParameter(const std::string& address, bool value);
    void sendString(const std::string& address, const std::string& value);
    void sendAllParameters();
    void sendGroupParameters(const std::string& groupName);
    
    // OSC Receive processing
    void processOscMessage(const ofxOscMessage& msg);
    
    // MIDI
    void setupMidi(const MidiSettings& settings);
    void closeMidi();
    void refreshMidiPorts();
    std::vector<std::string> getMidiPortNames() const;
    void connectMidiPort(int portIndex);
    void newMidiMessage(ofxMidiMessage& msg) override;
    void processMidiMessage(ofxMidiMessage& msg);
    
    // MIDI mapping
    void addMidiMapping(int ccNumber, const std::string& paramAddress, float minVal = 0.0f, float maxVal = 1.0f);
    void clearMidiMappings();
    void saveMidiMappings(const std::string& path);
    void loadMidiMappings(const std::string& path);
    
    // Enable/disable
    void setOscEnabled(bool enabled);
    bool isOscEnabled() const { return oscEnabled; }
    
    void setMidiEnabled(bool enabled);
    bool isMidiEnabled() const { return midiEnabled; }
    
    // Settings reload
    void reloadOscSettings();
    
private:
    ParameterManager() = default;
    ~ParameterManager() {
        // Ensure proper cleanup on destruction
        closeMidi();
        close();
    }
    ParameterManager(const ParameterManager&) = delete;
    ParameterManager& operator=(const ParameterManager&) = delete;
    
    // OSC
    ofxOscReceiver oscReceiver;
    ofxOscSender oscSender;
    bool oscEnabled = false;
    int oscReceivePort = 7000;
    std::string oscSendIP = "127.0.0.1";
    int oscSendPort = 7001;
    
    // MIDI
    std::unique_ptr<ofxMidiIn> midiIn;
    bool midiEnabled = false;
    std::vector<std::string> midiPortNames;
    
    // Parameter groups
    std::vector<std::shared_ptr<ParameterGroup>> groups;
    std::map<std::string, std::shared_ptr<ParameterBase>> allParameters;
    
    // MIDI mappings
    std::map<int, MidiMapping> midiMappings;
    std::map<int, bool> midiActive;  // For potentiometer latching
    static constexpr float MIDI_THRESHOLD = 0.035f;
    
    void rebuildParameterMap();
};

} // namespace dragonwaves

// Backwards compatibility
typedef dragonwaves::ParameterManager ParameterManager;
typedef dragonwaves::ParameterGroup ParameterGroup;
