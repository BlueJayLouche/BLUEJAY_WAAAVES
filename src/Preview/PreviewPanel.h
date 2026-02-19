#pragma once

#include "ofMain.h"
#include "ofxImGui.h"
#include "PreviewRenderer.h"
#include "PreviewWindow.h"
#include "ColorPicker.h"
#include "../ShaderPipeline/PipelineManager.h"

namespace dragonwaves {

//==============================================================================
// Preview Panel - ImGui integration for preview controls and GL window display
//==============================================================================
class PreviewPanel {
public:
    PreviewPanel();
    ~PreviewPanel();
    
    void setup(PipelineManager* pipeline);
    void update();
    void draw();
    
    // Settings
    void setVisible(bool visible);
    bool isVisible() const { return showPanel; }
    
    void setEnabled(bool enable) { enabled = enable; }
    bool isEnabled() const { return enabled; }
    
    // Toggle GL window
    void toggleWindow();
    void showWindow();
    void hideWindow();
    bool isWindowVisible() const;
    
    // Getters for external access
    PreviewRenderer& getRenderer() { return renderer; }
    ColorPicker& getColorPicker() { return colorPicker; }
    
    // Configuration
    void setUpdateRate(int fps);  // Limit preview FPS
    void setPreviewSize(int width, int height);
    void setWindowPosition(int x, int y);
    
    // Applied color callback (called when user clicks "Apply Color")
    std::function<void(ColorPicker::KeyTarget, ofColor)> onColorApplied;
    
private:
    PreviewRenderer renderer;
    ColorPicker colorPicker;
    PreviewWindow previewWindow;
    PipelineManager* pipeline = nullptr;
    
    bool showPanel = true;
    bool enabled = true;
    bool showCrosshair = true;
    bool windowMode = true;  // Use GL window instead of ImGui image
    
    int previewWidth = 320;
    int previewHeight = 180;
    int windowPosX = 100;
    int windowPosY = 100;
    float updateInterval = 1.0f / 30.0f;  // 30 FPS default
    float lastUpdateTime = 0.0f;
    
    // Picked color display
    ofColor lastPickedColor = ofColor::white;
    bool colorPickedThisFrame = false;
    
    void drawBlockSelector();
    void drawColorPickerSection();
    void drawSettingsSection();
    void drawWindowControls();
    
    // Handle color pick from window
    void onWindowColorPicked(ColorPicker::KeyTarget target, ofColor color);
};

} // namespace dragonwaves
