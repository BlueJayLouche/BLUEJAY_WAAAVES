#pragma once

#include "ShaderBlock.h"
#include "Block3Shader.h"  // For ParamModulation

namespace dragonwaves {

//==============================================================================
// Block 1: Channel mixing, feedback, and geometric pattern generation
//==============================================================================
class Block1Shader : public ShaderBlock {
public:
    Block1Shader();
    
    void setup(int width, int height) override;
    void process() override;
    
    // Input textures
    void setChannel1Texture(ofTexture& tex);
    void setChannel2Texture(ofTexture& tex);
    void setFeedbackTexture(ofTexture& tex);
    void setTemporalFilterTexture(ofTexture& tex);
    
    // Parameters - these are references that can be bound to ParameterManager
    struct Params {
        // Channel 1 adjust
        float ch1XDisplace = 0.0f;
        float ch1YDisplace = 0.0f;
        float ch1ZDisplace = 1.0f;
        float ch1Rotate = 0.0f;
        float ch1HueAttenuate = 1.0f;
        float ch1SaturationAttenuate = 1.0f;
        float ch1BrightAttenuate = 1.0f;
        float ch1Posterize = 16.0f;
        float ch1KaleidoscopeAmount = 0.0f;
        float ch1KaleidoscopeSlice = 0.0f;
        float ch1BlurAmount = 0.0f;
        float ch1BlurRadius = 1.0f;
        float ch1SharpenAmount = 0.0f;
        float ch1SharpenRadius = 1.0f;
        float ch1FiltersBoost = 0.0f;
        
        // Channel 2 mix and key
        float ch2MixAmount = 0.0f;
        float ch2KeyValueRed = 0.0f;
        float ch2KeyValueGreen = 0.0f;
        float ch2KeyValueBlue = 0.0f;
        float ch2KeyThreshold = 1.0f;
        float ch2KeySoft = 0.0f;
        
        // Channel 2 adjust
        float ch2XDisplace = 0.0f;
        float ch2YDisplace = 0.0f;
        float ch2ZDisplace = 1.0f;
        float ch2Rotate = 0.0f;
        float ch2HueAttenuate = 1.0f;
        float ch2SaturationAttenuate = 1.0f;
        float ch2BrightAttenuate = 1.0f;
        float ch2Posterize = 16.0f;
        float ch2KaleidoscopeAmount = 0.0f;
        float ch2KaleidoscopeSlice = 0.0f;
        float ch2BlurAmount = 0.0f;
        float ch2BlurRadius = 1.0f;
        float ch2SharpenAmount = 0.0f;
        float ch2SharpenRadius = 1.0f;
        float ch2FiltersBoost = 0.0f;
        
        // FB1 feedback
        float fb1MixAmount = 0.0f;
        float fb1KeyValueRed = 0.0f;
        float fb1KeyValueGreen = 0.0f;
        float fb1KeyValueBlue = 0.0f;
        float fb1KeyThreshold = 1.0f;
        float fb1KeySoft = 0.0f;
        float fb1XDisplace = 0.0f;
        float fb1YDisplace = 0.0f;
        float fb1ZDisplace = 1.0f;
        float fb1Rotate = 0.0f;
        float fb1ShearMatrix1 = 1.0f;
        float fb1ShearMatrix2 = 0.0f;
        float fb1ShearMatrix3 = 0.0f;
        float fb1ShearMatrix4 = 1.0f;
        float fb1KaleidoscopeAmount = 0.0f;
        float fb1KaleidoscopeSlice = 0.0f;
        
        // FB1 color
        float fb1HueOffset = 0.0f;
        float fb1SaturationOffset = 0.0f;
        float fb1BrightOffset = 0.0f;
        float fb1HueAttenuate = 1.0f;
        float fb1SaturationAttenuate = 1.0f;
        float fb1BrightAttenuate = 1.0f;
        float fb1HuePowmap = 1.0f;
        float fb1SaturationPowmap = 1.0f;
        float fb1BrightPowmap = 1.0f;
        float fb1HueShaper = 1.0f;
        float fb1Posterize = 16.0f;
        
        // FB1 filters
        float fb1BlurAmount = 0.0f;
        float fb1BlurRadius = 1.0f;
        float fb1SharpenAmount = 0.0f;
        float fb1SharpenRadius = 1.0f;
        float fb1TemporalFilter1Amount = 0.0f;
        float fb1TemporalFilter1Resonance = 0.0f;
        float fb1TemporalFilter2Amount = 0.0f;
        float fb1TemporalFilter2Resonance = 0.0f;
        float fb1FiltersBoost = 0.0f;
        
        // Switches (integers 0/1)
        int ch1InputSelect = 0;  // 0=input1, 1=input2
        int ch1GeoOverflow = 0;
        int ch1HMirror = 0;
        int ch1VMirror = 0;
        int ch1HFlip = 0;
        int ch1VFlip = 0;
        int ch1HueInvert = 0;
        int ch1SaturationInvert = 0;
        int ch1BrightInvert = 0;
        int ch1RGBInvert = 0;
        int ch1Solarize = 0;
        int ch1PosterizeSwitch = 0;
        int ch1HdAspectOn = 0;
        
        int ch2InputSelect = 1;  // 0=input1, 1=input2
        int ch2KeyOrder = 0;
        int ch2MixType = 0;
        int ch2MixOverflow = 0;
        int ch2KeyMode = 0;
        int ch2GeoOverflow = 0;
        int ch2HMirror = 0;
        int ch2VMirror = 0;
        int ch2HFlip = 0;
        int ch2VFlip = 0;
        int ch2HueInvert = 0;
        int ch2SaturationInvert = 0;
        int ch2BrightInvert = 0;
        int ch2RGBInvert = 0;
        int ch2Solarize = 0;
        int ch2PosterizeSwitch = 0;
        int ch2HdAspectOn = 0;
        
        int fb1KeyOrder = 0;
        int fb1MixType = 0;
        int fb1MixOverflow = 0;
        int fb1HMirror = 0;
        int fb1VMirror = 0;
        int fb1HFlip = 0;
        int fb1VFlip = 0;
        int fb1RotateMode = 0;
        int fb1GeoOverflow = 0;
        int fb1HueInvert = 0;
        int fb1SaturationInvert = 0;
        int fb1BrightInvert = 0;
        int fb1PosterizeSwitch = 0;
    };
    
    Params params;
    
    // Aspect ratio fixes
    float ch1HdAspectXFix = 1.0f;
    float ch1HdAspectYFix = 1.0f;
    float ch2HdAspectXFix = 1.0f;
    float ch2HdAspectYFix = 1.0f;
    float input1XYFix[2] = {0.0f, 0.0f};
    float input2XYFix[2] = {0.0f, 0.0f};
    
    // Modulation mappings for all parameters
    std::unordered_map<std::string, ParamModulation> modulations;
    
    // Initialize modulation mappings
    void initializeModulations();
    
    // Get modulation for a parameter
    ParamModulation* getModulation(const std::string& paramName);
    
    // Get current modulated value (for GUI feedback)
    float getModulatedValue(const std::string& paramName) const;
    
    // Apply all modulations (call before process())
    void applyModulations(const AudioAnalyzer& audioAnalyzer, const TempoManager& tempo, float deltaTime);
    
    // Get effective value (base + modulations)
    float getEffectiveValue(const std::string& paramName, float baseValue, const AudioAnalyzer& audioAnalyzer, const TempoManager& tempo, float deltaTime);
    
    // Serialization
    void loadModulations(const ofJson& json);
    ofJson saveModulations() const;
    
private:
    ofTexture* ch1Tex = nullptr;
    ofTexture* ch2Tex = nullptr;
    ofTexture* fbTex = nullptr;
    ofTexture* temporalTex = nullptr;
    ofTexture dummyTex;
    
    // Store last computed modulated values for GUI feedback
    mutable std::unordered_map<std::string, float> lastModulatedValues;
};

} // namespace dragonwaves
