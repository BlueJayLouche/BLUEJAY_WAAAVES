#pragma once

#include "ofMain.h"
#include "ofThread.h"
#include <queue>
#include <mutex>
#include <condition_variable>
#include <atomic>

namespace dragonwaves {

//==============================================================================
// Video Recorder Settings
//==============================================================================
struct VideoRecorderSettings {
    int fps = 30;
    int quality = 23;  // CRF value (0-51, lower = better, 23 is default)
    std::string codec = "hevc";  // "hevc", "h264", "prores"
    std::string outputFolder = "recorded";
    bool useHardwareEncoding = true;
};

//==============================================================================
// Frame buffer for async capture
//==============================================================================
struct RecordFrame {
    ofPixels pixels;
    int64_t timestamp;
    
    RecordFrame() : timestamp(0) {}
    RecordFrame(int w, int h, int channels) : timestamp(0) {
        pixels.allocate(w, h, channels);
    }
};

//==============================================================================
// Async Video Recorder using PBO and FFmpeg
//==============================================================================
class VideoRecorder : public ofThread {
public:
    VideoRecorder();
    ~VideoRecorder();
    
    // Setup with display settings
    void setup(int width, int height, const VideoRecorderSettings& settings = VideoRecorderSettings());
    
    // Start/stop recording
    bool startRecording(const std::string& filename = "");  // Auto-generates filename if empty
    void stopRecording();
    bool isRecording() const { return isRecording_.load(); }
    
    // Capture frame (call from main thread, non-blocking)
    void captureFrame(ofFbo& source);
    
    // Settings
    void setSettings(const VideoRecorderSettings& settings);
    const VideoRecorderSettings& getSettings() const { return settings_; }
    
    // Status
    float getRecordedSeconds() const;
    int getDroppedFrames() const { return droppedFrames_.load(); }
    int getQueuedFrames() const;
    
    // Generate unique filename
    static std::string generateFilename(const std::string& folder = "recorded");
    
private:
    // Background encoding thread
    void threadedFunction() override;
    
    // PBO setup for async readback
    void setupPBOs();
    void readbackPBO(ofFbo& source);
    
    // FFmpeg process
    bool startFFmpeg(const std::string& filename);
    void stopFFmpeg();
    bool writeFrameToFFmpeg(const ofPixels& pixels);
    
    // Settings
    VideoRecorderSettings settings_;
    int width_ = 0;
    int height_ = 0;
    
    // PBOs for async GPU readback (triple buffering)
    static constexpr int NUM_PBOS = 3;
    GLuint pboIds_[NUM_PBOS] = {0};
    int pboIndex_ = 0;
    bool pbosInitialized_ = false;
    
    // Frame queue (main thread writes, encoder thread reads)
    std::queue<RecordFrame> frameQueue_;
    mutable std::mutex queueMutex_;
    std::condition_variable queueCondition_;
    static constexpr int MAX_QUEUE_SIZE = 10;  // Drop frames if encoder can't keep up
    
    // FFmpeg pipe
    FILE* ffmpegPipe_ = nullptr;
    
    // State
    std::atomic<bool> isRecording_{false};
    std::atomic<bool> shouldStop_{false};
    std::atomic<int64_t> startTime_{0};
    std::atomic<int64_t> frameCount_{0};
    std::atomic<int> droppedFrames_{0};
    
    // Current filename
    std::string currentFilename_;
};

} // namespace dragonwaves

// Backwards compatibility
typedef dragonwaves::VideoRecorder VideoRecorder;
typedef dragonwaves::VideoRecorderSettings VideoRecorderSettings;
