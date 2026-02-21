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
    
    // Create the NDI finder to discover sources on the network
    receiver.CreateFinder();
    
    initialized = true;
    
    // Refresh sources to populate list
    refreshSources();
    
    ofLogNotice("NdiInput") << "Initialized";
    return true;
}

void NdiInput::update() {
    if (!initialized) return;
    
    // Receive directly into texture
    // ofxNDIreceiver handles format conversion (BGRA->RGBA) internally
    frameIsNew = receiver.ReceiveImage(texture);
    
    // Track receiver connection state
    receiverConnected = receiver.ReceiverConnected();
    
    // Calculate received FPS (only count actual new frames)
    float now = ofGetElapsedTimef();
    float delta = now - lastFrameTime;
    lastFrameTime = now;
    
    fpsTimer += delta;
    if (frameIsNew) {
        frameCounter++;
    }
    
    // Log FPS every 2 seconds
    if (fpsTimer >= 2.0f) {
        receivedFps = frameCounter / fpsTimer;
        float senderFps = receiver.GetSenderFps();
        int recvFps = receiver.GetFps();
        
        ofLogNotice("NdiInput") << "FPS: received=" << (int)receivedFps 
                                << " sender=" << (int)senderFps 
                                << " recv_calc=" << recvFps
                                << " connected=" << (receiverConnected ? "yes" : "no")
                                << " frameNew=" << (frameIsNew ? "yes" : "no")
                                << " texture=" << texture.getWidth() << "x" << texture.getHeight();
        
        frameCounter = 0;
        fpsTimer = 0;
    }
}

void NdiInput::close() {
    receiver.ReleaseReceiver();
    receiver.ReleaseFinder();
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
    if (selectedSourceIndex >= 0 && selectedSourceIndex < (int)sourceNames.size()) {
        return "NDI: " + sourceNames[selectedSourceIndex];
    }
    return "NDI: (No Source)";
}

void NdiInput::refreshSources() {
    // Find senders on the network - this updates the internal sender list
    // and returns the number of senders found
    int senderCount = receiver.FindSenders();
    
    // Get the updated NDI source list
    auto sources = receiver.GetSenderList();
    
    // Check if the list has changed
    bool listChanged = (sources.size() != sourceNames.size());
    if (!listChanged) {
        for (size_t i = 0; i < sources.size(); i++) {
            if (i >= sourceNames.size() || sources[i] != sourceNames[i]) {
                listChanged = true;
                break;
            }
        }
    }
    
    // Update our local copy
    sourceNames = sources;
    
    // Validate current selection
    if (selectedSourceIndex >= (int)sourceNames.size()) {
        selectedSourceIndex = sourceNames.empty() ? 0 : (int)sourceNames.size() - 1;
    }
    
    if (listChanged || senderCount > 0) {
        ofLogNotice("NdiInput") << "Found " << sourceNames.size() << " NDI sources";
        for (size_t i = 0; i < sourceNames.size(); i++) {
            ofLogNotice("NdiInput") << "  [" << i << "] " << sourceNames[i];
        }
    }
}

std::vector<std::string> NdiInput::getSourceNames() const {
    return sourceNames;
}

void NdiInput::selectSource(int index) {
    // Ensure sources are refreshed before selecting
    refreshSources();
    
    // Validate index bounds
    if (index < 0 || index >= (int)sourceNames.size()) {
        ofLogWarning("NdiInput") << "Invalid source index: " << index << " (available: " << sourceNames.size() << ")";
        return;
    }
    
    selectedSourceIndex = index;
    
    // Set the sender index before creating the receiver
    receiver.SetSenderIndex(index);
    
    // Release current and create new receiver for selected source
    receiver.ReleaseReceiver();
    
    // Create receiver for the selected sender (-1 means use the currently selected sender)
    bool created = receiver.CreateReceiver(-1);
    
    if (created) {
        ofLogNotice("NdiInput") << "Selected source: " << sourceNames[index];
    } else {
        ofLogError("NdiInput") << "Failed to create receiver for source: " << sourceNames[index];
    }
}

} // namespace dragonwaves
