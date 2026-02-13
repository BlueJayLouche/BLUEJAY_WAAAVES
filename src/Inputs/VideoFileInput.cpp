#include "VideoFileInput.h"

namespace dragonwaves {

VideoFileInput::VideoFileInput() {
}

VideoFileInput::~VideoFileInput() {
    close();
}

bool VideoFileInput::setup(int width, int height) {
    nativeWidth = width;
    nativeHeight = height;
    
    // Video player will be initialized when load() is called
    initialized = false;
    
    return true;
}

void VideoFileInput::update() {
    if (!initialized) return;
    
    player.update();
}

void VideoFileInput::close() {
    player.stop();
    player.close();
    initialized = false;
    filePath = "";
}

ofTexture& VideoFileInput::getTexture() {
    return player.getTexture();
}

bool VideoFileInput::isFrameNew() const {
    return initialized && player.isFrameNew();
}

bool VideoFileInput::isInitialized() const {
    return initialized;
}

std::string VideoFileInput::getName() const {
    if (!filePath.empty()) {
        // Extract filename from path
        size_t lastSlash = filePath.find_last_of("/\\");
        if (lastSlash != std::string::npos) {
            return "Video: " + filePath.substr(lastSlash + 1);
        }
        return "Video: " + filePath;
    }
    return "Video: (No File)";
}

bool VideoFileInput::load(const std::string& path) {
    filePath = path;
    
    bool loaded = player.load(path);
    if (loaded) {
        initialized = true;
        nativeWidth = player.getWidth();
        nativeHeight = player.getHeight();
        
        player.setLoopState(looping ? OF_LOOP_NORMAL : OF_LOOP_NONE);
        player.setSpeed(speed);
        
        ofLogNotice("VideoFileInput") << "Loaded: " << path 
                                       << " (" << nativeWidth << "x" << nativeHeight << ")";
    } else {
        ofLogError("VideoFileInput") << "Failed to load: " << path;
    }
    
    return loaded;
}

void VideoFileInput::play() {
    if (initialized) {
        player.play();
    }
}

void VideoFileInput::pause() {
    if (initialized) {
        player.setPaused(true);
    }
}

void VideoFileInput::stop() {
    if (initialized) {
        player.stop();
    }
}

void VideoFileInput::setLoop(bool loop) {
    looping = loop;
    if (initialized) {
        player.setLoopState(loop ? OF_LOOP_NORMAL : OF_LOOP_NONE);
    }
}

void VideoFileInput::setSpeed(float s) {
    speed = s;
    if (initialized) {
        player.setSpeed(s);
    }
}

void VideoFileInput::setPosition(float position) {
    if (initialized) {
        player.setPosition(position);
    }
}

float VideoFileInput::getPosition() const {
    if (initialized) {
        return player.getPosition();
    }
    return 0.0f;
}

float VideoFileInput::getDuration() const {
    if (initialized) {
        return player.getDuration();
    }
    return 0.0f;
}

bool VideoFileInput::isPlaying() const {
    return initialized && player.isPlaying();
}

} // namespace dragonwaves
