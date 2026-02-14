#pragma once

#include "ShaderBlock.h"
#include "../Audio/AudioAnalyzer.h"
#include "../Tempo/TempoManager.h"

namespace dragonwaves {

//==============================================================================
// Parameter modulation info - links a parameter to audio/BPM modulation
//==============================================================================
struct ParamModulation {
    AudioModulation audio;
    BpmModulation bpm;
    
    // Apply modulations to a base value
    float apply(float baseValue, const AudioAnalyzer& audioAnalyzer, const TempoManager& tempo, float deltaTime);
    
    void loadFromJson(const ofJson& json);
    void saveToJson(ofJson& json) const;
};

//==============================================================================
// Block 3: Final mixing with matrix mixer and colorization
//==============================================================================
class Block3Shader : public ShaderBlock {
public:
    Block3Shader();
    
    void setup(int width, int height) override;
    void process() override;
    
    // Input textures
    void setBlock1Texture(ofTexture& tex);
    void setBlock2Texture(ofTexture& tex);
    
    // Parameters
    struct Params {
        // Block1 geo (final stage)
        float block1XDisplace = 0.0f;
        float block1YDisplace = 0.0f;
        float block1ZDisplace = 1.0f;
        float block1Rotate = 0.0f;
        float block1ShearMatrix1 = 1.0f;
        float block1ShearMatrix2 = 0.0f;
        float block1ShearMatrix3 = 0.0f;
        float block1ShearMatrix4 = 1.0f;
        float block1KaleidoscopeAmount = 0.0f;
        float block1KaleidoscopeSlice = 0.0f;
        
        // Block1 colorize
        float block1ColorizeHueBand1 = 0.0f;
        float block1ColorizeSaturationBand1 = 1.0f;
        float block1ColorizeBrightBand1 = 1.0f;
        float block1ColorizeHueBand2 = 0.0f;
        float block1ColorizeSaturationBand2 = 1.0f;
        float block1ColorizeBrightBand2 = 1.0f;
        float block1ColorizeHueBand3 = 0.0f;
        float block1ColorizeSaturationBand3 = 1.0f;
        float block1ColorizeBrightBand3 = 1.0f;
        float block1ColorizeHueBand4 = 0.0f;
        float block1ColorizeSaturationBand4 = 1.0f;
        float block1ColorizeBrightBand4 = 1.0f;
        float block1ColorizeHueBand5 = 0.0f;
        float block1ColorizeSaturationBand5 = 1.0f;
        float block1ColorizeBrightBand5 = 1.0f;
        
        // Block1 filters
        float block1BlurAmount = 0.0f;
        float block1BlurRadius = 1.0f;
        float block1SharpenAmount = 0.0f;
        float block1SharpenRadius = 1.0f;
        float block1FiltersBoost = 0.0f;
        float block1Dither = 16.0f;
        
        // Block2 geo (final stage)
        float block2XDisplace = 0.0f;
        float block2YDisplace = 0.0f;
        float block2ZDisplace = 1.0f;
        float block2Rotate = 0.0f;
        float block2ShearMatrix1 = 1.0f;
        float block2ShearMatrix2 = 0.0f;
        float block2ShearMatrix3 = 0.0f;
        float block2ShearMatrix4 = 1.0f;
        float block2KaleidoscopeAmount = 0.0f;
        float block2KaleidoscopeSlice = 0.0f;
        
        // Block2 colorize
        float block2ColorizeHueBand1 = 0.0f;
        float block2ColorizeSaturationBand1 = 1.0f;
        float block2ColorizeBrightBand1 = 1.0f;
        float block2ColorizeHueBand2 = 0.0f;
        float block2ColorizeSaturationBand2 = 1.0f;
        float block2ColorizeBrightBand2 = 1.0f;
        float block2ColorizeHueBand3 = 0.0f;
        float block2ColorizeSaturationBand3 = 1.0f;
        float block2ColorizeBrightBand3 = 1.0f;
        float block2ColorizeHueBand4 = 0.0f;
        float block2ColorizeSaturationBand4 = 1.0f;
        float block2ColorizeBrightBand4 = 1.0f;
        float block2ColorizeHueBand5 = 0.0f;
        float block2ColorizeSaturationBand5 = 1.0f;
        float block2ColorizeBrightBand5 = 1.0f;
        
        // Block2 filters
        float block2BlurAmount = 0.0f;
        float block2BlurRadius = 1.0f;
        float block2SharpenAmount = 0.0f;
        float block2SharpenRadius = 1.0f;
        float block2FiltersBoost = 0.0f;
        float block2Dither = 16.0f;
        
        // Matrix mixer
        float matrixMixBgRedIntoFgRed = 0.0f;
        float matrixMixBgGreenIntoFgRed = 0.0f;
        float matrixMixBgBlueIntoFgRed = 0.0f;
        float matrixMixBgRedIntoFgGreen = 0.0f;
        float matrixMixBgGreenIntoFgGreen = 0.0f;
        float matrixMixBgBlueIntoFgGreen = 0.0f;
        float matrixMixBgRedIntoFgBlue = 0.0f;
        float matrixMixBgGreenIntoFgBlue = 0.0f;
        float matrixMixBgBlueIntoFgBlue = 0.0f;
        
        // Final mix and key
        float finalMixAmount = 0.0f;
        float finalKeyValueRed = 0.0f;
        float finalKeyValueGreen = 0.0f;
        float finalKeyValueBlue = 0.0f;
        float finalKeyThreshold = 1.0f;
        float finalKeySoft = 0.0f;
        
        // Switches
        int block1HMirror = 0;
        int block1VMirror = 0;
        int block1HFlip = 0;
        int block1VFlip = 0;
        int block1RotateMode = 0;
        int block1GeoOverflow = 0;
        int block1ColorizeSwitch = 0;
        int block1ColorizeHSB_RGB = 0;  // 0=HSB, 1=RGB
        int block1DitherSwitch = 0;
        int block1DitherType = 1;
        
        int block2HMirror = 0;
        int block2VMirror = 0;
        int block2HFlip = 0;
        int block2VFlip = 0;
        int block2RotateMode = 0;
        int block2GeoOverflow = 0;
        int block2ColorizeSwitch = 0;
        int block2ColorizeHSB_RGB = 0;
        int block2DitherSwitch = 0;
        int block2DitherType = 1;
        
        int matrixMixType = 0;
        int matrixMixOverflow = 0;
        
        int finalKeyOrder = 0;
        int finalMixType = 0;
        int finalMixOverflow = 0;
    };
    
    Params params;
    
    // Modulation mappings for all parameters
    // Access via getModulation("paramName")
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
    ofTexture* block1Tex = nullptr;
    ofTexture* block2Tex = nullptr;
    ofTexture dummyTex;
    
    // Store last computed modulated values for GUI feedback
    mutable std::unordered_map<std::string, float> lastModulatedValues;
};

} // namespace dragonwaves

// Backwards compatibility
typedef dragonwaves::Block3Shader Block3Shader;
typedef dragonwaves::ParamModulation ParamModulation;
