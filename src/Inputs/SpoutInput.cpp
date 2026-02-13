#include "SpoutInput.h"

namespace dragonwaves {

SpoutInput::SpoutInput() {
}

SpoutInput::~SpoutInput() {
    close();
}

bool SpoutInput::setup(int width, int height) {
    nativeWidth = width;
    nativeHeight = height;
    
    // Allocate texture
    texture.allocate(width, height, GL_RGBA);
    
    // Clear to black
    ofPixels blackPixels;
    blackPixels.allocate(width, height, OF_PIXELS_RGBA);
    blackPixels.setColor(ofColor::black);
    texture.loadData(blackPixels);
    
#if SPOUT_AVAILABLE
    receiver.init();
    initialized = true;
    refreshSources();
#else
    initialized = false;
    ofLogWarning("SpoutInput") << "Spout not available on this platform";
#endif
    
    return initialized;
}

void SpoutInput::update() {
    if (!initialized) return;
    
#if SPOUT_AVAILABLE
    // Receive texture
    frameIsNew = receiver.receive(texture);
#endif
}

void SpoutInput::close() {
#if SPOUT_AVAILABLE
    receiver.release();
#endif
    initialized = false;
}

ofTexture& SpoutInput::getTexture() {
    return texture;
}

bool SpoutInput::isFrameNew() const {
    return initialized && frameIsNew;
}

bool SpoutInput::isInitialized() const {
    return initialized;
}

std::string SpoutInput::getName() const {
#if SPOUT_AVAILABLE
    if (selectedSourceIndex >= 0 && selectedSourceIndex < sourceNames.size()) {
        return "Spout: " + sourceNames[selectedSourceIndex];
    }
#endif
    return "Spout: (No Source)";
}

void SpoutInput::refreshSources() {
#if SPOUT_AVAILABLE
    sourceNames.clear();
    
    // Get Spout sender list
    std::vector<std::string> senders = receiver.getSenderList();
    sourceNames = senders;
    
    ofLogNotice("SpoutInput") << "Found " << sourceNames.size() << " Spout senders";
#endif
}

std::vector<std::string> SpoutInput::getSourceNames() const {
#if SPOUT_AVAILABLE
    return sourceNames;
#else
    return std::vector<std::string>();
#endif
}

void SpoutInput::selectSource(int index) {
#if SPOUT_AVAILABLE
    if (index < 0 || index >= sourceNames.size()) return;
    
    selectedSourceIndex = index;
    std::string senderName = sourceNames[index];
    
    receiver.release();
    receiver.init();
    
    ofLogNotice("SpoutInput") << "Selected sender: " << senderName;
#endif
}

} // namespace dragonwaves
