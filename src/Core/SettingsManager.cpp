#include "SettingsManager.h"

namespace dragonwaves {

//==============================================================================
// DisplaySettings
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

void DisplaySettings::saveToXml(ofxXmlSettings& xml) {
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

//==============================================================================
// OscSettings
//==============================================================================
void OscSettings::loadFromXml(ofxXmlSettings& xml) {
    xml.pushTag("osc");
    
    enabled = xml.getValue("enabled", 0) == 1;
    receivePort = xml.getValue("receivePort", 7000);
    sendIP = xml.getValue("sendIP", "127.0.0.1");
    sendPort = xml.getValue("sendPort", 7001);
    
    xml.popTag();
}

void OscSettings::saveToXml(ofxXmlSettings& xml) {
    xml.addTag("osc");
    xml.pushTag("osc");
    
    xml.setValue("enabled", enabled ? 1 : 0);
    xml.setValue("receivePort", receivePort);
    xml.setValue("sendIP", sendIP);
    xml.setValue("sendPort", sendPort);
    
    xml.popTag();
}

//==============================================================================
// MidiSettings
//==============================================================================
void MidiSettings::loadFromXml(ofxXmlSettings& xml) {
    xml.pushTag("midi");
    
    selectedPort = xml.getValue("selectedPort", -1);
    deviceName = xml.getValue("deviceName", "");
    enabled = xml.getValue("enabled", 0) == 1;
    
    xml.popTag();
}

void MidiSettings::saveToXml(ofxXmlSettings& xml) {
    xml.addTag("midi");
    xml.pushTag("midi");
    
    xml.setValue("selectedPort", selectedPort);
    xml.setValue("deviceName", deviceName);
    xml.setValue("enabled", enabled ? 1 : 0);
    
    xml.popTag();
}

//==============================================================================
// InputSourceSettings
//==============================================================================
void InputSourceSettings::loadFromXml(ofxXmlSettings& xml) {
    xml.pushTag("inputSources");
    
    input1SourceType = xml.getValue("input1SourceType", 1);  // Default to Webcam (1)
    input2SourceType = xml.getValue("input2SourceType", 1);  // Default to Webcam (1)
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

void InputSourceSettings::saveToXml(ofxXmlSettings& xml) {
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
// SettingsManager
//==============================================================================
void SettingsManager::load() {
    ofxXmlSettings xml;
    std::string settingsPath = getSettingsFile();
    
    if (xml.loadFile(settingsPath)) {
        ofLogNotice("SettingsManager") << "Loading settings from " << settingsPath;
        
        // Load all settings sections
        if (xml.tagExists("display")) {
            display.loadFromXml(xml);
        }
        if (xml.tagExists("osc")) {
            osc.loadFromXml(xml);
        }
        if (xml.tagExists("midi")) {
            midi.loadFromXml(xml);
        }
        if (xml.tagExists("inputSources")) {
            inputSources.loadFromXml(xml);
        }
        
        // UI settings
        uiScaleIndex = xml.getValue("uiScaleIndex", 0);
        
        ofLogNotice("SettingsManager") << "Settings loaded successfully";
    } else {
        ofLogNotice("SettingsManager") << "No settings file found, using defaults";
        // Save defaults
        save();
    }
}

void SettingsManager::save() {
    ofxXmlSettings xml;
    
    // Save all settings sections
    display.saveToXml(xml);
    osc.saveToXml(xml);
    midi.saveToXml(xml);
    inputSources.saveToXml(xml);
    
    // UI settings
    xml.setValue("uiScaleIndex", uiScaleIndex);
    
    std::string settingsPath = getSettingsFile();
    if (xml.saveFile(settingsPath)) {
        ofLogNotice("SettingsManager") << "Settings saved to " << settingsPath;
    } else {
        ofLogError("SettingsManager") << "Failed to save settings to " << settingsPath;
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

} // namespace dragonwaves
