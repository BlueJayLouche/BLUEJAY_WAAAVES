#include "Block1Shader.h"
#include "Block3Shader.h"  // For ParamModulation

namespace dragonwaves {

Block1Shader::Block1Shader()
    : ShaderBlock("Block1", "shader1") {
    initializeModulations();
}

void Block1Shader::setup(int width, int height) {
    ShaderBlock::setup(width, height);
    
    // Allocate dummy texture for when inputs aren't available
    dummyTex.allocate(width, height, GL_RGBA);
    ofPixels pixels;
    pixels.allocate(width, height, OF_PIXELS_RGBA);
    pixels.setColor(ofColor::black);
    dummyTex.loadData(pixels);
    
    // Initialize parameter defaults
    params.ch1InputSelect = 0;
    params.ch2InputSelect = 1;
}

void Block1Shader::process() {
    ShaderBlock::process();
    
    // Bind textures
    if (ch1Tex && ch1Tex->isAllocated()) {
        shader.setUniformTexture("ch1Tex", *ch1Tex, 2);
    } else {
        shader.setUniformTexture("ch1Tex", dummyTex, 2);
    }
    
    if (ch2Tex && ch2Tex->isAllocated()) {
        shader.setUniformTexture("ch2Tex", *ch2Tex, 3);
    } else {
        shader.setUniformTexture("ch2Tex", dummyTex, 3);
    }
    
    if (fbTex && fbTex->isAllocated()) {
        shader.setUniformTexture("fb1Tex", *fbTex, 0);
    } else {
        shader.setUniformTexture("fb1Tex", dummyTex, 0);
    }
    
    if (temporalTex && temporalTex->isAllocated()) {
        shader.setUniformTexture("fb1TemporalFilter", *temporalTex, 1);
    } else {
        shader.setUniformTexture("fb1TemporalFilter", dummyTex, 1);
    }
    
    // Set resolution uniforms
    shader.setUniform1f("width", width);
    shader.setUniform1f("height", height);
    shader.setUniform1f("inverseWidth", 1.0f / width);
    shader.setUniform1f("inverseHeight", 1.0f / height);
    
    // Legacy resolution uniforms (for compatibility)
    shader.setUniform1f("inverseWidth1", 1.0f / width);
    shader.setUniform1f("inverseHeight1", 1.0f / height);
    shader.setUniform1f("input1Width", width);
    shader.setUniform1f("input1Height", height);
    shader.setUniform1f("hdFixX", 0.0f);
    shader.setUniform1f("hdFixY", 0.0f);
    shader.setUniform1f("ratio", 1.0f);
    
    // Channel 1 parameters
    shader.setUniform2f("ch1XYDisplace", params.ch1XDisplace, params.ch1YDisplace);
    shader.setUniform1f("ch1ZDisplace", params.ch1ZDisplace);
    shader.setUniform1f("ch1Rotate", params.ch1Rotate);
    shader.setUniform3f("ch1HSBAttenuate", params.ch1HueAttenuate, params.ch1SaturationAttenuate, params.ch1BrightAttenuate);
    shader.setUniform1f("ch1Posterize", params.ch1Posterize);
    shader.setUniform1f("ch1PosterizeInvert", 1.0f / params.ch1Posterize);
    shader.setUniform1i("ch1PosterizeSwitch", params.ch1PosterizeSwitch);
    shader.setUniform1f("ch1KaleidoscopeAmount", params.ch1KaleidoscopeAmount);
    shader.setUniform1f("ch1KaleidoscopeSlice", params.ch1KaleidoscopeSlice);
    shader.setUniform1f("ch1BlurAmount", params.ch1BlurAmount);
    shader.setUniform1f("ch1BlurRadius", params.ch1BlurRadius);
    shader.setUniform1f("ch1SharpenAmount", params.ch1SharpenAmount);
    shader.setUniform1f("ch1SharpenRadius", params.ch1SharpenRadius);
    shader.setUniform1f("ch1FiltersBoost", params.ch1FiltersBoost);
    
    shader.setUniform1i("ch1GeoOverflow", params.ch1GeoOverflow);
    shader.setUniform1i("ch1HMirror", params.ch1HMirror);
    shader.setUniform1i("ch1VMirror", params.ch1VMirror);
    shader.setUniform1i("ch1HFlip", params.ch1HFlip);
    shader.setUniform1i("ch1VFlip", params.ch1VFlip);
    shader.setUniform1i("ch1HueInvert", params.ch1HueInvert);
    shader.setUniform1i("ch1SaturationInvert", params.ch1SaturationInvert);
    shader.setUniform1i("ch1BrightInvert", params.ch1BrightInvert);
    shader.setUniform1i("ch1RGBInvert", params.ch1RGBInvert);
    shader.setUniform1i("ch1Solarize", params.ch1Solarize);
    
    // Channel 2 mix and key
    shader.setUniform1f("ch2MixAmount", params.ch2MixAmount);
    shader.setUniform3f("ch2KeyValue", params.ch2KeyValueRed, params.ch2KeyValueGreen, params.ch2KeyValueBlue);
    shader.setUniform1f("ch2KeyThreshold", params.ch2KeyThreshold);
    shader.setUniform1f("ch2KeySoft", params.ch2KeySoft);
    shader.setUniform1i("ch2KeyOrder", params.ch2KeyOrder);
    shader.setUniform1i("ch2MixType", params.ch2MixType);
    shader.setUniform1i("ch2MixOverflow", params.ch2MixOverflow);
    
    // Channel 2 parameters
    shader.setUniform2f("ch2XYDisplace", params.ch2XDisplace, params.ch2YDisplace);
    shader.setUniform1f("ch2ZDisplace", params.ch2ZDisplace);
    shader.setUniform1f("ch2Rotate", params.ch2Rotate);
    shader.setUniform3f("ch2HSBAttenuate", params.ch2HueAttenuate, params.ch2SaturationAttenuate, params.ch2BrightAttenuate);
    shader.setUniform1f("ch2Posterize", params.ch2Posterize);
    shader.setUniform1f("ch2PosterizeInvert", 1.0f / params.ch2Posterize);
    shader.setUniform1i("ch2PosterizeSwitch", params.ch2PosterizeSwitch);
    shader.setUniform1f("ch2KaleidoscopeAmount", params.ch2KaleidoscopeAmount);
    shader.setUniform1f("ch2KaleidoscopeSlice", params.ch2KaleidoscopeSlice);
    shader.setUniform1f("ch2BlurAmount", params.ch2BlurAmount);
    shader.setUniform1f("ch2BlurRadius", params.ch2BlurRadius);
    shader.setUniform1f("ch2SharpenAmount", params.ch2SharpenAmount);
    shader.setUniform1f("ch2SharpenRadius", params.ch2SharpenRadius);
    shader.setUniform1f("ch2FiltersBoost", params.ch2FiltersBoost);
    
    shader.setUniform1i("ch2GeoOverflow", params.ch2GeoOverflow);
    shader.setUniform1i("ch2HMirror", params.ch2HMirror);
    shader.setUniform1i("ch2VMirror", params.ch2VMirror);
    shader.setUniform1i("ch2HFlip", params.ch2HFlip);
    shader.setUniform1i("ch2VFlip", params.ch2VFlip);
    shader.setUniform1i("ch2HueInvert", params.ch2HueInvert);
    shader.setUniform1i("ch2SaturationInvert", params.ch2SaturationInvert);
    shader.setUniform1i("ch2BrightInvert", params.ch2BrightInvert);
    shader.setUniform1i("ch2RGBInvert", params.ch2RGBInvert);
    shader.setUniform1i("ch2Solarize", params.ch2Solarize);
    
    // FB1 parameters
    shader.setUniform1f("fb1MixAmount", params.fb1MixAmount);
    shader.setUniform3f("fb1KeyValue", params.fb1KeyValueRed, params.fb1KeyValueGreen, params.fb1KeyValueBlue);
    shader.setUniform1f("fb1KeyThreshold", params.fb1KeyThreshold);
    shader.setUniform1f("fb1KeySoft", params.fb1KeySoft);
    shader.setUniform1i("fb1KeyOrder", params.fb1KeyOrder);
    shader.setUniform1i("fb1MixType", params.fb1MixType);
    shader.setUniform1i("fb1MixOverflow", params.fb1MixOverflow);
    
    shader.setUniform2f("fb1XYDisplace", params.fb1XDisplace, params.fb1YDisplace);
    shader.setUniform1f("fb1ZDisplace", params.fb1ZDisplace);
    shader.setUniform1f("fb1Rotate", params.fb1Rotate);
    shader.setUniform4f("fb1ShearMatrix", params.fb1ShearMatrix1, params.fb1ShearMatrix2, 
                        params.fb1ShearMatrix3, params.fb1ShearMatrix4);
    shader.setUniform1f("fb1KaleidoscopeAmount", params.fb1KaleidoscopeAmount);
    shader.setUniform1f("fb1KaleidoscopeSlice", params.fb1KaleidoscopeSlice);
    
    shader.setUniform1i("fb1HMirror", params.fb1HMirror);
    shader.setUniform1i("fb1VMirror", params.fb1VMirror);
    shader.setUniform1i("fb1HFlip", params.fb1HFlip);
    shader.setUniform1i("fb1VFlip", params.fb1VFlip);
    shader.setUniform1i("fb1RotateMode", params.fb1RotateMode);
    shader.setUniform1i("fb1GeoOverflow", params.fb1GeoOverflow);
    
    shader.setUniform3f("fb1HSBOffset", params.fb1HueOffset, params.fb1SaturationOffset, params.fb1BrightOffset);
    shader.setUniform3f("fb1HSBAttenuate", params.fb1HueAttenuate, params.fb1SaturationAttenuate, params.fb1BrightAttenuate);
    shader.setUniform3f("fb1HSBPowmap", params.fb1HuePowmap, params.fb1SaturationPowmap, params.fb1BrightPowmap);
    shader.setUniform1f("fb1HueShaper", params.fb1HueShaper);
    shader.setUniform1f("fb1Posterize", params.fb1Posterize);
    shader.setUniform1f("fb1PosterizeInvert", 1.0f / params.fb1Posterize);
    shader.setUniform1i("fb1PosterizeSwitch", params.fb1PosterizeSwitch);
    
    shader.setUniform1i("fb1HueInvert", params.fb1HueInvert);
    shader.setUniform1i("fb1SaturationInvert", params.fb1SaturationInvert);
    shader.setUniform1i("fb1BrightInvert", params.fb1BrightInvert);
    
    shader.setUniform1f("fb1BlurAmount", params.fb1BlurAmount);
    shader.setUniform1f("fb1BlurRadius", params.fb1BlurRadius);
    shader.setUniform1f("fb1SharpenAmount", params.fb1SharpenAmount);
    shader.setUniform1f("fb1SharpenRadius", params.fb1SharpenRadius);
    shader.setUniform1f("fb1TemporalFilter1Amount", params.fb1TemporalFilter1Amount);
    shader.setUniform1f("fb1TemporalFilter1Resonance", params.fb1TemporalFilter1Resonance);
    shader.setUniform1f("fb1TemporalFilter2Amount", params.fb1TemporalFilter2Amount);
    shader.setUniform1f("fb1TemporalFilter2Resonance", params.fb1TemporalFilter2Resonance);
    shader.setUniform1f("fb1FiltersBoost", params.fb1FiltersBoost);
    
    // Aspect ratio fixes
    shader.setUniform1i("ch1HdAspectOn", params.ch1HdAspectOn);
    shader.setUniform2f("ch1HdAspectXYFix", ch1HdAspectXFix, ch1HdAspectYFix);
    shader.setUniform2f("input1XYFix", input1XYFix[0], input1XYFix[1]);
    
    shader.setUniform1i("ch2HdAspectOn", params.ch2HdAspectOn);
    shader.setUniform2f("ch2HdAspectXYFix", ch2HdAspectXFix, ch2HdAspectYFix);
    shader.setUniform2f("input2XYFix", input2XYFix[0], input2XYFix[1]);
    
    // Channel 1 aspect ratio and scaling (inputs are pre-scaled to internal resolution)
    shader.setUniform1f("ch1ScaleFix", 1.0f);
    shader.setUniform1f("ch1AspectRatio", 1.0f);
    shader.setUniform1f("ch1CribX", 0.0f);
    shader.setUniform1f("ch1HdZCrib", 0.0f);
    
    // Channel 2 aspect ratio and scaling
    shader.setUniform1f("ch2ScaleFix", 1.0f);
    shader.setUniform1f("ch2AspectRatio", 1.0f);
    shader.setUniform1f("ch2CribX", 0.0f);
    shader.setUniform1f("ch2HdZCrib", 0.0f);
    shader.setUniform1f("cribY", 0.0f);
    
    // Input select
    shader.setUniform1i("ch1InputSelect", params.ch1InputSelect);
    shader.setUniform1i("ch2InputSelect", params.ch2InputSelect);
}

void Block1Shader::setChannel1Texture(ofTexture& tex) {
    ch1Tex = &tex;
}

void Block1Shader::setChannel2Texture(ofTexture& tex) {
    ch2Tex = &tex;
}

void Block1Shader::setFeedbackTexture(ofTexture& tex) {
    fbTex = &tex;
}

void Block1Shader::setTemporalFilterTexture(ofTexture& tex) {
    temporalTex = &tex;
}

//==============================================================================
// Modulation Support
//==============================================================================
void Block1Shader::initializeModulations() {
    // Channel 1 adjust
    modulations["ch1XDisplace"] = ParamModulation();
    modulations["ch1YDisplace"] = ParamModulation();
    modulations["ch1ZDisplace"] = ParamModulation();
    modulations["ch1Rotate"] = ParamModulation();
    modulations["ch1HueAttenuate"] = ParamModulation();
    modulations["ch1SaturationAttenuate"] = ParamModulation();
    modulations["ch1BrightAttenuate"] = ParamModulation();
    modulations["ch1Posterize"] = ParamModulation();
    modulations["ch1KaleidoscopeAmount"] = ParamModulation();
    modulations["ch1KaleidoscopeSlice"] = ParamModulation();
    modulations["ch1BlurAmount"] = ParamModulation();
    modulations["ch1BlurRadius"] = ParamModulation();
    modulations["ch1SharpenAmount"] = ParamModulation();
    modulations["ch1SharpenRadius"] = ParamModulation();
    modulations["ch1FiltersBoost"] = ParamModulation();
    
    // Channel 2 mix and key
    modulations["ch2MixAmount"] = ParamModulation();
    modulations["ch2KeyValueRed"] = ParamModulation();
    modulations["ch2KeyValueGreen"] = ParamModulation();
    modulations["ch2KeyValueBlue"] = ParamModulation();
    modulations["ch2KeyThreshold"] = ParamModulation();
    modulations["ch2KeySoft"] = ParamModulation();
    
    // Channel 2 adjust
    modulations["ch2XDisplace"] = ParamModulation();
    modulations["ch2YDisplace"] = ParamModulation();
    modulations["ch2ZDisplace"] = ParamModulation();
    modulations["ch2Rotate"] = ParamModulation();
    modulations["ch2HueAttenuate"] = ParamModulation();
    modulations["ch2SaturationAttenuate"] = ParamModulation();
    modulations["ch2BrightAttenuate"] = ParamModulation();
    modulations["ch2Posterize"] = ParamModulation();
    modulations["ch2KaleidoscopeAmount"] = ParamModulation();
    modulations["ch2KaleidoscopeSlice"] = ParamModulation();
    modulations["ch2BlurAmount"] = ParamModulation();
    modulations["ch2BlurRadius"] = ParamModulation();
    modulations["ch2SharpenAmount"] = ParamModulation();
    modulations["ch2SharpenRadius"] = ParamModulation();
    modulations["ch2FiltersBoost"] = ParamModulation();
    
    // FB1 feedback
    modulations["fb1MixAmount"] = ParamModulation();
    modulations["fb1KeyValueRed"] = ParamModulation();
    modulations["fb1KeyValueGreen"] = ParamModulation();
    modulations["fb1KeyValueBlue"] = ParamModulation();
    modulations["fb1KeyThreshold"] = ParamModulation();
    modulations["fb1KeySoft"] = ParamModulation();
    modulations["fb1XDisplace"] = ParamModulation();
    modulations["fb1YDisplace"] = ParamModulation();
    modulations["fb1ZDisplace"] = ParamModulation();
    modulations["fb1Rotate"] = ParamModulation();
    modulations["fb1ShearMatrix1"] = ParamModulation();
    modulations["fb1ShearMatrix2"] = ParamModulation();
    modulations["fb1ShearMatrix3"] = ParamModulation();
    modulations["fb1ShearMatrix4"] = ParamModulation();
    modulations["fb1KaleidoscopeAmount"] = ParamModulation();
    modulations["fb1KaleidoscopeSlice"] = ParamModulation();
    
    // FB1 color
    modulations["fb1HueOffset"] = ParamModulation();
    modulations["fb1SaturationOffset"] = ParamModulation();
    modulations["fb1BrightOffset"] = ParamModulation();
    modulations["fb1HueAttenuate"] = ParamModulation();
    modulations["fb1SaturationAttenuate"] = ParamModulation();
    modulations["fb1BrightAttenuate"] = ParamModulation();
    modulations["fb1HuePowmap"] = ParamModulation();
    modulations["fb1SaturationPowmap"] = ParamModulation();
    modulations["fb1BrightPowmap"] = ParamModulation();
    modulations["fb1HueShaper"] = ParamModulation();
    modulations["fb1Posterize"] = ParamModulation();
    
    // FB1 filters
    modulations["fb1BlurAmount"] = ParamModulation();
    modulations["fb1BlurRadius"] = ParamModulation();
    modulations["fb1SharpenAmount"] = ParamModulation();
    modulations["fb1SharpenRadius"] = ParamModulation();
    modulations["fb1TemporalFilter1Amount"] = ParamModulation();
    modulations["fb1TemporalFilter1Resonance"] = ParamModulation();
    modulations["fb1TemporalFilter2Amount"] = ParamModulation();
    modulations["fb1TemporalFilter2Resonance"] = ParamModulation();
    modulations["fb1FiltersBoost"] = ParamModulation();
}

ParamModulation* Block1Shader::getModulation(const std::string& paramName) {
    auto it = modulations.find(paramName);
    if (it != modulations.end()) {
        return &(it->second);
    }
    return nullptr;
}

float Block1Shader::getModulatedValue(const std::string& paramName) const {
    auto it = lastModulatedValues.find(paramName);
    if (it != lastModulatedValues.end()) {
        return it->second;
    }
    return 0.0f;
}

void Block1Shader::applyModulations(const AudioAnalyzer& audioAnalyzer, const TempoManager& tempo, float deltaTime) {
    if (!audioAnalyzer.isEnabled() && !tempo.isEnabled()) return;
    
    auto& p = params;
    
    // Helper lambda to apply modulation
    auto applyMod = [&](const std::string& name, float& value) {
        auto* mod = getModulation(name);
        if (mod) {
            value = mod->apply(value, audioAnalyzer, tempo, deltaTime);
            lastModulatedValues[name] = value;
        }
    };
    
    // Channel 1 adjust
    applyMod("ch1XDisplace", p.ch1XDisplace);
    applyMod("ch1YDisplace", p.ch1YDisplace);
    applyMod("ch1ZDisplace", p.ch1ZDisplace);
    applyMod("ch1Rotate", p.ch1Rotate);
    applyMod("ch1HueAttenuate", p.ch1HueAttenuate);
    applyMod("ch1SaturationAttenuate", p.ch1SaturationAttenuate);
    applyMod("ch1BrightAttenuate", p.ch1BrightAttenuate);
    applyMod("ch1Posterize", p.ch1Posterize);
    applyMod("ch1KaleidoscopeAmount", p.ch1KaleidoscopeAmount);
    applyMod("ch1KaleidoscopeSlice", p.ch1KaleidoscopeSlice);
    applyMod("ch1BlurAmount", p.ch1BlurAmount);
    applyMod("ch1BlurRadius", p.ch1BlurRadius);
    applyMod("ch1SharpenAmount", p.ch1SharpenAmount);
    applyMod("ch1SharpenRadius", p.ch1SharpenRadius);
    applyMod("ch1FiltersBoost", p.ch1FiltersBoost);
    
    // Channel 2 mix and key
    applyMod("ch2MixAmount", p.ch2MixAmount);
    applyMod("ch2KeyValueRed", p.ch2KeyValueRed);
    applyMod("ch2KeyValueGreen", p.ch2KeyValueGreen);
    applyMod("ch2KeyValueBlue", p.ch2KeyValueBlue);
    applyMod("ch2KeyThreshold", p.ch2KeyThreshold);
    applyMod("ch2KeySoft", p.ch2KeySoft);
    
    // Channel 2 adjust
    applyMod("ch2XDisplace", p.ch2XDisplace);
    applyMod("ch2YDisplace", p.ch2YDisplace);
    applyMod("ch2ZDisplace", p.ch2ZDisplace);
    applyMod("ch2Rotate", p.ch2Rotate);
    applyMod("ch2HueAttenuate", p.ch2HueAttenuate);
    applyMod("ch2SaturationAttenuate", p.ch2SaturationAttenuate);
    applyMod("ch2BrightAttenuate", p.ch2BrightAttenuate);
    applyMod("ch2Posterize", p.ch2Posterize);
    applyMod("ch2KaleidoscopeAmount", p.ch2KaleidoscopeAmount);
    applyMod("ch2KaleidoscopeSlice", p.ch2KaleidoscopeSlice);
    applyMod("ch2BlurAmount", p.ch2BlurAmount);
    applyMod("ch2BlurRadius", p.ch2BlurRadius);
    applyMod("ch2SharpenAmount", p.ch2SharpenAmount);
    applyMod("ch2SharpenRadius", p.ch2SharpenRadius);
    applyMod("ch2FiltersBoost", p.ch2FiltersBoost);
    
    // FB1 feedback
    applyMod("fb1MixAmount", p.fb1MixAmount);
    applyMod("fb1KeyValueRed", p.fb1KeyValueRed);
    applyMod("fb1KeyValueGreen", p.fb1KeyValueGreen);
    applyMod("fb1KeyValueBlue", p.fb1KeyValueBlue);
    applyMod("fb1KeyThreshold", p.fb1KeyThreshold);
    applyMod("fb1KeySoft", p.fb1KeySoft);
    applyMod("fb1XDisplace", p.fb1XDisplace);
    applyMod("fb1YDisplace", p.fb1YDisplace);
    applyMod("fb1ZDisplace", p.fb1ZDisplace);
    applyMod("fb1Rotate", p.fb1Rotate);
    applyMod("fb1ShearMatrix1", p.fb1ShearMatrix1);
    applyMod("fb1ShearMatrix2", p.fb1ShearMatrix2);
    applyMod("fb1ShearMatrix3", p.fb1ShearMatrix3);
    applyMod("fb1ShearMatrix4", p.fb1ShearMatrix4);
    applyMod("fb1KaleidoscopeAmount", p.fb1KaleidoscopeAmount);
    applyMod("fb1KaleidoscopeSlice", p.fb1KaleidoscopeSlice);
    
    // FB1 color
    applyMod("fb1HueOffset", p.fb1HueOffset);
    applyMod("fb1SaturationOffset", p.fb1SaturationOffset);
    applyMod("fb1BrightOffset", p.fb1BrightOffset);
    applyMod("fb1HueAttenuate", p.fb1HueAttenuate);
    applyMod("fb1SaturationAttenuate", p.fb1SaturationAttenuate);
    applyMod("fb1BrightAttenuate", p.fb1BrightAttenuate);
    applyMod("fb1HuePowmap", p.fb1HuePowmap);
    applyMod("fb1SaturationPowmap", p.fb1SaturationPowmap);
    applyMod("fb1BrightPowmap", p.fb1BrightPowmap);
    applyMod("fb1HueShaper", p.fb1HueShaper);
    applyMod("fb1Posterize", p.fb1Posterize);
    
    // FB1 filters
    applyMod("fb1BlurAmount", p.fb1BlurAmount);
    applyMod("fb1BlurRadius", p.fb1BlurRadius);
    applyMod("fb1SharpenAmount", p.fb1SharpenAmount);
    applyMod("fb1SharpenRadius", p.fb1SharpenRadius);
    applyMod("fb1TemporalFilter1Amount", p.fb1TemporalFilter1Amount);
    applyMod("fb1TemporalFilter1Resonance", p.fb1TemporalFilter1Resonance);
    applyMod("fb1TemporalFilter2Amount", p.fb1TemporalFilter2Amount);
    applyMod("fb1TemporalFilter2Resonance", p.fb1TemporalFilter2Resonance);
    applyMod("fb1FiltersBoost", p.fb1FiltersBoost);
}

float Block1Shader::getEffectiveValue(const std::string& paramName, float baseValue, 
                                      const AudioAnalyzer& audioAnalyzer, const TempoManager& tempo, float deltaTime) {
    auto* mod = getModulation(paramName);
    if (mod) {
        float result = mod->apply(baseValue, audioAnalyzer, tempo, deltaTime);
        lastModulatedValues[paramName] = result;
        return result;
    }
    lastModulatedValues[paramName] = baseValue;
    return baseValue;
}

void Block1Shader::loadModulations(const ofJson& json) {
    for (auto& [key, value] : modulations) {
        if (json.contains(key)) {
            value.loadFromJson(json[key]);
        }
    }
}

ofJson Block1Shader::saveModulations() const {
    ofJson json;
    for (const auto& [key, value] : modulations) {
        value.saveToJson(json[key]);
    }
    return json;
}

} // namespace dragonwaves
