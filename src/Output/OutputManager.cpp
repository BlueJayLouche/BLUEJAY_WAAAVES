#include "OutputManager.h"

namespace dragonwaves {

//==============================================================================
// AsyncPixelTransfer
//==============================================================================
void AsyncPixelTransfer::setup(int w, int h) {
    width = w;
    height = h;
    
    pixels.allocate(width, height, OF_PIXELS_RGBA);
    
    size_t bufferSize = width * height * 4;  // RGBA
    
    glGenBuffers(2, pbo);
    glBindBuffer(GL_PIXEL_PACK_BUFFER, pbo[0]);
    glBufferData(GL_PIXEL_PACK_BUFFER, bufferSize, NULL, GL_STREAM_READ);
    glBindBuffer(GL_PIXEL_PACK_BUFFER, pbo[1]);
    glBufferData(GL_PIXEL_PACK_BUFFER, bufferSize, NULL, GL_STREAM_READ);
    glBindBuffer(GL_PIXEL_PACK_BUFFER, 0);
    
    pboIndex = 0;
    pboNextIndex = 1;
    frameCount = 0;
    initialized = true;
    
    ofLogNotice("AsyncPixelTransfer") << "Setup " << w << "x" << h;
}

void AsyncPixelTransfer::cleanup() {
    if (!initialized) return;
    
    glDeleteBuffers(2, pbo);
    pbo[0] = 0;
    pbo[1] = 0;
    initialized = false;
}

void AsyncPixelTransfer::resize(int w, int h) {
    cleanup();
    setup(w, h);
}

void AsyncPixelTransfer::beginTransfer(ofFbo& sourceFbo) {
    if (!initialized) return;
    
    // Bind PBO and read pixels
    sourceFbo.bind();
    glBindBuffer(GL_PIXEL_PACK_BUFFER, pbo[pboIndex]);
    glReadPixels(0, 0, width, height, GL_RGBA, GL_UNSIGNED_BYTE, 0);
    glBindBuffer(GL_PIXEL_PACK_BUFFER, 0);
    sourceFbo.unbind();
}

ofPixels& AsyncPixelTransfer::endTransfer() {
    if (!initialized) {
        return pixels;
    }
    
    // Read previous frame data
    if (frameCount > 0) {
        glBindBuffer(GL_PIXEL_PACK_BUFFER, pbo[pboNextIndex]);
        GLubyte* ptr = (GLubyte*)glMapBuffer(GL_PIXEL_PACK_BUFFER, GL_READ_ONLY);
        if (ptr) {
            pixels.setFromPixels(ptr, width, height, OF_PIXELS_RGBA);
            glUnmapBuffer(GL_PIXEL_PACK_BUFFER);
        }
        glBindBuffer(GL_PIXEL_PACK_BUFFER, 0);
    }
    
    // Swap indices
    std::swap(pboIndex, pboNextIndex);
    frameCount++;
    
    return pixels;
}

//==============================================================================
// OutputSender
//==============================================================================
OutputSender::OutputSender(const std::string& n)
    : name(n) {
}

//==============================================================================
// NdiOutputSender
//==============================================================================
NdiOutputSender::NdiOutputSender(const std::string& name)
    : OutputSender(name) {
}

NdiOutputSender::~NdiOutputSender() {
    // Explicit cleanup without locking - mutex may be in bad state during destruction
    try {
        if (active) {
            sender.ReleaseSender();
            active = false;
        }
        pboTransfer.cleanup();
    } catch (...) {
        // Ignore exceptions during destruction
    }
}

void NdiOutputSender::setup(int w, int h) {
    width = w;
    height = h;
    
    // Allocate scale FBO if needed
    scaleFbo.allocate(width, height, GL_RGBA);
    scaleFbo.begin();
    ofClear(0, 0, 0, 255);
    scaleFbo.end();
    
    // Setup PBO transfer
    pboTransfer.setup(width, height);
    
    ofLogNotice("NdiOutputSender") << name << " setup " << w << "x" << h;
}

void NdiOutputSender::send(ofTexture& texture) {
    if (!enabled || width == 0 || height == 0) return;
    
    std::lock_guard<std::mutex> lock(mtx);
    
    // Create sender if needed (only once)
    if (!active) {
        if (!sender.CreateSender(name.c_str(), width, height)) {
            ofLogError("NdiOutputSender") << "Failed to create sender: " << name;
            return;
        }
        active = true;
        ofLogNotice("NdiOutputSender") << "Created sender: " << name;
    }
    
    // Copy texture to scaleFbo at output resolution
    scaleFbo.begin();
    ofViewport(0, 0, width, height);
    ofSetupScreenOrtho(width, height);
    ofClear(0, 0, 0, 255);
    texture.draw(0, 0, width, height);
    scaleFbo.end();
    
    // Use async PBO transfer for non-blocking readback
    // This reads pixels from the PREVIOUS frame while rendering current frame
    pboTransfer.beginTransfer(scaleFbo);
    ofPixels& pixels = pboTransfer.endTransfer();
    
    if (pixels.isAllocated()) {
        sender.SendImage(pixels, false, false);
    }
}

void NdiOutputSender::close() {
    std::lock_guard<std::mutex> lock(mtx);
    if (active) {
        sender.ReleaseSender();
        active = false;
    }
    pboTransfer.cleanup();
}

void NdiOutputSender::setEnabled(bool e) {
    std::lock_guard<std::mutex> lock(mtx);
    enabled = e;
    if (!enabled && active) {
        sender.ReleaseSender();
        active = false;
    }
}

//==============================================================================
// SpoutOutputSender
//==============================================================================
SpoutOutputSender::SpoutOutputSender(const std::string& name)
    : OutputSender(name) {
}

SpoutOutputSender::~SpoutOutputSender() {
    close();
}

void SpoutOutputSender::setup(int w, int h) {
#if SPOUT_AVAILABLE
    width = w;
    height = h;
    
    flipFbo.allocate(width, height, GL_RGBA);
    flipFbo.begin();
    ofClear(0, 0, 0, 255);
    flipFbo.end();
    
    ofLogNotice("SpoutOutputSender") << name << " setup " << w << "x" << h;
#endif
}

void SpoutOutputSender::send(ofTexture& texture) {
#if SPOUT_AVAILABLE
    if (!enabled) return;
    
    // Initialize sender if needed
    if (!sender.isInitialized()) {
        sender.init(name, width, height, GL_RGBA);
        ofLogNotice("SpoutOutputSender") << "Initialized sender: " << name;
    }
    
    // Flip vertically (Spout expects flipped)
    flipFbo.begin();
    ofViewport(0, 0, width, height);
    ofSetupScreenOrtho(width, height);
    ofClear(0, 0, 0, 255);
    texture.draw(0, height, width, -height);
    flipFbo.end();
    
    // Send
    sender.send(flipFbo.getTexture());
#endif
}

void SpoutOutputSender::close() {
#if SPOUT_AVAILABLE
    sender.release();
#endif
}

void SpoutOutputSender::setEnabled(bool e) {
    enabled = e;
    if (!enabled) {
        close();
    }
}

//==============================================================================
// OutputManager
//==============================================================================
OutputManager::OutputManager() {
}

OutputManager::~OutputManager() {
    close();
}

void OutputManager::setup(const DisplaySettings& settings) {
    displaySettings = settings;
    
    // Create NDI senders
    ndiBlock1 = std::make_unique<NdiOutputSender>("GwBlock1");
    ndiBlock2 = std::make_unique<NdiOutputSender>("GwBlock2");
    ndiBlock3 = std::make_unique<NdiOutputSender>("GwBlock3");
    
    ndiBlock1->setup(settings.ndiSendWidth, settings.ndiSendHeight);
    ndiBlock2->setup(settings.ndiSendWidth, settings.ndiSendHeight);
    ndiBlock3->setup(settings.ndiSendWidth, settings.ndiSendHeight);
    
    // Create Spout senders (Windows only)
#if SPOUT_AVAILABLE
    spoutBlock1 = std::make_unique<SpoutOutputSender>("GwBlock1");
    spoutBlock2 = std::make_unique<SpoutOutputSender>("GwBlock2");
    spoutBlock3 = std::make_unique<SpoutOutputSender>("GwBlock3");
    
    spoutBlock1->setup(settings.spoutSendWidth, settings.spoutSendHeight);
    spoutBlock2->setup(settings.spoutSendWidth, settings.spoutSendHeight);
    spoutBlock3->setup(settings.spoutSendWidth, settings.spoutSendHeight);
#endif
    
    initialized = true;
    
    ofLogNotice("OutputManager") << "Setup complete";
}

void OutputManager::sendBlock1(ofTexture& texture) {
    if (!initialized) return;
    
    if (ndiBlock1 && ndiBlock1->isEnabled()) {
        ndiBlock1->send(texture);
    }
#if SPOUT_AVAILABLE
    if (spoutBlock1 && spoutBlock1->isEnabled()) {
        spoutBlock1->send(texture);
    }
#endif
}

void OutputManager::sendBlock2(ofTexture& texture) {
    if (!initialized) return;
    
    if (ndiBlock2 && ndiBlock2->isEnabled()) {
        ndiBlock2->send(texture);
    }
#if SPOUT_AVAILABLE
    if (spoutBlock2 && spoutBlock2->isEnabled()) {
        spoutBlock2->send(texture);
    }
#endif
}

void OutputManager::sendBlock3(ofTexture& texture) {
    if (!initialized) return;
    
    if (ndiBlock3 && ndiBlock3->isEnabled()) {
        ndiBlock3->send(texture);
    }
#if SPOUT_AVAILABLE
    if (spoutBlock3 && spoutBlock3->isEnabled()) {
        spoutBlock3->send(texture);
    }
#endif
}

void OutputManager::setNdiBlock1Enabled(bool enabled) {
    if (ndiBlock1) ndiBlock1->setEnabled(enabled);
}

void OutputManager::setNdiBlock2Enabled(bool enabled) {
    if (ndiBlock2) ndiBlock2->setEnabled(enabled);
}

void OutputManager::setNdiBlock3Enabled(bool enabled) {
    if (ndiBlock3) ndiBlock3->setEnabled(enabled);
}

void OutputManager::setSpoutBlock1Enabled(bool enabled) {
#if SPOUT_AVAILABLE
    if (spoutBlock1) spoutBlock1->setEnabled(enabled);
#endif
}

void OutputManager::setSpoutBlock2Enabled(bool enabled) {
#if SPOUT_AVAILABLE
    if (spoutBlock2) spoutBlock2->setEnabled(enabled);
#endif
}

void OutputManager::setSpoutBlock3Enabled(bool enabled) {
#if SPOUT_AVAILABLE
    if (spoutBlock3) spoutBlock3->setEnabled(enabled);
#endif
}

bool OutputManager::isNdiBlock1Enabled() const {
    return ndiBlock1 && ndiBlock1->isEnabled();
}

bool OutputManager::isNdiBlock2Enabled() const {
    return ndiBlock2 && ndiBlock2->isEnabled();
}

bool OutputManager::isNdiBlock3Enabled() const {
    return ndiBlock3 && ndiBlock3->isEnabled();
}

bool OutputManager::isSpoutBlock1Enabled() const {
#if SPOUT_AVAILABLE
    return spoutBlock1 && spoutBlock1->isEnabled();
#else
    return false;
#endif
}

bool OutputManager::isSpoutBlock2Enabled() const {
#if SPOUT_AVAILABLE
    return spoutBlock2 && spoutBlock2->isEnabled();
#else
    return false;
#endif
}

bool OutputManager::isSpoutBlock3Enabled() const {
#if SPOUT_AVAILABLE
    return spoutBlock3 && spoutBlock3->isEnabled();
#else
    return false;
#endif
}

void OutputManager::reinitialize(const DisplaySettings& settings) {
    displaySettings = settings;
    
    if (ndiBlock1) ndiBlock1->setup(settings.ndiSendWidth, settings.ndiSendHeight);
    if (ndiBlock2) ndiBlock2->setup(settings.ndiSendWidth, settings.ndiSendHeight);
    if (ndiBlock3) ndiBlock3->setup(settings.ndiSendWidth, settings.ndiSendHeight);
    
#if SPOUT_AVAILABLE
    if (spoutBlock1) spoutBlock1->setup(settings.spoutSendWidth, settings.spoutSendHeight);
    if (spoutBlock2) spoutBlock2->setup(settings.spoutSendWidth, settings.spoutSendHeight);
    if (spoutBlock3) spoutBlock3->setup(settings.spoutSendWidth, settings.spoutSendHeight);
#endif
    
    ofLogNotice("OutputManager") << "Reinitialized";
}

void OutputManager::close() {
    if (ndiBlock1) ndiBlock1->close();
    if (ndiBlock2) ndiBlock2->close();
    if (ndiBlock3) ndiBlock3->close();
    
#if SPOUT_AVAILABLE
    if (spoutBlock1) spoutBlock1->close();
    if (spoutBlock2) spoutBlock2->close();
    if (spoutBlock3) spoutBlock3->close();
#endif
    
    initialized = false;
}

} // namespace dragonwaves
