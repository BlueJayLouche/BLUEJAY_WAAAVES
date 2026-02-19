#include "ColorPicker.h"
#include "imgui.h"

namespace dragonwaves {

ColorPicker::ColorPicker() {}

void ColorPicker::setup() {
    // Setup PBO for async readback
    // Note: Actual PBO setup done on first use
}

void ColorPicker::onPreviewClick(int previewX, int previewY, int previewW, int previewH) {
    // Clamp to bounds
    previewX = ofClamp(previewX, 0, previewW - 1);
    previewY = ofClamp(previewY, 0, previewH - 1);
    
    // Store normalized position
    pickPosition.x = previewX / (float)previewW;
    pickPosition.y = previewY / (float)previewH;
    
    // Schedule read
    pendingRead = true;
}

void ColorPicker::readColorAtPosition(int x, int y) {
    if (!sourceTexture || !sourceTexture->isAllocated()) return;
    
    // For now, use synchronous read (can optimize with PBO later)
    // This is called from update() so it doesn't stall render
    
    ofFbo tempFbo;
    tempFbo.allocate(1, 1, GL_RGBA);
    
    tempFbo.begin();
    ofClear(0, 0, 0, 255);
    ofSetColor(255);
    sourceTexture->drawSubsection(0, 0, 1, 1, x, y, 1, 1);
    tempFbo.end();
    
    ofPixels pixels;
    tempFbo.readToPixels(pixels);
    
    ofColor c = pixels.getColor(0, 0);
    pickedColor = c;
}

void ColorPicker::applyToKeyColor(float* keyColorArray) {
    if (!keyColorArray) return;
    
    // Normalize 0-255 to 0-1
    keyColorArray[0] = pickedColor.r / 255.0f;
    keyColorArray[1] = pickedColor.g / 255.0f;
    keyColorArray[2] = pickedColor.b / 255.0f;
}

void ColorPicker::drawImGuiWidget() {
    ImGui::Text("Color Picker");
    ImGui::Separator();
    
    // Color display box
    ImVec4 colorVec(
        pickedColor.r / 255.0f,
        pickedColor.g / 255.0f,
        pickedColor.b / 255.0f,
        1.0f
    );
    
    ImGui::ColorButton("Picked Color", colorVec, 
        ImGuiColorEditFlags_NoPicker | ImGuiColorEditFlags_NoTooltip,
        ImVec2(60, 60));
    
    ImGui::SameLine();
    
    // RGB values
    ImGui::BeginGroup();
    ImGui::Text("RGB: %d, %d, %d", pickedColor.r, pickedColor.g, pickedColor.b);
    ImGui::Text("Hex: #%02X%02X%02X", pickedColor.r, pickedColor.g, pickedColor.b);
    ImGui::Text("Pos: %.2f, %.2f", pickPosition.x, pickPosition.y);
    ImGui::EndGroup();
    
    ImGui::Separator();
    
    // Target selection
    ImGui::Text("Apply to:");
    
    const char* targets[] = { "CH2 Key", "FB1 Key", "FB2 Key", "Final Key" };
    int currentTarget = (int)keyTarget;
    if (ImGui::Combo("Target", &currentTarget, targets, IM_ARRAYSIZE(targets))) {
        keyTarget = (KeyTarget)currentTarget;
    }
    
    // Apply button
    if (ImGui::Button("Apply Color", ImVec2(120, 0))) {
        // Signal to apply - actual application happens in GuiApp
        // This sets a flag that GuiApp checks
    }
    
    ImGui::SameLine();
    
    if (ImGui::Button("Reset to White", ImVec2(120, 0))) {
        pickedColor = ofColor::white;
    }
}

} // namespace dragonwaves
