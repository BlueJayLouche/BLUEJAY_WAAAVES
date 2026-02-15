#include "AudioAnalyzer.h"

namespace dragonwaves {

//==============================================================================
// Simple FFT Implementation (Cooley-Tukey radix-2)
//==============================================================================
void SimpleFFT::compute(std::vector<float>& input, std::vector<float>& output) {
    int N = input.size();
    if (N == 0) return;
    
    // Ensure output is correct size
    output.resize(N / 2 + 1);
    
    // Create complex data array
    std::vector<std::complex<float>> data(N);
    for (int i = 0; i < N; i++) {
        data[i] = std::complex<float>(input[i], 0.0f);
    }
    
    // Perform FFT
    fft(data);
    
    // Compute magnitude spectrum (only first N/2+1 values are unique for real input)
    for (int i = 0; i < N / 2 + 1 && i < output.size(); i++) {
        output[i] = std::abs(data[i]);
    }
}

void SimpleFFT::bitReversalPermutation(std::vector<std::complex<float>>& data) {
    int N = data.size();
    int j = 0;
    for (int i = 1; i < N; i++) {
        int bit = N >> 1;
        for (; j & bit; bit >>= 1) {
            j ^= bit;
        }
        j ^= bit;
        if (i < j) {
            std::swap(data[i], data[j]);
        }
    }
}

void SimpleFFT::fft(std::vector<std::complex<float>>& data) {
    int N = data.size();
    if (N <= 1) return;
    
    // Ensure N is power of 2
    if ((N & (N - 1)) != 0) return;
    
    // Bit reversal permutation
    bitReversalPermutation(data);
    
    // FFT computation
    for (int len = 2; len <= N; len <<= 1) {
        float ang = -2.0f * PI / len;
        std::complex<float> wlen(cosf(ang), sinf(ang));
        for (int i = 0; i < N; i += len) {
            std::complex<float> w(1.0f, 0.0f);
            for (int j = 0; j < len / 2; j++) {
                std::complex<float> u = data[i + j];
                std::complex<float> v = data[i + j + len / 2] * w;
                data[i + j] = u + v;
                data[i + j + len / 2] = u - v;
                w *= wlen;
            }
        }
    }
}

//==============================================================================
// AudioModulation
//==============================================================================
float AudioModulation::process(float fftValue, float deltaTime) {
    if (!enabled) {
        currentValue = 0.0f;
        return 0.0f;
    }
    
    // Apply attack/release smoothing
    float target = fftValue * amount * rangeScale;  // Scale by parameter range
    float rate = (target > currentValue) ? attack : release;
    currentValue += (target - currentValue) * rate * deltaTime * 60.0f; // Normalize to 60fps
    
    return currentValue;
}

void AudioModulation::loadFromJson(const ofJson& json) {
    if (json.contains("enabled")) enabled = json["enabled"].get<bool>();
    if (json.contains("fftBand")) fftBand = json["fftBand"].get<int>();
    if (json.contains("amount")) amount = json["amount"].get<float>();
    if (json.contains("useNormalization")) useNormalization = json["useNormalization"].get<bool>();
    if (json.contains("attack")) attack = json["attack"].get<float>();
    if (json.contains("release")) release = json["release"].get<float>();
    if (json.contains("rangeScale")) rangeScale = json["rangeScale"].get<float>();
}

void AudioModulation::saveToJson(ofJson& json) const {
    json["enabled"] = enabled;
    json["fftBand"] = fftBand;
    json["amount"] = amount;
    json["useNormalization"] = useNormalization;
    json["attack"] = attack;
    json["release"] = release;
    json["rangeScale"] = rangeScale;
}

//==============================================================================
// AudioAnalyzer
//==============================================================================
AudioAnalyzer::AudioAnalyzer() {
    // Initialize arrays
    bandValues.fill(0.0f);
    smoothedValues.fill(0.0f);
    peakValues.fill(0.0f);
    minValues.fill(0.0f);
    maxValues.fill(0.01f); // Start with small non-zero value
}

AudioAnalyzer::~AudioAnalyzer() {
    close();
}

void AudioAnalyzer::setup(const AudioSettings& newSettings) {
    settings = newSettings;
    
    if (!settings.enabled) {
        return;
    }
    
    // Rebuild the device ID mapping
    rebuildDeviceIdMap();
    
    // Set number of bins (must be power of 2)
    numBins = settings.numBins > 0 ? settings.numBins : 256;
    // Round to nearest power of 2
    int powerOf2 = 1;
    while (powerOf2 < numBins) powerOf2 <<= 1;
    numBins = powerOf2;
    
    fftBins.resize(numBins / 2 + 1);
    fftInputBuffer.resize(numBins);
    fftWindow.resize(numBins);
    
    // Setup audio buffer (circular buffer for continuous input)
    audioBuffer.resize(settings.fftSize);
    bufferWriteIndex = 0;
    
    // Create Hann window
    for (int i = 0; i < numBins; i++) {
        fftWindow[i] = 0.5f * (1.0f - cosf(2.0f * PI * i / (numBins - 1)));
    }
    
    // Get all available devices for logging
    auto devices = soundStream.getDeviceList();
    
    // Find the actual deviceID from our mapping
    currentDeviceId = -1;
    if (settings.inputDevice >= 0 && settings.inputDevice < (int)inputDeviceIds.size()) {
        currentDeviceId = inputDeviceIds[settings.inputDevice];
    }
    
    ofLogNotice("AudioAnalyzer") << "Setup with " << numBins << " FFT bins (power of 2)";
    ofLogNotice("AudioAnalyzer") << "Selected device index: " << settings.inputDevice 
                                  << " -> deviceID: " << currentDeviceId
                                  << " at " << settings.sampleRate << "Hz";
    
    // List available input devices for debugging
    ofLogNotice("AudioAnalyzer") << "Available input devices (filtered):";
    for (size_t i = 0; i < inputDeviceIds.size(); i++) {
        int devId = inputDeviceIds[i];
        for (const auto& device : devices) {
            if (device.deviceID == devId) {
                ofLogNotice("AudioAnalyzer") << "  [Index " << i << "] deviceID=" << devId << ": " << device.name;
                break;
            }
        }
    }
    
    // Setup sound stream using modern OF API
    ofSoundStreamSettings streamSettings;
    streamSettings.numInputChannels = 1;  // Mono input
    streamSettings.numOutputChannels = 0; // No output
    streamSettings.sampleRate = settings.sampleRate;
    streamSettings.bufferSize = settings.bufferSize;
    streamSettings.numBuffers = 2;
    streamSettings.setInListener(this);
    
    // Try to set the device by actual deviceID
    if (currentDeviceId >= 0) {
        for (const auto& device : devices) {
            if (device.deviceID == currentDeviceId && device.inputChannels > 0) {
                streamSettings.setInDevice(device);
                ofLogNotice("AudioAnalyzer") << "Using device: " << device.name << " (deviceID=" << device.deviceID << ")";
                break;
            }
        }
    } else {
        ofLogWarning("AudioAnalyzer") << "No valid device selected, using default";
    }
    
    streamSetup = soundStream.setup(streamSettings);
    
    if (!streamSetup) {
        ofLogError("AudioAnalyzer") << "Failed to setup sound stream!";
    } else {
        ofLogNotice("AudioAnalyzer") << "Sound stream setup successful";
    }
}

void AudioAnalyzer::close() {
    if (streamSetup) {
        soundStream.close();
        streamSetup = false;
    }
}

void AudioAnalyzer::update() {
    if (!settings.enabled || !streamSetup) {
        return;
    }
    
    // Compute FFT from audio buffer
    computeFFT();
    
    // Compute the 8 band values
    computeBandValues();
    
    // Update smoothing
    float smooth = settings.smoothing;
    for (int i = 0; i < 8; i++) {
        // Exponential smoothing
        smoothedValues[i] += (bandValues[i] - smoothedValues[i]) * (1.0f - smooth);
        
        // Peak hold with decay
        if (smoothedValues[i] > peakValues[i]) {
            peakValues[i] = smoothedValues[i];
        } else {
            peakValues[i] *= settings.peakDecay;
        }
    }
    
    // Update normalization
    if (settings.normalization) {
        updateNormalization();
    }
}

void AudioAnalyzer::computeFFT() {
    std::lock_guard<std::mutex> lock(audioMutex);
    
    // Copy from circular buffer to FFT input buffer
    // Use the most recent numBins samples
    for (int i = 0; i < numBins; i++) {
        int srcIdx = (bufferWriteIndex - numBins + i + audioBuffer.size()) % audioBuffer.size();
        fftInputBuffer[i] = audioBuffer[srcIdx] * fftWindow[i];
    }
    
    // Perform FFT
    SimpleFFT::compute(fftInputBuffer, fftBins);
}

void AudioAnalyzer::computeBandValues() {
    // Map FFT bins to 8 frequency bands using logarithmic scale
    // numBins typically covers 0 to sampleRate/2 (Nyquist frequency)
    float nyquist = settings.sampleRate / 2.0f;
    int fftSize = fftBins.size();
    
    // Band frequency ranges
    const float bandRanges[9] = {20, 60, 120, 250, 500, 2000, 4000, 8000, 16000};
    
    for (int band = 0; band < 8; band++) {
        float lowFreq = bandRanges[band];
        float highFreq = bandRanges[band + 1];
        
        // Convert frequencies to bin indices
        int lowBin = (int)((lowFreq / nyquist) * fftSize);
        int highBin = (int)((highFreq / nyquist) * fftSize);
        
        lowBin = ofClamp(lowBin, 0, fftSize - 1);
        highBin = ofClamp(highBin, lowBin, fftSize - 1);
        
        // Average energy in this band
        float sum = 0.0f;
        int count = 0;
        for (int i = lowBin; i <= highBin && i < fftSize; i++) {
            sum += fftBins[i];
            count++;
        }
        
        float avg = (count > 0) ? sum / count : 0.0f;
        
        // Apply amplitude scaling
        avg *= settings.amplitude;
        
        // Scale for better visibility (FFT output needs scaling)
        avg *= 2.0f / numBins;
        
        bandValues[band] = avg;
    }
}

float AudioAnalyzer::getBand(int bandIndex) const {
    if (bandIndex < 0 || bandIndex >= 8) return 0.0f;
    
    float value = smoothedValues[bandIndex];
    
    if (settings.normalization && maxValues[bandIndex] > minValues[bandIndex]) {
        value = (value - minValues[bandIndex]) / (maxValues[bandIndex] - minValues[bandIndex]);
        value = ofClamp(value, 0.0f, 1.0f);
    }
    
    return value;
}

float AudioAnalyzer::getBand(FFTBand band) const {
    return getBand(static_cast<int>(band));
}

float AudioAnalyzer::getRawBand(int bandIndex) const {
    if (bandIndex < 0 || bandIndex >= 8) return 0.0f;
    return bandValues[bandIndex];
}

float AudioAnalyzer::getPeak(int bandIndex) const {
    if (bandIndex < 0 || bandIndex >= 8) return 0.0f;
    return peakValues[bandIndex];
}

void AudioAnalyzer::setEnabled(bool enabled) {
    if (settings.enabled != enabled) {
        settings.enabled = enabled;
        if (enabled) {
            setup(settings);
        } else {
            close();
        }
    }
}

void AudioAnalyzer::rebuildDeviceIdMap() {
    inputDeviceIds.clear();
    auto deviceList = soundStream.getDeviceList();
    for (const auto& device : deviceList) {
        if (device.inputChannels > 0) {
            inputDeviceIds.push_back(device.deviceID);
        }
    }
    ofLogNotice("AudioAnalyzer") << "Rebuilt device map: " << inputDeviceIds.size() << " input devices found";
}

std::vector<std::string> AudioAnalyzer::getDeviceList() const {
    std::vector<std::string> devices;
    auto deviceList = soundStream.getDeviceList();
    for (const auto& device : deviceList) {
        if (device.inputChannels > 0) {
            devices.push_back(device.name);
        }
    }
    return devices;
}

void AudioAnalyzer::setDevice(int deviceIndex) {
    // Rebuild the device ID mapping to ensure it's up to date
    rebuildDeviceIdMap();
    
    // Validate the device index
    if (deviceIndex < 0 || deviceIndex >= (int)inputDeviceIds.size()) {
        ofLogWarning("AudioAnalyzer") << "Invalid device index: " << deviceIndex 
                                      << " (valid range: 0-" << (inputDeviceIds.size()-1) << ")";
        return;
    }
    
    if (settings.inputDevice != deviceIndex) {
        int oldDeviceId = currentDeviceId;
        settings.inputDevice = deviceIndex;
        currentDeviceId = inputDeviceIds[deviceIndex];
        
        ofLogNotice("AudioAnalyzer") << "Switching device: index " << deviceIndex 
                                      << " -> deviceID " << currentDeviceId
                                      << " (was deviceID " << oldDeviceId << ")";
        
        if (settings.enabled) {
            // Restart with new device
            close();
            setup(settings);
        }
    }
}

void AudioAnalyzer::audioIn(ofSoundBuffer& buffer) {
    const float* input = buffer.getBuffer().data();
    int nFrames = buffer.getNumFrames();
    int nChannels = buffer.getNumChannels();
    
    // Calculate volume (RMS)
    float vol = 0.0f;
    for (int i = 0; i < nFrames; i++) {
        float sample = 0.0f;
        for (int ch = 0; ch < nChannels; ch++) {
            sample += input[i * nChannels + ch];
        }
        sample /= nChannels;
        vol += sample * sample;
    }
    currentVolume = sqrtf(vol / nFrames);
    
    // Store audio in circular buffer
    std::lock_guard<std::mutex> lock(audioMutex);
    if (audioBuffer.size() < (size_t)settings.fftSize) {
        audioBuffer.resize(settings.fftSize);
    }
    
    // Copy to circular buffer
    for (int i = 0; i < nFrames; i++) {
        float sum = 0.0f;
        for (int ch = 0; ch < nChannels; ch++) {
            sum += input[i * nChannels + ch];
        }
        audioBuffer[bufferWriteIndex] = sum / nChannels;
        bufferWriteIndex = (bufferWriteIndex + 1) % audioBuffer.size();
    }
}

void AudioAnalyzer::audioIn(float* input, int bufferSize, int nChannels) {
    // Calculate volume (RMS)
    float vol = 0.0f;
    for (int i = 0; i < bufferSize; i++) {
        float sample = 0.0f;
        for (int ch = 0; ch < nChannels; ch++) {
            sample += input[i * nChannels + ch];
        }
        sample /= nChannels;
        vol += sample * sample;
    }
    currentVolume = sqrtf(vol / bufferSize);
    
    // Store audio in circular buffer
    std::lock_guard<std::mutex> lock(audioMutex);
    if (audioBuffer.size() < (size_t)settings.fftSize) {
        audioBuffer.resize(settings.fftSize);
    }
    
    // Copy to circular buffer
    for (int i = 0; i < bufferSize; i++) {
        float sum = 0.0f;
        for (int ch = 0; ch < nChannels; ch++) {
            sum += input[i * nChannels + ch];
        }
        audioBuffer[bufferWriteIndex] = sum / nChannels;
        bufferWriteIndex = (bufferWriteIndex + 1) % audioBuffer.size();
    }
}

void AudioAnalyzer::resetNormalization() {
    for (int i = 0; i < 8; i++) {
        minValues[i] = 0.0f;
        maxValues[i] = 0.01f;
    }
}

void AudioAnalyzer::updateNormalization() {
    for (int i = 0; i < 8; i++) {
        // Update min/max with exponential moving window
        if (smoothedValues[i] < minValues[i]) {
            minValues[i] = smoothedValues[i];
        } else {
            minValues[i] = minValues[i] * NORMALIZATION_DECAY + smoothedValues[i] * (1.0f - NORMALIZATION_DECAY);
        }
        
        if (smoothedValues[i] > maxValues[i]) {
            maxValues[i] = smoothedValues[i];
        } else {
            maxValues[i] = maxValues[i] * NORMALIZATION_DECAY + smoothedValues[i] * (1.0f - NORMALIZATION_DECAY);
        }
        
        // Ensure range is valid
        if (maxValues[i] <= minValues[i]) {
            maxValues[i] = minValues[i] + 0.001f;
        }
    }
}

} // namespace dragonwaves
