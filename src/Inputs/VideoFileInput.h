#pragma once

#include "InputSource.h"

namespace dragonwaves {

//==============================================================================
// Video file input source with looping
//==============================================================================
class VideoFileInput : public InputSource {
public:
    VideoFileInput();
    ~VideoFileInput();
    
    bool setup(int width, int height) override;
    void update() override;
    void close() override;
    
    ofTexture& getTexture() override;
    bool isFrameNew() const override;
    bool isInitialized() const override;
    InputType getType() const override { return InputType::VIDEO_FILE; }
    std::string getName() const override;
    
    // Video-specific methods
    bool load(const std::string& path);
    void play();
    void pause();
    void stop();
    void setLoop(bool loop);
    bool isLooping() const { return looping; }
    void setSpeed(float speed);
    float getSpeed() const { return speed; }
    void setPosition(float position);  // 0.0 to 1.0
    float getPosition() const;
    float getDuration() const;
    bool isPlaying() const;
    
    std::string getFilePath() const { return filePath; }
    
private:
    ofVideoPlayer player;
    std::string filePath;
    bool looping = true;
    float speed = 1.0f;
};

} // namespace dragonwaves
