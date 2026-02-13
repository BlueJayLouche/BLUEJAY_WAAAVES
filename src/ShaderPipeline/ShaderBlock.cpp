#include "ShaderBlock.h"

namespace dragonwaves {

ShaderBlock::ShaderBlock(const std::string& name, const std::string& shaderName)
    : name(name), shaderName(shaderName) {
}

void ShaderBlock::setup(int w, int h) {
    width = w;
    height = h;
    
    // Load shader using ShaderLoader
    if (!ShaderLoader::load(shader, shaderName)) {
        ofLogError("ShaderBlock") << "Failed to load shader: " << shaderName;
    }
    
    // Allocate output FBO
    allocateFbo(outputFbo, width, height);
    
    initialized = true;
    
    ofLogNotice("ShaderBlock") << name << " initialized at " << width << "x" << height;
}

void ShaderBlock::process() {
    if (!initialized) return;
    
    // Viewport and projection are already set by PipelineManager
    // which calls outputFbo.begin() before shader.begin()
    ofViewport(0, 0, width, height);
    ofSetupScreenOrtho(width, height);
    // Note: FBO begin/clear is handled by PipelineManager
}

void ShaderBlock::resize(int w, int h) {
    width = w;
    height = h;
    allocateFbo(outputFbo, width, height);
    
    ofLogNotice("ShaderBlock") << name << " resized to " << width << "x" << height;
}

void ShaderBlock::clear() {
    outputFbo.begin();
    ofClear(0, 0, 0, 255);
    outputFbo.end();
}

void ShaderBlock::allocateFbo(ofFbo& fbo, int w, int h) {
    ofFboSettings settings;
    settings.width = w;
    settings.height = h;
    settings.internalformat = GL_RGBA8;
    settings.useDepth = false;
    settings.useStencil = false;
    fbo.allocate(settings);
    fbo.begin();
    ofClear(0, 0, 0, 255);
    fbo.end();
}

} // namespace dragonwaves
