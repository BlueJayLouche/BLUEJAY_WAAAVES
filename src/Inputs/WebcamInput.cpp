#include "WebcamInput.h"

namespace dragonwaves {

WebcamInput::WebcamInput() {
}

WebcamInput::~WebcamInput() {
    close();
}

bool WebcamInput::setup(int width, int height) {
    nativeWidth = width;
    nativeHeight = height;
    
    grabber.setVerbose(true);
    grabber.setDeviceID(deviceID);
    grabber.setDesiredFrameRate(desiredFrameRate);
    
    initialized = grabber.setup(width, height);
    
    if (initialized) {
        ofLogNotice("WebcamInput") << "Initialized device " << deviceID 
                                    << " at " << width << "x" << height;
    } else {
        ofLogError("WebcamInput") << "Failed to initialize device " << deviceID;
    }
    
    return initialized;
}

void WebcamInput::update() {
    if (initialized) {
        grabber.update();
    }
}

void WebcamInput::close() {
    if (initialized) {
        grabber.close();
        initialized = false;
    }
}

ofTexture& WebcamInput::getTexture() {
    return grabber.getTexture();
}

bool WebcamInput::isFrameNew() const {
    return initialized && grabber.isFrameNew();
}

bool WebcamInput::isInitialized() const {
    return initialized;
}

std::string WebcamInput::getName() const {
    return "Webcam " + std::to_string(deviceID);
}

void WebcamInput::setDeviceID(int id) {
    deviceID = id;
}

std::vector<ofVideoDevice> WebcamInput::listDevices() {
    ofVideoGrabber tempGrabber;
    return tempGrabber.listDevices();
}

} // namespace dragonwaves
