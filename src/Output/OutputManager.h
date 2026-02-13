#pragma once

#include "ofMain.h"
#include "ofxNDIsender.h"
#include <mutex>

#if defined(TARGET_WIN32)
    #include "ofxSpout.h"
    #define SPOUT_AVAILABLE 1
#else
    #define SPOUT_AVAILABLE 0
#endif

#include "../Core/SettingsManager.h"

namespace dragonwaves {

//==============================================================================
// Async PBO transfer for efficient NDI sending
//==============================================================================
class AsyncPixelTransfer {
public:
    void setup(int width, int height);
    void cleanup();
    
    // Begin transfer - call after rendering to source FBO
    void beginTransfer(ofFbo& sourceFbo);
    
    // End transfer and get pixels - call before sending
    ofPixels& endTransfer();
    
    void resize(int width, int height);
    
private:
    GLuint pbo[2] = {0, 0};
    int pboIndex = 0;
    int pboNextIndex = 1;
    int frameCount = 0;
    ofPixels pixels;
    int width = 0;
    int height = 0;
    bool initialized = false;
};

//==============================================================================
// Output sender base class
//==============================================================================
class OutputSender {
public:
    OutputSender(const std::string& name);
    virtual ~OutputSender() = default;
    
    virtual void setup(int width, int height) = 0;
    virtual void send(ofTexture& texture) = 0;
    virtual void close() = 0;
    virtual bool isEnabled() const = 0;
    virtual void setEnabled(bool enabled) = 0;
    
    const std::string& getName() const { return name; }
    
protected:
    std::string name;
};

//==============================================================================
// NDI Sender
//==============================================================================
class NdiOutputSender : public OutputSender {
public:
    NdiOutputSender(const std::string& name);
    ~NdiOutputSender();
    
    void setup(int width, int height) override;
    void send(ofTexture& texture) override;
    void close() override;
    bool isEnabled() const override { return enabled; }
    void setEnabled(bool enabled) override;
    
private:
    ofxNDIsender sender;
    ofFbo scaleFbo;
    AsyncPixelTransfer pboTransfer;
    bool enabled = false;
    bool active = false;
    int width = 0;
    int height = 0;
    mutable std::mutex mtx;
};

//==============================================================================
// Spout Sender (Windows only)
//==============================================================================
class SpoutOutputSender : public OutputSender {
public:
    SpoutOutputSender(const std::string& name);
    ~SpoutOutputSender();
    
    void setup(int width, int height) override;
    void send(ofTexture& texture) override;
    void close() override;
    bool isEnabled() const override { return enabled; }
    void setEnabled(bool enabled) override;
    
private:
#if SPOUT_AVAILABLE
    ofxSpout::Sender sender;
    ofFbo flipFbo;
#endif
    bool enabled = false;
    int width = 0;
    int height = 0;
};

//==============================================================================
// Output Manager - handles all output senders
//==============================================================================
class OutputManager {
public:
    OutputManager();
    ~OutputManager();
    
    // Setup with display settings
    void setup(const DisplaySettings& settings);
    
    // Send outputs
    void sendBlock1(ofTexture& texture);
    void sendBlock2(ofTexture& texture);
    void sendBlock3(ofTexture& texture);
    
    // Enable/disable outputs
    void setNdiBlock1Enabled(bool enabled);
    void setNdiBlock2Enabled(bool enabled);
    void setNdiBlock3Enabled(bool enabled);
    void setSpoutBlock1Enabled(bool enabled);
    void setSpoutBlock2Enabled(bool enabled);
    void setSpoutBlock3Enabled(bool enabled);
    
    // Get status
    bool isNdiBlock1Enabled() const;
    bool isNdiBlock2Enabled() const;
    bool isNdiBlock3Enabled() const;
    bool isSpoutBlock1Enabled() const;
    bool isSpoutBlock2Enabled() const;
    bool isSpoutBlock3Enabled() const;
    
    // Reinitialize with new resolution
    void reinitialize(const DisplaySettings& settings);
    
    // Close all
    void close();
    
private:
    std::unique_ptr<NdiOutputSender> ndiBlock1;
    std::unique_ptr<NdiOutputSender> ndiBlock2;
    std::unique_ptr<NdiOutputSender> ndiBlock3;
    
#if SPOUT_AVAILABLE
    std::unique_ptr<SpoutOutputSender> spoutBlock1;
    std::unique_ptr<SpoutOutputSender> spoutBlock2;
    std::unique_ptr<SpoutOutputSender> spoutBlock3;
#endif
    
    DisplaySettings displaySettings;
    bool initialized = false;
};

} // namespace dragonwaves

// Backwards compatibility
typedef dragonwaves::OutputManager OutputManager;
