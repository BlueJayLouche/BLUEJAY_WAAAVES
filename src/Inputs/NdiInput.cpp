#include "NdiInput.h"

namespace dragonwaves {

NdiInput::NdiInput() {
}

NdiInput::~NdiInput() {
    close();
}

bool NdiInput::setup(int width, int height) {
    nativeWidth = width;
    nativeHeight = height;
    
    // Allocate texture at internal resolution
    texture.allocate(width, height, GL_RGBA);
    
    // Clear to black
    ofPixels blackPixels;
    blackPixels.allocate(width, height, OF_PIXELS_RGBA);
    blackPixels.setColor(ofColor::black);
    texture.loadData(blackPixels);
    
    initialized = true;
    
    // Refresh sources to populate list
    refreshSources();
    
    ofLogNotice("NdiInput") << "Initialized";
    return true;
}

void NdiInput::update() {
    if (!initialized) return;
    
    // Receive image into texture
    frameIsNew = receiver.ReceiveImage(texture);
}

void NdiInput::close() {
    receiver.ReleaseReceiver();
    initialized = false;
}

ofTexture& NdiInput::getTexture() {
    return texture;
}

bool NdiInput::isFrameNew() const {
    return frameIsNew;
}

bool NdiInput::isInitialized() const {
    return initialized;
}

std::string NdiInput::getName() const {
    if (selectedSourceIndex >= 0 && selectedSourceIndex < sourceNames.size()) {
        return "NDI: " + sourceNames[selectedSourceIndex];
    }
    return "NDI: (No Source)";
}

void NdiInput::refreshSources() {
    sourceNames.clear();
    
    // Get NDI source list
    auto sources = receiver.GetSenderList();
    
    for (const auto& source : sources) {
        sourceNames.push_back(source);
    }
    
    ofLogNotice("NdiInput") << "Found " << sourceNames.size() << " NDI sources";
}

std::vector<std::string> NdiInput::getSourceNames() const {
    return sourceNames;
}

void NdiInput::selectSource(int index) {
    if (index < 0 || index >= sourceNames.size()) return;
    
    selectedSourceIndex = index;
    
    // Release current and create new receiver for selected source
    receiver.ReleaseReceiver();
    
    receiver.CreateReceiver(index);
    
    ofLogNotice("NdiInput") << "Selected source: " << sourceNames[index];
}

} // namespace dragonwaves
