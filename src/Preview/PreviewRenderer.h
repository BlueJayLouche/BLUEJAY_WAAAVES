#pragma once

#include "ofMain.h"
#include "../ShaderPipeline/PipelineManager.h"

namespace dragonwaves {

//==============================================================================
// Preview Renderer - Lightweight preview window for block output
//==============================================================================
class PreviewRenderer {
public:
    PreviewRenderer();
    ~PreviewRenderer();
    
    // Setup with desired preview size
    void setup(int width = 320, int height = 180);
    
    // Update preview from pipeline output
    // Copies FBO texture to shareable regular texture
    void update(PipelineManager& pipeline, int drawMode);
    
    // Draw the preview at specified position (for in-app display)
    void draw(int x, int y, int w = 0, int h = 0);
    
    // Color picking
    ofColor pickColor(int x, int y);
    ofColor getLastPickedColor() const { return lastPickedColor; }
    
    // Draw mode selection
    void setPreviewDrawMode(int mode) { previewDrawMode = mode; }
    int getPreviewDrawMode() const { return previewDrawMode; }
    
    // Get current preview texture (shareable copy)
    ofTexture& getPreviewTexture() { return previewTexture; }
    
    // Get current preview pixels (for cross-context drawing)
    const ofPixels& getPreviewPixels() const { return previewPixels; }
    
    // Get texture dimensions
    int getWidth() const { return previewWidth; }
    int getHeight() const { return previewHeight; }
    
    // Enable/disable
    void setEnabled(bool enable) { enabled = enable; }
    bool isEnabled() const { return enabled; }
    
    // Performance metrics
    float getLastUpdateTime() const { return lastUpdateTimeMs; }
    
private:
    int previewWidth = 320;
    int previewHeight = 180;
    int previewDrawMode = 2;  // Default to BLOCK3
    
    // We need a regular texture (not FBO) for cross-context sharing
    ofTexture previewTexture;
    ofPixels previewPixels;  // For copying data
    
    ofColor lastPickedColor = ofColor::black;
    bool colorPending = false;
    
    bool enabled = true;
    bool initialized = false;
    bool textureNeedsUpdate = false;
    
    float lastUpdateTimeMs = 0.0f;
    
    // Get texture from appropriate block
    ofTexture& getBlockTexture(PipelineManager& pipeline, int blockNum);
    
    // Copy FBO texture to shareable texture
    void copyTextureData(ofTexture& sourceTexture);
};

} // namespace dragonwaves
