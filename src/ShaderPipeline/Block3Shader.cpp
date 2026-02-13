#include "Block3Shader.h"

namespace dragonwaves {

Block3Shader::Block3Shader()
    : ShaderBlock("Block3", "shader3") {
}

void Block3Shader::setup(int width, int height) {
    ShaderBlock::setup(width, height);
    
    dummyTex.allocate(width, height, GL_RGBA);
    ofPixels pixels;
    pixels.allocate(width, height, OF_PIXELS_RGBA);
    pixels.setColor(ofColor::black);
    dummyTex.loadData(pixels);
}

void Block3Shader::process() {
    ShaderBlock::process();
    
    // Bind textures - use units 0 and 1 for maximum compatibility
    if (block1Tex && block1Tex->isAllocated()) {
        shader.setUniformTexture("block1Output", *block1Tex, 0);
    } else {
        shader.setUniformTexture("block1Output", dummyTex, 0);
    }
    
    if (block2Tex && block2Tex->isAllocated()) {
        shader.setUniformTexture("block2Output", *block2Tex, 1);
    } else {
        shader.setUniformTexture("block2Output", dummyTex, 1);
    }
    
    // Resolution uniforms
    shader.setUniform1f("width", width);
    shader.setUniform1f("height", height);
    shader.setUniform1f("inverseWidth", 1.0f / width);
    shader.setUniform1f("inverseHeight", 1.0f / height);
    
    // Block1 geo (final stage)
    shader.setUniform2f("block1XYDisplace", params.block1XDisplace, params.block1YDisplace);
    float z1 = params.block1ZDisplace;
    if (z1 > 1.0f) {
        z1 = pow(2.0f, (z1 - 1.0f) * 8.0f);
        if (params.block1ZDisplace >= 2.0f) z1 = 1000.0f;
    }
    shader.setUniform1f("block1ZDisplace", z1);
    shader.setUniform1f("block1Rotate", params.block1Rotate);
    shader.setUniform4f("block1ShearMatrix", params.block1ShearMatrix1, params.block1ShearMatrix2,
                        params.block1ShearMatrix3, params.block1ShearMatrix4);
    shader.setUniform1f("block1KaleidoscopeAmount", params.block1KaleidoscopeAmount);
    shader.setUniform1f("block1KaleidoscopeSlice", params.block1KaleidoscopeSlice);
    
    shader.setUniform1i("block1HMirror", params.block1HMirror);
    shader.setUniform1i("block1VMirror", params.block1VMirror);
    shader.setUniform1i("block1HFlip", params.block1HFlip);
    shader.setUniform1i("block1VFlip", params.block1VFlip);
    shader.setUniform1i("block1RotateMode", params.block1RotateMode);
    shader.setUniform1i("block1GeoOverflow", params.block1GeoOverflow);
    
    // Block1 colorize
    shader.setUniform1i("block1ColorizeSwitch", params.block1ColorizeSwitch);
    shader.setUniform1i("block1ColorizeHSB_RGB", params.block1ColorizeHSB_RGB);
    shader.setUniform3f("block1ColorizeBand1", params.block1ColorizeHueBand1,
                        params.block1ColorizeSaturationBand1, params.block1ColorizeBrightBand1);
    shader.setUniform3f("block1ColorizeBand2", params.block1ColorizeHueBand2,
                        params.block1ColorizeSaturationBand2, params.block1ColorizeBrightBand2);
    shader.setUniform3f("block1ColorizeBand3", params.block1ColorizeHueBand3,
                        params.block1ColorizeSaturationBand3, params.block1ColorizeBrightBand3);
    shader.setUniform3f("block1ColorizeBand4", params.block1ColorizeHueBand4,
                        params.block1ColorizeSaturationBand4, params.block1ColorizeBrightBand4);
    shader.setUniform3f("block1ColorizeBand5", params.block1ColorizeHueBand5,
                        params.block1ColorizeSaturationBand5, params.block1ColorizeBrightBand5);
    
    // Block1 filters
    shader.setUniform1f("block1BlurAmount", params.block1BlurAmount);
    shader.setUniform1f("block1BlurRadius", params.block1BlurRadius);
    shader.setUniform1f("block1SharpenAmount", params.block1SharpenAmount);
    shader.setUniform1f("block1SharpenRadius", params.block1SharpenRadius);
    shader.setUniform1f("block1FiltersBoost", params.block1FiltersBoost);
    shader.setUniform1f("block1Dither", params.block1Dither);
    shader.setUniform1i("block1DitherSwitch", params.block1DitherSwitch);
    shader.setUniform1i("block1DitherType", params.block1DitherType);
    
    // Block2 geo (final stage)
    shader.setUniform2f("block2XYDisplace", params.block2XDisplace, params.block2YDisplace);
    float z2 = params.block2ZDisplace;
    if (z2 > 1.0f) {
        z2 = pow(2.0f, (z2 - 1.0f) * 8.0f);
        if (params.block2ZDisplace >= 2.0f) z2 = 1000.0f;
    }
    shader.setUniform1f("block2ZDisplace", z2);
    shader.setUniform1f("block2Rotate", params.block2Rotate);
    shader.setUniform4f("block2ShearMatrix", params.block2ShearMatrix1, params.block2ShearMatrix2,
                        params.block2ShearMatrix3, params.block2ShearMatrix4);
    shader.setUniform1f("block2KaleidoscopeAmount", params.block2KaleidoscopeAmount);
    shader.setUniform1f("block2KaleidoscopeSlice", params.block2KaleidoscopeSlice);
    
    shader.setUniform1i("block2HMirror", params.block2HMirror);
    shader.setUniform1i("block2VMirror", params.block2VMirror);
    shader.setUniform1i("block2HFlip", params.block2HFlip);
    shader.setUniform1i("block2VFlip", params.block2VFlip);
    shader.setUniform1i("block2RotateMode", params.block2RotateMode);
    shader.setUniform1i("block2GeoOverflow", params.block2GeoOverflow);
    
    // Block2 colorize
    shader.setUniform1i("block2ColorizeSwitch", params.block2ColorizeSwitch);
    shader.setUniform1i("block2ColorizeHSB_RGB", params.block2ColorizeHSB_RGB);
    shader.setUniform3f("block2ColorizeBand1", params.block2ColorizeHueBand1,
                        params.block2ColorizeSaturationBand1, params.block2ColorizeBrightBand1);
    shader.setUniform3f("block2ColorizeBand2", params.block2ColorizeHueBand2,
                        params.block2ColorizeSaturationBand2, params.block2ColorizeBrightBand2);
    shader.setUniform3f("block2ColorizeBand3", params.block2ColorizeHueBand3,
                        params.block2ColorizeSaturationBand3, params.block2ColorizeBrightBand3);
    shader.setUniform3f("block2ColorizeBand4", params.block2ColorizeHueBand4,
                        params.block2ColorizeSaturationBand4, params.block2ColorizeBrightBand4);
    shader.setUniform3f("block2ColorizeBand5", params.block2ColorizeHueBand5,
                        params.block2ColorizeSaturationBand5, params.block2ColorizeBrightBand5);
    
    // Block2 filters
    shader.setUniform1f("block2BlurAmount", params.block2BlurAmount);
    shader.setUniform1f("block2BlurRadius", params.block2BlurRadius);
    shader.setUniform1f("block2SharpenAmount", params.block2SharpenAmount);
    shader.setUniform1f("block2SharpenRadius", params.block2SharpenRadius);
    shader.setUniform1f("block2FiltersBoost", params.block2FiltersBoost);
    shader.setUniform1f("block2Dither", params.block2Dither);
    shader.setUniform1i("block2DitherSwitch", params.block2DitherSwitch);
    shader.setUniform1i("block2DitherType", params.block2DitherType);
    
    // Matrix mixer
    shader.setUniform1i("matrixMixType", params.matrixMixType);
    shader.setUniform1i("matrixMixOverflow", params.matrixMixOverflow);
    shader.setUniform3f("bgRGBIntoFgRed", params.matrixMixBgRedIntoFgRed,
                        params.matrixMixBgGreenIntoFgRed, params.matrixMixBgBlueIntoFgRed);
    shader.setUniform3f("bgRGBIntoFgGreen", params.matrixMixBgRedIntoFgGreen,
                        params.matrixMixBgGreenIntoFgGreen, params.matrixMixBgBlueIntoFgGreen);
    shader.setUniform3f("bgRGBIntoFgBlue", params.matrixMixBgRedIntoFgBlue,
                        params.matrixMixBgGreenIntoFgBlue, params.matrixMixBgBlueIntoFgBlue);
    
    // Final mix and key
    shader.setUniform1f("finalMixAmount", params.finalMixAmount);
    shader.setUniform3f("finalKeyValue", params.finalKeyValueRed, params.finalKeyValueGreen, params.finalKeyValueBlue);
    shader.setUniform1f("finalKeyThreshold", params.finalKeyThreshold);
    shader.setUniform1f("finalKeySoft", params.finalKeySoft);
    shader.setUniform1i("finalKeyOrder", params.finalKeyOrder);
    shader.setUniform1i("finalMixType", params.finalMixType);
    shader.setUniform1i("finalMixOverflow", params.finalMixOverflow);
}

void Block3Shader::setBlock1Texture(ofTexture& tex) {
    block1Tex = &tex;
}

void Block3Shader::setBlock2Texture(ofTexture& tex) {
    block2Tex = &tex;
}

} // namespace dragonwaves
