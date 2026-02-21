# Preview Window Integration Guide

This guide shows how to integrate the preview window into the existing BLUEJAY_WAAAVES application.

## Overview

The Preview Window is an ImGui panel that displays a scaled-down preview of any shader block (B1, B2, B3) and provides a color picker for chroma-keying. It lives in the **Control Window** (GuiApp) but displays content from the **Output Window** (ofApp).

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         DUAL WINDOW ARCHITECTURE                            │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│   ┌─────────────────────────────┐      ┌─────────────────────────────┐     │
│   │    CONTROL WINDOW           │      │      OUTPUT WINDOW          │     │
│   │    (GuiApp)                 │      │      (ofApp)                │     │
│   │                             │      │                             │     │
│   │  ┌─────────────────────┐    │      │   ┌───────────────────┐     │     │
│   │  │  [ImGui Interface]  │    │      │   │  Block 1 Shader   │     │     │
│   │  │                     │    │      │   ├───────────────────┤     │     │
│   │  │  ┌───────────────┐  │    │      │   │  Block 2 Shader   │     │     │
│   │  │  │ Preview Panel │  │◄───┼──────┼───│  Block 3 Shader   │     │     │
│   │  │  │  - Shows B1/  │  │    │      │   │      (Final)      │     │     │
│   │  │  │    B2/B3      │  │    │      │   └─────────┬─────────┘     │     │
│   │  │  │  - Color Pick │  │    │      │             │               │     │
│   │  │  └───────────────┘  │    │      │    ┌────────▼────────┐      │     │
│   │  │                     │    │      │    │    NDI/Spout    │      │     │
│   │  └─────────────────────┘    │      │    └─────────────────┘      │     │
│   └─────────────────────────────┘      └─────────────────────────────┘     │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Quick Start

### 1. Verify Preview Files Exist

The Preview module files should already exist in `src/Preview/`:
```
src/Preview/
├── PreviewRenderer.h       # Renders preview from pipeline FBOs
├── PreviewRenderer.cpp
├── ColorPicker.h           # Async pixel color sampling
├── ColorPicker.cpp
├── PreviewPanel.h          # ImGui UI wrapper
└── PreviewPanel.cpp
```

**If files are missing:** Copy them from the source location to `src/Preview/`

**Verify Project Configuration:**

**Xcode:**
- Ensure `src/Preview/` folder is in the project navigator
- All 6 files should be visible under "src" group

**Makefile:**
Verify in `config.make`:
```makefile
SOURCES += src/Preview/PreviewRenderer.cpp
SOURCES += src/Preview/ColorPicker.cpp
SOURCES += src/Preview/PreviewPanel.cpp
```

---

### 2. Modify main.cpp

Add the PreviewPanel include and forward declaration before `main()`:

```cpp
#include "ofMain.h"
#include "ofApp.h"
#include "GuiApp.h"
#include "ofAppGLFWWindow.h"

// Forward declaration for preview panel
namespace dragonwaves {
    class PreviewPanel;
}

//========================================================================
int main() {
    // ... existing OpenGL settings ...
    
    // Create and link apps
    shared_ptr<ofApp> mainApp(new ofApp);
    shared_ptr<GuiApp> guiApp(new GuiApp);
    mainApp->gui = guiApp;
    guiApp->mainApp = mainApp.get();
    mainApp->mainWindow = mainWindow;
    guiApp->guiWindow = guiWindow;
    
    // Initialize Preview Panel (created in ofApp::setup)
    // The previewPanel pointer will be passed from ofApp to GuiApp
    
    ofRunApp(guiWindow, guiApp);
    ofRunApp(mainWindow, mainApp);
    ofRunMainLoop();
}
```

**No other changes needed in main.cpp** - the preview panel is created and managed within ofApp and passed to GuiApp via the existing `mainApp->gui` reference.

---

### 3. Modify ofApp.h

```cpp
// Add at top
#include "Preview/PreviewPanel.h"

class ofApp : public ofBaseApp {
    // ... existing members ...
    
    // Preview window
    std::unique_ptr<dragonwaves::PreviewPanel> previewPanel;
    
    // Allow GuiApp to access preview panel
    friend class GuiApp;
};
```

---

### 4. Modify ofApp.cpp

```cpp
void ofApp::setup() {
    // ... existing setup code ...
    
    // Initialize preview panel (AFTER pipeline is created)
    previewPanel = std::make_unique<dragonwaves::PreviewPanel>();
    previewPanel->setup(pipeline.get());
    
    // Optional: Set up color applied callback
    previewPanel->onColorApplied = [this](dragonwaves::ColorPicker::KeyTarget target, ofColor color) {
        // Convert to normalized float (0-1 range)
        float r = color.r / 255.0f;
        float g = color.g / 255.0f;
        float b = color.b / 255.0f;
        
        // Apply to appropriate GUI parameters
        switch (target) {
            case dragonwaves::ColorPicker::CH2_KEY:
                gui->ch2MixAndKey[1] = r;
                gui->ch2MixAndKey[2] = g;
                gui->ch2MixAndKey[3] = b;
                break;
            case dragonwaves::ColorPicker::FB1_KEY:
                gui->fb1MixAndKey[1] = r;
                gui->fb1MixAndKey[2] = g;
                gui->fb1MixAndKey[3] = b;
                break;
            case dragonwaves::ColorPicker::FB2_KEY:
                gui->fb2MixAndKey[1] = r;
                gui->fb2MixAndKey[2] = g;
                gui->fb2MixAndKey[3] = b;
                break;
            case dragonwaves::ColorPicker::FINAL_KEY:
                // Add final mix key parameters if they exist
                break;
        }
        
        // Send OSC notification
        if (gui->mainApp) {
            gui->mainApp->sendOscParameter("/gravity/preview/colorPicked", 1.0f);
        }
    };
    
    // Pass preview panel to GuiApp (must be done AFTER gui is assigned)
    if (gui) {
        gui->previewPanel = previewPanel.get();
    }
}

void ofApp::update() {
    // ... existing update code ...
    
    // Update preview at throttled rate
    if (previewPanel && gui && gui->showPreviewWindow) {
        previewPanel->update();
    }
}

void ofApp::exit() {
    // Clean up preview panel before pipeline is destroyed
    previewPanel.reset();
    
    // ... existing exit code ...
}
```

---

### 5. Modify GuiApp.h

```cpp
// Forward declaration (add with other forward declarations)
namespace dragonwaves {
    class PreviewPanel;
}

class GuiApp : public ofBaseApp, public ofxMidiListener {
public:
    // ... existing members ...
    
    // Preview window
    bool showPreviewWindow = true;
    
    // Access to preview panel (set by ofApp)
    dragonwaves::PreviewPanel* previewPanel = nullptr;
};
```

---

### 6. Modify GuiApp.cpp

```cpp
void GuiApp::draw() {
    // ... existing GUI code (menu bar, panels, etc.) ...
    
    // Draw preview panel
    if (previewPanel && showPreviewWindow) {
        previewPanel->draw();
    }
    
    // ... rest of GUI ...
}
```

Add a menu item to toggle preview (typically in your main menu bar):
```cpp
// In your menu bar or settings section
if (ImGui::BeginMenu("Window")) {
    ImGui::MenuItem("Preview & Color Picker", NULL, &showPreviewWindow);
    ImGui::EndMenu();
}
```

---

## Configuration Options

### Change Preview Size

```cpp
// In ofApp::setup()
previewPanel->setPreviewSize(240, 135);  // Smaller
previewPanel->setPreviewSize(640, 360);  // Larger (more GPU memory)
```

### Change Update Rate

```cpp
// Limit to 15 FPS to save GPU
previewPanel->setUpdateRate(15);
```

### Disable by Default

```cpp
// In ofApp::setup()
previewPanel->setVisible(false);  // Hidden by default
gui->showPreviewWindow = false;
```

---

## Data Flow Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                            DATA FLOW                                        │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│   ofApp (Output Window)              PreviewPanel (in GuiApp)               │
│   ┌─────────────────────┐            ┌─────────────────────┐               │
│   │                     │            │                     │               │
│   │  PipelineManager    │            │  PreviewRenderer    │               │
│   │  ┌───────────────┐  │            │  ┌───────────────┐  │               │
│   │  │  Block1 FBO   │──┼────────────┼─>│  Preview FBO  │  │               │
│   │  ├───────────────┤  │  copy via  │  │  (320x180)    │  │               │
│   │  │  Block2 FBO   │──┼──draw()───>│  └───────┬───────┘  │               │
│   │  ├───────────────┤  │            │          │          │               │
│   │  │  Block3 FBO   │──┘            │    ImGui::Image()   │               │
│   │  └───────────────┘               │          │          │               │
│   │                     │            │          ▼          │               │
│   │                     │            │  ┌───────────────┐  │               │
│   │                     │            │  │  ColorPicker  │  │               │
│   │                     │            │  │  (async PBO   │  │               │
│   │                     │            │  │   readback)   │  │               │
│   │                     │            │  └───────────────┘  │               │
│   └─────────────────────┘            └─────────────────────┘               │
│                                                                             │
│   User clicks preview ──> Async pixel read ──> Color applied to GUI params │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Performance Considerations

### Memory Usage
- Preview FBO: ~220KB (320x180 RGBA)
- Pixel buffer: ~220KB
- Total: < 1MB overhead

### GPU Time
- Copy operation: ~0.05ms per update
- At 30 FPS: ~1.5ms per second (0.15% of frame time)
- When hidden: 0 cost

### Recommendations
1. **Keep default size** (320x180) - good balance of visibility and performance
2. **Limit to 30 FPS** - visual feedback doesn't need 60 FPS
3. **Hide when not needed** - zero cost when disabled
4. **Use B3/All mode** - shows final output, most useful

---

## Troubleshooting

### Preview is black
- Check that pipeline is producing output
- Verify draw mode is set correctly (0=B1, 1=B2, 2=B3)
- Check OpenGL texture binding
- Ensure preview panel is updated in ofApp::update()

### Color picker not working
- Ensure you click inside preview image
- Check that pixel readback is working (some GPUs/drivers)
- Try synchronous readback in ColorPicker.cpp if async fails

### Preview not showing in GUI
- Verify `gui->previewPanel` is set in ofApp::setup()
- Check that `showPreviewWindow` is true
- Ensure PreviewPanel::draw() is called in GuiApp::draw()

### Performance impact
- Reduce preview size
- Lower update FPS
- Hide preview when not actively using

### Crash on startup
- Verify all files are added to project
- Check for null pointers (pipeline, gui)
- Ensure OpenGL context is active before creating preview panel
- Make sure pipeline is created BEFORE preview panel in ofApp::setup()

### Build errors
- Verify `#include "Preview/PreviewPanel.h"` path is correct
- Check that all 6 Preview source files are compiled
- Ensure C++14 or later is enabled (for std::make_unique)

---

## Advanced Usage

### Custom Color Application

```cpp
previewPanel->onColorApplied = [this](auto target, ofColor color) {
    // Custom logic here
    // e.g., Apply to multiple keys, log colors, etc.
};
```

### Access Raw Preview Texture

```cpp
ofTexture& tex = previewPanel->getRenderer().getPreviewTexture();
// Use for custom drawing, recording, etc.
```

### Programmatic Control

```cpp
// Change displayed block
previewPanel->getRenderer().setPreviewDrawMode(1);  // Show Block2

// Get picked color
ofColor c = previewPanel->getColorPicker().getPickedColor();

// Check if visible
bool visible = previewPanel->isVisible();
```

---

## Future Enhancements

See `PREVIEW_WINDOW_PLAN.md` for planned features:
- Histogram display
- Waveform/vectorscope
- Zoom functionality
- Color memory slots
