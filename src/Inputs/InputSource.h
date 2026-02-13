#pragma once

#include "ofMain.h"

namespace dragonwaves {

// Input source types
enum class InputType {
    NONE,
    WEBCAM,
    NDI,
    SPOUT,
    VIDEO_FILE
};

//==============================================================================
// Base class for all input sources
//==============================================================================
class InputSource {
public:
    InputSource() = default;
    virtual ~InputSource() = default;
    
    // Initialize the input source
    virtual bool setup(int width, int height) = 0;
    
    // Update the input (call each frame)
    virtual void update() = 0;
    
    // Close/release the input
    virtual void close() = 0;
    
    // Get the current texture
    virtual ofTexture& getTexture() = 0;
    
    // Check if a new frame is available
    virtual bool isFrameNew() const = 0;
    
    // Check if the input is initialized and ready
    virtual bool isInitialized() const = 0;
    
    // Get input type
    virtual InputType getType() const = 0;
    
    // Get input name
    virtual std::string getName() const = 0;
    
    // Draw into an FBO at specified resolution
    void drawToFbo(ofFbo& fbo);
    
    // Get native width/height
    virtual int getNativeWidth() const { return nativeWidth; }
    virtual int getNativeHeight() const { return nativeHeight; }
    
protected:
    int nativeWidth = 0;
    int nativeHeight = 0;
    bool initialized = false;
};

} // namespace dragonwaves
