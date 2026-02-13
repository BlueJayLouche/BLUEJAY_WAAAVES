#pragma once

#include "ShaderBlock.h"

namespace dragonwaves {

//==============================================================================
// Block 2: Secondary processing with block2 input and FB2
//==============================================================================
class Block2Shader : public ShaderBlock {
public:
    Block2Shader();
    
    void setup(int width, int height) override;
    void process() override;
    
    // Input textures
    void setBlock1Texture(ofTexture& tex);
    void setInputTexture(ofTexture& tex);
    void setFeedbackTexture(ofTexture& tex);
    void setTemporalFilterTexture(ofTexture& tex);
    
    // Parameters
    struct Params {
        // Block2 input adjust
        float block2InputXDisplace = 0.0f;
        float block2InputYDisplace = 0.0f;
        float block2InputZDisplace = 1.0f;
        float block2InputRotate = 0.0f;
        float block2InputHueAttenuate = 1.0f;
        float block2InputSaturationAttenuate = 1.0f;
        float block2InputBrightAttenuate = 1.0f;
        float block2InputPosterize = 16.0f;
        float block2InputKaleidoscopeAmount = 0.0f;
        float block2InputKaleidoscopeSlice = 0.0f;
        float block2InputBlurAmount = 0.0f;
        float block2InputBlurRadius = 1.0f;
        float block2InputSharpenAmount = 0.0f;
        float block2InputSharpenRadius = 1.0f;
        float block2InputFiltersBoost = 0.0f;
        
        // FB2 feedback
        float fb2MixAmount = 0.0f;
        float fb2KeyValueRed = 0.0f;
        float fb2KeyValueGreen = 0.0f;
        float fb2KeyValueBlue = 0.0f;
        float fb2KeyThreshold = 1.0f;
        float fb2KeySoft = 0.0f;
        float fb2XDisplace = 0.0f;
        float fb2YDisplace = 0.0f;
        float fb2ZDisplace = 1.0f;
        float fb2Rotate = 0.0f;
        float fb2ShearMatrix1 = 1.0f;
        float fb2ShearMatrix2 = 0.0f;
        float fb2ShearMatrix3 = 0.0f;
        float fb2ShearMatrix4 = 1.0f;
        float fb2KaleidoscopeAmount = 0.0f;
        float fb2KaleidoscopeSlice = 0.0f;
        
        // FB2 color
        float fb2HueOffset = 0.0f;
        float fb2SaturationOffset = 0.0f;
        float fb2BrightOffset = 0.0f;
        float fb2HueAttenuate = 1.0f;
        float fb2SaturationAttenuate = 1.0f;
        float fb2BrightAttenuate = 1.0f;
        float fb2HuePowmap = 1.0f;
        float fb2SaturationPowmap = 1.0f;
        float fb2BrightPowmap = 1.0f;
        float fb2HueShaper = 1.0f;
        float fb2Posterize = 16.0f;
        
        // FB2 filters
        float fb2BlurAmount = 0.0f;
        float fb2BlurRadius = 1.0f;
        float fb2SharpenAmount = 0.0f;
        float fb2SharpenRadius = 1.0f;
        float fb2TemporalFilter1Amount = 0.0f;
        float fb2TemporalFilter1Resonance = 0.0f;
        float fb2TemporalFilter2Amount = 0.0f;
        float fb2TemporalFilter2Resonance = 0.0f;
        float fb2FiltersBoost = 0.0f;
        
        // Switches
        int block2InputSelect = 0;  // 0=block1, 1=input1, 2=input2
        int block2InputMasterSwitch = 0;
        int block2InputGeoOverflow = 0;
        int block2InputHMirror = 0;
        int block2InputVMirror = 0;
        int block2InputHFlip = 0;
        int block2InputVFlip = 0;
        int block2InputHueInvert = 0;
        int block2InputSaturationInvert = 0;
        int block2InputBrightInvert = 0;
        int block2InputRGBInvert = 0;
        int block2InputSolarize = 0;
        int block2InputPosterizeSwitch = 0;
        int block2InputHdAspectOn = 0;
        
        int fb2KeyOrder = 0;
        int fb2MixType = 0;
        int fb2MixOverflow = 0;
        int fb2HMirror = 0;
        int fb2VMirror = 0;
        int fb2HFlip = 0;
        int fb2VFlip = 0;
        int fb2RotateMode = 0;
        int fb2GeoOverflow = 0;
        int fb2HueInvert = 0;
        int fb2SaturationInvert = 0;
        int fb2BrightInvert = 0;
        int fb2PosterizeSwitch = 0;
    };
    
    Params params;
    
    float block2InputHdAspectXFix = 1.0f;
    float block2InputHdAspectYFix = 1.0f;
    
private:
    ofTexture* block1Tex = nullptr;
    ofTexture* inputTex = nullptr;
    ofTexture* fbTex = nullptr;
    ofTexture* temporalTex = nullptr;
    ofTexture dummyTex;
};

} // namespace dragonwaves
