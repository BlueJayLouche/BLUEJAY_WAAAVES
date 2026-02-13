#pragma once

#include "ofMain.h"
#include "InputSource.h"
#include "WebcamInput.h"
#include "NdiInput.h"
#include "SpoutInput.h"
#include "VideoFileInput.h"
#include "../Core/SettingsManager.h"

namespace dragonwaves {

//==============================================================================
// Input slot configuration
//==============================================================================
struct InputSlot {
    int slotIndex;
    std::shared_ptr<InputSource> source;
    ofFbo fbo;  // Scaled to internal resolution
    InputType configuredType = InputType::NONE;
    int configuredDeviceID = 0;
    int configuredSourceIndex = 0;
    std::string configuredVideoPath;
    
    void allocateFbo(int width, int height);
    void update();
    ofTexture& getOutputTexture();
};

//==============================================================================
// Central input management
//==============================================================================
class InputManager {
public:
    InputManager();
    ~InputManager();
    
    // Initialize with display settings
    void setup(const DisplaySettings& settings);
    
    // Update all inputs
    void update();
    
    // Configure inputs
    void configureInput1(InputType type, int deviceOrSourceIndex = 0, const std::string& videoPath = "");
    void configureInput2(InputType type, int deviceOrSourceIndex = 0, const std::string& videoPath = "");
    
    // Get output textures (scaled to internal resolution)
    ofTexture& getInput1Texture();
    ofTexture& getInput2Texture();
    
    // Get source textures (native resolution)
    ofTexture& getInput1SourceTexture();
    ofTexture& getInput2SourceTexture();
    
    // Check if inputs are ready
    bool isInput1Ready() const;
    bool isInput2Ready() const;
    
    // Check if new frames available
    bool isInput1FrameNew() const;
    bool isInput2FrameNew() const;
    
    // Get current input types
    InputType getInput1Type() const;
    InputType getInput2Type() const;
    
    // Reinitialize with new resolution
    void reinitialize(const DisplaySettings& settings);
    
    // Source management
    void refreshNdiSources();
    void refreshSpoutSources();
    std::vector<std::string> getNdiSourceNames() const;
    std::vector<std::string> getSpoutSourceNames() const;
    
    // Getters for specific input sources
    std::shared_ptr<NdiInput> getNdiInput1() { return ndiInput1; }
    std::shared_ptr<NdiInput> getNdiInput2() { return ndiInput2; }
    std::shared_ptr<SpoutInput> getSpoutInput1() { return spoutInput1; }
    std::shared_ptr<SpoutInput> getSpoutInput2() { return spoutInput2; }
    std::shared_ptr<VideoFileInput> getVideoInput1() { return videoInput1; }
    std::shared_ptr<VideoFileInput> getVideoInput2() { return videoInput2; }
    
private:
    InputSlot slot1;
    InputSlot slot2;
    
    // Shared input sources (can be switched between slots)
    std::shared_ptr<WebcamInput> webcam1;
    std::shared_ptr<WebcamInput> webcam2;
    std::shared_ptr<NdiInput> ndiInput1;
    std::shared_ptr<NdiInput> ndiInput2;
    std::shared_ptr<SpoutInput> spoutInput1;
    std::shared_ptr<SpoutInput> spoutInput2;
    std::shared_ptr<VideoFileInput> videoInput1;
    std::shared_ptr<VideoFileInput> videoInput2;
    
    DisplaySettings displaySettings;
    
    void setupInputSource(InputSlot& slot, InputType type, int deviceOrSourceIndex, const std::string& videoPath);
    void allocateFbos();
};

} // namespace dragonwaves

// Backwards compatibility
typedef dragonwaves::InputManager InputManager;
typedef dragonwaves::InputType InputType;
