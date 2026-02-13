#pragma once

#include "InputSource.h"

namespace dragonwaves {

//==============================================================================
// Webcam input source
//==============================================================================
class WebcamInput : public InputSource {
public:
    WebcamInput();
    ~WebcamInput();
    
    bool setup(int width, int height) override;
    void update() override;
    void close() override;
    
    ofTexture& getTexture() override;
    bool isFrameNew() const override;
    bool isInitialized() const override;
    InputType getType() const override { return InputType::WEBCAM; }
    std::string getName() const override;
    
    // Device management
    void setDeviceID(int deviceID);
    int getDeviceID() const { return deviceID; }
    
    // Static methods for device enumeration
    static std::vector<ofVideoDevice> listDevices();
    
private:
    ofVideoGrabber grabber;
    int deviceID = 0;
    int desiredFrameRate = 30;
};

} // namespace dragonwaves
