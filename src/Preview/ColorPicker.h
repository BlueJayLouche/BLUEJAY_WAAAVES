#pragma once

#include "ofMain.h"

namespace dragonwaves {

//==============================================================================
// Color Picker - Interactive color sampling from preview
//==============================================================================
class ColorPicker {
public:
    ColorPicker();
    
    void setup();
    
    // Call when user clicks on preview
    void onPreviewClick(int previewX, int previewY, int previewW, int previewH);
    
    // Set source texture for color picking
    void setSourceTexture(ofTexture* tex) { sourceTexture = tex; }
    
    // Get picked color
    ofColor getPickedColor() const { return pickedColor; }
    ofColor getHoveredColor() const { return hoveredColor; }
    
    // Set picked color directly (from external picker)
    void setPickedColor(ofColor color) { pickedColor = color; }
    
    // Get normalized position (0-1) of last pick
    glm::vec2 getPickPosition() const { return pickPosition; }
    
    // Apply picked color to GUI key parameters
    void applyToKeyColor(float* keyColorArray);
    
    // Target block selection (for key application)
    enum KeyTarget {
        CH2_KEY = 0,   // Block1 channel 2
        FB1_KEY,       // Block1 feedback
        FB2_KEY,       // Block2 feedback
        FINAL_KEY      // Block3 final mix
    };
    
    void setKeyTarget(KeyTarget target) { keyTarget = target; }
    KeyTarget getKeyTarget() const { return keyTarget; }
    
    // ImGui widget
    void drawImGuiWidget();
    
private:
    ofTexture* sourceTexture = nullptr;
    ofPixels pixelBuffer;
    
    ofColor pickedColor = ofColor::white;
    ofColor hoveredColor = ofColor::white;
    glm::vec2 pickPosition = glm::vec2(0.5f, 0.5f);
    glm::vec2 hoverPosition = glm::vec2(0.5f, 0.5f);
    
    KeyTarget keyTarget = CH2_KEY;
    
    bool pendingRead = false;
    
    void readColorAtPosition(int x, int y);
};

} // namespace dragonwaves
