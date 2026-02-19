#include "PreviewPanel.h"
#include "imgui.h"
#include "ofxImGui.h"

namespace dragonwaves {

PreviewPanel::PreviewPanel() {}

PreviewPanel::~PreviewPanel() {
    // Window will be cleaned up automatically
}

void PreviewPanel::setup(PipelineManager* pipe) {
    pipeline = pipe;
    
    renderer.setup(previewWidth, previewHeight);
    colorPicker.setup();
    
    // Setup preview window
    if (windowMode) {
        previewWindow.setup(&renderer, &colorPicker);
        previewWindow.setPosition(windowPosX, windowPosY);
        
        // Setup callback for color picking
        previewWindow.onColorPicked = [this](ColorPicker::KeyTarget target, ofColor color) {
            this->lastPickedColor = color;
            this->colorPickedThisFrame = true;
        };
    }
    
    ofLogNotice("PreviewPanel") << "Setup complete (window mode: " << (windowMode ? "yes" : "no") << ")";
}

void PreviewPanel::update() {
    if (!enabled || !pipeline) return;
    
    float now = ofGetElapsedTimef();
    if (now - lastUpdateTime < updateInterval) return;
    
    // Get current draw mode from renderer
    int drawMode = renderer.getPreviewDrawMode();
    renderer.update(*pipeline, drawMode);
    
    // Update preview window with pixels (avoids cross-context texture issues)
    if (windowMode) {
        previewWindow.setPreviewPixels(renderer.getPreviewPixels());
    }
    
    lastUpdateTime = now;
}

void PreviewPanel::draw() {
    if (!showPanel || !enabled) return;
    
    // Draw preview window content if visible
    if (windowMode) {
        previewWindow.draw();
    }
    
    ImGui::Begin("Preview & Color Picker", &showPanel, 
        ImGuiWindowFlags_NoCollapse);
    
    drawWindowControls();
    ImGui::Separator();
    
    drawBlockSelector();
    ImGui::Separator();
    
    // Show picked color info
    ImGui::Text("Click in the preview window to sample colors");
    ImGui::TextDisabled("(ESC to hide window, SPACE to sample)");
    ImGui::Separator();
    
    drawColorPickerSection();
    ImGui::Separator();
    
    drawSettingsSection();
    
    ImGui::End();
}

void PreviewPanel::drawWindowControls() {
    ImGui::Text("Display:");
    ImGui::SameLine();
    
    if (ImGui::Button(isWindowVisible() ? "Hide Window" : "Show Window")) {
        toggleWindow();
    }
    
    ImGui::SameLine();
    
    if (ImGui::Button("Reset Position")) {
        setWindowPosition(100, 100);
    }
}

void PreviewPanel::drawBlockSelector() {
    ImGui::Text("Source:");
    ImGui::SameLine();
    
    int currentMode = renderer.getPreviewDrawMode();
    int newMode = currentMode;
    
    if (ImGui::RadioButton("B1", &newMode, 0)) {}
    ImGui::SameLine();
    if (ImGui::RadioButton("B2", &newMode, 1)) {}
    ImGui::SameLine();
    if (ImGui::RadioButton("B3", &newMode, 2)) {}
    
    if (newMode != currentMode) {
        renderer.setPreviewDrawMode(newMode);
    }
}

void PreviewPanel::drawColorPickerSection() {
    // Use last picked color or current picker color
    ofColor picked = colorPickedThisFrame ? lastPickedColor : colorPicker.getPickedColor();
    colorPickedThisFrame = false;  // Reset flag
    
    ImGui::Text("Picked Color:");
    
    // Color box
    ImVec4 colorVec(
        picked.r / 255.0f,
        picked.g / 255.0f,
        picked.b / 255.0f,
        1.0f
    );
    
    ImGui::ColorButton("##picked", colorVec,
        ImGuiColorEditFlags_NoPicker | ImGuiColorEditFlags_NoTooltip,
        ImVec2(50, 50));
    
    ImGui::SameLine();
    
    // RGB info
    ImGui::BeginGroup();
    ImGui::Text("R: %d  G: %d  B: %d", picked.r, picked.g, picked.b);
    ImGui::Text("Hex: #%02X%02X%02X", picked.r, picked.g, picked.b);
    ImGui::EndGroup();
    
    // Target selection
    ImGui::Text("Apply to:");
    
    const char* targets[] = { "CH2 Key", "FB1 Key", "FB2 Key", "Final Key" };
    int currentTarget = (int)colorPicker.getKeyTarget();
    
    if (ImGui::Combo("##target", &currentTarget, targets, IM_ARRAYSIZE(targets))) {
        colorPicker.setKeyTarget((ColorPicker::KeyTarget)currentTarget);
    }
    
    // Apply button
    if (ImGui::Button("Apply Color", ImVec2(100, 0))) {
        if (onColorApplied) {
            onColorApplied(colorPicker.getKeyTarget(), picked);
        }
    }
    
    ImGui::SameLine();
    
    if (ImGui::Button("Reset", ImVec2(60, 0))) {
        lastPickedColor = ofColor::white;
        colorPicker.setSourceTexture(nullptr);
    }
}

void PreviewPanel::drawSettingsSection() {
    if (!ImGui::CollapsingHeader("Settings")) return;
    
    // Update rate
    int fps = (int)(1.0f / updateInterval);
    if (ImGui::SliderInt("Preview FPS", &fps, 10, 60)) {
        updateInterval = 1.0f / fps;
    }
    
    // Show crosshair (window only)
    if (windowMode) {
        ImGui::Checkbox("Show Crosshair", &showCrosshair);
    }
    
    // Performance info
    ImGui::TextDisabled("Update time: %.3f ms", renderer.getLastUpdateTime());
    
    // Window position
    if (ImGui::InputInt("Window X", &windowPosX)) {
        setWindowPosition(windowPosX, windowPosY);
    }
    if (ImGui::InputInt("Window Y", &windowPosY)) {
        setWindowPosition(windowPosX, windowPosY);
    }
}

void PreviewPanel::setUpdateRate(int fps) {
    updateInterval = 1.0f / ofClamp(fps, 10, 60);
}

void PreviewPanel::setPreviewSize(int width, int height) {
    previewWidth = width;
    previewHeight = height;
    renderer.setup(width, height);
}

void PreviewPanel::setWindowPosition(int x, int y) {
    windowPosX = x;
    windowPosY = y;
    previewWindow.setPosition(x, y);
}

void PreviewPanel::setVisible(bool visible) {
    showPanel = visible;
    if (visible) {
        showWindow();
    } else {
        hideWindow();
    }
}

void PreviewPanel::toggleWindow() {
    previewWindow.toggle();
}

void PreviewPanel::showWindow() {
    previewWindow.show();
}

void PreviewPanel::hideWindow() {
    previewWindow.hide();
}

bool PreviewPanel::isWindowVisible() const {
    return previewWindow.isVisible();
}

void PreviewPanel::onWindowColorPicked(ColorPicker::KeyTarget target, ofColor color) {
    if (onColorApplied) {
        onColorApplied(target, color);
    }
}

} // namespace dragonwaves
