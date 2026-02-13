#include "Block1Shader.h"

namespace dragonwaves {

Block1Shader::Block1Shader()
    : ShaderBlock("Block1", "shader1") {
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

} // namespace dragonwaves
