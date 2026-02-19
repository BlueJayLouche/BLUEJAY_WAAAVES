#include "PreviewWindow.h"
#include "PreviewRenderer.h"
#include <GLFW/glfw3.h>

namespace dragonwaves {

PreviewWindow::PreviewWindow() {}

PreviewWindow::~PreviewWindow() {
    if (glfwWindow) {
        // Don't destroy if it's the shared context window
        glfwSetWindowShouldClose(glfwWindow, true);
    }
}

void PreviewWindow::setPreviewPixels(const ofPixels& pixels) {
    ofScopedLock lock(pixelsMutex);
    // Just copy the pixels, don't create the texture yet
    // Texture will be created in the preview window's context during draw()
    if (!localPixels.isAllocated() || 
        localPixels.getWidth() != pixels.getWidth() || 
        localPixels.getHeight() != pixels.getHeight()) {
        localPixels.allocate(pixels.getWidth(), pixels.getHeight(), pixels.getNumChannels());
    }
    localPixels = pixels;
    pixelsDirty = true;
}

bool PreviewWindow::setup(PreviewRenderer* rend, ColorPicker* picker) {
    if (initialized) return true;
    
    colorPicker = picker;
    
    if (!rend) {
        ofLogError("PreviewWindow") << "No renderer provided";
        return false;
    }
    
    // We need to share context with the OUTPUT window (ofApp), not the GUI window
    // The output window is where the pipeline textures are rendered
    // Get it from the mainApp pointer if available
    GLFWwindow* outputGLFWWindow = nullptr;
    
    // Try to get the ofApp window (output window) from mainApp if set
    // Since we don't have direct access to mainApp here, we need to iterate windows
    // or use the fact that ofApp is usually the second window created
    
    // Alternative: use the currently active window but we need to ensure
    // we're sharing with the right one
    
    // For now, try to get any GLFW window and use it for sharing
    // The textures should be shared across all contexts if properly set up
    auto currentWindow = dynamic_cast<ofAppGLFWWindow*>(ofGetWindowPtr());
    if (!currentWindow) {
        ofLogError("PreviewWindow") << "Could not get current window";
        return false;
    }
    
    GLFWwindow* currentGLFWWindow = currentWindow->getGLFWWindow();
    if (!currentGLFWWindow) {
        ofLogError("PreviewWindow") << "Could not get GLFW window";
        return false;
    }
    
    ofLogNotice("PreviewWindow") << "Creating window with shared context";
    
    // Create window with shared context
    glfwWindowHint(GLFW_VISIBLE, visible ? GLFW_TRUE : GLFW_FALSE);
    glfwWindowHint(GLFW_RESIZABLE, GLFW_FALSE);
    glfwWindowHint(GLFW_DECORATED, GLFW_TRUE);
    glfwWindowHint(GLFW_FOCUSED, GLFW_FALSE);
    
    // On macOS with retina, we may want to disable automatic scaling
    // GLFW_COCOA_RETINA_FRAMEBUFFER controls whether to use full resolution
    #ifdef __APPLE__
    glfwWindowHint(GLFW_COCOA_RETINA_FRAMEBUFFER, GLFW_TRUE);
    ofLogNotice("PreviewWindow") << "macOS detected - enabling retina support";
    #endif
    
    glfwWindow = glfwCreateWindow(windowWidth, windowHeight, "Preview - Click to Pick Color", 
                                  NULL, currentGLFWWindow);
    
    if (!glfwWindow) {
        ofLogError("PreviewWindow") << "Failed to create GLFW window";
        return false;
    }
    
    // Query actual framebuffer size for diagnostics
    int actualFbWidth, actualFbHeight;
    glfwGetFramebufferSize(glfwWindow, &actualFbWidth, &actualFbHeight);
    
    ofLogNotice("PreviewWindow") << "Created: " << windowWidth << "x" << windowHeight
                                 << " (framebuffer: " << actualFbWidth << "x" << actualFbHeight << ")";
    
    // Set position
    glfwSetWindowPos(glfwWindow, windowX, windowY);
    
    // Setup callbacks
    setupCallbacks();
    
    // Store this instance in the window user pointer
    glfwSetWindowUserPointer(glfwWindow, this);
    
    initialized = true;
    
    return true;
}

void PreviewWindow::setupCallbacks() {
    if (!glfwWindow) return;
    
    glfwSetMouseButtonCallback(glfwWindow, mouseButtonCallback);
    glfwSetCursorPosCallback(glfwWindow, cursorPosCallback);
    glfwSetKeyCallback(glfwWindow, keyCallback);
    glfwSetWindowCloseCallback(glfwWindow, windowCloseCallback);
}

void PreviewWindow::update() {
    if (!initialized || !glfwWindow) return;
    
    // Poll events for this window
    glfwPollEvents();
    
    // Check if window should close
    if (glfwWindowShouldClose(glfwWindow)) {
        visible = false;
        glfwSetWindowShouldClose(glfwWindow, false);  // Reset for next time
    }
}

void PreviewWindow::draw() {
    if (!initialized || !glfwWindow || !visible) return;
    
    // Get the main window's context to restore later
    auto mainWindow = dynamic_cast<ofAppGLFWWindow*>(ofGetWindowPtr());
    
    // Switch to preview window context
    glfwMakeContextCurrent(glfwWindow);
    
    // Get framebuffer size
    int fbWidth, fbHeight;
    glfwGetFramebufferSize(glfwWindow, &fbWidth, &fbHeight);
    
    // Configure OF's global renderer for this window's dimensions
    auto renderer = ofGetGLRenderer();
    if (renderer) {
        renderer->viewport(0, 0, fbWidth, fbHeight, false);
        
        // Manual ortho setup: bottom-left origin (standard OF)
        glm::mat4 orthoMat = glm::ortho(0.f, (float)fbWidth, 0.f, (float)fbHeight, -1.f, 1.f);
        renderer->matrixMode(OF_MATRIX_PROJECTION);
        renderer->loadMatrix(orthoMat);
        renderer->matrixMode(OF_MATRIX_MODELVIEW);
        renderer->loadMatrix(glm::mat4(1.0));
    }
    
    ofClear(0, 0, 0, 255);
    
    // Update FBO if needed
    ofScopedLock lock(pixelsMutex);
    if (pixelsDirty && localPixels.isAllocated()) {
        int pixW = localPixels.getWidth();
        int pixH = localPixels.getHeight();
        
        if (!previewFbo.isAllocated() || 
            previewFbo.getWidth() != pixW ||
            previewFbo.getHeight() != pixH) {
            previewFbo.allocate(pixW, pixH, GL_RGBA);
        }
        
        // Load pixels directly into FBO texture
        previewFbo.getTexture().loadData(localPixels);
        pixelsDirty = false;
    }
    
    if (previewFbo.isAllocated()) {
        // Draw FBO filling the window (aspect ratio is handled by renderer)
        previewFbo.draw(0, 0, fbWidth, fbHeight);
    }
    
    // Swap buffers
    glfwSwapBuffers(glfwWindow);
    
    // Restore main window context
    if (mainWindow) {
        glfwMakeContextCurrent(mainWindow->getGLFWWindow());
    }
}

void PreviewWindow::show() {
    if (!glfwWindow) return;
    visible = true;
    glfwShowWindow(glfwWindow);
}

void PreviewWindow::hide() {
    if (!glfwWindow) return;
    visible = false;
    glfwHideWindow(glfwWindow);
}

void PreviewWindow::toggle() {
    if (visible) hide();
    else show();
}

bool PreviewWindow::isVisible() const {
    return visible && glfwWindow != nullptr;
}

void PreviewWindow::setPosition(int x, int y) {
    windowX = x;
    windowY = y;
    if (glfwWindow) {
        glfwSetWindowPos(glfwWindow, x, y);
    }
}

ofVec2f PreviewWindow::getPosition() const {
    int x, y;
    if (glfwWindow) {
        glfwGetWindowPos(glfwWindow, &x, &y);
        return ofVec2f(x, y);
    }
    return ofVec2f(windowX, windowY);
}

bool PreviewWindow::shouldClose() const {
    return glfwWindow && glfwWindowShouldClose(glfwWindow);
}

// Static callbacks
void PreviewWindow::mouseButtonCallback(GLFWwindow* window, int button, int action, int mods) {
    PreviewWindow* instance = getInstance(window);
    if (instance) instance->onMouseButton(button, action, mods);
}

void PreviewWindow::cursorPosCallback(GLFWwindow* window, double xpos, double ypos) {
    PreviewWindow* instance = getInstance(window);
    if (instance) instance->onCursorPos(xpos, ypos);
}

void PreviewWindow::keyCallback(GLFWwindow* window, int key, int scancode, int action, int mods) {
    PreviewWindow* instance = getInstance(window);
    if (instance) instance->onKey(key, scancode, action, mods);
}

void PreviewWindow::windowCloseCallback(GLFWwindow* window) {
    PreviewWindow* instance = getInstance(window);
    if (instance) instance->onWindowClose();
}

// Instance callbacks
void PreviewWindow::onMouseButton(int button, int action, int mods) {
    if (button == GLFW_MOUSE_BUTTON_LEFT) {
        if (action == GLFW_PRESS) {
            mousePressed = true;
            performColorPick();
        } else if (action == GLFW_RELEASE) {
            mousePressed = false;
        }
    }
}

void PreviewWindow::onCursorPos(double xpos, double ypos) {
    mouseX = (int)xpos;
    mouseY = (int)ypos;
}

void PreviewWindow::onKey(int key, int scancode, int action, int mods) {
    if (action == GLFW_PRESS || action == GLFW_REPEAT) {
        switch (key) {
            case GLFW_KEY_ESCAPE:
                hide();
                break;
            case GLFW_KEY_SPACE:
                performColorPick();
                break;
        }
    }
}

void PreviewWindow::onWindowClose() {
    visible = false;
}

void PreviewWindow::performColorPick() {
    if (!colorPicker) return;
    
    // Get window size (logical coordinates)
    int winWidth, winHeight;
    glfwGetWindowSize(glfwWindow, &winWidth, &winHeight);
    
    // Get framebuffer size (actual pixels)
    int fbWidth, fbHeight;
    glfwGetFramebufferSize(glfwWindow, &fbWidth, &fbHeight);
    
    // Get pixel buffer dimensions (source resolution)
    ofScopedLock lock(pixelsMutex);
    if (!localPixels.isAllocated()) return;
    
    int pixW = localPixels.getWidth();
    int pixH = localPixels.getHeight();
    
    // Map mouse position from window to framebuffer coordinates
    float scaleX = (float)fbWidth / winWidth;
    float scaleY = (float)fbHeight / winHeight;
    int fbMouseX = (int)(mouseX * scaleX);
    int fbMouseY = (int)(mouseY * scaleY);
    
    // Clamp to framebuffer bounds
    fbMouseX = ofClamp(fbMouseX, 0, fbWidth - 1);
    fbMouseY = ofClamp(fbMouseY, 0, fbHeight - 1);
    
    // Calculate how image is drawn (aspect-correct, filling window)
    float pixAspect = (float)pixW / (float)pixH;
    float fbAspect = (float)fbWidth / (float)fbHeight;
    
    float drawX = 0, drawY = 0, drawW = fbWidth, drawH = fbHeight;
    if (pixAspect > fbAspect) {
        drawH = fbWidth / pixAspect;
        drawY = (fbHeight - drawH) / 2.0f;
    } else {
        drawW = fbHeight * pixAspect;
        drawX = (fbWidth - drawW) / 2.0f;
    }
    
    // Map framebuffer mouse to pixel coordinates
    int pixX, pixY;
    if (fbMouseX >= drawX && fbMouseX < drawX + drawW &&
        fbMouseY >= drawY && fbMouseY < drawY + drawH) {
        // Mouse is over the image
        pixX = (int)((float)(fbMouseX - drawX) / drawW * pixW);
        pixY = (int)((float)(fbMouseY - drawY) / drawH * pixH);
    } else {
        // Mouse is in letterbox area - map to nearest edge
        pixX = (int)((float)fbMouseX / fbWidth * pixW);
        pixY = (int)((float)fbMouseY / fbHeight * pixH);
    }
    pixX = ofClamp(pixX, 0, pixW - 1);
    pixY = ofClamp(pixY, 0, pixH - 1);
    
    // Read color from local pixels
    ofColor picked = localPixels.getColor(pixX, pixY);
    
    // Store in color picker
    colorPicker->setPickedColor(picked);
    colorPicker->onPreviewClick(pixX, pixY, pixW, pixH);
    
    ofLogNotice("PreviewWindow") << "Picked: R=" << (int)picked.r 
                                 << " G=" << (int)picked.g 
                                 << " B=" << (int)picked.b
                                 << " at " << pixX << "," << pixY;
    
    // Trigger callback if set
    if (onColorPicked) {
        onColorPicked(colorPicker->getKeyTarget(), picked);
    }
}

PreviewWindow* PreviewWindow::getInstance(GLFWwindow* window) {
    return static_cast<PreviewWindow*>(glfwGetWindowUserPointer(window));
}

} // namespace dragonwaves
