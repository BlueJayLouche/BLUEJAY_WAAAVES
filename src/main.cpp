#include "ofMain.h"
#include "ofApp.h"
#include "GuiApp.h"
#include "ofAppGLFWWindow.h"

//========================================================================
int main() {
#if defined(__APPLE__) && (defined(__arm64__) || defined(__aarch64__))
    // Apple Silicon - can use desktop OpenGL
    ofGLWindowSettings settings;
    settings.setGLVersion(3, 2);
    settings.setSize(1920, 1080);
    ofLogNotice("main") << "Using OpenGL 3.2 renderer for Apple Silicon";
#elif defined(__arm__) || defined(__aarch64__)
    // Non-Apple ARM platform (Raspberry Pi, etc.)
    ofGLESWindowSettings settings;
    settings.glesVersion = 2;
    settings.setSize(640, 480);
    ofLogNotice("main") << "Using OpenGL ES2 renderer for ARM";
#else
    // Desktop x86/x64 platform
    ofGLWindowSettings settings;
    settings.setGLVersion(3, 2);
    settings.setSize(1920, 1080);
    ofLogNotice("main") << "Using OpenGL 3.2 renderer";
#endif
    
    
    // GUI WINDOW - starts maximized (no position set, let OS handle it)
    //    settings.setSize(1920, 1080);
    //    settings.resizable = true;
    //    settings.decorated = true;
    
    shared_ptr<ofAppBaseWindow> guiWindow = ofCreateWindow(settings);
    guiWindow->setWindowTitle("Gravity Waaaves - Control");
    
    
    
    
    // GUI WINDOW - starts maximized (no position set, let OS handle it)
    //    settings.setSize(1920, 1080);
    //    settings.resizable = true;
    //    settings.decorated = true;
    //    settings.maximized = true;
    
    
    // OUTPUT WINDOW
    // For single monitor testing, offset to overlap or use smaller size
    // Create fresh settings for second window to avoid macOS pixel format issues
#if defined(__APPLE__) && (defined(__arm64__) || defined(__aarch64__))
    ofGLWindowSettings mainSettings;
    mainSettings.setGLVersion(3, 2);
#else
    ofGLWindowSettings mainSettings;
    mainSettings.setGLVersion(3, 2);
#endif
    mainSettings.setSize(1280, 720);
    mainSettings.setPosition(glm::vec2(100, 100));  // Adjust based on your setup
//    mainSettings.resizable = true;
//    mainSettings.decorated = true;  // Set to true for testing, false for fullscreen output
    shared_ptr<ofAppBaseWindow> mainWindow = ofCreateWindow(mainSettings);
    mainWindow->setWindowTitle("Gravity Waaaves - Output");
    
    // Create and link apps
    shared_ptr<ofApp> mainApp(new ofApp);
    shared_ptr<GuiApp> guiApp(new GuiApp);
    mainApp->gui = guiApp;
    guiApp->mainApp = mainApp.get();
    mainApp->mainWindow = mainWindow;
    guiApp->guiWindow = guiWindow;
    
    // Note: Audio and Tempo references will be set in ofApp::setup() after they're created
    
    ofRunApp(guiWindow, guiApp);
    ofRunApp(mainWindow, mainApp);
    ofRunMainLoop();
    
//    // Create window and initialize app
//    auto window = ofCreateWindow(settings);
//    ofRunApp(window, make_shared<ofApp>());
//    ofRunMainLoop();
    
}
