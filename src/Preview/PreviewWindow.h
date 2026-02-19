#pragma once

#include "ofMain.h"
#include "ofAppGLFWWindow.h"
#include <GLFW/glfw3.h>
#include "PreviewRenderer.h"
#include "ColorPicker.h"

namespace dragonwaves {

class PreviewRenderer;  // Forward declaration

//==============================================================================
// Preview Window - Dedicated GL window for video preview
//==============================================================================
class PreviewWindow {
public:
    PreviewWindow();
    ~PreviewWindow();
    
    // Create the preview window
    // parentWindow: the main output window to share context with
    bool setup(PreviewRenderer* renderer, ColorPicker* colorPicker);
    
    // Update window (call every frame)
    void update();
    
    // Draw the preview content
    void draw();
    
    // Show/hide window
    void show();
    void hide();
    void toggle();
    bool isVisible() const;
    
    // Position control
    void setPosition(int x, int y);
    ofVec2f getPosition() const;
    
    // Check if window should close
    bool shouldClose() const;
    
    // Get GLFW window handle
    GLFWwindow* getGLFWWindow() const { return glfwWindow; }
    
    // Set preview pixels (called from PreviewRenderer)
    void setPreviewPixels(const ofPixels& pixels);
    
    // Callback for color picked
    std::function<void(ColorPicker::KeyTarget, ofColor)> onColorPicked;

private:
    ColorPicker* colorPicker = nullptr;
    
    shared_ptr<ofAppGLFWWindow> previewOfWindow;
    GLFWwindow* glfwWindow = nullptr;
    
    int windowWidth = 640;   // Larger window to show scaled preview
    int windowHeight = 360;
    int windowX = 100;
    int windowY = 100;
    
    bool visible = false;
    bool initialized = false;
    
    // Local copy of pixels for drawing (avoids cross-context texture issues)
    ofPixels localPixels;
    ofFbo previewFbo;
    bool pixelsDirty = false;
    ofMutex pixelsMutex;
    
    // Mouse handling
    bool mousePressed = false;
    int mouseX = 0;
    int mouseY = 0;
    
    // Setup callbacks
    void setupCallbacks();
    
    // GLFW callbacks (static, forward to instance)
    static void mouseButtonCallback(GLFWwindow* window, int button, int action, int mods);
    static void cursorPosCallback(GLFWwindow* window, double xpos, double ypos);
    static void keyCallback(GLFWwindow* window, int key, int scancode, int action, int mods);
    static void windowCloseCallback(GLFWwindow* window);
    
    // Instance callbacks
    void onMouseButton(int button, int action, int mods);
    void onCursorPos(double xpos, double ypos);
    void onKey(int key, int scancode, int action, int mods);
    void onWindowClose();
    
    // Perform color pick at mouse position
    void performColorPick();
    
    // Get the PreviewWindow instance from GLFW window
    static PreviewWindow* getInstance(GLFWwindow* window);
};

} // namespace dragonwaves
