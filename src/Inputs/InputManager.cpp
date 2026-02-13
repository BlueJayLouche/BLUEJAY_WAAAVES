#include "InputManager.h"

namespace dragonwaves {

//==============================================================================
// InputSlot
//==============================================================================
void InputSlot::allocateFbo(int width, int height) {
    ofFboSettings settings;
    settings.width = width;
    settings.height = height;
    settings.internalformat = GL_RGBA8;
    settings.useDepth = false;
    settings.useStencil = false;
    fbo.allocate(settings);
    fbo.begin();
    ofClear(0, 0, 0, 255);
    fbo.end();
}

void InputSlot::update() {
    if (source && source->isInitialized()) {
        source->update();
        if (source->isFrameNew()) {
            // Draw to FBO at internal resolution
            fbo.begin();
            ofViewport(0, 0, fbo.getWidth(), fbo.getHeight());
            ofSetupScreenOrtho(fbo.getWidth(), fbo.getHeight());
            ofClear(0, 0, 0, 255);
            source->getTexture().draw(0, 0, fbo.getWidth(), fbo.getHeight());
            fbo.end();
        }
    }
}

ofTexture& InputSlot::getOutputTexture() {
    return fbo.getTexture();
}

//==============================================================================
// InputManager
//==============================================================================
InputManager::InputManager() {
}

InputManager::~InputManager() {
}

void InputManager::setup(const DisplaySettings& settings) {
    displaySettings = settings;
    
    // Create shared input sources
    webcam1 = std::make_shared<WebcamInput>();
    webcam2 = std::make_shared<WebcamInput>();
    ndiInput1 = std::make_shared<NdiInput>();
    ndiInput2 = std::make_shared<NdiInput>();
    spoutInput1 = std::make_shared<SpoutInput>();
    spoutInput2 = std::make_shared<SpoutInput>();
    videoInput1 = std::make_shared<VideoFileInput>();
    videoInput2 = std::make_shared<VideoFileInput>();
    
    // Allocate slot FBOs
    slot1.allocateFbo(settings.internalWidth, settings.internalHeight);
    slot2.allocateFbo(settings.internalWidth, settings.internalHeight);
    
    // Setup slots with default configuration (webcam)
    slot1.slotIndex = 1;
    slot2.slotIndex = 2;
    
    ofLogNotice("InputManager") << "Setup complete. Internal resolution: " 
                                 << settings.internalWidth << "x" << settings.internalHeight;
}

void InputManager::update() {
    slot1.update();
    slot2.update();
}

void InputManager::configureInput1(InputType type, int deviceOrSourceIndex, const std::string& videoPath) {
    setupInputSource(slot1, type, deviceOrSourceIndex, videoPath);
}

void InputManager::configureInput2(InputType type, int deviceOrSourceIndex, const std::string& videoPath) {
    setupInputSource(slot2, type, deviceOrSourceIndex, videoPath);
}

void InputManager::setupInputSource(InputSlot& slot, InputType type, int deviceOrSourceIndex, const std::string& videoPath) {
    
    // Determine which new source object will be used
    std::shared_ptr<InputSource> newSource = nullptr;
    switch (type) {
        case InputType::WEBCAM:
            newSource = (slot.slotIndex == 1) ? webcam1 : webcam2;
            break;
        case InputType::NDI:
            newSource = (slot.slotIndex == 1) ? ndiInput1 : ndiInput2;
            break;
        case InputType::SPOUT:
            newSource = (slot.slotIndex == 1) ? spoutInput1 : spoutInput2;
            break;
        case InputType::VIDEO_FILE:
            newSource = (slot.slotIndex == 1) ? videoInput1 : videoInput2;
            break;
        default:
            newSource = nullptr;
            break;
    }
    
    // Only close the current source if we're switching to a DIFFERENT source type
    // If we're reusing the same source object (e.g., changing webcam device ID),
    // don't close it - just reconfigure it
    if (slot.source && slot.source != newSource) {
        slot.source->close();
    }
    
    slot.configuredType = type;
    slot.configuredDeviceID = deviceOrSourceIndex;
    slot.configuredSourceIndex = deviceOrSourceIndex;
    slot.configuredVideoPath = videoPath;
    slot.source = newSource;
    
    switch (type) {
        case InputType::WEBCAM:
            if (slot.slotIndex == 1) {
                webcam1->close();  // Close first to ensure clean state
                webcam1->setDeviceID(deviceOrSourceIndex);
                webcam1->setup(displaySettings.input1Width, displaySettings.input1Height);
            } else {
                webcam2->close();  // Close first to ensure clean state
                webcam2->setDeviceID(deviceOrSourceIndex);
                webcam2->setup(displaySettings.input2Width, displaySettings.input2Height);
            }
            break;
            
        case InputType::NDI:
            if (slot.slotIndex == 1) {
                // NDI can reconfigure without full close/setup cycle
                if (!ndiInput1->isInitialized()) {
                    ndiInput1->setup(displaySettings.internalWidth, displaySettings.internalHeight);
                }
                if (deviceOrSourceIndex >= 0) {
                    ndiInput1->selectSource(deviceOrSourceIndex);
                }
            } else {
                if (!ndiInput2->isInitialized()) {
                    ndiInput2->setup(displaySettings.internalWidth, displaySettings.internalHeight);
                }
                if (deviceOrSourceIndex >= 0) {
                    ndiInput2->selectSource(deviceOrSourceIndex);
                }
            }
            break;
            
        case InputType::SPOUT:
            if (slot.slotIndex == 1) {
                if (!spoutInput1->isInitialized()) {
                    spoutInput1->setup(displaySettings.internalWidth, displaySettings.internalHeight);
                }
                if (deviceOrSourceIndex >= 0) {
                    spoutInput1->selectSource(deviceOrSourceIndex);
                }
            } else {
                if (!spoutInput2->isInitialized()) {
                    spoutInput2->setup(displaySettings.internalWidth, displaySettings.internalHeight);
                }
                if (deviceOrSourceIndex >= 0) {
                    spoutInput2->selectSource(deviceOrSourceIndex);
                }
            }
            break;
            
        case InputType::VIDEO_FILE:
            if (slot.slotIndex == 1) {
                videoInput1->close();
                videoInput1->setup(displaySettings.internalWidth, displaySettings.internalHeight);
                if (!videoPath.empty()) {
                    videoInput1->load(videoPath);
                    videoInput1->play();
                }
            } else {
                videoInput2->close();
                videoInput2->setup(displaySettings.internalWidth, displaySettings.internalHeight);
                if (!videoPath.empty()) {
                    videoInput2->load(videoPath);
                    videoInput2->play();
                }
            }
            break;
            
        default:
            slot.source = nullptr;
            break;
    }
}

ofTexture& InputManager::getInput1Texture() {
    return slot1.getOutputTexture();
}

ofTexture& InputManager::getInput2Texture() {
    return slot2.getOutputTexture();
}

ofTexture& InputManager::getInput1SourceTexture() {
    if (slot1.source) {
        return slot1.source->getTexture();
    }
    return slot1.getOutputTexture();
}

ofTexture& InputManager::getInput2SourceTexture() {
    if (slot2.source) {
        return slot2.source->getTexture();
    }
    return slot2.getOutputTexture();
}

bool InputManager::isInput1Ready() const {
    return slot1.source && slot1.source->isInitialized();
}

bool InputManager::isInput2Ready() const {
    return slot2.source && slot2.source->isInitialized();
}

bool InputManager::isInput1FrameNew() const {
    return slot1.source && slot1.source->isFrameNew();
}

bool InputManager::isInput2FrameNew() const {
    return slot2.source && slot2.source->isFrameNew();
}

InputType InputManager::getInput1Type() const {
    return slot1.configuredType;
}

InputType InputManager::getInput2Type() const {
    return slot2.configuredType;
}

void InputManager::reinitialize(const DisplaySettings& settings) {
    displaySettings = settings;
    
    // Reallocate FBOs
    slot1.allocateFbo(settings.internalWidth, settings.internalHeight);
    slot2.allocateFbo(settings.internalWidth, settings.internalHeight);
    
    // Reconfigure sources with new settings
    InputType type1 = slot1.configuredType;
    InputType type2 = slot2.configuredType;
    int index1 = slot1.configuredSourceIndex;
    int index2 = slot2.configuredSourceIndex;
    std::string path1 = slot1.configuredVideoPath;
    std::string path2 = slot2.configuredVideoPath;
    
    setupInputSource(slot1, type1, index1, path1);
    setupInputSource(slot2, type2, index2, path2);
    
    ofLogNotice("InputManager") << "Reinitialized with new resolution";
}

void InputManager::refreshNdiSources() {
    if (ndiInput1) ndiInput1->refreshSources();
    if (ndiInput2) ndiInput2->refreshSources();
}

void InputManager::refreshSpoutSources() {
    if (spoutInput1) spoutInput1->refreshSources();
    if (spoutInput2) spoutInput2->refreshSources();
}

std::vector<std::string> InputManager::getNdiSourceNames() const {
    if (ndiInput1) {
        return ndiInput1->getSourceNames();
    }
    return std::vector<std::string>();
}

std::vector<std::string> InputManager::getSpoutSourceNames() const {
    if (spoutInput1) {
        return spoutInput1->getSourceNames();
    }
    return std::vector<std::string>();
}

void InputManager::allocateFbos() {
    slot1.allocateFbo(displaySettings.internalWidth, displaySettings.internalHeight);
    slot2.allocateFbo(displaySettings.internalWidth, displaySettings.internalHeight);
}

} // namespace dragonwaves
