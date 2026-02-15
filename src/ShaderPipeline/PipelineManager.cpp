#include "PipelineManager.h"

namespace dragonwaves {

//==============================================================================
// DelayBuffer
//==============================================================================
void DelayBuffer::setup(int w, int h) {
    width = w;
    height = h;
    
    for (int i = 0; i < MAX_FRAMES; i++) {
        ofFboSettings settings;
        settings.width = w;
        settings.height = h;
        settings.internalformat = GL_RGBA8;
        settings.useDepth = false;
        settings.useStencil = false;
        frames[i].allocate(settings);
        frames[i].begin();
        ofClear(0, 0, 0, 255);
        frames[i].end();
    }
    
    writeIndex = 0;
    initialized = true;
    
    ofLogNotice("DelayBuffer") << "Setup with " << MAX_FRAMES << " frames at " << w << "x" << h;
}

void DelayBuffer::resize(int w, int h) {
    if (!initialized) {
        setup(w, h);
        return;
    }
    
    width = w;
    height = h;
    
    for (int i = 0; i < MAX_FRAMES; i++) {
        ofFboSettings settings;
        settings.width = w;
        settings.height = h;
        settings.internalformat = GL_RGBA8;
        settings.useDepth = false;
        settings.useStencil = false;
        frames[i].allocate(settings);
        frames[i].begin();
        ofClear(0, 0, 0, 255);
        frames[i].end();
    }
    
    ofLogNotice("DelayBuffer") << "Resized to " << w << "x" << h;
}

void DelayBuffer::pushFrame(ofFbo& frame) {
    if (!initialized) return;
    
    frames[writeIndex].begin();
    ofViewport(0, 0, width, height);
    ofSetupScreenOrtho(width, height);
    ofClear(0, 0, 0, 255);
    frame.getTexture().draw(0, 0, width, height);
    frames[writeIndex].end();
    
    writeIndex = (writeIndex + 1) % MAX_FRAMES;
}

ofTexture& DelayBuffer::getFrame(int delay) {
    if (!initialized || delay < 0 || delay >= MAX_FRAMES) {
        return frames[0].getTexture();
    }
    
    // Calculate read index based on delay
    int readIndex = (writeIndex - delay - 1 + MAX_FRAMES) % MAX_FRAMES;
    return frames[readIndex].getTexture();
}

void DelayBuffer::clear() {
    for (int i = 0; i < MAX_FRAMES; i++) {
        frames[i].begin();
        ofClear(0, 0, 0, 255);
        frames[i].end();
    }
    writeIndex = 0;
}

//==============================================================================
// PipelineManager
//==============================================================================
PipelineManager::PipelineManager() {
}

PipelineManager::~PipelineManager() {
}

void PipelineManager::setup(const DisplaySettings& settings) {
    displaySettings = settings;
    
    // Setup shader blocks
    block1.setup(settings.internalWidth, settings.internalHeight);
    block2.setup(settings.internalWidth, settings.internalHeight);
    block3.setup(settings.outputWidth, settings.outputHeight);
    
    // Setup delay buffers
    fb1Delay.setup(settings.internalWidth, settings.internalHeight);
    fb2Delay.setup(settings.internalWidth, settings.internalHeight);
    
    // Initialize cached mesh for Block3
    updateBlock3Mesh(settings.outputWidth, settings.outputHeight);
    
    allocateDummyTexture();
    
    initialized = true;
    
    ofLogNotice("PipelineManager") << "Setup complete";
}

void PipelineManager::updateBlock3Mesh(int width, int height) {
    block3Mesh.clear();
    block3Mesh.setMode(OF_PRIMITIVE_TRIANGLE_FAN);
    block3Mesh.addVertex(ofVec3f(0, 0, 0));
    block3Mesh.addTexCoord(ofVec2f(0, 0));
    block3Mesh.addVertex(ofVec3f(width, 0, 0));
    block3Mesh.addTexCoord(ofVec2f(1, 0));
    block3Mesh.addVertex(ofVec3f(width, height, 0));
    block3Mesh.addTexCoord(ofVec2f(1, 1));
    block3Mesh.addVertex(ofVec3f(0, height, 0));
    block3Mesh.addTexCoord(ofVec2f(0, 1));
}

void PipelineManager::allocateDummyTexture() {
    dummyTexture.allocate(displaySettings.internalWidth, displaySettings.internalHeight, GL_RGBA);
    ofPixels pixels;
    pixels.allocate(displaySettings.internalWidth, displaySettings.internalHeight, OF_PIXELS_RGBA);
    pixels.setColor(ofColor::black);
    dummyTexture.loadData(pixels);
}

void PipelineManager::processFrame() {
    if (!initialized) return;
    
    // ===== BLOCK 1 =====
    // Draw feedback texture first (delayed frame)
    ofTexture& fb1Tex = fb1Delay.getFrame(fb1DelayTime);
    ofTexture& fb1Temporal = fb1Delay.getFrame(0);  // Most recent for temporal filter
    
    block1.setFeedbackTexture(fb1Tex);
    block1.setTemporalFilterTexture(fb1Temporal);
    
    // Set input textures based on ch1InputSelect and ch2InputSelect
    // ch1InputSelect: 0=input1, 1=input2
    // ch2InputSelect: 0=input1, 1=input2
    ofTexture* ch1Tex = (block1.params.ch1InputSelect == 0) ? input1Tex : input2Tex;
    ofTexture* ch2Tex = (block1.params.ch2InputSelect == 0) ? input1Tex : input2Tex;
    
    if (ch1Tex && ch1Tex->isAllocated()) {
        block1.setChannel1Texture(*ch1Tex);
    } else {
        block1.setChannel1Texture(dummyTexture);
    }
    
    if (ch2Tex && ch2Tex->isAllocated()) {
        block1.setChannel2Texture(*ch2Tex);
    } else {
        block1.setChannel2Texture(dummyTexture);
    }
    
    // Process block 1
    block1.getOutput().begin();
    ofViewport(0, 0, block1.getOutput().getWidth(), block1.getOutput().getHeight());
    ofSetupScreenOrtho(block1.getOutput().getWidth(), block1.getOutput().getHeight());
    ofClear(0, 0, 0, 255);
    
    // Unbind only texture units used by Block1 (0-3) to prevent FBO self-binding issues
    // Unit 0: fb1Tex (bound by draw), Units 1-3: ch1Tex, ch2Tex, temporal (bound by uniforms)
    for (int i = 0; i < 4; i++) {
        glActiveTexture(GL_TEXTURE0 + i);
        glBindTexture(GL_TEXTURE_2D, 0);
    }
    glActiveTexture(GL_TEXTURE0);
    
    // Start shader and set uniforms
    block1.getShader().begin();
    block1.process();
    
    // Draw feedback texture - this binds it to texture unit 0
    // The shader samples from tex0 uniform which is also bound to unit 0
    fb1Tex.draw(0, 0, block1.getOutput().getWidth(), block1.getOutput().getHeight());
    
    block1.getShader().end();
    block1.getOutput().end();
    
    // Store frame for feedback
    fb1Delay.pushFrame(block1.getOutput());
    
    // ===== BLOCK 2 =====
    ofTexture& fb2Tex = fb2Delay.getFrame(fb2DelayTime);
    ofTexture& fb2Temporal = fb2Delay.getFrame(0);
    
    block2.setBlock1Texture(block1.getOutputTexture());
    block2.setFeedbackTexture(fb2Tex);
    block2.setTemporalFilterTexture(fb2Temporal);
    
    // Set input texture based on block2InputSelect
    if (block2.params.block2InputSelect == 0) {
        block2.setInputTexture(block1.getOutputTexture());
    } else if (block2.params.block2InputSelect == 1 && input1Tex) {
        block2.setInputTexture(*input1Tex);
    } else if (block2.params.block2InputSelect == 2 && input2Tex) {
        block2.setInputTexture(*input2Tex);
    } else {
        block2.setInputTexture(dummyTexture);
    }
    
    block2.getOutput().begin();
    ofViewport(0, 0, block2.getOutput().getWidth(), block2.getOutput().getHeight());
    ofSetupScreenOrtho(block2.getOutput().getWidth(), block2.getOutput().getHeight());
    ofClear(0, 0, 0, 255);
    
    // Unbind only texture units used by Block2 (4-6) to prevent FBO self-binding issues
    for (int i = 4; i < 7; i++) {
        glActiveTexture(GL_TEXTURE0 + i);
        glBindTexture(GL_TEXTURE_2D, 0);
    }
    glActiveTexture(GL_TEXTURE0);
    
    // Start shader and set uniforms
    block2.getShader().begin();
    block2.process();
    
    // Draw feedback texture - this binds it to texture unit 0
    // The shader samples from tex0 uniform which is also bound to unit 0
    fb2Tex.draw(0, 0, block2.getOutput().getWidth(), block2.getOutput().getHeight());
    
    block2.getShader().end();
    block2.getOutput().end();
    
    fb2Delay.pushFrame(block2.getOutput());
    
    // ===== BLOCK 3 =====
    block3.setBlock1Texture(block1.getOutputTexture());
    block3.setBlock2Texture(block2.getOutputTexture());
    
    block3.getOutput().begin();
    ofViewport(0, 0, block3.getOutput().getWidth(), block3.getOutput().getHeight());
    ofSetupScreenOrtho(block3.getOutput().getWidth(), block3.getOutput().getHeight());
    ofClear(0, 0, 0, 255);
    
    // Unbind only texture units used by Block3 (0-1) to prevent FBO self-binding issues
    for (int i = 0; i < 2; i++) {
        glActiveTexture(GL_TEXTURE0 + i);
        glBindTexture(GL_TEXTURE_2D, 0);
    }
    glActiveTexture(GL_TEXTURE0);
    
    // Start shader and set uniforms
    block3.getShader().begin();
    block3.process();
    
    // Debug: verify params value right before drawing
    static int processDebugCounter = 0;
    if (processDebugCounter++ % 60 == 0) {
        ofLogNotice("processFrame") << "Before draw: block1XDisplace=" << block3.params.block1XDisplace;
    }
    
    // Draw cached mesh (avoid recreation every frame)
    block3Mesh.draw();
    
    block3.getShader().end();
    block3.getOutput().end();
}

void PipelineManager::setInput1Texture(ofTexture& tex) {
    input1Tex = &tex;
}

void PipelineManager::setInput2Texture(ofTexture& tex) {
    input2Tex = &tex;
}

ofTexture& PipelineManager::getBlock1Output() {
    return block1.getOutputTexture();
}

ofTexture& PipelineManager::getBlock2Output() {
    return block2.getOutputTexture();
}

ofTexture& PipelineManager::getFinalOutput() {
    return block3.getOutputTexture();
}

ofFbo& PipelineManager::getBlock1Fbo() {
    return block1.getOutput();
}

ofFbo& PipelineManager::getBlock2Fbo() {
    return block2.getOutput();
}

ofFbo& PipelineManager::getBlock3Fbo() {
    return block3.getOutput();
}

void PipelineManager::reinitialize(const DisplaySettings& settings) {
    displaySettings = settings;
    
    block1.resize(settings.internalWidth, settings.internalHeight);
    block2.resize(settings.internalWidth, settings.internalHeight);
    block3.resize(settings.outputWidth, settings.outputHeight);
    
    fb1Delay.resize(settings.internalWidth, settings.internalHeight);
    fb2Delay.resize(settings.internalWidth, settings.internalHeight);
    
    // Update cached mesh for new dimensions
    updateBlock3Mesh(settings.outputWidth, settings.outputHeight);
    
    allocateDummyTexture();
    
    ofLogNotice("PipelineManager") << "Reinitialized with new resolution";
}

void PipelineManager::clearFB1() {
    fb1Delay.clear();
}

void PipelineManager::clearFB2() {
    fb2Delay.clear();
}

void PipelineManager::clearAll() {
    clearFB1();
    clearFB2();
    block1.clear();
    block2.clear();
    block3.clear();
}

void PipelineManager::setFB1DelayTime(int frames) {
    fb1DelayTime = ofClamp(frames, 1, DelayBuffer::MAX_FRAMES - 1);
}

void PipelineManager::setFB2DelayTime(int frames) {
    fb2DelayTime = ofClamp(frames, 1, DelayBuffer::MAX_FRAMES - 1);
}

void PipelineManager::updateModulations(float deltaTime) {
    if (!audioAnalyzer && !tempoManager) return;
    
    // Update audio analyzer
    if (audioAnalyzer) {
        audioAnalyzer->update();
    }
    
    // Update tempo manager
    if (tempoManager) {
        tempoManager->update(deltaTime);
    }
    
    // Create dummy objects if not available (modulation will be 0)
    static AudioAnalyzer dummyAudio;
    static TempoManager dummyTempo;
    const AudioAnalyzer& audio = audioAnalyzer ? *audioAnalyzer : dummyAudio;
    const TempoManager& tempo = tempoManager ? *tempoManager : dummyTempo;
    
    // Apply modulations to Block1 parameters
    block1.applyModulations(audio, tempo, deltaTime);
    
    // Apply modulations to Block2 parameters
    block2.applyModulations(audio, tempo, deltaTime);
    
    // Apply modulations to Block3 parameters
    block3.applyModulations(audio, tempo, deltaTime);
}

float PipelineManager::getModulatedValue(int blockNum, const std::string& paramName) const {
    switch (blockNum) {
        case 1: return block1.getModulatedValue(paramName);
        case 2: return block2.getModulatedValue(paramName);
        case 3: default: return block3.getModulatedValue(paramName);
    }
}

} // namespace dragonwaves
