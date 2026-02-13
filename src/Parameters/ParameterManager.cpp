#include "ParameterManager.h"

namespace dragonwaves {

//==============================================================================
// ParameterGroup
//==============================================================================
void ParameterGroup::addParameter(std::shared_ptr<ParameterBase> param) {
    parameters.push_back(param);
    addressMap[param->getOscAddress()] = param;
    nameMap[param->getName()] = param;
}

std::shared_ptr<ParameterBase> ParameterGroup::getParameter(const std::string& address) const {
    auto it = addressMap.find(address);
    if (it != addressMap.end()) {
        return it->second;
    }
    return nullptr;
}

std::shared_ptr<ParameterBase> ParameterGroup::getParameterByName(const std::string& name) const {
    auto it = nameMap.find(name);
    if (it != nameMap.end()) {
        return it->second;
    }
    return nullptr;
}

//==============================================================================
// ParameterManager
//==============================================================================
void ParameterManager::setup(const OscSettings& settings) {
    oscReceivePort = settings.receivePort;
    oscSendIP = settings.sendIP;
    oscSendPort = settings.sendPort;
    
    if (settings.enabled) {
        setOscEnabled(true);
    }
    
    ofLogNotice("ParameterManager") << "Setup complete";
}

void ParameterManager::close() {
    setOscEnabled(false);
}

void ParameterManager::update() {
    if (!oscEnabled) return;
    
    // Process incoming OSC messages
    while (oscReceiver.hasWaitingMessages()) {
        ofxOscMessage msg;
        oscReceiver.getNextMessage(msg);
        processOscMessage(msg);
    }
}

void ParameterManager::registerGroup(std::shared_ptr<ParameterGroup> group) {
    groups.push_back(group);
    rebuildParameterMap();
}

std::shared_ptr<ParameterGroup> ParameterManager::getGroup(const std::string& name) const {
    for (auto& group : groups) {
        if (group->getName() == name) {
            return group;
        }
    }
    return nullptr;
}

std::shared_ptr<ParameterBase> ParameterManager::getParameter(const std::string& oscAddress) const {
    auto it = allParameters.find(oscAddress);
    if (it != allParameters.end()) {
        return it->second;
    }
    return nullptr;
}

void ParameterManager::rebuildParameterMap() {
    allParameters.clear();
    for (auto& group : groups) {
        for (auto& param : group->getParameters()) {
            allParameters[param->getOscAddress()] = param;
        }
    }
}

void ParameterManager::processOscMessage(const ofxOscMessage& msg) {
    std::string address = msg.getAddress();
    
    auto param = getParameter(address);
    if (param) {
        if (msg.getArgType(0) == OFXOSC_TYPE_FLOAT) {
            param->setFromFloat(msg.getArgAsFloat(0));
            param->notifyChanged();
        } else if (msg.getArgType(0) == OFXOSC_TYPE_INT32) {
            param->setFromInt(msg.getArgAsInt32(0));
            param->notifyChanged();
        }
    } else {
        ofLogVerbose("ParameterManager") << "Unknown OSC address: " << address;
    }
}

void ParameterManager::sendParameter(const std::string& address, float value) {
    if (!oscEnabled) return;
    
    ofxOscMessage msg;
    msg.setAddress(address);
    msg.addFloatArg(value);
    oscSender.sendMessage(msg);
}

void ParameterManager::sendParameter(const std::string& address, int value) {
    if (!oscEnabled) return;
    
    ofxOscMessage msg;
    msg.setAddress(address);
    msg.addIntArg(value);
    oscSender.sendMessage(msg);
}

void ParameterManager::sendParameter(const std::string& address, bool value) {
    sendParameter(address, value ? 1.0f : 0.0f);
}

void ParameterManager::sendString(const std::string& address, const std::string& value) {
    if (!oscEnabled) return;
    
    ofxOscMessage msg;
    msg.setAddress(address);
    msg.addStringArg(value);
    oscSender.sendMessage(msg);
}

void ParameterManager::sendAllParameters() {
    if (!oscEnabled) return;
    
    for (auto& pair : allParameters) {
        auto& param = pair.second;
        sendParameter(param->getOscAddress(), param->getAsFloat());
    }
}

void ParameterManager::sendGroupParameters(const std::string& groupName) {
    if (!oscEnabled) return;
    
    auto group = getGroup(groupName);
    if (group) {
        for (auto& param : group->getParameters()) {
            sendParameter(param->getOscAddress(), param->getAsFloat());
        }
    }
}

void ParameterManager::setOscEnabled(bool enabled) {
    if (enabled == oscEnabled) return;
    
    if (enabled) {
        oscReceiver.setup(oscReceivePort);
        oscSender.setup(oscSendIP, oscSendPort);
        ofLogNotice("ParameterManager") << "OSC enabled on port " << oscReceivePort;
    } else {
        oscReceiver.stop();
        ofLogNotice("ParameterManager") << "OSC disabled";
    }
    
    oscEnabled = enabled;
}

void ParameterManager::reloadOscSettings() {
    if (!oscEnabled) return;
    
    oscReceiver.stop();
    oscReceiver.setup(oscReceivePort);
    oscSender.setup(oscSendIP, oscSendPort);
    
    ofLogNotice("ParameterManager") << "OSC settings reloaded";
}

void ParameterManager::setupMidi(const MidiSettings& settings) {
    midiIn = std::make_unique<ofxMidiIn>();
    midiPortNames.clear();
    
    // List available ports
    midiPortNames = midiIn->getInPortList();
    
    if (settings.enabled && settings.selectedPort >= 0) {
        connectMidiPort(settings.selectedPort);
    }
}

void ParameterManager::closeMidi() {
    if (midiIn) {
        midiIn->closePort();
        midiIn.reset();
    }
    midiEnabled = false;
}

void ParameterManager::refreshMidiPorts() {
    if (!midiIn) return;
    midiPortNames = midiIn->getInPortList();
}

std::vector<std::string> ParameterManager::getMidiPortNames() const {
    return midiPortNames;
}

void ParameterManager::connectMidiPort(int portIndex) {
    if (!midiIn) return;
    
    midiIn->closePort();
    midiIn->openPort(portIndex);
    midiIn->addListener(this);
    midiIn->addListener(this);
    midiEnabled = true;
    
    ofLogNotice("ParameterManager") << "Connected to MIDI port " << portIndex;
}

void ParameterManager::newMidiMessage(ofxMidiMessage& msg) {
    // Forward to processing function
    processMidiMessage(msg);
}

void ParameterManager::processMidiMessage(ofxMidiMessage& msg) {
    if (msg.status == MIDI_CONTROL_CHANGE) {
        int cc = msg.control;
        float value = msg.value / 127.0f;
        
        auto it = midiMappings.find(cc);
        if (it != midiMappings.end()) {
            const MidiMapping& mapping = it->second;
            
            // Potentiometer latching logic
            auto param = getParameter(mapping.paramAddress);
            if (param) {
                float currentVal = param->getAsFloat();
                float mappedValue = ofMap(value, 0, 1, mapping.minValue, mapping.maxValue);
                
                if (!midiActive[cc]) {
                    // Check if controller is close to current value
                    if (abs(mappedValue - currentVal) < MIDI_THRESHOLD) {
                        midiActive[cc] = true;
                    }
                } else {
                    param->setFromFloat(mappedValue);
                    param->notifyChanged();
                    
                    // Send OSC update
                    sendParameter(mapping.paramAddress, param->getAsFloat());
                }
            }
        }
    }
}

void ParameterManager::addMidiMapping(int ccNumber, const std::string& paramAddress, 
                                       float minVal, float maxVal) {
    MidiMapping mapping;
    mapping.ccNumber = ccNumber;
    mapping.paramAddress = paramAddress;
    mapping.minValue = minVal;
    mapping.maxValue = maxVal;
    
    midiMappings[ccNumber] = mapping;
    midiActive[ccNumber] = false;
}

void ParameterManager::clearMidiMappings() {
    midiMappings.clear();
    midiActive.clear();
}

void ParameterManager::saveMidiMappings(const std::string& path) {
    ofJson json;
    for (auto& pair : midiMappings) {
        ofJson mapping;
        mapping["cc"] = pair.second.ccNumber;
        mapping["address"] = pair.second.paramAddress;
        mapping["min"] = pair.second.minValue;
        mapping["max"] = pair.second.maxValue;
        json.push_back(mapping);
    }
    ofSaveJson(path, json);
}

void ParameterManager::loadMidiMappings(const std::string& path) {
    ofJson json = ofLoadJson(path);
    if (json.is_array()) {
        clearMidiMappings();
        for (auto& mapping : json) {
            if (mapping.contains("cc") && mapping.contains("address")) {
                addMidiMapping(
                    mapping["cc"],
                    mapping["address"],
                    mapping.value("min", 0.0f),
                    mapping.value("max", 1.0f)
                );
            }
        }
    }
}

void ParameterManager::setMidiEnabled(bool enabled) {
    midiEnabled = enabled;
}

} // namespace dragonwaves
