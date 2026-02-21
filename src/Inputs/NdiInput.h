#pragma once

#include "InputSource.h"
#include "ofxNDIreceiver.h"

namespace dragonwaves {

//==============================================================================
// NDI input source
//==============================================================================
class NdiInput : public InputSource {
public:
    NdiInput();
    ~NdiInput();
    
    bool setup(int width, int height) override;
    void update() override;
    void close() override;
    
    ofTexture& getTexture() override;
    bool isFrameNew() const override;
    bool isInitialized() const override;
    InputType getType() const override { return InputType::NDI; }
    std::string getName() const override;
    
    // NDI-specific methods
    void refreshSources();
    std::vector<std::string> getSourceNames() const;
    void selectSource(int index);
    int getSelectedSourceIndex() const { return selectedSourceIndex; }
    
    // Get receiver reference for advanced control
    ofxNDIreceiver& getReceiver() { return receiver; }
    
    // Performance diagnostics
    bool isReceiverConnected() const { return receiverConnected; }
    float getReceivedFps() const { return receivedFps; }
    
private:
    ofxNDIreceiver receiver;
    ofTexture texture;
    std::vector<std::string> sourceNames;
    int selectedSourceIndex = 0;
    int maxSources = 10;
    bool frameIsNew = false;
    
    // Performance diagnostics
    float lastFrameTime = 0;
    float receivedFps = 0;
    int frameCounter = 0;
    float fpsTimer = 0;
    bool receiverConnected = false;
};

} // namespace dragonwaves
