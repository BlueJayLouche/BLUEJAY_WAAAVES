#include "Block2Shader.h"
#include "Block3Shader.h"  // For ParamModulation

namespace dragonwaves {

Block2Shader::Block2Shader()
    : ShaderBlock("Block2", "shader2") {
    initializeModulations();
}

void Block2Shader::setup(int width, int height) {
    ShaderBlock::setup(width, height);
    
    dummyTex.allocate(width, height, GL_RGBA);
    ofPixels pixels;
    pixels.allocate(width, height, OF_PIXELS_RGBA);
    pixels.setColor(ofColor::black);
    dummyTex.loadData(pixels);
    
    params.block2InputSelect = 0;  // Default to block1 input
}

void Block2Shader::process() {
    ShaderBlock::process();
    
    // Bind textures based on block2InputSelect
    // 0 = use block1 output (fb texture), 1 = input1, 2 = input2
    if (params.block2InputSelect == 0) {
        // Use block1 output
        if (block1Tex && block1Tex->isAllocated()) {
            shader.setUniformTexture("block2InputTex", *block1Tex, 6);
        } else {
            shader.setUniformTexture("block2InputTex", dummyTex, 6);
        }
    } else {
        // Use external input (input1 or input2)
        if (inputTex && inputTex->isAllocated()) {
            shader.setUniformTexture("block2InputTex", *inputTex, 6);
        } else {
            shader.setUniformTexture("block2InputTex", dummyTex, 6);
        }
    }
    
    if (fbTex && fbTex->isAllocated()) {
        shader.setUniformTexture("tex0", *fbTex, 4);
    } else {
        shader.setUniformTexture("tex0", dummyTex, 4);
    }
    
    if (temporalTex && temporalTex->isAllocated()) {
        shader.setUniformTexture("fb2TemporalFilter", *temporalTex, 5);
    } else {
        shader.setUniformTexture("fb2TemporalFilter", dummyTex, 5);
    }
    
    // Resolution uniforms
    shader.setUniform1f("width", width);
    shader.setUniform1f("height", height);
    shader.setUniform1f("inverseWidth", 1.0f / width);
    shader.setUniform1f("inverseHeight", 1.0f / height);
    
    // Block2 input parameters
    shader.setUniform1i("block2InputMasterSwitch", params.block2InputMasterSwitch);
    shader.setUniform1f("block2InputWidth", width);
    shader.setUniform1f("block2InputHeight", height);
    shader.setUniform1f("block2InputWidthHalf", width * 0.5f);
    shader.setUniform1f("block2InputHeightHalf", height * 0.5f);
    
    shader.setUniform2f("block2InputXYDisplace", params.block2InputXDisplace, params.block2InputYDisplace);
    shader.setUniform1f("block2InputZDisplace", params.block2InputZDisplace);
    shader.setUniform1f("block2InputRotate", params.block2InputRotate);
    shader.setUniform3f("block2InputHSBAttenuate", params.block2InputHueAttenuate, 
                        params.block2InputSaturationAttenuate, params.block2InputBrightAttenuate);
    shader.setUniform1f("block2InputPosterize", params.block2InputPosterize);
    shader.setUniform1f("block2InputPosterizeInvert", 1.0f / params.block2InputPosterize);
    shader.setUniform1i("block2InputPosterizeSwitch", params.block2InputPosterizeSwitch);
    shader.setUniform1f("block2InputKaleidoscopeAmount", params.block2InputKaleidoscopeAmount);
    shader.setUniform1f("block2InputKaleidoscopeSlice", params.block2InputKaleidoscopeSlice);
    shader.setUniform1f("block2InputBlurAmount", params.block2InputBlurAmount);
    shader.setUniform1f("block2InputBlurRadius", params.block2InputBlurRadius);
    shader.setUniform1f("block2InputSharpenAmount", params.block2InputSharpenAmount);
    shader.setUniform1f("block2InputSharpenRadius", params.block2InputSharpenRadius);
    shader.setUniform1f("block2InputFiltersBoost", params.block2InputFiltersBoost);
    
    shader.setUniform1i("block2InputGeoOverflow", params.block2InputGeoOverflow);
    shader.setUniform1i("block2InputHMirror", params.block2InputHMirror);
    shader.setUniform1i("block2InputVMirror", params.block2InputVMirror);
    shader.setUniform1i("block2InputHFlip", params.block2InputHFlip);
    shader.setUniform1i("block2InputVFlip", params.block2InputVFlip);
    shader.setUniform1i("block2InputHueInvert", params.block2InputHueInvert);
    shader.setUniform1i("block2InputSaturationInvert", params.block2InputSaturationInvert);
    shader.setUniform1i("block2InputBrightInvert", params.block2InputBrightInvert);
    shader.setUniform1i("block2InputRGBInvert", params.block2InputRGBInvert);
    shader.setUniform1i("block2InputSolarize", params.block2InputSolarize);
    
    shader.setUniform1i("block2InputHdAspectOn", params.block2InputHdAspectOn);
    shader.setUniform2f("block2InputHdAspectXYFix", block2InputHdAspectXFix, block2InputHdAspectYFix);
    
    // FB2 parameters
    shader.setUniform1f("fb2MixAmount", params.fb2MixAmount);
    shader.setUniform3f("fb2KeyValue", params.fb2KeyValueRed, params.fb2KeyValueGreen, params.fb2KeyValueBlue);
    shader.setUniform1f("fb2KeyThreshold", params.fb2KeyThreshold);
    shader.setUniform1f("fb2KeySoft", params.fb2KeySoft);
    shader.setUniform1i("fb2KeyOrder", params.fb2KeyOrder);
    shader.setUniform1i("fb2MixType", params.fb2MixType);
    shader.setUniform1i("fb2MixOverflow", params.fb2MixOverflow);
    
    shader.setUniform2f("fb2XYDisplace", params.fb2XDisplace, params.fb2YDisplace);
    shader.setUniform1f("fb2ZDisplace", params.fb2ZDisplace);
    shader.setUniform1f("fb2Rotate", params.fb2Rotate);
    shader.setUniform4f("fb2ShearMatrix", params.fb2ShearMatrix1, params.fb2ShearMatrix2,
                        params.fb2ShearMatrix3, params.fb2ShearMatrix4);
    shader.setUniform1f("fb2KaleidoscopeAmount", params.fb2KaleidoscopeAmount);
    shader.setUniform1f("fb2KaleidoscopeSlice", params.fb2KaleidoscopeSlice);
    
    shader.setUniform1i("fb2HMirror", params.fb2HMirror);
    shader.setUniform1i("fb2VMirror", params.fb2VMirror);
    shader.setUniform1i("fb2HFlip", params.fb2HFlip);
    shader.setUniform1i("fb2VFlip", params.fb2VFlip);
    shader.setUniform1i("fb2RotateMode", params.fb2RotateMode);
    shader.setUniform1i("fb2GeoOverflow", params.fb2GeoOverflow);
    
    shader.setUniform3f("fb2HSBOffset", params.fb2HueOffset, params.fb2SaturationOffset, params.fb2BrightOffset);
    shader.setUniform3f("fb2HSBAttenuate", params.fb2HueAttenuate, params.fb2SaturationAttenuate, params.fb2BrightAttenuate);
    shader.setUniform3f("fb2HSBPowmap", params.fb2HuePowmap, params.fb2SaturationPowmap, params.fb2BrightPowmap);
    shader.setUniform1f("fb2HueShaper", params.fb2HueShaper);
    shader.setUniform1f("fb2Posterize", params.fb2Posterize);
    shader.setUniform1f("fb2PosterizeInvert", 1.0f / params.fb2Posterize);
    shader.setUniform1i("fb2PosterizeSwitch", params.fb2PosterizeSwitch);
    
    shader.setUniform1i("fb2HueInvert", params.fb2HueInvert);
    shader.setUniform1i("fb2SaturationInvert", params.fb2SaturationInvert);
    shader.setUniform1i("fb2BrightInvert", params.fb2BrightInvert);
    
    shader.setUniform1f("fb2BlurAmount", params.fb2BlurAmount);
    shader.setUniform1f("fb2BlurRadius", params.fb2BlurRadius);
    shader.setUniform1f("fb2SharpenAmount", params.fb2SharpenAmount);
    shader.setUniform1f("fb2SharpenRadius", params.fb2SharpenRadius);
    shader.setUniform1f("fb2TemporalFilter1Amount", params.fb2TemporalFilter1Amount);
    shader.setUniform1f("fb2TemporalFilter1Resonance", params.fb2TemporalFilter1Resonance);
    shader.setUniform1f("fb2TemporalFilter2Amount", params.fb2TemporalFilter2Amount);
    shader.setUniform1f("fb2TemporalFilter2Resonance", params.fb2TemporalFilter2Resonance);
    shader.setUniform1f("fb2FiltersBoost", params.fb2FiltersBoost);
    
    // Input select
    shader.setUniform1i("block2InputSelect", params.block2InputSelect);
}

void Block2Shader::setBlock1Texture(ofTexture& tex) {
    block1Tex = &tex;
}

void Block2Shader::setInputTexture(ofTexture& tex) {
    inputTex = &tex;
}

void Block2Shader::setFeedbackTexture(ofTexture& tex) {
    fbTex = &tex;
}

void Block2Shader::setTemporalFilterTexture(ofTexture& tex) {
    temporalTex = &tex;
}

//==============================================================================
// Modulation Support
//==============================================================================
void Block2Shader::initializeModulations() {
    // Block2 input adjust
    modulations["block2InputXDisplace"] = ParamModulation();
    modulations["block2InputYDisplace"] = ParamModulation();
    modulations["block2InputZDisplace"] = ParamModulation();
    modulations["block2InputRotate"] = ParamModulation();
    modulations["block2InputHueAttenuate"] = ParamModulation();
    modulations["block2InputSaturationAttenuate"] = ParamModulation();
    modulations["block2InputBrightAttenuate"] = ParamModulation();
    modulations["block2InputPosterize"] = ParamModulation();
    modulations["block2InputKaleidoscopeAmount"] = ParamModulation();
    modulations["block2InputKaleidoscopeSlice"] = ParamModulation();
    modulations["block2InputBlurAmount"] = ParamModulation();
    modulations["block2InputBlurRadius"] = ParamModulation();
    modulations["block2InputSharpenAmount"] = ParamModulation();
    modulations["block2InputSharpenRadius"] = ParamModulation();
    modulations["block2InputFiltersBoost"] = ParamModulation();
    
    // FB2 feedback
    modulations["fb2MixAmount"] = ParamModulation();
    modulations["fb2KeyValueRed"] = ParamModulation();
    modulations["fb2KeyValueGreen"] = ParamModulation();
    modulations["fb2KeyValueBlue"] = ParamModulation();
    modulations["fb2KeyThreshold"] = ParamModulation();
    modulations["fb2KeySoft"] = ParamModulation();
    modulations["fb2XDisplace"] = ParamModulation();
    modulations["fb2YDisplace"] = ParamModulation();
    modulations["fb2ZDisplace"] = ParamModulation();
    modulations["fb2Rotate"] = ParamModulation();
    modulations["fb2ShearMatrix1"] = ParamModulation();
    modulations["fb2ShearMatrix2"] = ParamModulation();
    modulations["fb2ShearMatrix3"] = ParamModulation();
    modulations["fb2ShearMatrix4"] = ParamModulation();
    modulations["fb2KaleidoscopeAmount"] = ParamModulation();
    modulations["fb2KaleidoscopeSlice"] = ParamModulation();
    
    // FB2 color
    modulations["fb2HueOffset"] = ParamModulation();
    modulations["fb2SaturationOffset"] = ParamModulation();
    modulations["fb2BrightOffset"] = ParamModulation();
    modulations["fb2HueAttenuate"] = ParamModulation();
    modulations["fb2SaturationAttenuate"] = ParamModulation();
    modulations["fb2BrightAttenuate"] = ParamModulation();
    modulations["fb2HuePowmap"] = ParamModulation();
    modulations["fb2SaturationPowmap"] = ParamModulation();
    modulations["fb2BrightPowmap"] = ParamModulation();
    modulations["fb2HueShaper"] = ParamModulation();
    modulations["fb2Posterize"] = ParamModulation();
    
    // FB2 filters
    modulations["fb2BlurAmount"] = ParamModulation();
    modulations["fb2BlurRadius"] = ParamModulation();
    modulations["fb2SharpenAmount"] = ParamModulation();
    modulations["fb2SharpenRadius"] = ParamModulation();
    modulations["fb2TemporalFilter1Amount"] = ParamModulation();
    modulations["fb2TemporalFilter1Resonance"] = ParamModulation();
    modulations["fb2TemporalFilter2Amount"] = ParamModulation();
    modulations["fb2TemporalFilter2Resonance"] = ParamModulation();
    modulations["fb2FiltersBoost"] = ParamModulation();
}

ParamModulation* Block2Shader::getModulation(const std::string& paramName) {
    auto it = modulations.find(paramName);
    if (it != modulations.end()) {
        return &(it->second);
    }
    return nullptr;
}

float Block2Shader::getModulatedValue(const std::string& paramName) const {
    auto it = lastModulatedValues.find(paramName);
    if (it != lastModulatedValues.end()) {
        return it->second;
    }
    return 0.0f;
}

void Block2Shader::applyModulations(const AudioAnalyzer& audioAnalyzer, const TempoManager& tempo, float deltaTime) {
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
    
    // Block2 input adjust
    applyMod("block2InputXDisplace", p.block2InputXDisplace);
    applyMod("block2InputYDisplace", p.block2InputYDisplace);
    applyMod("block2InputZDisplace", p.block2InputZDisplace);
    applyMod("block2InputRotate", p.block2InputRotate);
    applyMod("block2InputHueAttenuate", p.block2InputHueAttenuate);
    applyMod("block2InputSaturationAttenuate", p.block2InputSaturationAttenuate);
    applyMod("block2InputBrightAttenuate", p.block2InputBrightAttenuate);
    applyMod("block2InputPosterize", p.block2InputPosterize);
    applyMod("block2InputKaleidoscopeAmount", p.block2InputKaleidoscopeAmount);
    applyMod("block2InputKaleidoscopeSlice", p.block2InputKaleidoscopeSlice);
    applyMod("block2InputBlurAmount", p.block2InputBlurAmount);
    applyMod("block2InputBlurRadius", p.block2InputBlurRadius);
    applyMod("block2InputSharpenAmount", p.block2InputSharpenAmount);
    applyMod("block2InputSharpenRadius", p.block2InputSharpenRadius);
    applyMod("block2InputFiltersBoost", p.block2InputFiltersBoost);
    
    // FB2 feedback
    applyMod("fb2MixAmount", p.fb2MixAmount);
    applyMod("fb2KeyValueRed", p.fb2KeyValueRed);
    applyMod("fb2KeyValueGreen", p.fb2KeyValueGreen);
    applyMod("fb2KeyValueBlue", p.fb2KeyValueBlue);
    applyMod("fb2KeyThreshold", p.fb2KeyThreshold);
    applyMod("fb2KeySoft", p.fb2KeySoft);
    applyMod("fb2XDisplace", p.fb2XDisplace);
    applyMod("fb2YDisplace", p.fb2YDisplace);
    applyMod("fb2ZDisplace", p.fb2ZDisplace);
    applyMod("fb2Rotate", p.fb2Rotate);
    applyMod("fb2ShearMatrix1", p.fb2ShearMatrix1);
    applyMod("fb2ShearMatrix2", p.fb2ShearMatrix2);
    applyMod("fb2ShearMatrix3", p.fb2ShearMatrix3);
    applyMod("fb2ShearMatrix4", p.fb2ShearMatrix4);
    applyMod("fb2KaleidoscopeAmount", p.fb2KaleidoscopeAmount);
    applyMod("fb2KaleidoscopeSlice", p.fb2KaleidoscopeSlice);
    
    // FB2 color
    applyMod("fb2HueOffset", p.fb2HueOffset);
    applyMod("fb2SaturationOffset", p.fb2SaturationOffset);
    applyMod("fb2BrightOffset", p.fb2BrightOffset);
    applyMod("fb2HueAttenuate", p.fb2HueAttenuate);
    applyMod("fb2SaturationAttenuate", p.fb2SaturationAttenuate);
    applyMod("fb2BrightAttenuate", p.fb2BrightAttenuate);
    applyMod("fb2HuePowmap", p.fb2HuePowmap);
    applyMod("fb2SaturationPowmap", p.fb2SaturationPowmap);
    applyMod("fb2BrightPowmap", p.fb2BrightPowmap);
    applyMod("fb2HueShaper", p.fb2HueShaper);
    applyMod("fb2Posterize", p.fb2Posterize);
    
    // FB2 filters
    applyMod("fb2BlurAmount", p.fb2BlurAmount);
    applyMod("fb2BlurRadius", p.fb2BlurRadius);
    applyMod("fb2SharpenAmount", p.fb2SharpenAmount);
    applyMod("fb2SharpenRadius", p.fb2SharpenRadius);
    applyMod("fb2TemporalFilter1Amount", p.fb2TemporalFilter1Amount);
    applyMod("fb2TemporalFilter1Resonance", p.fb2TemporalFilter1Resonance);
    applyMod("fb2TemporalFilter2Amount", p.fb2TemporalFilter2Amount);
    applyMod("fb2TemporalFilter2Resonance", p.fb2TemporalFilter2Resonance);
    applyMod("fb2FiltersBoost", p.fb2FiltersBoost);
}

float Block2Shader::getEffectiveValue(const std::string& paramName, float baseValue, 
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

void Block2Shader::loadModulations(const ofJson& json) {
    for (auto& [key, value] : modulations) {
        if (json.contains(key)) {
            value.loadFromJson(json[key]);
        }
    }
}

ofJson Block2Shader::saveModulations() const {
    ofJson json;
    for (const auto& [key, value] : modulations) {
        value.saveToJson(json[key]);
    }
    return json;
}

} // namespace dragonwaves
