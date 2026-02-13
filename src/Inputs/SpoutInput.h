#pragma once

#include "ofMain.h"

#if defined(TARGET_WIN32)
    #include "ofxSpout.h"
    #define SPOUT_AVAILABLE 1
#else
    #define SPOUT_AVAILABLE 0
#endif

#include "InputSource.h"

namespace dragonwaves {

//==============================================================================
// Spout input source (Windows only)
//==============================================================================
class SpoutInput : public InputSource {
public:
    SpoutInput();
    ~SpoutInput();
    
    bool setup(int width, int height) override;
    void update() override;
    void close() override;
    
    ofTexture& getTexture() override;
    bool isFrameNew() const override;
    bool isInitialized() const override;
    InputType getType() const override { return InputType::SPOUT; }
    std::string getName() const override;
    
    // Spout-specific methods
    void refreshSources();
    std::vector<std::string> getSourceNames() const;
    void selectSource(int index);
    int getSelectedSourceIndex() const { return selectedSourceIndex; }
    
private:
#if SPOUT_AVAILABLE
    ofxSpout::Receiver receiver;
    std::vector<std::string> sourceNames;
#endif
    ofTexture texture;
    int selectedSourceIndex = 0;
    bool frameIsNew = false;
};

} // namespace dragonwaves
