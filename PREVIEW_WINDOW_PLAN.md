# Preview Window Implementation Plan

## Overview
A compact, performant preview window with drawblock selection and color dropper for chroma-keying. Designed to have minimal impact on the main rendering pipeline.

---

## 1. Architecture

### 1.1 Design Principles
- **Zero Re-rendering**: Preview copies from existing FBOs, never re-runs shaders
- **Async Pixel Read**: Color picking uses asynchronous pixel readback to avoid GPU stalls
- **Fixed Small Size**: Preview is capped at 320x180 (16:9) or 240x180 (4:3)
- **Optional**: Can be toggled on/off to save resources

### 1.2 Component Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                        ofApp                                     │
│  ┌─────────────────┐    ┌──────────────────┐    ┌───────────┐  │
│  │ PipelineManager │    │ PreviewRenderer  │    │ ColorPicker│  │
│  │                 │    │                  │    │            │  │
│  │ ┌─────────────┐ │    │ ┌──────────────┐ │    │ ┌────────┐ │  │
│  │ │ Block1 FBO  │─┼────┼>│ Preview FBO  │ │    │ │ Pixels │ │  │
│  │ ├─────────────┤ │    │ │ 320x180      │ │    │ └────────┘ │  │
│  │ │ Block2 FBO  │─┼────┼>│ (scaled copy)│ │    │            │  │
│  │ ├─────────────┤ │    │ └──────────────┘ │    └────────────┘  │
│  │ │ Block3 FBO  │─┼────┘                  │                      │
│  │ └─────────────┘ │                       │                      │
│  └─────────────────┘                       │                      │
│                                            │                      │
│  ┌──────────────────────────────────────┐ │                      │
│  │ GuiApp                                │ │                      │
│  │  ┌────────────────────────────────┐  │ │                      │
│  │  │ ImGui Preview Panel            │  │ │                      │
│  │  │ ┌──────────────────────────┐   │  │ │                      │
│  │  │ │ Preview Image (ImGui)    │   │  │ │                      │
│  │  │ │ [B1][B2][B3][All]        │   │  │ │                      │
│  │  │ │ Click to pick color      │   │  │ │                      │
│  │  │ └──────────────────────────┘   │  │ │                      │
│  │  │ ┌──────────────────────────┐   │  │ │                      │
│  │  │ │ Color Info               │   │  │ │                      │
│  │  │ │ RGB: 255, 0, 128         │   │  │ │                      │
│  │  │ │ [Apply to CH2 Key]       │   │  │ │                      │
│  │  │ └──────────────────────────┘   │  │ │                      │
│  │  └────────────────────────────────┘  │ │                      │
│  └──────────────────────────────────────┘ │                      │
└───────────────────────────────────────────┴──────────────────────┘
```

---

## 2. Implementation Details

### 2.1 Files to Create

```
src/Preview/
├── PreviewRenderer.h/.cpp    # Main preview rendering
├── ColorPicker.h/.cpp        # Color sampling logic
└── PreviewPanel.h/.cpp       # ImGui UI integration
```

### 2.2 Files to Modify

```
src/ofApp.h
- Add: std::unique_ptr<PreviewRenderer> previewRenderer;
- Add: bool showPreviewWindow = true;

src/ofApp.cpp
- setup(): Initialize previewRenderer
- update(): Call previewRenderer->update() if enabled
- draw(): Optional - preview can be separate window

src/GuiApp.h
- Add preview panel controls
- Add color picker state

src/GuiApp.cpp
- draw(): Add ImGui preview panel
```

---

## 3. Key Features

### 3.1 Drawblock Selection

**UI Layout:**
```
┌─────────────────────────────┐
│ Preview: [B1] [B2] [B3] [✓All] │  <- Toggle buttons
│ ┌─────────────────────────┐ │
│ │                         │ │
│ │     Preview Image       │ │  <- Shows selected block
│ │     (click to pick)     │ │
│ │                         │ │
│ └─────────────────────────┘ │
└─────────────────────────────┘
```

**Implementation:**
```cpp
// In GuiApp::draw()
ImGui::Begin("Preview");

// Block selection buttons
if (ImGui::Button("B1")) previewRenderer.setPreviewDrawMode(0);
ImGui::SameLine();
if (ImGui::Button("B2")) previewRenderer.setPreviewDrawMode(1);
ImGui::SameLine();
if (ImGui::Button("B3")) previewRenderer.setPreviewDrawMode(2);
ImGui::SameLine();
if (ImGui::Button("All")) previewRenderer.setPreviewDrawMode(2); // Same as B3

// Preview image using ImGui::Image
ofTexture& tex = previewRenderer.getPreviewTexture();
ImTextureID texID = (ImTextureID)(uintptr_t)tex.getTextureData().textureID;
ImVec2 size(previewWidth, previewHeight);

// Handle click for color picking
if (ImGui::ImageButton("preview", texID, size)) {
    ImVec2 mousePos = ImGui::GetMousePos();
    ImVec2 widgetPos = ImGui::GetItemRectMin();
    int x = mousePos.x - widgetPos.x;
    int y = mousePos.y - widgetPos.y;
    colorPicker.onPreviewClick(x, y, previewWidth, previewHeight);
}

ImGui::End();
```

### 3.2 Color Dropper

**UI Layout:**
```
┌─────────────────────────────┐
│ Color Picker                │
│ ┌─────┬───────────────────┐ │
│ │ ▓▓▓ │ RGB: 255, 128, 0  │ │  <- Live color display
│ │ ▓▓▓ │ Hex: #FF8000      │ │
│ └─────┴───────────────────┘ │
│ [Apply to CH2 Key]          │  <- Target selection + apply
│ [Apply to FB1 Key]          │
│ [Apply to FB2 Key]          │
└─────────────────────────────┘
```

**Implementation Strategy:**

1. **Hover Preview**: Show color under cursor in real-time (use last frame's pixel data)
2. **Click to Pick**: Click freezes the color selection
3. **Async Readback**: Use PBO (Pixel Buffer Object) for non-blocking GPU→CPU transfer

```cpp
void ColorPicker::onPreviewClick(int x, int y, int previewW, int previewH) {
    // Convert to normalized coordinates
    pickPosition.x = x / (float)previewW;
    pickPosition.y = y / (float)previewH;
    
    // Schedule async read (actual read happens next frame)
    pendingRead = true;
}

void ColorPicker::update() {
    if (pendingRead && sourceTexture) {
        // Use PBO for async readback
        if (!pboAllocated) {
            glGenBuffers(1, &pboID);
            glBindBuffer(GL_PIXEL_PACK_BUFFER, pboID);
            glBufferData(GL_PIXEL_PACK_BUFFER, 4, NULL, GL_STREAM_READ);
            glBindBuffer(GL_PIXEL_PACK_BUFFER, 0);
            pboAllocated = true;
        }
        
        // Read pixel at pick position
        int x = pickPosition.x * sourceTexture->getWidth();
        int y = pickPosition.y * sourceTexture->getHeight();
        
        sourceTexture->bind();
        glBindBuffer(GL_PIXEL_PACK_BUFFER, pboID);
        glReadPixels(x, y, 1, 1, GL_RGBA, GL_UNSIGNED_BYTE, 0);
        glBindBuffer(GL_PIXEL_PACK_BUFFER, 0);
        sourceTexture->unbind();
        
        // Map buffer to get data (non-blocking if using PBO properly)
        glBindBuffer(GL_PIXEL_PACK_BUFFER, pboID);
        GLubyte* data = (GLubyte*)glMapBuffer(GL_PIXEL_PACK_BUFFER, GL_READ_ONLY);
        if (data) {
            pickedColor = ofColor(data[0], data[1], data[2]);
            glUnmapBuffer(GL_PIXEL_PACK_BUFFER);
        }
        glBindBuffer(GL_PIXEL_PACK_BUFFER, 0);
        
        pendingRead = false;
    }
}
```

---

## 4. Performance Optimizations

### 4.1 Zero-Cost Preview (When Hidden)
```cpp
void PreviewRenderer::update(PipelineManager& pipeline, int drawMode) {
    if (!enabled || !visible) return;  // Early exit
    // ... rest of update
}
```

### 4.2 Throttled Updates
Update preview at lower FPS than main render:
```cpp
// In ofApp::update()
float now = ofGetElapsedTimef();
if (now - lastPreviewUpdate > previewUpdateInterval) {  // e.g., 30fps max
    previewRenderer->update(*pipeline, gui->drawMode);
    lastPreviewUpdate = now;
}
```

### 4.3 Efficient Scaling
Use GPU bilinear filtering instead of shader:
```cpp
// In PreviewRenderer::copyFromBlockX()
srcTex.draw(0, 0, previewWidth, previewHeight);  // Hardware scaling
```

### 4.4 Lazy Pixel Read
Only read pixels when color picker is active:
```cpp
void PreviewRenderer::asyncReadPixels() {
    if (!colorPickerActive) return;  // Skip if not needed
    // ... read pixels
}
```

### 4.5 Expected Performance Impact
- **Preview Update**: ~0.05ms per frame (FBO copy only)
- **Color Pick**: ~0.1ms (one-time, only on click)
- **Memory**: +~1MB (320x180 RGBA FBO + pixels buffer)

---

## 5. Integration Steps

### Step 1: Create Preview Module
1. Create `src/Preview/` directory
2. Add `PreviewRenderer.h/cpp`
3. Add to Xcode project / Makefile

### Step 2: Integrate into ofApp
```cpp
// ofApp.h
#include "Preview/PreviewRenderer.h"

class ofApp : public ofBaseApp {
    // ... existing members
    std::unique_ptr<dragonwaves::PreviewRenderer> previewRenderer;
};

// ofApp.cpp
void ofApp::setup() {
    // ... existing setup
    previewRenderer = std::make_unique<PreviewRenderer>();
    previewRenderer->setup(320, 180);
}

void ofApp::update() {
    // ... existing update
    if (gui && gui->showPreviewWindow) {
        previewRenderer->update(*pipeline, gui->previewDrawMode);
    }
}
```

### Step 3: Add ImGui Panel
```cpp
// In GuiApp::draw()
void GuiApp::drawPreviewPanel() {
    if (!showPreviewWindow) return;
    
    ImGui::Begin("Preview & Color Picker", &showPreviewWindow);
    
    // Block selection
    // Preview image
    // Color picker controls
    
    ImGui::End();
}
```

### Step 4: Wire Color Application
```cpp
void GuiApp::applyPickedColorToKey() {
    ofColor c = colorPicker.getPickedColor();
    
    // Normalize to 0-1 range for shader uniforms
    float r = c.r / 255.0f;
    float g = c.g / 255.0f;
    float b = c.b / 255.0f;
    
    switch (colorPicker.getKeyTarget()) {
        case CH2_KEY:
            ch2MixAndKey[1] = r;
            ch2MixAndKey[2] = g;
            ch2MixAndKey[3] = b;
            break;
        case FB1_KEY:
            fb1MixAndKey[1] = r;
            fb1MixAndKey[2] = g;
            fb1MixAndKey[3] = b;
            break;
        // ... etc
    }
}
```

---

## 6. UI Mockup

```
┌─────────────────────────────────────────────────────────────┐
│ Preview & Color Picker                          [X]         │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Source: [B1] [B2] [B3✓] [All]                             │
│  ┌─────────────────────────────┐                            │
│  │                             │  ┌──────────────────────┐  │
│  │     [Preview Image]         │  │ Picked Color         │  │
│  │                             │  │ ┌────┐ R: 255       │  │
│  │     Click to sample         │  │ │████│ G: 0         │  │
│  │                             │  │ │████│ B: 128       │  │
│  │                             │  │ └────┘ Hex: #FF0080  │  │
│  └─────────────────────────────┘  └──────────────────────┘  │
│                                                             │
│  Apply to:  (•) CH2 Key  ( ) FB1 Key  ( ) FB2 Key         │
│                                                             │
│  [      Apply Color      ]  [ Reset to White ]             │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│  Settings                                                   │
│  [✓] Show Preview    [✓] Show Crosshair    FPS: 30          │
└─────────────────────────────────────────────────────────────┘
```

---

## 7. Testing Checklist

- [ ] Preview shows correct block when switching draw modes
- [ ] Preview updates at configured FPS (not every frame)
- [ ] Color picker shows correct RGB values
- [ ] Color applies to correct key target
- [ ] No performance degradation when preview is hidden
- [ ] Works with all input types (webcam, NDI, Spout)
- [ ] Crosshair shows exact pixel being sampled
- [ ] Preview aspect ratio matches output

---

## 8. Future Enhancements

1. **Histogram Display**: Show RGB histogram of current block
2. **Waveform/Vectorscope**: Broadcast-style video analysis
3. **Split Screen**: A/B comparison between blocks
4. **Zoom**: 2x/4x zoom for precise color picking
5. **Color Memory**: Save/restore multiple key colors
