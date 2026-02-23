#include "VideoRecorder.h"
#include "ofUtils.h"

#if !defined(TARGET_WIN32)
    #include <fcntl.h>
#endif

namespace dragonwaves {

//==============================================================================
VideoRecorder::VideoRecorder() {
}

//==============================================================================
VideoRecorder::~VideoRecorder() {
    stopRecording();
    
    // Clean up PBOs
    if (pbosInitialized_) {
        glDeleteBuffers(NUM_PBOS, pboIds_);
    }
}

//==============================================================================
void VideoRecorder::setup(int width, int height, const VideoRecorderSettings& settings) {
    width_ = width;
    height_ = height;
    settings_ = settings;
    
    // Ensure output directory exists
    ofDirectory dir(settings_.outputFolder);
    if (!dir.exists()) {
        dir.create(true);
    }
    
    // Setup PBOs for async readback
    setupPBOs();
    
    ofLogNotice("VideoRecorder") << "Setup: " << width_ << "x" << height_ 
                                 << " @ " << settings_.fps << "fps"
                                 << " codec: " << settings_.codec;
}

//==============================================================================
void VideoRecorder::setupPBOs() {
    if (pbosInitialized_ || width_ == 0 || height_ == 0) return;
    
    glGenBuffers(NUM_PBOS, pboIds_);
    
    int dataSize = width_ * height_ * 4;  // RGBA
    
    for (int i = 0; i < NUM_PBOS; i++) {
        glBindBuffer(GL_PIXEL_PACK_BUFFER, pboIds_[i]);
        glBufferData(GL_PIXEL_PACK_BUFFER, dataSize, nullptr, GL_STREAM_READ);
    }
    
    glBindBuffer(GL_PIXEL_PACK_BUFFER, 0);
    pbosInitialized_ = true;
    
    ofLogNotice("VideoRecorder") << "PBOs initialized";
}

//==============================================================================
void VideoRecorder::setSettings(const VideoRecorderSettings& settings) {
    settings_ = settings;
}

//==============================================================================
bool VideoRecorder::startRecording(const std::string& filename) {
    if (isRecording_.load()) {
        ofLogWarning("VideoRecorder") << "Already recording";
        return false;
    }
    
    if (width_ == 0 || height_ == 0) {
        ofLogError("VideoRecorder") << "Not setup";
        return false;
    }
    
    // Generate filename if not provided
    if (filename.empty()) {
        currentFilename_ = generateFilename(settings_.outputFolder);
    } else {
        currentFilename_ = filename;
    }
    
    // Reset counters
    droppedFrames_ = 0;
    frameCount_ = 0;
    startTime_ = ofGetElapsedTimeMillis();
    shouldStop_ = false;
    
    // Start FFmpeg
    if (!startFFmpeg(currentFilename_)) {
        ofLogError("VideoRecorder") << "Failed to start FFmpeg";
        return false;
    }
    
    // Start encoding thread
    isRecording_ = true;
    startThread();
    
    ofLogNotice("VideoRecorder") << "Started recording: " << currentFilename_;
    return true;
}

//==============================================================================
void VideoRecorder::stopRecording() {
    if (!isRecording_.load()) return;
    
    ofLogNotice("VideoRecorder") << "Stopping recording...";
    
    // Signal stop
    shouldStop_ = true;
    isRecording_ = false;
    
    // Wake up encoder thread
    queueCondition_.notify_all();
    
    // Wait for thread to finish
    waitForThread(true);
    
    // Stop FFmpeg
    stopFFmpeg();
    
    // Clear queue
    std::lock_guard<std::mutex> lock(queueMutex_);
    while (!frameQueue_.empty()) {
        frameQueue_.pop();
    }
    
    float duration = getRecordedSeconds();
    ofLogNotice("VideoRecorder") << "Stopped. Recorded " << frameCount_ 
                                 << " frames (" << duration << "s)"
                                 << " Dropped: " << droppedFrames_;
}

//==============================================================================
void VideoRecorder::captureFrame(ofFbo& source) {
    if (!isRecording_.load() || !pbosInitialized_) return;
    
    // Async PBO readback
    readbackPBO(source);
}

//==============================================================================
void VideoRecorder::readbackPBO(ofFbo& source) {
    // Use next PBO
    int nextPboIndex = (pboIndex_ + 1) % NUM_PBOS;
    int readPboIndex = (pboIndex_ + 2) % NUM_PBOS;
    
    // Read from oldest PBO (3 frames ago) - ensures GPU is done
    glBindBuffer(GL_PIXEL_PACK_BUFFER, pboIds_[readPboIndex]);
    
    GLubyte* ptr = (GLubyte*)glMapBuffer(GL_PIXEL_PACK_BUFFER, GL_READ_ONLY);
    if (ptr) {
        // Try to add to queue (non-blocking)
        RecordFrame frame;
        frame.pixels.allocate(width_, height_, 4);  // RGBA
        frame.pixels.setFromPixels(ptr, width_, height_, 4);
        frame.timestamp = ofGetElapsedTimeMillis();
        
        {
            std::unique_lock<std::mutex> lock(queueMutex_);
            
            if (frameQueue_.size() < MAX_QUEUE_SIZE) {
                frameQueue_.push(std::move(frame));
                queueCondition_.notify_one();
            } else {
                droppedFrames_++;
            }
        }
        
        glUnmapBuffer(GL_PIXEL_PACK_BUFFER);
    }
    
    glBindBuffer(GL_PIXEL_PACK_BUFFER, pboIds_[nextPboIndex]);
    
    // Initiate async read from FBO to PBO
    source.bind();
    glGetTexImage(GL_TEXTURE_2D, 0, GL_RGBA, GL_UNSIGNED_BYTE, 0);
    source.unbind();
    
    glBindBuffer(GL_PIXEL_PACK_BUFFER, 0);
    
    pboIndex_ = nextPboIndex;
}

//==============================================================================
void VideoRecorder::threadedFunction() {
    while (isThreadRunning()) {
        RecordFrame frame;
        
        // Wait for frame or stop signal
        {
            std::unique_lock<std::mutex> lock(queueMutex_);
            queueCondition_.wait(lock, [this] { 
                return !frameQueue_.empty() || shouldStop_.load(); 
            });
            
            if (shouldStop_.load() && frameQueue_.empty()) {
                break;
            }
            
            if (!frameQueue_.empty()) {
                frame = std::move(frameQueue_.front());
                frameQueue_.pop();
            }
        }
        
        // Write to FFmpeg
        if (frame.pixels.isAllocated()) {
            writeFrameToFFmpeg(frame.pixels);
            frameCount_++;
        }
    }
}

//==============================================================================
bool VideoRecorder::startFFmpeg(const std::string& filename) {
    // Build FFmpeg command
    std::stringstream cmd;
    cmd << "ffmpeg -y ";  // Overwrite output
    
    // Input format
    cmd << "-f rawvideo -pix_fmt rgba ";
    cmd << "-s " << width_ << "x" << height_ << " ";
    cmd << "-r " << settings_.fps << " ";
    cmd << "-i - ";  // Read from stdin
    
    // Codec selection - optimized for real-time performance
    if (settings_.codec == "hevc") {
        #if defined(TARGET_OSX)
            if (settings_.useHardwareEncoding) {
                cmd << "-c:v hevc_videotoolbox ";
                cmd << "-allow_sw 1 ";
                cmd << "-b:v " << (settings_.quality < 20 ? "12M" : "6M") << " ";
                cmd << "-realtime 1 ";  // Prioritize speed
            } else {
                cmd << "-c:v libx265 -crf " << settings_.quality << " ";
                cmd << "-preset ultrafast ";  // Fastest preset
                cmd << "-tune fastdecode ";
                cmd << "-threads 4 ";  // Limit threads to avoid starving app
            }
        #else
            cmd << "-c:v libx265 -crf " << settings_.quality << " ";
            cmd << "-preset ultrafast ";
            cmd << "-tune fastdecode ";
            cmd << "-threads 4 ";
        #endif
        cmd << "-tag:v hvc1 ";
    } 
    else if (settings_.codec == "h264") {
        #if defined(TARGET_OSX)
            if (settings_.useHardwareEncoding) {
                cmd << "-c:v h264_videotoolbox ";
                cmd << "-allow_sw 1 ";
                cmd << "-b:v " << (settings_.quality < 20 ? "12M" : "6M") << " ";
                cmd << "-realtime 1 ";
            } else {
                cmd << "-c:v libx264 -crf " << settings_.quality << " ";
                cmd << "-preset ultrafast ";
                cmd << "-tune fastdecode ";
                cmd << "-threads 4 ";
            }
        #elif defined(TARGET_WIN32)
            if (settings_.useHardwareEncoding) {
                cmd << "-c:v h264_nvenc ";
                cmd << "-rc vbr ";
                cmd << "-cq " << settings_.quality << " ";
                cmd << "-preset p1 ";  // Fastest NVENC preset
            } else {
                cmd << "-c:v libx264 -crf " << settings_.quality << " ";
                cmd << "-preset ultrafast ";
                cmd << "-tune fastdecode ";
                cmd << "-threads 4 ";
            }
        #else
            cmd << "-c:v libx264 -crf " << settings_.quality << " ";
            cmd << "-preset ultrafast ";
            cmd << "-tune fastdecode ";
            cmd << "-threads 4 ";
        #endif
    }
    else if (settings_.codec == "prores") {
        cmd << "-c:v prores_ks -profile:v 3 ";  // ProRes 422
    }
    
    // Pixel format and output
    cmd << "-pix_fmt yuv420p ";  // For compatibility
    cmd << "-movflags +faststart ";  // Web optimization
    cmd << "\"" << ofToDataPath(filename, true) << "\" ";
    
    // Redirect stderr
    cmd << "2>&1";
    
    ofLogNotice("VideoRecorder") << "FFmpeg: " << cmd.str();
    
    // Open pipe - use unbuffered mode for lower latency
    #if defined(TARGET_WIN32)
        ffmpegPipe_ = _popen(cmd.str().c_str(), "wb");
    #else
        // Use "w" for text mode - FFmpeg handles binary via the protocol
        ffmpegPipe_ = popen(cmd.str().c_str(), "w");
    #endif
    
    if (!ffmpegPipe_) {
        ofLogError("VideoRecorder") << "Failed to open FFmpeg pipe";
        return false;
    }
    
    // Small delay to let FFmpeg initialize
    ofSleepMillis(100);
    
    // Check if FFmpeg is still running by testing the file descriptor
    // (This is a simple check - more robust would be checking process status)
    #if !defined(TARGET_WIN32)
        int fd = fileno(ffmpegPipe_);
        if (fcntl(fd, F_GETFD) == -1) {
            ofLogError("VideoRecorder") << "FFmpeg pipe closed immediately (encoder may have failed)";
            ffmpegPipe_ = nullptr;
            return false;
        }
    #endif
    
    return true;
}

//==============================================================================
void VideoRecorder::stopFFmpeg() {
    if (ffmpegPipe_) {
        #if defined(TARGET_WIN32)
            _pclose(ffmpegPipe_);
        #else
            pclose(ffmpegPipe_);
        #endif
        ffmpegPipe_ = nullptr;
    }
}

//==============================================================================
bool VideoRecorder::writeFrameToFFmpeg(const ofPixels& pixels) {
    if (!ffmpegPipe_) return false;
    
    size_t written = fwrite(pixels.getData(), 1, pixels.size(), ffmpegPipe_);
    return written == pixels.size();
}

//==============================================================================
float VideoRecorder::getRecordedSeconds() const {
    return (ofGetElapsedTimeMillis() - startTime_.load()) / 1000.0f;
}

//==============================================================================
int VideoRecorder::getQueuedFrames() const {
    std::lock_guard<std::mutex> lock(queueMutex_);
    return frameQueue_.size();
}

//==============================================================================
std::string VideoRecorder::generateFilename(const std::string& folder) {
    // Create folder if needed
    ofDirectory dir(folder);
    if (!dir.exists()) {
        dir.create(true);
    }
    
    // Generate timestamp: YYYYMMDD_HHMMSS
    std::string timestamp = ofGetTimestampString("%Y%m%d_%H%M%S");
    
    // Extension based on codec
    std::string ext = "mp4";
    
    return folder + "/recording_" + timestamp + "." + ext;
}

} // namespace dragonwaves
