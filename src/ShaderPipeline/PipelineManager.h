#pragma once

#include "Block1Shader.h"
#include "Block2Shader.h"
#include "Block3Shader.h"
#include "../Core/SettingsManager.h"

namespace dragonwaves {

//==============================================================================
// Frame buffer for delay/feedback
//==============================================================================
class DelayBuffer {
public:
    static constexpr int MAX_FRAMES = 120;
    
    void setup(int width, int height);
    void resize(int width, int height);
    
    // Push new frame to buffer
    void pushFrame(ofFbo& frame);
    
    // Get frame at specified delay (0 = most recent)
    ofTexture& getFrame(int delay);
    
    // Clear all frames
    void clear();
    
    int getSize() const { return MAX_FRAMES; }
    
private:
    std::array<ofFbo, MAX_FRAMES> frames;
    int writeIndex = 0;
    int width = 0;
    int height = 0;
    bool initialized = false;
};

//==============================================================================
// Main shader pipeline manager
//==============================================================================
class PipelineManager {
public:
    PipelineManager();
    ~PipelineManager();
    
    // Initialize with settings
    void setup(const DisplaySettings& settings);
    
    // Process one frame through the pipeline
    void processFrame();
    
    // Input textures
    void setInput1Texture(ofTexture& tex);
    void setInput2Texture(ofTexture& tex);
    
    // Get outputs
    ofTexture& getBlock1Output();
    ofTexture& getBlock2Output();
    ofTexture& getFinalOutput();
    ofFbo& getBlock1Fbo();
    ofFbo& getBlock2Fbo();
    ofFbo& getBlock3Fbo();
    
    // Get shader blocks for parameter access
    Block1Shader& getBlock1() { return block1; }
    Block2Shader& getBlock2() { return block2; }
    Block3Shader& getBlock3() { return block3; }
    
    // Get delay buffers
    DelayBuffer& getFB1DelayBuffer() { return fb1Delay; }
    DelayBuffer& getFB2DelayBuffer() { return fb2Delay; }
    
    // Reinitialize with new resolution
    void reinitialize(const DisplaySettings& settings);
    
    // Clear feedback buffers
    void clearFB1();
    void clearFB2();
    void clearAll();
    
    // Draw mode
    enum DrawMode {
        DRAW_BLOCK1 = 0,
        DRAW_BLOCK2,
        DRAW_BLOCK3,
        DRAW_ALL_BLOCKS
    };
    
    void setDrawMode(DrawMode mode) { drawMode = mode; }
    DrawMode getDrawMode() const { return drawMode; }
    
    // Feedback delay times
    void setFB1DelayTime(int frames);
    void setFB2DelayTime(int frames);
    int getFB1DelayTime() const { return fb1DelayTime; }
    int getFB2DelayTime() const { return fb2DelayTime; }
    
private:
    Block1Shader block1;
    Block2Shader block2;
    Block3Shader block3;
    
    DelayBuffer fb1Delay;
    DelayBuffer fb2Delay;
    
    ofTexture* input1Tex = nullptr;
    ofTexture* input2Tex = nullptr;
    ofTexture dummyTexture;
    
    DisplaySettings displaySettings;
    
    DrawMode drawMode = DRAW_BLOCK3;
    
    int fb1DelayTime = 1;
    int fb2DelayTime = 1;
    
    bool initialized = false;
    
    // Cached mesh for Block3 rendering (avoid recreation every frame)
    ofMesh block3Mesh;
    void updateBlock3Mesh(int width, int height);
    
    void allocateDummyTexture();
};

} // namespace dragonwaves

// Backwards compatibility
typedef dragonwaves::PipelineManager PipelineManager;
