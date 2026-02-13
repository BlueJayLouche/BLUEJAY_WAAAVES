#pragma once

#include "ofMain.h"
#include "../ShaderLoader.h"

namespace dragonwaves {

//==============================================================================
// Base class for shader blocks
//==============================================================================
class ShaderBlock {
public:
    ShaderBlock(const std::string& name, const std::string& shaderName);
    virtual ~ShaderBlock() = default;
    
    // Setup with resolution
    virtual void setup(int width, int height);
    
    // Process the shader - to be called between begin()/end()
    virtual void process();
    
    // Get the output FBO
    ofFbo& getOutput() { return outputFbo; }
    ofTexture& getOutputTexture() { return outputFbo.getTexture(); }
    
    // Resize
    virtual void resize(int width, int height);
    
    // Clear
    virtual void clear();
    
    const std::string& getName() const { return name; }
    
    // Shader access for PipelineManager
    ofShader& getShader() { return shader; }
    
protected:
    std::string name;
    std::string shaderName;
    ofShader shader;
    ofFbo outputFbo;
    int width = 0;
    int height = 0;
    bool initialized = false;
    
    // Helper to allocate GPU-only FBO
    void allocateFbo(ofFbo& fbo, int w, int h);
};

} // namespace dragonwaves
