#include "ofApp.h"
#include "ofAppGLFWWindow.h"
#include <GLFW/glfw3.h>

// Modular includes
#include "Core/SettingsManager.h"
#include "Core/PresetManager.h"
#include "Inputs/InputManager.h"
#include "ShaderPipeline/PipelineManager.h"
#include "Output/OutputManager.h"
#include "Geometry/GeometryRenderer.h"
#include "Parameters/ParameterManager.h"

using namespace dragonwaves;

//========================================================================
// ofApp implementation using modular architecture
//========================================================================

//--------------------------------------------------------------
ofApp::ofApp() {
    // Constructor - members will be default constructed
}

//--------------------------------------------------------------
ofApp::~ofApp() {
    // Destructor should be minimal - all cleanup happens in exit()
    // The unique_ptrs will be automatically destroyed in reverse order of declaration
    // Note: DO NOT access singletons here as they may already be destroyed
}

//--------------------------------------------------------------
void ofApp::setup(){
    ofDisableArbTex();
    ofEnableNormalizedTexCoords();
    ofSetFrameRate(30);
    ofSetVerticalSync(false);
    ofBackground(0);
    ofHideCursor();
    
    // Load settings
    SettingsManager::getInstance().load();
    auto& settings = SettingsManager::getInstance();
    
    // Apply display settings
    ofSetFrameRate(settings.getDisplay().targetFPS);
    
    // CRITICAL: Sync SettingsManager to GUI BEFORE InputManager setup
    // GuiApp::setup() runs before ofApp::setup() and loads settings.json,
    // so we must overwrite those values with config.json values here
    if (gui) {
        gui->input1SourceType = settings.getInputSources().input1SourceType;
        gui->input2SourceType = settings.getInputSources().input2SourceType;
        gui->input1DeviceID = settings.getInputSources().input1DeviceID;
        gui->input2DeviceID = settings.getInputSources().input2DeviceID;
        gui->input1NdiSourceIndex = settings.getInputSources().input1NdiSourceIndex;
        gui->input2NdiSourceIndex = settings.getInputSources().input2NdiSourceIndex;
#if OFAPP_HAS_SPOUT
        gui->input1SpoutSourceIndex = settings.getInputSources().input1SpoutSourceIndex;
        gui->input2SpoutSourceIndex = settings.getInputSources().input2SpoutSourceIndex;
#endif
        ofLogNotice("ofApp") << "Synced input settings from config.json (SettingsManager) to GUI";
    }
    
    // Initialize input manager
    inputManager = std::make_unique<InputManager>();
    inputManager->setup(settings.getDisplay());
    
    // Configure inputs from settings (using the same values we synced to GUI)
    InputType input1Type = (InputType)settings.getInputSources().input1SourceType;
    InputType input2Type = (InputType)settings.getInputSources().input2SourceType;
    
    // Determine the correct device/index based on input type
    int input1DeviceOrIndex = 0;
    int input2DeviceOrIndex = 0;
    
    switch (input1Type) {
        case InputType::WEBCAM:
            input1DeviceOrIndex = settings.getInputSources().input1DeviceID;
            break;
        case InputType::NDI:
            input1DeviceOrIndex = settings.getInputSources().input1NdiSourceIndex;
            break;
        default:
            input1DeviceOrIndex = 0;
            break;
    }
    
    switch (input2Type) {
        case InputType::WEBCAM:
            input2DeviceOrIndex = settings.getInputSources().input2DeviceID;
            break;
        case InputType::NDI:
            input2DeviceOrIndex = settings.getInputSources().input2NdiSourceIndex;
            break;
        default:
            input2DeviceOrIndex = 0;
            break;
    }
    
    inputManager->configureInput1(input1Type, input1DeviceOrIndex);
    inputManager->configureInput2(input2Type, input2DeviceOrIndex);
    
    ofLogNotice("ofApp") << "Configured inputs from config.json: Input1=" 
                         << (int)input1Type << ":" << input1DeviceOrIndex 
                         << ", Input2=" << (int)input2Type << ":" << input2DeviceOrIndex;
    
    // Initial sync of NDI source names to GUI
    if (gui) {
        gui->ndiSourceNames = inputManager->getNdiSourceNames();
        ofLogNotice("ofApp") << "Initial NDI source list: " << gui->ndiSourceNames.size() << " sources";
    }
    
    // Initialize shader pipeline
    pipeline = std::make_unique<PipelineManager>();
    pipeline->setup(settings.getDisplay());
    
    // Initialize output manager
    outputManager = std::make_unique<OutputManager>();
    outputManager->setup(settings.getDisplay());
    
    // Initialize geometry manager
    geometryManager = std::make_unique<GeometryManager>();
    geometryManager->setup();
    
    // Initialize audio analyzer
    audioAnalyzer = std::make_unique<AudioAnalyzer>();
    audioAnalyzer->setup(settings.getAudio());
    
    // Initialize tempo manager
    tempoManager = std::make_unique<TempoManager>();
    tempoManager->setup(settings.getTempo());
    
    // Connect to pipeline
    if (pipeline) {
        pipeline->setAudioAnalyzer(audioAnalyzer.get());
        pipeline->setTempoManager(tempoManager.get());
    }
    
    // Connect to GUI
    if (gui) {
        gui->setAudioAnalyzer(audioAnalyzer.get());
        gui->setTempoManager(tempoManager.get());
        gui->syncAudioSettingsFromAnalyzer();  // Sync GUI with loaded settings
    }
    
    // Initialize preset manager
    PresetManager::getInstance().setup();
    
    // Initialize preview panel (AFTER pipeline is created)
    previewPanel = std::make_unique<dragonwaves::PreviewPanel>();
    previewPanel->setup(pipeline.get());
    
    // Set up color applied callback
    previewPanel->onColorApplied = [this](dragonwaves::ColorPicker::KeyTarget target, ofColor color) {
        // Convert to normalized float (0-1 range)
        float r = color.r / 255.0f;
        float g = color.g / 255.0f;
        float b = color.b / 255.0f;
        
        // Apply to appropriate GUI parameters
        switch (target) {
            case dragonwaves::ColorPicker::CH2_KEY:
                gui->ch2MixAndKey[1] = r;
                gui->ch2MixAndKey[2] = g;
                gui->ch2MixAndKey[3] = b;
                break;
            case dragonwaves::ColorPicker::FB1_KEY:
                gui->fb1MixAndKey[1] = r;
                gui->fb1MixAndKey[2] = g;
                gui->fb1MixAndKey[3] = b;
                break;
            case dragonwaves::ColorPicker::FB2_KEY:
                gui->fb2MixAndKey[1] = r;
                gui->fb2MixAndKey[2] = g;
                gui->fb2MixAndKey[3] = b;
                break;
            case dragonwaves::ColorPicker::FINAL_KEY:
                // Final mix key - using finalMixAndKey if available
                gui->finalMixAndKey[1] = r;
                gui->finalMixAndKey[2] = g;
                gui->finalMixAndKey[3] = b;
                break;
        }
        
        // Send OSC notification
        sendOscParameter("/gravity/preview/colorPicked", 1.0f);
    };
    
    // Pass preview panel to GuiApp
    if (gui) {
        gui->previewPanel = previewPanel.get();
    }
    
    // Setup OSC/Parameter manager
    ParameterManager::getInstance().setup(settings.getOsc());
    
    // Register Audio and Tempo parameters with OSC
    registerAudioTempoOscParams();
    
    // Initialize LFO thetas
    resetLfoThetas();
    
    // Register callback for settings reload (file watching)
    settings.onSettingsChanged([this]() {
        ofLogNotice("ofApp") << "Settings file changed, syncing to GUI...";
        this->syncSettingsManagerToGui();
    });
    
    // Legacy GUI reference for compatibility
    if (gui) {
        setupOsc();
    }
    
    ofLogNotice("ofApp") << "Setup complete";
}

//--------------------------------------------------------------
void ofApp::update(){
    // Update settings manager (file watching for runtime reload)
    SettingsManager::getInstance().update();
    
    // Update parameter manager (process OSC)
    ParameterManager::getInstance().update();
    
    // Check for input reinitialization
    if (gui && gui->reinitializeInputs) {
        reinitializeInputs();
        gui->reinitializeInputs = false;
    }
    
    // Check for source refresh
    if (gui && gui->refreshNdiSources) {
        inputManager->refreshNdiSources();
        // Sync the refreshed source names to the GUI
        gui->ndiSourceNames = inputManager->getNdiSourceNames();
        ofLogNotice("ofApp") << "NDI sources refreshed: " << gui->ndiSourceNames.size() << " sources found";
        gui->refreshNdiSources = false;
    }
    
    #if OFAPP_HAS_SPOUT
    if (gui && gui->refreshSpoutSources) {
        inputManager->refreshSpoutSources();
        gui->refreshSpoutSources = false;
    }
    #endif
    
    // Check for resolution change
    if (gui && gui->resolutionChangeRequested) {
        applyResolutionChange();
        gui->resolutionChangeRequested = false;
    }
    
    // Check for FPS change
    if (gui && gui->fpsChangeRequested) {
        ofSetFrameRate(gui->targetFPS);
        gui->fpsChangeRequested = false;
    }
    
    // Update inputs
    inputManager->update();
    
    // Update LFOs
    updateLfos();
    
    // Update geometry patterns - only if any geometry is enabled
    if (gui && (gui->block1LineSwitch || gui->block1SevenStarSwitch || 
        gui->block1LissaBallSwitch || gui->block1HypercubeSwitch ||
        gui->block1LissajousCurveSwitch)) {
        geometryManager->update();
    }
    
    // Update audio analyzer
    if (audioAnalyzer) {
        audioAnalyzer->update();
    }
    
    // Note: Tempo manager is updated in pipeline->updateModulations() to avoid double updates
    
    // Process OSC messages (legacy)
    if (oscEnabled) {
        processOscMessages();
    }
    
    // Update preview panel (always update texture, window visibility is controlled separately)
    if (previewPanel) {
        previewPanel->update();
    }
}

//--------------------------------------------------------------
void ofApp::draw(){
    if (!pipeline) return;
    
    // Sync parameters from GUI to pipeline
    syncGuiToPipeline();
    
    // Apply audio/BPM modulations (after GUI sync, before shader processing)
    if (pipeline && (audioAnalyzer || tempoManager)) {
        pipeline->updateModulations(ofGetLastFrameTime());
    }
    
    // Set input textures
    pipeline->setInput1Texture(inputManager->getInput1Texture());
    pipeline->setInput2Texture(inputManager->getInput2Texture());
    
    // Draw geometry patterns FIRST (before shader processing)
    // This ensures geometry is rendered into the FBOs before they're used as textures
    drawGeometryPatterns();
    
    // Process shader pipeline
    pipeline->processFrame();
    
    // Send outputs
    sendOutputs();
    
    // Draw to screen based on draw mode
    drawOutput();
    
    // Clear framebuffers for next frame (only if requested)
    clearFramebuffers();
}

//--------------------------------------------------------------
// LFO wave generation function
float ofApp::lfo(float amp, float rate, int shape) {
    float waveValue = 0.0f;
    
    switch(shape) {
        case 0: // Sine (default)
            waveValue = sin(rate);
            break;
        case 1: // Triangle
            waveValue = (2.0f / PI) * asin(sin(rate));
            break;
        case 2: // Ramp (rising sawtooth)
            waveValue = (2.0f / TWO_PI) * fmod(rate + PI, TWO_PI) - 1.0f;
            break;
        case 3: // Saw (falling sawtooth)
            waveValue = 1.0f - (2.0f / TWO_PI) * fmod(rate + PI, TWO_PI);
            break;
        case 4: // Square (50% duty cycle)
            waveValue = (sin(rate) >= 0.0f) ? 1.0f : -1.0f;
            break;
        default: // Fallback to sine
            waveValue = sin(rate);
            break;
    }
    
    return amp * waveValue;
}

//--------------------------------------------------------------
// Helper to calculate effective LFO rate (handles tempo sync)
float ofApp::getEffectiveLfoRate(bool* syncEnabled, int* divisionIndex, float rawRate, const float* divisionValues) {
    if (!syncEnabled || !*syncEnabled) {
        // Sync disabled - use raw rate
        return rawRate;
    }
    
    // Sync enabled - calculate rate based on BPM
    if (!tempoManager || !tempoManager->isEnabled() || !tempoManager->isPlaying()) {
        // Tempo not available - fall back to raw rate
        return rawRate;
    }
    
    // Get cycles per beat from division
    int divIdx = divisionIndex ? *divisionIndex : 2; // Default to 1/4
    divIdx = ofClamp(divIdx, 0, 7);
    float cyclesPerBeat = divisionValues[divIdx];
    
    // Calculate phase increment per frame based on actual frame time
    // We want: phase increment = cyclesPerBeat * (BPM/60) * TWO_PI * deltaTime
    // But since lfoRateC is applied as a multiplier in updateLfos, we return a normalized value
    float bpm = tempoManager->getBpm();
    float beatsPerSecond = bpm / 60.0f;
    float cyclesPerSecond = cyclesPerBeat * beatsPerSecond;
    
    // Get actual delta time for frame-rate independent calculation
    float deltaTime = ofGetLastFrameTime();
    
    // Calculate what rate would give us the desired cycles per second
    // lfoRateC * rate * (TWO_PI / (lfoRateC * deltaTime)) = cycles per second
    // We want: lfoRateC * normalizedRate = cyclesPerSecond * TWO_PI * deltaTime
    float desiredPhaseIncrement = cyclesPerSecond * TWO_PI * deltaTime;
    float normalizedRate = desiredPhaseIncrement / lfoRateC;
    
    return normalizedRate;
}

//--------------------------------------------------------------
void ofApp::syncGuiToSettingsManager() {
    if (!gui) return;
    
    auto& settings = SettingsManager::getInstance();
    auto& displaySettings = settings.getDisplay();
    auto& inputSettings = settings.getInputSources();
    auto& oscSettings = settings.getOsc();
    auto& midiSettings = settings.getMidi();
    
    // Sync display settings
    displaySettings.input1Width = gui->input1Width;
    displaySettings.input1Height = gui->input1Height;
    displaySettings.input2Width = gui->input2Width;
    displaySettings.input2Height = gui->input2Height;
    displaySettings.internalWidth = gui->internalWidth;
    displaySettings.internalHeight = gui->internalHeight;
    displaySettings.outputWidth = gui->outputWidth;
    displaySettings.outputHeight = gui->outputHeight;
    displaySettings.ndiSendWidth = gui->ndiSendWidth;
    displaySettings.ndiSendHeight = gui->ndiSendHeight;
    displaySettings.targetFPS = gui->targetFPS;
    
    // Sync input source settings
    inputSettings.input1SourceType = gui->input1SourceType;
    inputSettings.input2SourceType = gui->input2SourceType;
    inputSettings.input1DeviceID = gui->input1DeviceID;
    inputSettings.input2DeviceID = gui->input2DeviceID;
    inputSettings.input1NdiSourceIndex = gui->input1NdiSourceIndex;
    inputSettings.input2NdiSourceIndex = gui->input2NdiSourceIndex;
#if OFAPP_HAS_SPOUT
    inputSettings.input1SpoutSourceIndex = gui->input1SpoutSourceIndex;
    inputSettings.input2SpoutSourceIndex = gui->input2SpoutSourceIndex;
#endif
    
    // Sync OSC settings
    oscSettings.enabled = gui->oscEnabled;
    oscSettings.receivePort = gui->oscReceivePort;
    oscSettings.sendIP = std::string(gui->oscSendIP);
    oscSettings.sendPort = gui->oscSendPort;
    
    // Sync MIDI settings
    midiSettings.selectedPort = gui->selectedMidiPort;
    midiSettings.deviceName = (gui->selectedMidiPort >= 0 && gui->selectedMidiPort < (int)gui->midiDeviceNames.size()) 
        ? gui->midiDeviceNames[gui->selectedMidiPort] 
        : "";
    midiSettings.enabled = gui->midiConnected;
    
    // Sync UI scale
    settings.setUIScaleIndex(gui->uiScaleIndex);
    
    ofLogNotice("ofApp") << "GUI settings synced to SettingsManager";
}

//--------------------------------------------------------------
void ofApp::syncSettingsManagerToGui() {
    if (!gui) return;
    
    auto& settings = SettingsManager::getInstance();
    auto& displaySettings = settings.getDisplay();
    auto& inputSettings = settings.getInputSources();
    auto& oscSettings = settings.getOsc();
    auto& midiSettings = settings.getMidi();
    
    // Sync display settings to GUI
    gui->input1Width = displaySettings.input1Width;
    gui->input1Height = displaySettings.input1Height;
    gui->input2Width = displaySettings.input2Width;
    gui->input2Height = displaySettings.input2Height;
    gui->internalWidth = displaySettings.internalWidth;
    gui->internalHeight = displaySettings.internalHeight;
    gui->outputWidth = displaySettings.outputWidth;
    gui->outputHeight = displaySettings.outputHeight;
    gui->ndiSendWidth = displaySettings.ndiSendWidth;
    gui->ndiSendHeight = displaySettings.ndiSendHeight;
    gui->targetFPS = displaySettings.targetFPS;
    
    // Sync input source settings to GUI
    gui->input1SourceType = inputSettings.input1SourceType;
    gui->input2SourceType = inputSettings.input2SourceType;
    gui->input1DeviceID = inputSettings.input1DeviceID;
    gui->input2DeviceID = inputSettings.input2DeviceID;
    gui->input1NdiSourceIndex = inputSettings.input1NdiSourceIndex;
    gui->input2NdiSourceIndex = inputSettings.input2NdiSourceIndex;
#if OFAPP_HAS_SPOUT
    gui->input1SpoutSourceIndex = inputSettings.input1SpoutSourceIndex;
    gui->input2SpoutSourceIndex = inputSettings.input2SpoutSourceIndex;
#endif
    
    // NOTE: We do NOT automatically reinitialize inputs when settings are reloaded
    // from file. This prevents interruption of video during use.
    // The user must explicitly click the "Reinitialize Inputs" button to apply
    // input source changes. The GUI values above are synced so the user can see
    // what settings are loaded and decide when to apply them.
    
    // Sync OSC settings to GUI
    gui->oscEnabled = oscSettings.enabled;
    gui->oscReceivePort = oscSettings.receivePort;
    strncpy(gui->oscSendIP, oscSettings.sendIP.c_str(), sizeof(gui->oscSendIP) - 1);
    gui->oscSendIP[sizeof(gui->oscSendIP) - 1] = '\0';  // Ensure null termination
    gui->oscSendPort = oscSettings.sendPort;
    
    // Sync MIDI settings to GUI
    gui->selectedMidiPort = midiSettings.selectedPort;
    // Note: We don't override device name if port is already set correctly
    
    // Sync UI scale to GUI
    gui->uiScaleIndex = settings.getUIScaleIndex();
    
    // NOTE: We do NOT automatically apply resolution/FPS changes when settings are reloaded.
    // This prevents interruption of video during use.
    // The user must explicitly click the "Apply Resolution" button in the GUI to apply changes.
    // We just clear the flags so they don't trigger later.
    if (settings.hasResolutionChanged()) {
        settings.clearResolutionChanged();
        ofLogNotice("ofApp") << "Resolution settings changed in config.json - click 'Apply Resolution' button to apply";
    }
    if (settings.hasFPSChanged()) {
        settings.clearFPSChanged();
        ofLogNotice("ofApp") << "FPS settings changed in config.json - click 'Apply Resolution' button to apply";
    }
    
    // Reload OSC settings
    gui->oscSettingsReloadRequested = true;
    
    ofLogNotice("ofApp") << "SettingsManager synced to GUI (config.json reloaded). Input settings updated but NOT applied - click 'Reinitialize Inputs' button to apply changes.";
}

//--------------------------------------------------------------
void ofApp::syncGuiToPipeline() {
    if (!gui) return;
    
    auto& block1 = pipeline->getBlock1();
    auto& block2 = pipeline->getBlock2();
    auto& block3 = pipeline->getBlock3();
    
    // ========================================
    // BLOCK 1 - Channel 1 (with LFO modulation)
    // ========================================
    // Base values from GUI
    float ch1XDisplace = -640.0f * gui->ch1Adjust[0];
    float ch1YDisplace = 480.0f * gui->ch1Adjust[1];
    float ch1ZDisplace = 1.0f + gui->ch1Adjust[2];
    float ch1Rotate = PI * gui->ch1Adjust[3];
    float ch1HueAttenuate = 1.0f + gui->ch1Adjust[4];
    float ch1SaturationAttenuate = 1.0f + gui->ch1Adjust[5];
    float ch1BrightAttenuate = 1.0f + gui->ch1Adjust[6];
    float ch1KaleidoscopeSlice = PI * gui->ch1Adjust[9];
    
    // Apply LFO modulation (even indices are amplitude, odd are rate which updates theta)
    ch1XDisplace += lfo(ch1XDisplaceC * gui->ch1AdjustLfo[0], ch1XDisplaceTheta, gui->ch1AdjustLfoShape[0]);
    ch1YDisplace += lfo(ch1YDisplaceC * gui->ch1AdjustLfo[2], ch1YDisplaceTheta, gui->ch1AdjustLfoShape[1]);
    ch1ZDisplace += lfo(ch1ZDisplaceC * gui->ch1AdjustLfo[4], ch1ZDisplaceTheta, gui->ch1AdjustLfoShape[2]);
    ch1Rotate += lfo(ch1RotateC * gui->ch1AdjustLfo[6], ch1RotateTheta, gui->ch1AdjustLfoShape[3]);
    ch1HueAttenuate += lfo(gui->ch1AdjustLfo[8], ch1HueAttenuateTheta, gui->ch1AdjustLfoShape[4]);
    ch1SaturationAttenuate += lfo(gui->ch1AdjustLfo[10], ch1SaturationAttenuateTheta, gui->ch1AdjustLfoShape[5]);
    ch1BrightAttenuate += lfo(gui->ch1AdjustLfo[12], ch1BrightAttenuateTheta, gui->ch1AdjustLfoShape[6]);
    ch1KaleidoscopeSlice += lfo(ch1KaleidoscopeSliceC * gui->ch1AdjustLfo[14], ch1KaleidoscopeSliceTheta, gui->ch1AdjustLfoShape[7]);
    
    block1.params.ch1XDisplace = ch1XDisplace;
    block1.params.ch1YDisplace = ch1YDisplace;
    block1.params.ch1ZDisplace = ch1ZDisplace;
    block1.params.ch1Rotate = ch1Rotate;
    block1.params.ch1HueAttenuate = ch1HueAttenuate;
    block1.params.ch1SaturationAttenuate = ch1SaturationAttenuate;
    block1.params.ch1BrightAttenuate = ch1BrightAttenuate;
    block1.params.ch1KaleidoscopeSlice = ch1KaleidoscopeSlice;
    
    // Other Channel 1 params (no LFO)
    block1.params.ch1Posterize = 15.0f * (1.0f - gui->ch1Adjust[7]) + 1.0f;
    block1.params.ch1PosterizeSwitch = gui->ch1Adjust[7] > 0 ? 1 : 0;
    block1.params.ch1KaleidoscopeAmount = floor(21.0f * gui->ch1Adjust[8]);
    block1.params.ch1BlurAmount = gui->ch1Adjust[10];
    block1.params.ch1BlurRadius = 9.0f * gui->ch1Adjust[11] + 1.0f;
    block1.params.ch1SharpenAmount = gui->ch1Adjust[12];
    block1.params.ch1SharpenRadius = 9.0f * gui->ch1Adjust[13] + 1.0f;
    block1.params.ch1FiltersBoost = gui->ch1Adjust[14];
    
    // Channel 1 switches
    block1.params.ch1InputSelect = gui->ch1InputSelect;
    block1.params.ch1GeoOverflow = gui->ch1GeoOverflow;
    block1.params.ch1HMirror = gui->ch1HMirror ? 1 : 0;
    block1.params.ch1VMirror = gui->ch1VMirror ? 1 : 0;
    block1.params.ch1HFlip = gui->ch1HFlip ? 1 : 0;
    block1.params.ch1VFlip = gui->ch1VFlip ? 1 : 0;
    block1.params.ch1HueInvert = gui->ch1HueInvert ? 1 : 0;
    block1.params.ch1SaturationInvert = gui->ch1SaturationInvert ? 1 : 0;
    block1.params.ch1BrightInvert = gui->ch1BrightInvert ? 1 : 0;
    block1.params.ch1RGBInvert = gui->ch1RGBInvert ? 1 : 0;
    block1.params.ch1Solarize = gui->ch1Solarize ? 1 : 0;
    block1.params.ch1HdAspectOn = gui->ch1AspectRatioSwitch;
    
    // ========================================
    // BLOCK 1 - Channel 2 Mix (with LFO)
    // ========================================
    float ch2MixAmount = 2.0f * gui->ch2MixAndKey[0];
    float ch2KeyThreshold = (ROOT_THREE + 0.001f) * gui->ch2MixAndKey[4];
    float ch2KeySoft = gui->ch2MixAndKey[5];
    
    // Apply LFO
    ch2MixAmount += lfo(mixAmountC * gui->ch2MixAndKeyLfo[0], ch2MixAmountTheta, gui->ch2MixAndKeyLfoShape[0]);
    ch2KeyThreshold += lfo(keyThresholdC * gui->ch2MixAndKeyLfo[2], ch2KeyThresholdTheta, gui->ch2MixAndKeyLfoShape[1]);
    ch2KeySoft += lfo(gui->ch2MixAndKeyLfo[4], ch2KeySoftTheta, gui->ch2MixAndKeyLfoShape[2]);
    
    block1.params.ch2MixAmount = ch2MixAmount;
    block1.params.ch2KeyThreshold = ch2KeyThreshold;
    block1.params.ch2KeySoft = ch2KeySoft;
    block1.params.ch2KeyValueRed = gui->ch2MixAndKey[1];
    block1.params.ch2KeyValueGreen = gui->ch2MixAndKey[2];
    block1.params.ch2KeyValueBlue = gui->ch2MixAndKey[3];
    block1.params.ch2KeyOrder = gui->ch2KeyOrder;
    block1.params.ch2MixType = gui->ch2MixType;
    block1.params.ch2MixOverflow = gui->ch2MixOverflow;
    
    // ========================================
    // BLOCK 1 - Channel 2 Adjust (with LFO)
    // ========================================
    float ch2XDisplace = -640.0f * gui->ch2Adjust[0];
    float ch2YDisplace = 480.0f * gui->ch2Adjust[1];
    float ch2ZDisplace = 1.0f + gui->ch2Adjust[2];
    float ch2Rotate = PI * gui->ch2Adjust[3];
    float ch2HueAttenuate = 1.0f + gui->ch2Adjust[4];
    float ch2SaturationAttenuate = 1.0f + gui->ch2Adjust[5];
    float ch2BrightAttenuate = 1.0f + gui->ch2Adjust[6];
    float ch2KaleidoscopeSlice = PI * gui->ch2Adjust[9];
    
    // Apply LFO
    ch2XDisplace += lfo(ch2XDisplaceC * gui->ch2AdjustLfo[0], ch2XDisplaceTheta, gui->ch2AdjustLfoShape[0]);
    ch2YDisplace += lfo(ch2YDisplaceC * gui->ch2AdjustLfo[2], ch2YDisplaceTheta, gui->ch2AdjustLfoShape[1]);
    ch2ZDisplace += lfo(ch2ZDisplaceC * gui->ch2AdjustLfo[4], ch2ZDisplaceTheta, gui->ch2AdjustLfoShape[2]);
    ch2Rotate += lfo(ch2RotateC * gui->ch2AdjustLfo[6], ch2RotateTheta, gui->ch2AdjustLfoShape[3]);
    ch2HueAttenuate += lfo(gui->ch2AdjustLfo[8], ch2HueAttenuateTheta, gui->ch2AdjustLfoShape[4]);
    ch2SaturationAttenuate += lfo(gui->ch2AdjustLfo[10], ch2SaturationAttenuateTheta, gui->ch2AdjustLfoShape[5]);
    ch2BrightAttenuate += lfo(gui->ch2AdjustLfo[12], ch2BrightAttenuateTheta, gui->ch2AdjustLfoShape[6]);
    ch2KaleidoscopeSlice += lfo(ch2KaleidoscopeSliceC * gui->ch2AdjustLfo[14], ch2KaleidoscopeSliceTheta, gui->ch2AdjustLfoShape[7]);
    
    block1.params.ch2XDisplace = ch2XDisplace;
    block1.params.ch2YDisplace = ch2YDisplace;
    block1.params.ch2ZDisplace = ch2ZDisplace;
    block1.params.ch2Rotate = ch2Rotate;
    block1.params.ch2HueAttenuate = ch2HueAttenuate;
    block1.params.ch2SaturationAttenuate = ch2SaturationAttenuate;
    block1.params.ch2BrightAttenuate = ch2BrightAttenuate;
    block1.params.ch2KaleidoscopeSlice = ch2KaleidoscopeSlice;
    
    // Other Channel 2 params (no LFO)
    block1.params.ch2Posterize = 15.0f * (1.0f - gui->ch2Adjust[7]) + 1.0f;
    block1.params.ch2PosterizeSwitch = gui->ch2Adjust[7] > 0 ? 1 : 0;
    block1.params.ch2KaleidoscopeAmount = floor(21.0f * gui->ch2Adjust[8]);
    block1.params.ch2BlurAmount = gui->ch2Adjust[10];
    block1.params.ch2BlurRadius = 9.0f * gui->ch2Adjust[11] + 1.0f;
    block1.params.ch2SharpenAmount = gui->ch2Adjust[12];
    block1.params.ch2SharpenRadius = 9.0f * gui->ch2Adjust[13] + 1.0f;
    block1.params.ch2FiltersBoost = gui->ch2Adjust[14];
    
    // Channel 2 switches
    block1.params.ch2InputSelect = gui->ch2InputSelect;
    block1.params.ch2GeoOverflow = gui->ch2GeoOverflow;
    block1.params.ch2HMirror = gui->ch2HMirror ? 1 : 0;
    block1.params.ch2VMirror = gui->ch2VMirror ? 1 : 0;
    block1.params.ch2HFlip = gui->ch2HFlip ? 1 : 0;
    block1.params.ch2VFlip = gui->ch2VFlip ? 1 : 0;
    block1.params.ch2HueInvert = gui->ch2HueInvert ? 1 : 0;
    block1.params.ch2SaturationInvert = gui->ch2SaturationInvert ? 1 : 0;
    block1.params.ch2BrightInvert = gui->ch2BrightInvert ? 1 : 0;
    block1.params.ch2RGBInvert = gui->ch2RGBInvert ? 1 : 0;
    block1.params.ch2Solarize = gui->ch2Solarize ? 1 : 0;
    block1.params.ch2HdAspectOn = gui->ch2AspectRatioSwitch;
    
    // ========================================
    // BLOCK 1 - FB1 Mix & Key (with LFO)
    // ========================================
    float fb1MixAmount = 2.0f * gui->fb1MixAndKey[0];
    float fb1KeyThreshold = (ROOT_THREE + 0.001f) * gui->fb1MixAndKey[4];
    float fb1KeySoft = gui->fb1MixAndKey[5];
    
    // Apply LFO
    fb1MixAmount += lfo(mixAmountC * gui->fb1MixAndKeyLfo[0], fb1MixAmountTheta, gui->fb1MixAndKeyLfoShape[0]);
    fb1KeyThreshold += lfo(keyThresholdC * gui->fb1MixAndKeyLfo[2], fb1KeyThresholdTheta, gui->fb1MixAndKeyLfoShape[1]);
    fb1KeySoft += lfo(gui->fb1MixAndKeyLfo[4], fb1KeySoftTheta, gui->fb1MixAndKeyLfoShape[2]);
    
    block1.params.fb1MixAmount = fb1MixAmount;
    block1.params.fb1KeyThreshold = fb1KeyThreshold;
    block1.params.fb1KeySoft = fb1KeySoft;
    block1.params.fb1KeyValueRed = gui->fb1MixAndKey[1];
    block1.params.fb1KeyValueGreen = gui->fb1MixAndKey[2];
    block1.params.fb1KeyValueBlue = gui->fb1MixAndKey[3];
    block1.params.fb1KeyOrder = gui->fb1KeyOrder;
    block1.params.fb1MixType = gui->fb1MixType;
    block1.params.fb1MixOverflow = gui->fb1MixOverflow;
    
    // ========================================
    // BLOCK 1 - FB1 Geo1 (with LFO)
    // ========================================
    float fb1XDisplace = -80.0f * gui->fb1Geo1[0];
    float fb1YDisplace = 80.0f * gui->fb1Geo1[1];
    float fb1ZDisplace = 1.0f + 0.5f * gui->fb1Geo1[2];
    float fb1Rotate = PI * gui->fb1Geo1[3];
    
    // Apply LFO (Geo1Lfo1)
    fb1XDisplace += lfo(fb1XDisplaceC * gui->fb1Geo1Lfo1[0], fb1XDisplaceTheta, gui->fb1Geo1Lfo1Shape[0]);
    fb1YDisplace += lfo(fb1YDisplaceC * gui->fb1Geo1Lfo1[2], fb1YDisplaceTheta, gui->fb1Geo1Lfo1Shape[1]);
    fb1ZDisplace += lfo(fb1ZDisplaceC * gui->fb1Geo1Lfo1[4], fb1ZDisplaceTheta, gui->fb1Geo1Lfo1Shape[2]);
    fb1Rotate += lfo(fb1RotateC * gui->fb1Geo1Lfo1[6], fb1RotateTheta, gui->fb1Geo1Lfo1Shape[3]);
    
    block1.params.fb1XDisplace = fb1XDisplace;
    block1.params.fb1YDisplace = fb1YDisplace;
    block1.params.fb1ZDisplace = fb1ZDisplace;
    block1.params.fb1Rotate = fb1Rotate;
    
    // FB1 Shear Matrix (with LFO from Geo1Lfo2)
    float fb1ShearMatrix1 = 0.25f * (1.0f / 0.25f + gui->fb1Geo1[4]);
    float fb1ShearMatrix2 = -0.25f * gui->fb1Geo1[6];
    float fb1ShearMatrix3 = 0.25f * gui->fb1Geo1[7];
    float fb1ShearMatrix4 = 0.25f * (1.0f / 0.25f + gui->fb1Geo1[5]);
    float fb1KaleidoscopeSlice = PI * gui->fb1Geo1[9];
    
    // Apply LFO (Geo1Lfo2)
    fb1ShearMatrix1 += lfo(fb1ShearMatrix1C * gui->fb1Geo1Lfo2[0], fb1ShearMatrix1Theta, gui->fb1Geo1Lfo2Shape[0]);
    fb1ShearMatrix2 += lfo(fb1ShearMatrix2C * gui->fb1Geo1Lfo2[4], fb1ShearMatrix2Theta, gui->fb1Geo1Lfo2Shape[2]);
    fb1ShearMatrix3 += lfo(fb1ShearMatrix3C * gui->fb1Geo1Lfo2[6], fb1ShearMatrix3Theta, gui->fb1Geo1Lfo2Shape[3]);
    fb1ShearMatrix4 += lfo(fb1ShearMatrix4C * gui->fb1Geo1Lfo2[2], fb1ShearMatrix4Theta, gui->fb1Geo1Lfo2Shape[1]);
    fb1KaleidoscopeSlice += lfo(fb1KaleidoscopeSliceC * gui->fb1Geo1Lfo2[8], fb1KaleidoscopeSliceTheta, gui->fb1Geo1Lfo2Shape[4]);
    
    block1.params.fb1ShearMatrix1 = fb1ShearMatrix1;
    block1.params.fb1ShearMatrix2 = fb1ShearMatrix2;
    block1.params.fb1ShearMatrix3 = fb1ShearMatrix3;
    block1.params.fb1ShearMatrix4 = fb1ShearMatrix4;
    block1.params.fb1KaleidoscopeSlice = fb1KaleidoscopeSlice;
    block1.params.fb1KaleidoscopeAmount = floor(21.0f * gui->fb1Geo1[8]);
    
    // ========================================
    // BLOCK 1 - FB1 Color (with LFO)
    // ========================================
    float fb1HueAttenuate = 1.0f + 0.25f * gui->fb1Color1[3];
    float fb1SaturationAttenuate = 1.0f + 0.25f * gui->fb1Color1[4];
    float fb1BrightAttenuate = 1.0f + 0.25f * gui->fb1Color1[5];
    
    // Apply LFO (Color1Lfo1)
    fb1HueAttenuate += lfo(fb1HueAttenuateC * gui->fb1Color1Lfo1[0], fb1HueAttenuateTheta, gui->fb1Color1Lfo1Shape[0]);
    fb1SaturationAttenuate += lfo(fb1SaturationAttenuateC * gui->fb1Color1Lfo1[2], fb1SaturationAttenuateTheta, gui->fb1Color1Lfo1Shape[1]);
    fb1BrightAttenuate += lfo(fb1BrightAttenuateC * gui->fb1Color1Lfo1[4], fb1BrightAttenuateTheta, gui->fb1Color1Lfo1Shape[2]);
    
    block1.params.fb1HueAttenuate = fb1HueAttenuate;
    block1.params.fb1SaturationAttenuate = fb1SaturationAttenuate;
    block1.params.fb1BrightAttenuate = fb1BrightAttenuate;
    
    // Other FB1 Color params (no LFO)
    block1.params.fb1HueOffset = 0.25f * gui->fb1Color1[0];
    block1.params.fb1SaturationOffset = 0.25f * gui->fb1Color1[1];
    block1.params.fb1BrightOffset = 0.25f * gui->fb1Color1[2];
    block1.params.fb1HuePowmap = 1.0f + 0.1f * gui->fb1Color1[6];
    block1.params.fb1SaturationPowmap = 1.0f + 0.1f * gui->fb1Color1[7];
    block1.params.fb1BrightPowmap = 1.0f + 0.1f * gui->fb1Color1[8];
    block1.params.fb1HueShaper = gui->fb1Color1[9];
    block1.params.fb1Posterize = 15.0f * (1.0f - gui->fb1Color1[10]) + 1.0f;
    block1.params.fb1PosterizeSwitch = gui->fb1Color1[10] > 0 ? 1 : 0;
    
    // FB1 Filters (no LFO)
    block1.params.fb1BlurAmount = gui->fb1Filters[0];
    block1.params.fb1BlurRadius = 9.0f * gui->fb1Filters[1] + 1.0f;
    block1.params.fb1SharpenAmount = 0.6f * gui->fb1Filters[2];
    block1.params.fb1SharpenRadius = 9.0f * gui->fb1Filters[3] + 1.0f;
    block1.params.fb1TemporalFilter1Amount = 2.0f * gui->fb1Filters[4];
    block1.params.fb1TemporalFilter1Resonance = gui->fb1Filters[5];
    block1.params.fb1TemporalFilter2Amount = 2.0f * gui->fb1Filters[6];
    block1.params.fb1TemporalFilter2Resonance = gui->fb1Filters[7];
    block1.params.fb1FiltersBoost = gui->fb1Filters[8];
    
    // FB1 switches
    block1.params.fb1HMirror = gui->fb1HMirror ? 1 : 0;
    block1.params.fb1VMirror = gui->fb1VMirror ? 1 : 0;
    block1.params.fb1HFlip = gui->fb1HFlip ? 1 : 0;
    block1.params.fb1VFlip = gui->fb1VFlip ? 1 : 0;
    block1.params.fb1RotateMode = gui->fb1RotateMode ? 1 : 0;
    block1.params.fb1GeoOverflow = gui->fb1GeoOverflow;
    block1.params.fb1HueInvert = gui->fb1HueInvert ? 1 : 0;
    block1.params.fb1SaturationInvert = gui->fb1SaturationInvert ? 1 : 0;
    block1.params.fb1BrightInvert = gui->fb1BrightInvert ? 1 : 0;
    
    // FB1 delay time
    pipeline->setFB1DelayTime(gui->fb1DelayTime);
    
    // ========================================
    // BLOCK 2 - Input Adjust (with LFO)
    // ========================================
    float block2InputXDisplace = -640.0f * gui->block2InputAdjust[0];
    float block2InputYDisplace = 480.0f * gui->block2InputAdjust[1];
    float block2InputZDisplace = 1.0f + gui->block2InputAdjust[2];
    float block2InputRotate = PI * gui->block2InputAdjust[3];
    float block2InputHueAttenuate = 1.0f + gui->block2InputAdjust[4];
    float block2InputSaturationAttenuate = 1.0f + gui->block2InputAdjust[5];
    float block2InputBrightAttenuate = 1.0f + gui->block2InputAdjust[6];
    float block2InputKaleidoscopeSlice = PI * gui->block2InputAdjust[9];
    
    // Apply LFO
    block2InputXDisplace += lfo(block2InputXDisplaceC * gui->block2InputAdjustLfo[0], block2InputXDisplaceTheta, gui->block2InputAdjustLfoShape[0]);
    block2InputYDisplace += lfo(block2InputYDisplaceC * gui->block2InputAdjustLfo[2], block2InputYDisplaceTheta, gui->block2InputAdjustLfoShape[1]);
    block2InputZDisplace += lfo(block2InputZDisplaceC * gui->block2InputAdjustLfo[4], block2InputZDisplaceTheta, gui->block2InputAdjustLfoShape[2]);
    block2InputRotate += lfo(block2InputRotateC * gui->block2InputAdjustLfo[6], block2InputRotateTheta, gui->block2InputAdjustLfoShape[3]);
    block2InputHueAttenuate += lfo(gui->block2InputAdjustLfo[8], block2InputHueAttenuateTheta, gui->block2InputAdjustLfoShape[4]);
    block2InputSaturationAttenuate += lfo(gui->block2InputAdjustLfo[10], block2InputSaturationAttenuateTheta, gui->block2InputAdjustLfoShape[5]);
    block2InputBrightAttenuate += lfo(gui->block2InputAdjustLfo[12], block2InputBrightAttenuateTheta, gui->block2InputAdjustLfoShape[6]);
    block2InputKaleidoscopeSlice += lfo(block2InputKaleidoscopeSliceC * gui->block2InputAdjustLfo[14], block2InputKaleidoscopeSliceTheta, gui->block2InputAdjustLfoShape[7]);
    
    block2.params.block2InputXDisplace = block2InputXDisplace;
    block2.params.block2InputYDisplace = block2InputYDisplace;
    block2.params.block2InputZDisplace = block2InputZDisplace;
    block2.params.block2InputRotate = block2InputRotate;
    block2.params.block2InputHueAttenuate = block2InputHueAttenuate;
    block2.params.block2InputSaturationAttenuate = block2InputSaturationAttenuate;
    block2.params.block2InputBrightAttenuate = block2InputBrightAttenuate;
    block2.params.block2InputKaleidoscopeSlice = block2InputKaleidoscopeSlice;
    
    // Other Block2 Input params (no LFO)
    block2.params.block2InputSelect = gui->block2InputSelect;
    block2.params.block2InputPosterize = 15.0f * (1.0f - gui->block2InputAdjust[7]) + 1.0f;
    block2.params.block2InputPosterizeSwitch = gui->block2InputAdjust[7] > 0 ? 1 : 0;
    block2.params.block2InputKaleidoscopeAmount = floor(21.0f * gui->block2InputAdjust[8]);
    // Block2 Input switches
    block2.params.block2InputHMirror = gui->block2InputHMirror ? 1 : 0;
    block2.params.block2InputVMirror = gui->block2InputVMirror ? 1 : 0;
    block2.params.block2InputHFlip = gui->block2InputHFlip ? 1 : 0;
    block2.params.block2InputVFlip = gui->block2InputVFlip ? 1 : 0;
    block2.params.block2InputHueInvert = gui->block2InputHueInvert ? 1 : 0;
    block2.params.block2InputSaturationInvert = gui->block2InputSaturationInvert ? 1 : 0;
    block2.params.block2InputBrightInvert = gui->block2InputBrightInvert ? 1 : 0;
    block2.params.block2InputRGBInvert = gui->block2InputRGBInvert ? 1 : 0;
    block2.params.block2InputSolarize = gui->block2InputSolarize ? 1 : 0;
    block2.params.block2InputGeoOverflow = gui->block2InputGeoOverflow;
    
    // ========================================
    // BLOCK 2 - FB2 Mix & Key (with LFO)
    // ========================================
    float fb2MixAmount = 2.0f * gui->fb2MixAndKey[0];
    float fb2KeyThreshold = (ROOT_THREE + 0.001f) * gui->fb2MixAndKey[4];
    float fb2KeySoft = gui->fb2MixAndKey[5];
    
    // Apply LFO
    fb2MixAmount += lfo(mixAmountC * gui->fb2MixAndKeyLfo[0], fb2MixAmountTheta, gui->fb2MixAndKeyLfoShape[0]);
    fb2KeyThreshold += lfo(keyThresholdC * gui->fb2MixAndKeyLfo[2], fb2KeyThresholdTheta, gui->fb2MixAndKeyLfoShape[1]);
    fb2KeySoft += lfo(gui->fb2MixAndKeyLfo[4], fb2KeySoftTheta, gui->fb2MixAndKeyLfoShape[2]);
    
    block2.params.fb2MixAmount = fb2MixAmount;
    block2.params.fb2KeyThreshold = fb2KeyThreshold;
    block2.params.fb2KeySoft = fb2KeySoft;
    block2.params.fb2KeyValueRed = gui->fb2MixAndKey[1];
    block2.params.fb2KeyValueGreen = gui->fb2MixAndKey[2];
    block2.params.fb2KeyValueBlue = gui->fb2MixAndKey[3];
    block2.params.fb2KeyOrder = gui->fb2KeyOrder;
    block2.params.fb2MixType = gui->fb2MixType;
    block2.params.fb2MixOverflow = gui->fb2MixOverflow;
    
    // ========================================
    // BLOCK 2 - FB2 Geo1 (with LFO)
    // ========================================
    float fb2XDisplace = -80.0f * gui->fb2Geo1[0];
    float fb2YDisplace = 80.0f * gui->fb2Geo1[1];
    float fb2ZDisplace = 1.0f + 0.5f * gui->fb2Geo1[2];
    float fb2Rotate = PI * gui->fb2Geo1[3];
    
    // Apply LFO (Geo1Lfo1)
    fb2XDisplace += lfo(fb2XDisplaceC * gui->fb2Geo1Lfo1[0], fb2XDisplaceTheta, gui->fb2Geo1Lfo1Shape[0]);
    fb2YDisplace += lfo(fb2YDisplaceC * gui->fb2Geo1Lfo1[2], fb2YDisplaceTheta, gui->fb2Geo1Lfo1Shape[1]);
    fb2ZDisplace += lfo(fb2ZDisplaceC * gui->fb2Geo1Lfo1[4], fb2ZDisplaceTheta, gui->fb2Geo1Lfo1Shape[2]);
    fb2Rotate += lfo(fb2RotateC * gui->fb2Geo1Lfo1[6], fb2RotateTheta, gui->fb2Geo1Lfo1Shape[3]);
    
    block2.params.fb2XDisplace = fb2XDisplace;
    block2.params.fb2YDisplace = fb2YDisplace;
    block2.params.fb2ZDisplace = fb2ZDisplace;
    block2.params.fb2Rotate = fb2Rotate;
    
    // FB2 Shear Matrix (with LFO from Geo1Lfo2)
    float fb2ShearMatrix1 = 0.25f * (1.0f / 0.25f + gui->fb2Geo1[4]);
    float fb2ShearMatrix2 = -0.25f * gui->fb2Geo1[6];
    float fb2ShearMatrix3 = 0.25f * gui->fb2Geo1[7];
    float fb2ShearMatrix4 = 0.25f * (1.0f / 0.25f + gui->fb2Geo1[5]);
    float fb2KaleidoscopeSlice = PI * gui->fb2Geo1[9];
    
    // Apply LFO (Geo1Lfo2)
    fb2ShearMatrix1 += lfo(fb2ShearMatrix1C * gui->fb2Geo1Lfo2[0], fb2ShearMatrix1Theta, gui->fb2Geo1Lfo2Shape[0]);
    fb2ShearMatrix2 += lfo(fb2ShearMatrix2C * gui->fb2Geo1Lfo2[4], fb2ShearMatrix2Theta, gui->fb2Geo1Lfo2Shape[2]);
    fb2ShearMatrix3 += lfo(fb2ShearMatrix3C * gui->fb2Geo1Lfo2[6], fb2ShearMatrix3Theta, gui->fb2Geo1Lfo2Shape[3]);
    fb2ShearMatrix4 += lfo(fb2ShearMatrix4C * gui->fb2Geo1Lfo2[2], fb2ShearMatrix4Theta, gui->fb2Geo1Lfo2Shape[1]);
    fb2KaleidoscopeSlice += lfo(fb2KaleidoscopeSliceC * gui->fb2Geo1Lfo2[8], fb2KaleidoscopeSliceTheta, gui->fb2Geo1Lfo2Shape[4]);
    
    block2.params.fb2ShearMatrix1 = fb2ShearMatrix1;
    block2.params.fb2ShearMatrix2 = fb2ShearMatrix2;
    block2.params.fb2ShearMatrix3 = fb2ShearMatrix3;
    block2.params.fb2ShearMatrix4 = fb2ShearMatrix4;
    block2.params.fb2KaleidoscopeSlice = fb2KaleidoscopeSlice;
    block2.params.fb2KaleidoscopeAmount = floor(21.0f * gui->fb2Geo1[8]);
    // FB2 Geo switches
    block2.params.fb2HMirror = gui->block2HMirror ? 1 : 0;
    block2.params.fb2VMirror = gui->block2VMirror ? 1 : 0;
    block2.params.fb2HFlip = gui->block2HFlip ? 1 : 0;
    block2.params.fb2VFlip = gui->block2VFlip ? 1 : 0;
    block2.params.fb2RotateMode = gui->block2RotateMode ? 1 : 0;
    
    // ========================================
    // BLOCK 2 - FB2 Color (with LFO)
    // ========================================
    float fb2HueAttenuate = 1.0f + 0.25f * gui->fb2Color1[3];
    float fb2SaturationAttenuate = 1.0f + 0.25f * gui->fb2Color1[4];
    float fb2BrightAttenuate = 1.0f + 0.25f * gui->fb2Color1[5];
    
    // Apply LFO (Color1Lfo1)
    fb2HueAttenuate += lfo(fb2HueAttenuateC * gui->fb2Color1Lfo1[0], fb2HueAttenuateTheta, gui->fb2Color1Lfo1Shape[0]);
    fb2SaturationAttenuate += lfo(fb2SaturationAttenuateC * gui->fb2Color1Lfo1[2], fb2SaturationAttenuateTheta, gui->fb2Color1Lfo1Shape[1]);
    fb2BrightAttenuate += lfo(fb2BrightAttenuateC * gui->fb2Color1Lfo1[4], fb2BrightAttenuateTheta, gui->fb2Color1Lfo1Shape[2]);
    
    block2.params.fb2HueAttenuate = fb2HueAttenuate;
    block2.params.fb2SaturationAttenuate = fb2SaturationAttenuate;
    block2.params.fb2BrightAttenuate = fb2BrightAttenuate;
    
    // Other FB2 params (no LFO)
    block2.params.fb2HueOffset = 0.25f * gui->fb2Color1[0];
    block2.params.fb2SaturationOffset = 0.25f * gui->fb2Color1[1];
    block2.params.fb2BrightOffset = 0.25f * gui->fb2Color1[2];
    block2.params.fb2HuePowmap = 1.0f + 0.1f * gui->fb2Color1[6];
    block2.params.fb2SaturationPowmap = 1.0f + 0.1f * gui->fb2Color1[7];
    block2.params.fb2BrightPowmap = 1.0f + 0.1f * gui->fb2Color1[8];
    block2.params.fb2HueShaper = gui->fb2Color1[9];
    block2.params.fb2Posterize = 15.0f * (1.0f - gui->fb2Color1[10]) + 1.0f;
    block2.params.fb2PosterizeSwitch = gui->fb2Color1[10] > 0 ? 1 : 0;
    
    // ========================================
    // BLOCK 2 - FB2 Filters (no LFO)
    // ========================================
    block2.params.fb2BlurAmount = gui->fb2Filters[0];
    block2.params.fb2BlurRadius = fb2FilterRadiusC * gui->fb2Filters[1] + 1.0f;
    block2.params.fb2SharpenAmount = fb2SharpenAmountC * gui->fb2Filters[2];
    block2.params.fb2SharpenRadius = fb2FilterRadiusC * gui->fb2Filters[3] + 1.0f;
    block2.params.fb2TemporalFilter1Amount = fb2TemporalFilterAmountC * gui->fb2Filters[4];
    block2.params.fb2TemporalFilter1Resonance = gui->fb2Filters[5];
    block2.params.fb2TemporalFilter2Amount = fb2TemporalFilterAmountC * gui->fb2Filters[6];
    block2.params.fb2TemporalFilter2Resonance = gui->fb2Filters[7];
    block2.params.fb2FiltersBoost = gui->fb2Filters[8];
    
    // FB2 delay time
    pipeline->setFB2DelayTime(gui->fb2DelayTime);
    
    // ========================================
    // BLOCK 3 - Block1 Geo (with LFO)
    // ========================================
    float block1XDisplace = -1280.0f * gui->block1Geo[0];
    float block1YDisplace = 720.0f * gui->block1Geo[1];
    float block1ZDisplace = 1.0f + gui->block1Geo[2];
    float block1Rotate = PI * gui->block1Geo[3];
    
    // Apply LFO (Geo1Lfo1)
    block1XDisplace += lfo(block1XDisplaceC * gui->block1Geo1Lfo1[0], block1XDisplaceTheta, gui->block1Geo1Lfo1Shape[0]);
    block1YDisplace += lfo(block1YDisplaceC * gui->block1Geo1Lfo1[2], block1YDisplaceTheta, gui->block1Geo1Lfo1Shape[1]);
    block1ZDisplace += lfo(block1ZDisplaceC * gui->block1Geo1Lfo1[4], block1ZDisplaceTheta, gui->block1Geo1Lfo1Shape[2]);
    block1Rotate += lfo(block1RotateC * gui->block1Geo1Lfo1[6], block1RotateTheta, gui->block1Geo1Lfo1Shape[3]);
    
    block3.params.block1XDisplace = block1XDisplace;
    block3.params.block1YDisplace = block1YDisplace;
    block3.params.block1ZDisplace = block1ZDisplace;
    block3.params.block1Rotate = block1Rotate;
    
    // Block1 Shear Matrix (with LFO from Geo1Lfo2)
    float block1ShearMatrix1 = 1.0f * (1.0f + gui->block1Geo[4]);
    float block1ShearMatrix2 = -1.0f * gui->block1Geo[6];
    float block1ShearMatrix3 = 1.0f * gui->block1Geo[7];
    float block1ShearMatrix4 = 1.0f * (1.0f + gui->block1Geo[5]);
    float block1KaleidoscopeSlice = PI * gui->block1Geo[9];
    
    // Apply LFO (Geo1Lfo2)
    block1ShearMatrix1 += lfo(block1ShearMatrix1C * gui->block1Geo1Lfo2[0], block1ShearMatrix1Theta, gui->block1Geo1Lfo2Shape[0]);
    block1ShearMatrix2 += lfo(block1ShearMatrix2C * gui->block1Geo1Lfo2[4], block1ShearMatrix2Theta, gui->block1Geo1Lfo2Shape[2]);
    block1ShearMatrix3 += lfo(block1ShearMatrix3C * gui->block1Geo1Lfo2[6], block1ShearMatrix3Theta, gui->block1Geo1Lfo2Shape[3]);
    block1ShearMatrix4 += lfo(block1ShearMatrix4C * gui->block1Geo1Lfo2[2], block1ShearMatrix4Theta, gui->block1Geo1Lfo2Shape[1]);
    block1KaleidoscopeSlice += lfo(block1KaleidoscopeSliceC * gui->block1Geo1Lfo2[8], block1KaleidoscopeSliceTheta, gui->block1Geo1Lfo2Shape[4]);
    
    block3.params.block1ShearMatrix1 = block1ShearMatrix1;
    block3.params.block1ShearMatrix2 = block1ShearMatrix2;
    block3.params.block1ShearMatrix3 = block1ShearMatrix3;
    block3.params.block1ShearMatrix4 = block1ShearMatrix4;
    block3.params.block1KaleidoscopeSlice = block1KaleidoscopeSlice;
    block3.params.block1KaleidoscopeAmount = floor(21.0f * gui->block1Geo[8]);
    
    // ========================================
    // BLOCK 3 - Block1 Colorize (with LFO)
    // ========================================
    // Base values
    float block1ColorizeHueBand1 = gui->block1Colorize[0];
    float block1ColorizeSaturationBand1 = gui->block1Colorize[1];
    float block1ColorizeBrightBand1 = gui->block1Colorize[2];
    float block1ColorizeHueBand2 = gui->block1Colorize[3];
    float block1ColorizeSaturationBand2 = gui->block1Colorize[4];
    float block1ColorizeBrightBand2 = gui->block1Colorize[5];
    float block1ColorizeHueBand3 = gui->block1Colorize[6];
    float block1ColorizeSaturationBand3 = gui->block1Colorize[7];
    float block1ColorizeBrightBand3 = gui->block1Colorize[8];
    float block1ColorizeHueBand4 = gui->block1Colorize[9];
    float block1ColorizeSaturationBand4 = gui->block1Colorize[10];
    float block1ColorizeBrightBand4 = gui->block1Colorize[11];
    float block1ColorizeHueBand5 = gui->block1Colorize[12];
    float block1ColorizeSaturationBand5 = gui->block1Colorize[13];
    float block1ColorizeBrightBand5 = gui->block1Colorize[14];
    
    // Apply LFO (ColorizeLfo1 - bands 1-2)
    block1ColorizeHueBand1 += lfo(gui->block1ColorizeLfo1[0], block1ColorizeHueBand1Theta, gui->block1ColorizeLfo1Shape[0]);
    block1ColorizeSaturationBand1 += lfo(gui->block1ColorizeLfo1[1], block1ColorizeSaturationBand1Theta, gui->block1ColorizeLfo1Shape[1]);
    block1ColorizeBrightBand1 += lfo(gui->block1ColorizeLfo1[2], block1ColorizeBrightBand1Theta, gui->block1ColorizeLfo1Shape[2]);
    block1ColorizeHueBand2 += lfo(gui->block1ColorizeLfo1[6], block1ColorizeHueBand2Theta, gui->block1ColorizeLfo1Shape[3]);
    block1ColorizeSaturationBand2 += lfo(gui->block1ColorizeLfo1[7], block1ColorizeSaturationBand2Theta, gui->block1ColorizeLfo1Shape[4]);
    block1ColorizeBrightBand2 += lfo(gui->block1ColorizeLfo1[8], block1ColorizeBrightBand2Theta, gui->block1ColorizeLfo1Shape[5]);
    
    // Apply LFO (ColorizeLfo2 - bands 3-4)
    block1ColorizeHueBand3 += lfo(gui->block1ColorizeLfo2[0], block1ColorizeHueBand3Theta, gui->block1ColorizeLfo2Shape[0]);
    block1ColorizeSaturationBand3 += lfo(gui->block1ColorizeLfo2[1], block1ColorizeSaturationBand3Theta, gui->block1ColorizeLfo2Shape[1]);
    block1ColorizeBrightBand3 += lfo(gui->block1ColorizeLfo2[2], block1ColorizeBrightBand3Theta, gui->block1ColorizeLfo2Shape[2]);
    block1ColorizeHueBand4 += lfo(gui->block1ColorizeLfo2[6], block1ColorizeHueBand4Theta, gui->block1ColorizeLfo2Shape[3]);
    block1ColorizeSaturationBand4 += lfo(gui->block1ColorizeLfo2[7], block1ColorizeSaturationBand4Theta, gui->block1ColorizeLfo2Shape[4]);
    block1ColorizeBrightBand4 += lfo(gui->block1ColorizeLfo2[8], block1ColorizeBrightBand4Theta, gui->block1ColorizeLfo2Shape[5]);
    
    // Apply LFO (ColorizeLfo3 - band 5)
    block1ColorizeHueBand5 += lfo(gui->block1ColorizeLfo3[0], block1ColorizeHueBand5Theta, gui->block1ColorizeLfo3Shape[0]);
    block1ColorizeSaturationBand5 += lfo(gui->block1ColorizeLfo3[1], block1ColorizeSaturationBand5Theta, gui->block1ColorizeLfo3Shape[1]);
    block1ColorizeBrightBand5 += lfo(gui->block1ColorizeLfo3[2], block1ColorizeBrightBand5Theta, gui->block1ColorizeLfo3Shape[2]);
    
    block3.params.block1ColorizeHueBand1 = block1ColorizeHueBand1;
    block3.params.block1ColorizeSaturationBand1 = block1ColorizeSaturationBand1;
    block3.params.block1ColorizeBrightBand1 = block1ColorizeBrightBand1;
    block3.params.block1ColorizeHueBand2 = block1ColorizeHueBand2;
    block3.params.block1ColorizeSaturationBand2 = block1ColorizeSaturationBand2;
    block3.params.block1ColorizeBrightBand2 = block1ColorizeBrightBand2;
    block3.params.block1ColorizeHueBand3 = block1ColorizeHueBand3;
    block3.params.block1ColorizeSaturationBand3 = block1ColorizeSaturationBand3;
    block3.params.block1ColorizeBrightBand3 = block1ColorizeBrightBand3;
    block3.params.block1ColorizeHueBand4 = block1ColorizeHueBand4;
    block3.params.block1ColorizeSaturationBand4 = block1ColorizeSaturationBand4;
    block3.params.block1ColorizeBrightBand4 = block1ColorizeBrightBand4;
    block3.params.block1ColorizeHueBand5 = block1ColorizeHueBand5;
    block3.params.block1ColorizeSaturationBand5 = block1ColorizeSaturationBand5;
    block3.params.block1ColorizeBrightBand5 = block1ColorizeBrightBand5;
    
    block3.params.block1ColorizeSwitch = gui->block1ColorizeSwitch ? 1 : 0;
    block3.params.block1ColorizeHSB_RGB = gui->block1ColorizeHSB_RGB ? 1 : 0;
    
    // Block 3 filters (no LFO)
    block3.params.block1BlurAmount = gui->block1Filters[0];
    block3.params.block1BlurRadius = 9.0f * gui->block1Filters[1] + 1.0f;
    block3.params.block1SharpenAmount = gui->block1Filters[2];
    block3.params.block1SharpenRadius = 9.0f * gui->block1Filters[3] + 1.0f;
    block3.params.block1FiltersBoost = gui->block1Filters[4];
    block3.params.block1Dither = 15.0f * (1.0f - gui->block1Filters[5]) + 2.0f;
    block3.params.block1DitherSwitch = gui->block1Filters[5] > 0 ? 1 : 0;
    block3.params.block1DitherType = gui->block1DitherType;
    
    // Block 3 switches
    block3.params.block1HMirror = gui->block1HMirror ? 1 : 0;
    block3.params.block1VMirror = gui->block1VMirror ? 1 : 0;
    block3.params.block1HFlip = gui->block1HFlip ? 1 : 0;
    block3.params.block1VFlip = gui->block1VFlip ? 1 : 0;
    block3.params.block1RotateMode = gui->block1RotateMode ? 1 : 0;
    block3.params.block1GeoOverflow = gui->block1GeoOverflow;
    
    // ========================================
    // BLOCK 3 - Block2 Geo (with LFO)
    // ========================================
    float block2XDisplace = -1280.0f * gui->block2Geo[0];
    float block2YDisplace = 720.0f * gui->block2Geo[1];
    float block2ZDisplace = 1.0f + gui->block2Geo[2];
    float block2Rotate = PI * gui->block2Geo[3];
    
    // Apply LFO (Geo1Lfo1)
    block2XDisplace += lfo(block2XDisplaceC * gui->block2Geo1Lfo1[0], block2XDisplaceTheta, gui->block2Geo1Lfo1Shape[0]);
    block2YDisplace += lfo(block2YDisplaceC * gui->block2Geo1Lfo1[2], block2YDisplaceTheta, gui->block2Geo1Lfo1Shape[1]);
    block2ZDisplace += lfo(block2ZDisplaceC * gui->block2Geo1Lfo1[4], block2ZDisplaceTheta, gui->block2Geo1Lfo1Shape[2]);
    block2Rotate += lfo(block2RotateC * gui->block2Geo1Lfo1[6], block2RotateTheta, gui->block2Geo1Lfo1Shape[3]);
    
    block3.params.block2XDisplace = block2XDisplace;
    block3.params.block2YDisplace = block2YDisplace;
    block3.params.block2ZDisplace = block2ZDisplace;
    block3.params.block2Rotate = block2Rotate;
    
    // Block2 Shear Matrix (with LFO from Geo1Lfo2)
    float block2ShearMatrix1 = 1.0f * (1.0f + gui->block2Geo[4]);
    float block2ShearMatrix2 = -1.0f * gui->block2Geo[6];
    float block2ShearMatrix3 = 1.0f * gui->block2Geo[7];
    float block2ShearMatrix4 = 1.0f * (1.0f + gui->block2Geo[5]);
    float block2KaleidoscopeSlice = PI * gui->block2Geo[9];
    
    // Apply LFO (Geo1Lfo2)
    block2ShearMatrix1 += lfo(block2ShearMatrix1C * gui->block2Geo1Lfo2[0], block2ShearMatrix1Theta, gui->block2Geo1Lfo2Shape[0]);
    block2ShearMatrix2 += lfo(block2ShearMatrix2C * gui->block2Geo1Lfo2[4], block2ShearMatrix2Theta, gui->block2Geo1Lfo2Shape[2]);
    block2ShearMatrix3 += lfo(block2ShearMatrix3C * gui->block2Geo1Lfo2[6], block2ShearMatrix3Theta, gui->block2Geo1Lfo2Shape[3]);
    block2ShearMatrix4 += lfo(block2ShearMatrix4C * gui->block2Geo1Lfo2[2], block2ShearMatrix4Theta, gui->block2Geo1Lfo2Shape[1]);
    block2KaleidoscopeSlice += lfo(block2KaleidoscopeSliceC * gui->block2Geo1Lfo2[8], block2KaleidoscopeSliceTheta, gui->block2Geo1Lfo2Shape[4]);
    
    block3.params.block2ShearMatrix1 = block2ShearMatrix1;
    block3.params.block2ShearMatrix2 = block2ShearMatrix2;
    block3.params.block2ShearMatrix3 = block2ShearMatrix3;
    block3.params.block2ShearMatrix4 = block2ShearMatrix4;
    block3.params.block2KaleidoscopeSlice = block2KaleidoscopeSlice;
    block3.params.block2KaleidoscopeAmount = floor(21.0f * gui->block2Geo[8]);
    
    // ========================================
    // BLOCK 3 - Block2 Colorize (with LFO)
    // ========================================
    // Base values
    float block2ColorizeHueBand1 = gui->block2Colorize[0];
    float block2ColorizeSaturationBand1 = gui->block2Colorize[1];
    float block2ColorizeBrightBand1 = gui->block2Colorize[2];
    float block2ColorizeHueBand2 = gui->block2Colorize[3];
    float block2ColorizeSaturationBand2 = gui->block2Colorize[4];
    float block2ColorizeBrightBand2 = gui->block2Colorize[5];
    float block2ColorizeHueBand3 = gui->block2Colorize[6];
    float block2ColorizeSaturationBand3 = gui->block2Colorize[7];
    float block2ColorizeBrightBand3 = gui->block2Colorize[8];
    float block2ColorizeHueBand4 = gui->block2Colorize[9];
    float block2ColorizeSaturationBand4 = gui->block2Colorize[10];
    float block2ColorizeBrightBand4 = gui->block2Colorize[11];
    float block2ColorizeHueBand5 = gui->block2Colorize[12];
    float block2ColorizeSaturationBand5 = gui->block2Colorize[13];
    float block2ColorizeBrightBand5 = gui->block2Colorize[14];
    
    // Apply LFO (ColorizeLfo1 - bands 1-2)
    block2ColorizeHueBand1 += lfo(gui->block2ColorizeLfo1[0], block2ColorizeHueBand1Theta, gui->block2ColorizeLfo1Shape[0]);
    block2ColorizeSaturationBand1 += lfo(gui->block2ColorizeLfo1[1], block2ColorizeSaturationBand1Theta, gui->block2ColorizeLfo1Shape[1]);
    block2ColorizeBrightBand1 += lfo(gui->block2ColorizeLfo1[2], block2ColorizeBrightBand1Theta, gui->block2ColorizeLfo1Shape[2]);
    block2ColorizeHueBand2 += lfo(gui->block2ColorizeLfo1[6], block2ColorizeHueBand2Theta, gui->block2ColorizeLfo1Shape[3]);
    block2ColorizeSaturationBand2 += lfo(gui->block2ColorizeLfo1[7], block2ColorizeSaturationBand2Theta, gui->block2ColorizeLfo1Shape[4]);
    block2ColorizeBrightBand2 += lfo(gui->block2ColorizeLfo1[8], block2ColorizeBrightBand2Theta, gui->block2ColorizeLfo1Shape[5]);
    
    // Apply LFO (ColorizeLfo2 - bands 3-4)
    block2ColorizeHueBand3 += lfo(gui->block2ColorizeLfo2[0], block2ColorizeHueBand3Theta, gui->block2ColorizeLfo2Shape[0]);
    block2ColorizeSaturationBand3 += lfo(gui->block2ColorizeLfo2[1], block2ColorizeSaturationBand3Theta, gui->block2ColorizeLfo2Shape[1]);
    block2ColorizeBrightBand3 += lfo(gui->block2ColorizeLfo2[2], block2ColorizeBrightBand3Theta, gui->block2ColorizeLfo2Shape[2]);
    block2ColorizeHueBand4 += lfo(gui->block2ColorizeLfo2[6], block2ColorizeHueBand4Theta, gui->block2ColorizeLfo2Shape[3]);
    block2ColorizeSaturationBand4 += lfo(gui->block2ColorizeLfo2[7], block2ColorizeSaturationBand4Theta, gui->block2ColorizeLfo2Shape[4]);
    block2ColorizeBrightBand4 += lfo(gui->block2ColorizeLfo2[8], block2ColorizeBrightBand4Theta, gui->block2ColorizeLfo2Shape[5]);
    
    // Apply LFO (ColorizeLfo3 - band 5)
    block2ColorizeHueBand5 += lfo(gui->block2ColorizeLfo3[0], block2ColorizeHueBand5Theta, gui->block2ColorizeLfo3Shape[0]);
    block2ColorizeSaturationBand5 += lfo(gui->block2ColorizeLfo3[1], block2ColorizeSaturationBand5Theta, gui->block2ColorizeLfo3Shape[1]);
    block2ColorizeBrightBand5 += lfo(gui->block2ColorizeLfo3[2], block2ColorizeBrightBand5Theta, gui->block2ColorizeLfo3Shape[2]);
    
    block3.params.block2ColorizeHueBand1 = block2ColorizeHueBand1;
    block3.params.block2ColorizeSaturationBand1 = block2ColorizeSaturationBand1;
    block3.params.block2ColorizeBrightBand1 = block2ColorizeBrightBand1;
    block3.params.block2ColorizeHueBand2 = block2ColorizeHueBand2;
    block3.params.block2ColorizeSaturationBand2 = block2ColorizeSaturationBand2;
    block3.params.block2ColorizeBrightBand2 = block2ColorizeBrightBand2;
    block3.params.block2ColorizeHueBand3 = block2ColorizeHueBand3;
    block3.params.block2ColorizeSaturationBand3 = block2ColorizeSaturationBand3;
    block3.params.block2ColorizeBrightBand3 = block2ColorizeBrightBand3;
    block3.params.block2ColorizeHueBand4 = block2ColorizeHueBand4;
    block3.params.block2ColorizeSaturationBand4 = block2ColorizeSaturationBand4;
    block3.params.block2ColorizeBrightBand4 = block2ColorizeBrightBand4;
    block3.params.block2ColorizeHueBand5 = block2ColorizeHueBand5;
    block3.params.block2ColorizeSaturationBand5 = block2ColorizeSaturationBand5;
    block3.params.block2ColorizeBrightBand5 = block2ColorizeBrightBand5;
    
    block3.params.block2ColorizeSwitch = gui->block2ColorizeSwitch ? 1 : 0;
    block3.params.block2ColorizeHSB_RGB = gui->block2ColorizeHSB_RGB ? 1 : 0;
    
    // Block2 filters (no LFO)
    block3.params.block2BlurAmount = gui->block2Filters[0];
    block3.params.block2BlurRadius = 9.0f * gui->block2Filters[1] + 1.0f;
    block3.params.block2SharpenAmount = gui->block2Filters[2];
    block3.params.block2SharpenRadius = 9.0f * gui->block2Filters[3] + 1.0f;
    block3.params.block2FiltersBoost = gui->block2Filters[4];
    block3.params.block2Dither = 15.0f * (1.0f - gui->block2Filters[5]) + 2.0f;
    block3.params.block2DitherSwitch = gui->block2Filters[5] > 0 ? 1 : 0;
    block3.params.block2DitherType = gui->block2DitherType;
    
    // Block2 switches
    block3.params.block2HMirror = gui->block2HMirror ? 1 : 0;
    block3.params.block2VMirror = gui->block2VMirror ? 1 : 0;
    block3.params.block2HFlip = gui->block2HFlip ? 1 : 0;
    block3.params.block2VFlip = gui->block2VFlip ? 1 : 0;
    block3.params.block2RotateMode = gui->block2RotateMode ? 1 : 0;
    block3.params.block2GeoOverflow = gui->block2GeoOverflow;
    
    // ========================================
    // BLOCK 3 - Matrix Mixer (with LFO)
    // ========================================
    float matrixMixBgRedIntoFgRed = 6.0f * gui->matrixMix[0];
    float matrixMixBgGreenIntoFgRed = 6.0f * gui->matrixMix[1];
    float matrixMixBgBlueIntoFgRed = 6.0f * gui->matrixMix[2];
    float matrixMixBgRedIntoFgGreen = 6.0f * gui->matrixMix[3];
    float matrixMixBgGreenIntoFgGreen = 6.0f * gui->matrixMix[4];
    float matrixMixBgBlueIntoFgGreen = 6.0f * gui->matrixMix[5];
    float matrixMixBgRedIntoFgBlue = 6.0f * gui->matrixMix[6];
    float matrixMixBgGreenIntoFgBlue = 6.0f * gui->matrixMix[7];
    float matrixMixBgBlueIntoFgBlue = 6.0f * gui->matrixMix[8];
    
    // Apply LFO (MatrixMixLfo1)
    matrixMixBgRedIntoFgRed += lfo(matrixMixC * gui->matrixMixLfo1[0], matrixMixBgRedIntoFgRedTheta, gui->matrixMixLfo1Shape[0]);
    matrixMixBgGreenIntoFgRed += lfo(matrixMixC * gui->matrixMixLfo1[1], matrixMixBgGreenIntoFgRedTheta, gui->matrixMixLfo1Shape[1]);
    matrixMixBgBlueIntoFgRed += lfo(matrixMixC * gui->matrixMixLfo1[2], matrixMixBgBlueIntoFgRedTheta, gui->matrixMixLfo1Shape[2]);
    matrixMixBgRedIntoFgGreen += lfo(matrixMixC * gui->matrixMixLfo1[6], matrixMixBgRedIntoFgGreenTheta, gui->matrixMixLfo1Shape[3]);
    matrixMixBgGreenIntoFgGreen += lfo(matrixMixC * gui->matrixMixLfo1[7], matrixMixBgGreenIntoFgGreenTheta, gui->matrixMixLfo1Shape[4]);
    matrixMixBgBlueIntoFgGreen += lfo(matrixMixC * gui->matrixMixLfo1[8], matrixMixBgBlueIntoFgGreenTheta, gui->matrixMixLfo1Shape[5]);
    
    // Apply LFO (MatrixMixLfo2)
    matrixMixBgRedIntoFgBlue += lfo(matrixMixC * gui->matrixMixLfo2[0], matrixMixBgRedIntoFgBlueTheta, gui->matrixMixLfo2Shape[0]);
    matrixMixBgGreenIntoFgBlue += lfo(matrixMixC * gui->matrixMixLfo2[1], matrixMixBgGreenIntoFgBlueTheta, gui->matrixMixLfo2Shape[1]);
    matrixMixBgBlueIntoFgBlue += lfo(matrixMixC * gui->matrixMixLfo2[2], matrixMixBgBlueIntoFgBlueTheta, gui->matrixMixLfo2Shape[2]);
    
    block3.params.matrixMixBgRedIntoFgRed = matrixMixBgRedIntoFgRed;
    block3.params.matrixMixBgGreenIntoFgRed = matrixMixBgGreenIntoFgRed;
    block3.params.matrixMixBgBlueIntoFgRed = matrixMixBgBlueIntoFgRed;
    block3.params.matrixMixBgRedIntoFgGreen = matrixMixBgRedIntoFgGreen;
    block3.params.matrixMixBgGreenIntoFgGreen = matrixMixBgGreenIntoFgGreen;
    block3.params.matrixMixBgBlueIntoFgGreen = matrixMixBgBlueIntoFgGreen;
    block3.params.matrixMixBgRedIntoFgBlue = matrixMixBgRedIntoFgBlue;
    block3.params.matrixMixBgGreenIntoFgBlue = matrixMixBgGreenIntoFgBlue;
    block3.params.matrixMixBgBlueIntoFgBlue = matrixMixBgBlueIntoFgBlue;
    
    block3.params.matrixMixType = gui->matrixMixType;
    block3.params.matrixMixOverflow = gui->matrixMixOverflow;
    
    // ========================================
    // BLOCK 3 - Final Mix (with LFO)
    // ========================================
    float finalMixAmount = 2.0f * gui->finalMixAndKey[0];
    float finalKeyThreshold = (ROOT_THREE + 0.001f) * gui->finalMixAndKey[4];
    float finalKeySoft = gui->finalMixAndKey[5];
    
    // Apply LFO
    finalMixAmount += lfo(mixAmountC * gui->finalMixAndKeyLfo[0], finalMixAmountTheta, gui->finalMixAndKeyLfoShape[0]);
    finalKeyThreshold += lfo(keyThresholdC * gui->finalMixAndKeyLfo[2], finalKeyThresholdTheta, gui->finalMixAndKeyLfoShape[1]);
    finalKeySoft += lfo(gui->finalMixAndKeyLfo[4], finalKeySoftTheta, gui->finalMixAndKeyLfoShape[2]);
    
    block3.params.finalMixAmount = finalMixAmount;
    block3.params.finalKeyThreshold = finalKeyThreshold;
    block3.params.finalKeySoft = finalKeySoft;
    block3.params.finalKeyValueRed = gui->finalMixAndKey[1];
    block3.params.finalKeyValueGreen = gui->finalMixAndKey[2];
    block3.params.finalKeyValueBlue = gui->finalMixAndKey[3];
    block3.params.finalKeyOrder = gui->finalKeyOrder;
    block3.params.finalMixType = gui->finalMixType;
    block3.params.finalMixOverflow = gui->finalMixOverflow;
    
    // Draw mode
    pipeline->setDrawMode((PipelineManager::DrawMode)gui->drawMode);
    
    // NDI/Spout enable
    outputManager->setNdiBlock3Enabled(gui->ndiSendBlock3);
#if OFAPP_HAS_SPOUT
    outputManager->setSpoutBlock3Enabled(gui->spoutSendBlock3);
#endif
}

//--------------------------------------------------------------
void ofApp::drawGeometryPatterns() {
    if (!geometryManager || !pipeline || !gui) return;
    
    // Check if any geometry is enabled
    bool hasGeometry = gui->block1LineSwitch || gui->block1SevenStarSwitch || 
                       gui->block1LissaBallSwitch || gui->block1HypercubeSwitch ||
                       gui->block1LissajousCurveSwitch;
    
    if (!hasGeometry) {
        // Disable all geometry
        geometryManager->getHypercube().setEnabled(false);
        geometryManager->getLine().setEnabled(false);
        geometryManager->getSevenStar().setEnabled(false);
        geometryManager->getSpiralEllipse().setEnabled(false);
        geometryManager->getLissajous1().setEnabled(false);
        return;
    }
    
    // Block 1 geometry params
    if (gui->block1HypercubeSwitch) {
        auto& pattern = geometryManager->getHypercube();
        pattern.setEnabled(true);
        pattern.thetaRate = gui->hypercube_theta_rate;
        pattern.phiRate = gui->hypercube_phi_rate;
        pattern.size = gui->hypercube_size;
    } else {
        geometryManager->getHypercube().setEnabled(false);
    }
    
    if (gui->block1LineSwitch) {
        geometryManager->getLine().setEnabled(true);
    } else {
        geometryManager->getLine().setEnabled(false);
    }
    
    if (gui->block1SevenStarSwitch) {
        geometryManager->getSevenStar().setEnabled(true);
    } else {
        geometryManager->getSevenStar().setEnabled(false);
    }
    
    if (gui->block1LissaBallSwitch) {
        geometryManager->getSpiralEllipse().setEnabled(true);
    } else {
        geometryManager->getSpiralEllipse().setEnabled(false);
    }
    
    if (gui->block1LissajousCurveSwitch) {
        auto& pattern = geometryManager->getLissajous1();
        pattern.setEnabled(true);
        pattern.xFreq = gui->lissajous1XFreq;
        pattern.yFreq = gui->lissajous1YFreq;
        pattern.speed = gui->lissajous1Speed;
        pattern.size = gui->lissajous1Size;
    } else {
        geometryManager->getLissajous1().setEnabled(false);
    }
    
    // Save current graphics state
    ofPushStyle();
    ofPushView();
    
    // Unbind any textures to prevent FBO self-binding issues
    for (int i = 0; i < 8; i++) {
        glActiveTexture(GL_TEXTURE0 + i);
        glBindTexture(GL_TEXTURE_2D, 0);
    }
    glActiveTexture(GL_TEXTURE0);
    
    // Draw on Block 1
    int w = pipeline->getBlock1Fbo().getWidth();
    int h = pipeline->getBlock1Fbo().getHeight();
    pipeline->getBlock1Fbo().begin();
    {
        ofClear(0, 0, 0, 0);
        ofSetupScreenPerspective(w, h);
        geometryManager->drawPatterns(w, h);
    }
    pipeline->getBlock1Fbo().end();
    
    // Restore graphics state
    ofPopView();
    ofPopStyle();
}

//--------------------------------------------------------------
void ofApp::sendOutputs() {
    if (!outputManager) return;
    
    outputManager->sendBlock3(pipeline->getFinalOutput());
}

//--------------------------------------------------------------
void ofApp::drawOutput() {
    if (!pipeline || !gui) return;
    
    ofSetupScreen();
    
    switch (gui->drawMode) {
        case 0:
            pipeline->getBlock1Output().draw(0, 0, ofGetWidth(), ofGetHeight());
            break;
        case 1:
            pipeline->getBlock2Output().draw(0, 0, ofGetWidth(), ofGetHeight());
            break;
        case 2:
            pipeline->getFinalOutput().draw(0, 0, ofGetWidth(), ofGetHeight());
            break;
        case 3:
            pipeline->getBlock1Output().draw(0, 0, ofGetWidth()/2, ofGetHeight()/2);
            pipeline->getBlock2Output().draw(ofGetWidth()/2, 0, ofGetWidth()/2, ofGetHeight()/2);
            pipeline->getFinalOutput().draw(0, ofGetHeight()/2, ofGetWidth()/2, ofGetHeight()/2);
            break;
    }
}

//--------------------------------------------------------------
void ofApp::clearFramebuffers() {
    if (!pipeline) return;
    
    // Only clear feedback buffers when explicitly requested
    // Don't clear the main FBOs here - they contain the rendered output!
    if (gui && gui->fb1FramebufferClearSwitch) {
        pipeline->clearFB1();
        gui->fb1FramebufferClearSwitch = false;
        ofLogNotice("ofApp") << "FB1 feedback buffer cleared";
    }
    if (gui && gui->fb2FramebufferClearSwitch) {
        pipeline->clearFB2();
        gui->fb2FramebufferClearSwitch = false;
        ofLogNotice("ofApp") << "FB2 feedback buffer cleared";
    }
}

//--------------------------------------------------------------
void ofApp::reinitializeInputs() {
    if (!gui) return;
    
    ofLogNotice("ofApp") << "Reinitializing video inputs...";
    
    // Configure Input 1 based on GUI settings
    InputType type1 = (InputType)gui->input1SourceType;
    int deviceOrIndex1 = 0;
    
    switch (type1) {
        case InputType::WEBCAM:
            deviceOrIndex1 = gui->input1DeviceID;
            ofLogNotice("ofApp") << "Input 1: Webcam Device " << deviceOrIndex1;
            break;
        case InputType::NDI:
            deviceOrIndex1 = gui->input1NdiSourceIndex;
            ofLogNotice("ofApp") << "Input 1: NDI Source Index " << deviceOrIndex1;
            break;
#if OFAPP_HAS_SPOUT
        case InputType::SPOUT:
            deviceOrIndex1 = gui->input1SpoutSourceIndex;
            ofLogNotice("ofApp") << "Input 1: Spout Source Index " << deviceOrIndex1;
            break;
#endif
        case InputType::VIDEO_FILE:
            ofLogNotice("ofApp") << "Input 1: Video File (not yet implemented)";
            break;
        default:
            break;
    }
    
    inputManager->configureInput1(type1, deviceOrIndex1);
    
    // Configure Input 2 based on GUI settings
    InputType type2 = (InputType)gui->input2SourceType;
    int deviceOrIndex2 = 0;
    
    switch (type2) {
        case InputType::WEBCAM:
            deviceOrIndex2 = gui->input2DeviceID;
            ofLogNotice("ofApp") << "Input 2: Webcam Device " << deviceOrIndex2;
            break;
        case InputType::NDI:
            deviceOrIndex2 = gui->input2NdiSourceIndex;
            ofLogNotice("ofApp") << "Input 2: NDI Source Index " << deviceOrIndex2;
            break;
#if OFAPP_HAS_SPOUT
        case InputType::SPOUT:
            deviceOrIndex2 = gui->input2SpoutSourceIndex;
            ofLogNotice("ofApp") << "Input 2: Spout Source Index " << deviceOrIndex2;
            break;
#endif
        case InputType::VIDEO_FILE:
            ofLogNotice("ofApp") << "Input 2: Video File (not yet implemented)";
            break;
        default:
            break;
    }
    
    inputManager->configureInput2(type2, deviceOrIndex2);
    
    // Save input settings to XML for persistence
    auto& settings = SettingsManager::getInstance();
    auto& inputSettings = settings.getInputSources();
    
    inputSettings.input1SourceType = gui->input1SourceType;
    inputSettings.input2SourceType = gui->input2SourceType;
    inputSettings.input1DeviceID = gui->input1DeviceID;
    inputSettings.input2DeviceID = gui->input2DeviceID;
    inputSettings.input1NdiSourceIndex = gui->input1NdiSourceIndex;
    inputSettings.input2NdiSourceIndex = gui->input2NdiSourceIndex;
#if OFAPP_HAS_SPOUT
    inputSettings.input1SpoutSourceIndex = gui->input1SpoutSourceIndex;
    inputSettings.input2SpoutSourceIndex = gui->input2SpoutSourceIndex;
#endif
    
    settings.save();
    ofLogNotice("ofApp") << "Input settings saved to config.json";
    
    // Also save GUI's JSON settings to keep them in sync
    if (gui) {
        gui->saveVideoOscSettings();
        ofLogNotice("ofApp") << "GUI settings saved to settings.json";
    }
}

//--------------------------------------------------------------
void ofApp::applyResolutionChange() {
    if (!gui) return;
    
    auto& settings = SettingsManager::getInstance();
    DisplaySettings newSettings = settings.getDisplay();
    
    newSettings.input1Width = gui->input1Width;
    newSettings.input1Height = gui->input1Height;
    newSettings.input2Width = gui->input2Width;
    newSettings.input2Height = gui->input2Height;
    newSettings.internalWidth = gui->internalWidth;
    newSettings.internalHeight = gui->internalHeight;
    newSettings.outputWidth = gui->outputWidth;
    newSettings.outputHeight = gui->outputHeight;
    newSettings.ndiSendWidth = gui->ndiSendWidth;
    newSettings.ndiSendHeight = gui->ndiSendHeight;
    
    settings.applyDisplaySettings(newSettings);
    
    // Reinitialize everything
    inputManager->reinitialize(newSettings);
    pipeline->reinitialize(newSettings);
    outputManager->reinitialize(newSettings);
    
    settings.save();
}

//--------------------------------------------------------------
void ofApp::updateLfos() {
    if (!gui) return;
    
    // Get beat division values for tempo sync
    const float* divValues = dragonwaves::BeatDivisionValues;
    
    // ========================================
    // BLOCK 1 LFO Theta Updates
    // ========================================
    
    // Channel 1 adjust LFO - only update if amplitude is non-zero
    if (gui->ch1AdjustLfo[0] != 0.0f) ch1XDisplaceTheta += lfoRateC * getEffectiveLfoRate(&gui->ch1AdjustLfoSync[1], &gui->ch1AdjustLfoDivision[1], gui->ch1AdjustLfo[1], divValues);
    if (gui->ch1AdjustLfo[2] != 0.0f) ch1YDisplaceTheta += lfoRateC * getEffectiveLfoRate(&gui->ch1AdjustLfoSync[3], &gui->ch1AdjustLfoDivision[3], gui->ch1AdjustLfo[3], divValues);
    if (gui->ch1AdjustLfo[4] != 0.0f) ch1ZDisplaceTheta += lfoRateC * getEffectiveLfoRate(&gui->ch1AdjustLfoSync[5], &gui->ch1AdjustLfoDivision[5], gui->ch1AdjustLfo[5], divValues);
    if (gui->ch1AdjustLfo[6] != 0.0f) ch1RotateTheta += lfoRateC * getEffectiveLfoRate(&gui->ch1AdjustLfoSync[7], &gui->ch1AdjustLfoDivision[7], gui->ch1AdjustLfo[7], divValues);
    if (gui->ch1AdjustLfo[8] != 0.0f) ch1HueAttenuateTheta += lfoRateC * getEffectiveLfoRate(&gui->ch1AdjustLfoSync[9], &gui->ch1AdjustLfoDivision[9], gui->ch1AdjustLfo[9], divValues);
    if (gui->ch1AdjustLfo[10] != 0.0f) ch1SaturationAttenuateTheta += lfoRateC * getEffectiveLfoRate(&gui->ch1AdjustLfoSync[11], &gui->ch1AdjustLfoDivision[11], gui->ch1AdjustLfo[11], divValues);
    if (gui->ch1AdjustLfo[12] != 0.0f) ch1BrightAttenuateTheta += lfoRateC * getEffectiveLfoRate(&gui->ch1AdjustLfoSync[13], &gui->ch1AdjustLfoDivision[13], gui->ch1AdjustLfo[13], divValues);
    if (gui->ch1AdjustLfo[14] != 0.0f) ch1KaleidoscopeSliceTheta += lfoRateC * getEffectiveLfoRate(&gui->ch1AdjustLfoSync[15], &gui->ch1AdjustLfoDivision[15], gui->ch1AdjustLfo[15], divValues);
    
    // Channel 2 mix and key LFO
    if (gui->ch2MixAndKeyLfo[0] != 0.0f) ch2MixAmountTheta += lfoRateC * getEffectiveLfoRate(&gui->ch2MixAndKeyLfoSync[1], &gui->ch2MixAndKeyLfoDivision[1], gui->ch2MixAndKeyLfo[1], divValues);
    if (gui->ch2MixAndKeyLfo[2] != 0.0f) ch2KeyThresholdTheta += lfoRateC * getEffectiveLfoRate(&gui->ch2MixAndKeyLfoSync[3], &gui->ch2MixAndKeyLfoDivision[3], gui->ch2MixAndKeyLfo[3], divValues);
    if (gui->ch2MixAndKeyLfo[4] != 0.0f) ch2KeySoftTheta += lfoRateC * getEffectiveLfoRate(&gui->ch2MixAndKeyLfoSync[5], &gui->ch2MixAndKeyLfoDivision[5], gui->ch2MixAndKeyLfo[5], divValues);
    
    // Channel 2 adjust LFO
    if (gui->ch2AdjustLfo[0] != 0.0f) ch2XDisplaceTheta += lfoRateC * getEffectiveLfoRate(&gui->ch2AdjustLfoSync[1], &gui->ch2AdjustLfoDivision[1], gui->ch2AdjustLfo[1], divValues);
    if (gui->ch2AdjustLfo[2] != 0.0f) ch2YDisplaceTheta += lfoRateC * getEffectiveLfoRate(&gui->ch2AdjustLfoSync[3], &gui->ch2AdjustLfoDivision[3], gui->ch2AdjustLfo[3], divValues);
    if (gui->ch2AdjustLfo[4] != 0.0f) ch2ZDisplaceTheta += lfoRateC * getEffectiveLfoRate(&gui->ch2AdjustLfoSync[5], &gui->ch2AdjustLfoDivision[5], gui->ch2AdjustLfo[5], divValues);
    if (gui->ch2AdjustLfo[6] != 0.0f) ch2RotateTheta += lfoRateC * getEffectiveLfoRate(&gui->ch2AdjustLfoSync[7], &gui->ch2AdjustLfoDivision[7], gui->ch2AdjustLfo[7], divValues);
    if (gui->ch2AdjustLfo[8] != 0.0f) ch2HueAttenuateTheta += lfoRateC * getEffectiveLfoRate(&gui->ch2AdjustLfoSync[9], &gui->ch2AdjustLfoDivision[9], gui->ch2AdjustLfo[9], divValues);
    if (gui->ch2AdjustLfo[10] != 0.0f) ch2SaturationAttenuateTheta += lfoRateC * getEffectiveLfoRate(&gui->ch2AdjustLfoSync[11], &gui->ch2AdjustLfoDivision[11], gui->ch2AdjustLfo[11], divValues);
    if (gui->ch2AdjustLfo[12] != 0.0f) ch2BrightAttenuateTheta += lfoRateC * getEffectiveLfoRate(&gui->ch2AdjustLfoSync[13], &gui->ch2AdjustLfoDivision[13], gui->ch2AdjustLfo[13], divValues);
    if (gui->ch2AdjustLfo[14] != 0.0f) ch2KaleidoscopeSliceTheta += lfoRateC * getEffectiveLfoRate(&gui->ch2AdjustLfoSync[15], &gui->ch2AdjustLfoDivision[15], gui->ch2AdjustLfo[15], divValues);
    
    // FB1 mix and key LFO
    if (gui->fb1MixAndKeyLfo[0] != 0.0f) fb1MixAmountTheta += lfoRateC * getEffectiveLfoRate(&gui->fb1MixAndKeyLfoSync[1], &gui->fb1MixAndKeyLfoDivision[1], gui->fb1MixAndKeyLfo[1], divValues);
    if (gui->fb1MixAndKeyLfo[2] != 0.0f) fb1KeyThresholdTheta += lfoRateC * getEffectiveLfoRate(&gui->fb1MixAndKeyLfoSync[3], &gui->fb1MixAndKeyLfoDivision[3], gui->fb1MixAndKeyLfo[3], divValues);
    if (gui->fb1MixAndKeyLfo[4] != 0.0f) fb1KeySoftTheta += lfoRateC * getEffectiveLfoRate(&gui->fb1MixAndKeyLfoSync[5], &gui->fb1MixAndKeyLfoDivision[5], gui->fb1MixAndKeyLfo[5], divValues);
    
    // FB1 geo1 LFO (first set)
    if (gui->fb1Geo1Lfo1[0] != 0.0f) fb1XDisplaceTheta += lfoRateC * getEffectiveLfoRate(&gui->fb1Geo1Lfo1Sync[1], &gui->fb1Geo1Lfo1Division[1], gui->fb1Geo1Lfo1[1], divValues);
    if (gui->fb1Geo1Lfo1[2] != 0.0f) fb1YDisplaceTheta += lfoRateC * getEffectiveLfoRate(&gui->fb1Geo1Lfo1Sync[3], &gui->fb1Geo1Lfo1Division[3], gui->fb1Geo1Lfo1[3], divValues);
    if (gui->fb1Geo1Lfo1[4] != 0.0f) fb1ZDisplaceTheta += lfoRateC * getEffectiveLfoRate(&gui->fb1Geo1Lfo1Sync[5], &gui->fb1Geo1Lfo1Division[5], gui->fb1Geo1Lfo1[5], divValues);
    if (gui->fb1Geo1Lfo1[6] != 0.0f) fb1RotateTheta += lfoRateC * getEffectiveLfoRate(&gui->fb1Geo1Lfo1Sync[7], &gui->fb1Geo1Lfo1Division[7], gui->fb1Geo1Lfo1[7], divValues);
    
    // FB1 geo1 LFO (second set - shear matrix and kaleidoscope)
    if (gui->fb1Geo1Lfo2[0] != 0.0f) fb1ShearMatrix1Theta += lfoRateC * getEffectiveLfoRate(&gui->fb1Geo1Lfo2Sync[1], &gui->fb1Geo1Lfo2Division[1], gui->fb1Geo1Lfo2[1], divValues);
    if (gui->fb1Geo1Lfo2[4] != 0.0f) fb1ShearMatrix2Theta += lfoRateC * getEffectiveLfoRate(&gui->fb1Geo1Lfo2Sync[5], &gui->fb1Geo1Lfo2Division[5], gui->fb1Geo1Lfo2[5], divValues);
    if (gui->fb1Geo1Lfo2[6] != 0.0f) fb1ShearMatrix3Theta += lfoRateC * getEffectiveLfoRate(&gui->fb1Geo1Lfo2Sync[7], &gui->fb1Geo1Lfo2Division[7], gui->fb1Geo1Lfo2[7], divValues);
    if (gui->fb1Geo1Lfo2[2] != 0.0f) fb1ShearMatrix4Theta += lfoRateC * getEffectiveLfoRate(&gui->fb1Geo1Lfo2Sync[3], &gui->fb1Geo1Lfo2Division[3], gui->fb1Geo1Lfo2[3], divValues);
    if (gui->fb1Geo1Lfo2[8] != 0.0f) fb1KaleidoscopeSliceTheta += lfoRateC * getEffectiveLfoRate(&gui->fb1Geo1Lfo2Sync[9], &gui->fb1Geo1Lfo2Division[9], gui->fb1Geo1Lfo2[9], divValues);
    
    // FB1 color LFO
    if (gui->fb1Color1Lfo1[0] != 0.0f) fb1HueAttenuateTheta += lfoRateC * getEffectiveLfoRate(&gui->fb1Color1Lfo1Sync[1], &gui->fb1Color1Lfo1Division[1], gui->fb1Color1Lfo1[1], divValues);
    if (gui->fb1Color1Lfo1[2] != 0.0f) fb1SaturationAttenuateTheta += lfoRateC * getEffectiveLfoRate(&gui->fb1Color1Lfo1Sync[3], &gui->fb1Color1Lfo1Division[3], gui->fb1Color1Lfo1[3], divValues);
    if (gui->fb1Color1Lfo1[4] != 0.0f) fb1BrightAttenuateTheta += lfoRateC * getEffectiveLfoRate(&gui->fb1Color1Lfo1Sync[5], &gui->fb1Color1Lfo1Division[5], gui->fb1Color1Lfo1[5], divValues);
    
    // ========================================
    // BLOCK 2 LFO Theta Updates
    // ========================================
    
    // Block2 input adjust LFO
    if (gui->block2InputAdjustLfo[0] != 0.0f) block2InputXDisplaceTheta += lfoRateC * getEffectiveLfoRate(&gui->block2InputAdjustLfoSync[1], &gui->block2InputAdjustLfoDivision[1], gui->block2InputAdjustLfo[1], divValues);
    if (gui->block2InputAdjustLfo[2] != 0.0f) block2InputYDisplaceTheta += lfoRateC * getEffectiveLfoRate(&gui->block2InputAdjustLfoSync[3], &gui->block2InputAdjustLfoDivision[3], gui->block2InputAdjustLfo[3], divValues);
    if (gui->block2InputAdjustLfo[4] != 0.0f) block2InputZDisplaceTheta += lfoRateC * getEffectiveLfoRate(&gui->block2InputAdjustLfoSync[5], &gui->block2InputAdjustLfoDivision[5], gui->block2InputAdjustLfo[5], divValues);
    if (gui->block2InputAdjustLfo[6] != 0.0f) block2InputRotateTheta += lfoRateC * getEffectiveLfoRate(&gui->block2InputAdjustLfoSync[7], &gui->block2InputAdjustLfoDivision[7], gui->block2InputAdjustLfo[7], divValues);
    if (gui->block2InputAdjustLfo[8] != 0.0f) block2InputHueAttenuateTheta += lfoRateC * getEffectiveLfoRate(&gui->block2InputAdjustLfoSync[9], &gui->block2InputAdjustLfoDivision[9], gui->block2InputAdjustLfo[9], divValues);
    if (gui->block2InputAdjustLfo[10] != 0.0f) block2InputSaturationAttenuateTheta += lfoRateC * getEffectiveLfoRate(&gui->block2InputAdjustLfoSync[11], &gui->block2InputAdjustLfoDivision[11], gui->block2InputAdjustLfo[11], divValues);
    if (gui->block2InputAdjustLfo[12] != 0.0f) block2InputBrightAttenuateTheta += lfoRateC * getEffectiveLfoRate(&gui->block2InputAdjustLfoSync[13], &gui->block2InputAdjustLfoDivision[13], gui->block2InputAdjustLfo[13], divValues);
    if (gui->block2InputAdjustLfo[14] != 0.0f) block2InputKaleidoscopeSliceTheta += lfoRateC * getEffectiveLfoRate(&gui->block2InputAdjustLfoSync[15], &gui->block2InputAdjustLfoDivision[15], gui->block2InputAdjustLfo[15], divValues);
    
    // FB2 mix and key LFO
    if (gui->fb2MixAndKeyLfo[0] != 0.0f) fb2MixAmountTheta += lfoRateC * getEffectiveLfoRate(&gui->fb2MixAndKeyLfoSync[1], &gui->fb2MixAndKeyLfoDivision[1], gui->fb2MixAndKeyLfo[1], divValues);
    if (gui->fb2MixAndKeyLfo[2] != 0.0f) fb2KeyThresholdTheta += lfoRateC * getEffectiveLfoRate(&gui->fb2MixAndKeyLfoSync[3], &gui->fb2MixAndKeyLfoDivision[3], gui->fb2MixAndKeyLfo[3], divValues);
    if (gui->fb2MixAndKeyLfo[4] != 0.0f) fb2KeySoftTheta += lfoRateC * getEffectiveLfoRate(&gui->fb2MixAndKeyLfoSync[5], &gui->fb2MixAndKeyLfoDivision[5], gui->fb2MixAndKeyLfo[5], divValues);
    
    // FB2 geo1 LFO (first set)
    if (gui->fb2Geo1Lfo1[0] != 0.0f) fb2XDisplaceTheta += lfoRateC * getEffectiveLfoRate(&gui->fb2Geo1Lfo1Sync[1], &gui->fb2Geo1Lfo1Division[1], gui->fb2Geo1Lfo1[1], divValues);
    if (gui->fb2Geo1Lfo1[2] != 0.0f) fb2YDisplaceTheta += lfoRateC * getEffectiveLfoRate(&gui->fb2Geo1Lfo1Sync[3], &gui->fb2Geo1Lfo1Division[3], gui->fb2Geo1Lfo1[3], divValues);
    if (gui->fb2Geo1Lfo1[4] != 0.0f) fb2ZDisplaceTheta += lfoRateC * getEffectiveLfoRate(&gui->fb2Geo1Lfo1Sync[5], &gui->fb2Geo1Lfo1Division[5], gui->fb2Geo1Lfo1[5], divValues);
    if (gui->fb2Geo1Lfo1[6] != 0.0f) fb2RotateTheta += lfoRateC * getEffectiveLfoRate(&gui->fb2Geo1Lfo1Sync[7], &gui->fb2Geo1Lfo1Division[7], gui->fb2Geo1Lfo1[7], divValues);
    
    // FB2 geo1 LFO (second set - shear matrix and kaleidoscope)
    if (gui->fb2Geo1Lfo2[0] != 0.0f) fb2ShearMatrix1Theta += lfoRateC * getEffectiveLfoRate(&gui->fb2Geo1Lfo2Sync[1], &gui->fb2Geo1Lfo2Division[1], gui->fb2Geo1Lfo2[1], divValues);
    if (gui->fb2Geo1Lfo2[4] != 0.0f) fb2ShearMatrix2Theta += lfoRateC * getEffectiveLfoRate(&gui->fb2Geo1Lfo2Sync[5], &gui->fb2Geo1Lfo2Division[5], gui->fb2Geo1Lfo2[5], divValues);
    if (gui->fb2Geo1Lfo2[6] != 0.0f) fb2ShearMatrix3Theta += lfoRateC * getEffectiveLfoRate(&gui->fb2Geo1Lfo2Sync[7], &gui->fb2Geo1Lfo2Division[7], gui->fb2Geo1Lfo2[7], divValues);
    if (gui->fb2Geo1Lfo2[2] != 0.0f) fb2ShearMatrix4Theta += lfoRateC * getEffectiveLfoRate(&gui->fb2Geo1Lfo2Sync[3], &gui->fb2Geo1Lfo2Division[3], gui->fb2Geo1Lfo2[3], divValues);
    if (gui->fb2Geo1Lfo2[8] != 0.0f) fb2KaleidoscopeSliceTheta += lfoRateC * getEffectiveLfoRate(&gui->fb2Geo1Lfo2Sync[9], &gui->fb2Geo1Lfo2Division[9], gui->fb2Geo1Lfo2[9], divValues);
    
    // FB2 color LFO
    if (gui->fb2Color1Lfo1[0] != 0.0f) fb2HueAttenuateTheta += lfoRateC * getEffectiveLfoRate(&gui->fb2Color1Lfo1Sync[1], &gui->fb2Color1Lfo1Division[1], gui->fb2Color1Lfo1[1], divValues);
    if (gui->fb2Color1Lfo1[2] != 0.0f) fb2SaturationAttenuateTheta += lfoRateC * getEffectiveLfoRate(&gui->fb2Color1Lfo1Sync[3], &gui->fb2Color1Lfo1Division[3], gui->fb2Color1Lfo1[3], divValues);
    if (gui->fb2Color1Lfo1[4] != 0.0f) fb2BrightAttenuateTheta += lfoRateC * getEffectiveLfoRate(&gui->fb2Color1Lfo1Sync[5], &gui->fb2Color1Lfo1Division[5], gui->fb2Color1Lfo1[5], divValues);
    
    // ========================================
    // BLOCK 3 LFO Theta Updates
    // ========================================
    
    // Block1 geo LFO (first set)
    if (gui->block1Geo1Lfo1[0] != 0.0f) block1XDisplaceTheta += lfoRateC * getEffectiveLfoRate(&gui->block1Geo1Lfo1Sync[1], &gui->block1Geo1Lfo1Division[1], gui->block1Geo1Lfo1[1], divValues);
    if (gui->block1Geo1Lfo1[2] != 0.0f) block1YDisplaceTheta += lfoRateC * getEffectiveLfoRate(&gui->block1Geo1Lfo1Sync[3], &gui->block1Geo1Lfo1Division[3], gui->block1Geo1Lfo1[3], divValues);
    if (gui->block1Geo1Lfo1[4] != 0.0f) block1ZDisplaceTheta += lfoRateC * getEffectiveLfoRate(&gui->block1Geo1Lfo1Sync[5], &gui->block1Geo1Lfo1Division[5], gui->block1Geo1Lfo1[5], divValues);
    if (gui->block1Geo1Lfo1[6] != 0.0f) block1RotateTheta += lfoRateC * getEffectiveLfoRate(&gui->block1Geo1Lfo1Sync[7], &gui->block1Geo1Lfo1Division[7], gui->block1Geo1Lfo1[7], divValues);
    
    // Block1 geo LFO (second set - shear matrix and kaleidoscope)
    if (gui->block1Geo1Lfo2[0] != 0.0f) block1ShearMatrix1Theta += lfoRateC * getEffectiveLfoRate(&gui->block1Geo1Lfo2Sync[1], &gui->block1Geo1Lfo2Division[1], gui->block1Geo1Lfo2[1], divValues);
    if (gui->block1Geo1Lfo2[4] != 0.0f) block1ShearMatrix2Theta += lfoRateC * getEffectiveLfoRate(&gui->block1Geo1Lfo2Sync[5], &gui->block1Geo1Lfo2Division[5], gui->block1Geo1Lfo2[5], divValues);
    if (gui->block1Geo1Lfo2[6] != 0.0f) block1ShearMatrix3Theta += lfoRateC * getEffectiveLfoRate(&gui->block1Geo1Lfo2Sync[7], &gui->block1Geo1Lfo2Division[7], gui->block1Geo1Lfo2[7], divValues);
    if (gui->block1Geo1Lfo2[2] != 0.0f) block1ShearMatrix4Theta += lfoRateC * getEffectiveLfoRate(&gui->block1Geo1Lfo2Sync[3], &gui->block1Geo1Lfo2Division[3], gui->block1Geo1Lfo2[3], divValues);
    if (gui->block1Geo1Lfo2[8] != 0.0f) block1KaleidoscopeSliceTheta += lfoRateC * getEffectiveLfoRate(&gui->block1Geo1Lfo2Sync[9], &gui->block1Geo1Lfo2Division[9], gui->block1Geo1Lfo2[9], divValues);
    
    // Block1 colorize LFO (bands 1-2) - indices 0,1,2,6,7,8 are amplitudes
    if (gui->block1ColorizeLfo1[0] != 0.0f) block1ColorizeHueBand1Theta += lfoRateC * getEffectiveLfoRate(&gui->block1ColorizeLfo1Sync[3], &gui->block1ColorizeLfo1Division[3], gui->block1ColorizeLfo1[3], divValues);
    if (gui->block1ColorizeLfo1[1] != 0.0f) block1ColorizeSaturationBand1Theta += lfoRateC * getEffectiveLfoRate(&gui->block1ColorizeLfo1Sync[4], &gui->block1ColorizeLfo1Division[4], gui->block1ColorizeLfo1[4], divValues);
    if (gui->block1ColorizeLfo1[2] != 0.0f) block1ColorizeBrightBand1Theta += lfoRateC * getEffectiveLfoRate(&gui->block1ColorizeLfo1Sync[5], &gui->block1ColorizeLfo1Division[5], gui->block1ColorizeLfo1[5], divValues);
    if (gui->block1ColorizeLfo1[6] != 0.0f) block1ColorizeHueBand2Theta += lfoRateC * getEffectiveLfoRate(&gui->block1ColorizeLfo1Sync[9], &gui->block1ColorizeLfo1Division[9], gui->block1ColorizeLfo1[9], divValues);
    if (gui->block1ColorizeLfo1[7] != 0.0f) block1ColorizeSaturationBand2Theta += lfoRateC * getEffectiveLfoRate(&gui->block1ColorizeLfo1Sync[10], &gui->block1ColorizeLfo1Division[10], gui->block1ColorizeLfo1[10], divValues);
    if (gui->block1ColorizeLfo1[8] != 0.0f) block1ColorizeBrightBand2Theta += lfoRateC * getEffectiveLfoRate(&gui->block1ColorizeLfo1Sync[11], &gui->block1ColorizeLfo1Division[11], gui->block1ColorizeLfo1[11], divValues);
    
    // Block1 colorize LFO (bands 3-4) - indices 0,1,2,6,7,8 are amplitudes
    if (gui->block1ColorizeLfo2[0] != 0.0f) block1ColorizeHueBand3Theta += lfoRateC * getEffectiveLfoRate(&gui->block1ColorizeLfo2Sync[3], &gui->block1ColorizeLfo2Division[3], gui->block1ColorizeLfo2[3], divValues);
    if (gui->block1ColorizeLfo2[1] != 0.0f) block1ColorizeSaturationBand3Theta += lfoRateC * getEffectiveLfoRate(&gui->block1ColorizeLfo2Sync[4], &gui->block1ColorizeLfo2Division[4], gui->block1ColorizeLfo2[4], divValues);
    if (gui->block1ColorizeLfo2[2] != 0.0f) block1ColorizeBrightBand3Theta += lfoRateC * getEffectiveLfoRate(&gui->block1ColorizeLfo2Sync[5], &gui->block1ColorizeLfo2Division[5], gui->block1ColorizeLfo2[5], divValues);
    if (gui->block1ColorizeLfo2[6] != 0.0f) block1ColorizeHueBand4Theta += lfoRateC * getEffectiveLfoRate(&gui->block1ColorizeLfo2Sync[9], &gui->block1ColorizeLfo2Division[9], gui->block1ColorizeLfo2[9], divValues);
    if (gui->block1ColorizeLfo2[7] != 0.0f) block1ColorizeSaturationBand4Theta += lfoRateC * getEffectiveLfoRate(&gui->block1ColorizeLfo2Sync[10], &gui->block1ColorizeLfo2Division[10], gui->block1ColorizeLfo2[10], divValues);
    if (gui->block1ColorizeLfo2[8] != 0.0f) block1ColorizeBrightBand4Theta += lfoRateC * getEffectiveLfoRate(&gui->block1ColorizeLfo2Sync[11], &gui->block1ColorizeLfo2Division[11], gui->block1ColorizeLfo2[11], divValues);
    
    // Block1 colorize LFO (band 5) - indices 0,1,2 are amplitudes
    if (gui->block1ColorizeLfo3[0] != 0.0f) block1ColorizeHueBand5Theta += lfoRateC * getEffectiveLfoRate(&gui->block1ColorizeLfo3Sync[3], &gui->block1ColorizeLfo3Division[3], gui->block1ColorizeLfo3[3], divValues);
    if (gui->block1ColorizeLfo3[1] != 0.0f) block1ColorizeSaturationBand5Theta += lfoRateC * getEffectiveLfoRate(&gui->block1ColorizeLfo3Sync[4], &gui->block1ColorizeLfo3Division[4], gui->block1ColorizeLfo3[4], divValues);
    if (gui->block1ColorizeLfo3[2] != 0.0f) block1ColorizeBrightBand5Theta += lfoRateC * getEffectiveLfoRate(&gui->block1ColorizeLfo3Sync[5], &gui->block1ColorizeLfo3Division[5], gui->block1ColorizeLfo3[5], divValues);
    
    // Block2 geo LFO (first set)
    if (gui->block2Geo1Lfo1[0] != 0.0f) block2XDisplaceTheta += lfoRateC * getEffectiveLfoRate(&gui->block2Geo1Lfo1Sync[1], &gui->block2Geo1Lfo1Division[1], gui->block2Geo1Lfo1[1], divValues);
    if (gui->block2Geo1Lfo1[2] != 0.0f) block2YDisplaceTheta += lfoRateC * getEffectiveLfoRate(&gui->block2Geo1Lfo1Sync[3], &gui->block2Geo1Lfo1Division[3], gui->block2Geo1Lfo1[3], divValues);
    if (gui->block2Geo1Lfo1[4] != 0.0f) block2ZDisplaceTheta += lfoRateC * getEffectiveLfoRate(&gui->block2Geo1Lfo1Sync[5], &gui->block2Geo1Lfo1Division[5], gui->block2Geo1Lfo1[5], divValues);
    if (gui->block2Geo1Lfo1[6] != 0.0f) block2RotateTheta += lfoRateC * getEffectiveLfoRate(&gui->block2Geo1Lfo1Sync[7], &gui->block2Geo1Lfo1Division[7], gui->block2Geo1Lfo1[7], divValues);
    
    // Block2 geo LFO (second set - shear matrix and kaleidoscope)
    if (gui->block2Geo1Lfo2[0] != 0.0f) block2ShearMatrix1Theta += lfoRateC * getEffectiveLfoRate(&gui->block2Geo1Lfo2Sync[1], &gui->block2Geo1Lfo2Division[1], gui->block2Geo1Lfo2[1], divValues);
    if (gui->block2Geo1Lfo2[4] != 0.0f) block2ShearMatrix2Theta += lfoRateC * getEffectiveLfoRate(&gui->block2Geo1Lfo2Sync[5], &gui->block2Geo1Lfo2Division[5], gui->block2Geo1Lfo2[5], divValues);
    if (gui->block2Geo1Lfo2[6] != 0.0f) block2ShearMatrix3Theta += lfoRateC * getEffectiveLfoRate(&gui->block2Geo1Lfo2Sync[7], &gui->block2Geo1Lfo2Division[7], gui->block2Geo1Lfo2[7], divValues);
    if (gui->block2Geo1Lfo2[2] != 0.0f) block2ShearMatrix4Theta += lfoRateC * getEffectiveLfoRate(&gui->block2Geo1Lfo2Sync[3], &gui->block2Geo1Lfo2Division[3], gui->block2Geo1Lfo2[3], divValues);
    if (gui->block2Geo1Lfo2[8] != 0.0f) block2KaleidoscopeSliceTheta += lfoRateC * getEffectiveLfoRate(&gui->block2Geo1Lfo2Sync[9], &gui->block2Geo1Lfo2Division[9], gui->block2Geo1Lfo2[9], divValues);
    
    // Block2 colorize LFO (bands 1-2) - indices 0,1,2,6,7,8 are amplitudes
    if (gui->block2ColorizeLfo1[0] != 0.0f) block2ColorizeHueBand1Theta += lfoRateC * getEffectiveLfoRate(&gui->block2ColorizeLfo1Sync[3], &gui->block2ColorizeLfo1Division[3], gui->block2ColorizeLfo1[3], divValues);
    if (gui->block2ColorizeLfo1[1] != 0.0f) block2ColorizeSaturationBand1Theta += lfoRateC * getEffectiveLfoRate(&gui->block2ColorizeLfo1Sync[4], &gui->block2ColorizeLfo1Division[4], gui->block2ColorizeLfo1[4], divValues);
    if (gui->block2ColorizeLfo1[2] != 0.0f) block2ColorizeBrightBand1Theta += lfoRateC * getEffectiveLfoRate(&gui->block2ColorizeLfo1Sync[5], &gui->block2ColorizeLfo1Division[5], gui->block2ColorizeLfo1[5], divValues);
    if (gui->block2ColorizeLfo1[6] != 0.0f) block2ColorizeHueBand2Theta += lfoRateC * getEffectiveLfoRate(&gui->block2ColorizeLfo1Sync[9], &gui->block2ColorizeLfo1Division[9], gui->block2ColorizeLfo1[9], divValues);
    if (gui->block2ColorizeLfo1[7] != 0.0f) block2ColorizeSaturationBand2Theta += lfoRateC * getEffectiveLfoRate(&gui->block2ColorizeLfo1Sync[10], &gui->block2ColorizeLfo1Division[10], gui->block2ColorizeLfo1[10], divValues);
    if (gui->block2ColorizeLfo1[8] != 0.0f) block2ColorizeBrightBand2Theta += lfoRateC * getEffectiveLfoRate(&gui->block2ColorizeLfo1Sync[11], &gui->block2ColorizeLfo1Division[11], gui->block2ColorizeLfo1[11], divValues);
    
    // Block2 colorize LFO (bands 3-4) - indices 0,1,2,6,7,8 are amplitudes
    if (gui->block2ColorizeLfo2[0] != 0.0f) block2ColorizeHueBand3Theta += lfoRateC * getEffectiveLfoRate(&gui->block2ColorizeLfo2Sync[3], &gui->block2ColorizeLfo2Division[3], gui->block2ColorizeLfo2[3], divValues);
    if (gui->block2ColorizeLfo2[1] != 0.0f) block2ColorizeSaturationBand3Theta += lfoRateC * getEffectiveLfoRate(&gui->block2ColorizeLfo2Sync[4], &gui->block2ColorizeLfo2Division[4], gui->block2ColorizeLfo2[4], divValues);
    if (gui->block2ColorizeLfo2[2] != 0.0f) block2ColorizeBrightBand3Theta += lfoRateC * getEffectiveLfoRate(&gui->block2ColorizeLfo2Sync[5], &gui->block2ColorizeLfo2Division[5], gui->block2ColorizeLfo2[5], divValues);
    if (gui->block2ColorizeLfo2[6] != 0.0f) block2ColorizeHueBand4Theta += lfoRateC * getEffectiveLfoRate(&gui->block2ColorizeLfo2Sync[9], &gui->block2ColorizeLfo2Division[9], gui->block2ColorizeLfo2[9], divValues);
    if (gui->block2ColorizeLfo2[7] != 0.0f) block2ColorizeSaturationBand4Theta += lfoRateC * getEffectiveLfoRate(&gui->block2ColorizeLfo2Sync[10], &gui->block2ColorizeLfo2Division[10], gui->block2ColorizeLfo2[10], divValues);
    if (gui->block2ColorizeLfo2[8] != 0.0f) block2ColorizeBrightBand4Theta += lfoRateC * getEffectiveLfoRate(&gui->block2ColorizeLfo2Sync[11], &gui->block2ColorizeLfo2Division[11], gui->block2ColorizeLfo2[11], divValues);
    
    // Block2 colorize LFO (band 5) - indices 0,1,2 are amplitudes
    if (gui->block2ColorizeLfo3[0] != 0.0f) block2ColorizeHueBand5Theta += lfoRateC * getEffectiveLfoRate(&gui->block2ColorizeLfo3Sync[3], &gui->block2ColorizeLfo3Division[3], gui->block2ColorizeLfo3[3], divValues);
    if (gui->block2ColorizeLfo3[1] != 0.0f) block2ColorizeSaturationBand5Theta += lfoRateC * getEffectiveLfoRate(&gui->block2ColorizeLfo3Sync[4], &gui->block2ColorizeLfo3Division[4], gui->block2ColorizeLfo3[4], divValues);
    if (gui->block2ColorizeLfo3[2] != 0.0f) block2ColorizeBrightBand5Theta += lfoRateC * getEffectiveLfoRate(&gui->block2ColorizeLfo3Sync[5], &gui->block2ColorizeLfo3Division[5], gui->block2ColorizeLfo3[5], divValues);
    
    // Matrix mix LFO - matrixMixLfo1 uses indices 0,1,2,6,7,8 for amplitudes; matrixMixLfo2 uses 0,1,2
    if (gui->matrixMixLfo1[0] != 0.0f) matrixMixBgRedIntoFgRedTheta += lfoRateC * getEffectiveLfoRate(&gui->matrixMixLfo1Sync[3], &gui->matrixMixLfo1Division[3], gui->matrixMixLfo1[3], divValues);
    if (gui->matrixMixLfo1[1] != 0.0f) matrixMixBgGreenIntoFgRedTheta += lfoRateC * getEffectiveLfoRate(&gui->matrixMixLfo1Sync[4], &gui->matrixMixLfo1Division[4], gui->matrixMixLfo1[4], divValues);
    if (gui->matrixMixLfo1[2] != 0.0f) matrixMixBgBlueIntoFgRedTheta += lfoRateC * getEffectiveLfoRate(&gui->matrixMixLfo1Sync[5], &gui->matrixMixLfo1Division[5], gui->matrixMixLfo1[5], divValues);
    if (gui->matrixMixLfo1[6] != 0.0f) matrixMixBgRedIntoFgGreenTheta += lfoRateC * getEffectiveLfoRate(&gui->matrixMixLfo1Sync[9], &gui->matrixMixLfo1Division[9], gui->matrixMixLfo1[9], divValues);
    if (gui->matrixMixLfo1[7] != 0.0f) matrixMixBgGreenIntoFgGreenTheta += lfoRateC * getEffectiveLfoRate(&gui->matrixMixLfo1Sync[10], &gui->matrixMixLfo1Division[10], gui->matrixMixLfo1[10], divValues);
    if (gui->matrixMixLfo1[8] != 0.0f) matrixMixBgBlueIntoFgGreenTheta += lfoRateC * getEffectiveLfoRate(&gui->matrixMixLfo1Sync[11], &gui->matrixMixLfo1Division[11], gui->matrixMixLfo1[11], divValues);
    if (gui->matrixMixLfo2[0] != 0.0f) matrixMixBgRedIntoFgBlueTheta += lfoRateC * getEffectiveLfoRate(&gui->matrixMixLfo2Sync[3], &gui->matrixMixLfo2Division[3], gui->matrixMixLfo2[3], divValues);
    if (gui->matrixMixLfo2[1] != 0.0f) matrixMixBgGreenIntoFgBlueTheta += lfoRateC * getEffectiveLfoRate(&gui->matrixMixLfo2Sync[4], &gui->matrixMixLfo2Division[4], gui->matrixMixLfo2[4], divValues);
    if (gui->matrixMixLfo2[2] != 0.0f) matrixMixBgBlueIntoFgBlueTheta += lfoRateC * getEffectiveLfoRate(&gui->matrixMixLfo2Sync[5], &gui->matrixMixLfo2Division[5], gui->matrixMixLfo2[5], divValues);
    
    // Final mix LFO
    if (gui->finalMixAndKeyLfo[0] != 0.0f) finalMixAmountTheta += lfoRateC * getEffectiveLfoRate(&gui->finalMixAndKeyLfoSync[1], &gui->finalMixAndKeyLfoDivision[1], gui->finalMixAndKeyLfo[1], divValues);
    if (gui->finalMixAndKeyLfo[2] != 0.0f) finalKeyThresholdTheta += lfoRateC * getEffectiveLfoRate(&gui->finalMixAndKeyLfoSync[3], &gui->finalMixAndKeyLfoDivision[3], gui->finalMixAndKeyLfo[3], divValues);
    if (gui->finalMixAndKeyLfo[4] != 0.0f) finalKeySoftTheta += lfoRateC * getEffectiveLfoRate(&gui->finalMixAndKeyLfoSync[5], &gui->finalMixAndKeyLfoDivision[5], gui->finalMixAndKeyLfo[5], divValues);
}

//--------------------------------------------------------------
void ofApp::resetLfoThetas() {
    // Block 1
    ch1XDisplaceTheta = 0;
    ch1YDisplaceTheta = 0;
    ch1ZDisplaceTheta = 0;
    ch1RotateTheta = 0;
    ch1HueAttenuateTheta = 0;
    ch1SaturationAttenuateTheta = 0;
    ch1BrightAttenuateTheta = 0;
    ch1KaleidoscopeSliceTheta = 0;
    
    ch2MixAmountTheta = 0;
    ch2KeyThresholdTheta = 0;
    ch2KeySoftTheta = 0;
    
    ch2XDisplaceTheta = 0;
    ch2YDisplaceTheta = 0;
    ch2ZDisplaceTheta = 0;
    ch2RotateTheta = 0;
    ch2HueAttenuateTheta = 0;
    ch2SaturationAttenuateTheta = 0;
    ch2BrightAttenuateTheta = 0;
    ch2KaleidoscopeSliceTheta = 0;
    
    fb1MixAmountTheta = 0;
    fb1KeyThresholdTheta = 0;
    fb1KeySoftTheta = 0;
    
    fb1XDisplaceTheta = 0;
    fb1YDisplaceTheta = 0;
    fb1ZDisplaceTheta = 0;
    fb1RotateTheta = 0;
    
    fb1ShearMatrix1Theta = 0;
    fb1ShearMatrix2Theta = 0;
    fb1ShearMatrix3Theta = 0;
    fb1ShearMatrix4Theta = 0;
    fb1KaleidoscopeSliceTheta = 0;
    
    fb1HueAttenuateTheta = 0;
    fb1SaturationAttenuateTheta = 0;
    fb1BrightAttenuateTheta = 0;
    
    // Block 2
    block2InputXDisplaceTheta = 0;
    block2InputYDisplaceTheta = 0;
    block2InputZDisplaceTheta = 0;
    block2InputRotateTheta = 0;
    block2InputHueAttenuateTheta = 0;
    block2InputSaturationAttenuateTheta = 0;
    block2InputBrightAttenuateTheta = 0;
    block2InputKaleidoscopeSliceTheta = 0;
    
    fb2MixAmountTheta = 0;
    fb2KeyThresholdTheta = 0;
    fb2KeySoftTheta = 0;
    
    fb2XDisplaceTheta = 0;
    fb2YDisplaceTheta = 0;
    fb2ZDisplaceTheta = 0;
    fb2RotateTheta = 0;
    
    fb2ShearMatrix1Theta = 0;
    fb2ShearMatrix2Theta = 0;
    fb2ShearMatrix3Theta = 0;
    fb2ShearMatrix4Theta = 0;
    fb2KaleidoscopeSliceTheta = 0;
    
    fb2HueAttenuateTheta = 0;
    fb2SaturationAttenuateTheta = 0;
    fb2BrightAttenuateTheta = 0;
    
    // Block 3
    block1XDisplaceTheta = 0;
    block1YDisplaceTheta = 0;
    block1ZDisplaceTheta = 0;
    block1RotateTheta = 0;
    
    block1ShearMatrix1Theta = 0;
    block1ShearMatrix2Theta = 0;
    block1ShearMatrix3Theta = 0;
    block1ShearMatrix4Theta = 0;
    block1KaleidoscopeSliceTheta = 0;
    
    block1ColorizeHueBand1Theta = 0;
    block1ColorizeSaturationBand1Theta = 0;
    block1ColorizeBrightBand1Theta = 0;
    block1ColorizeHueBand2Theta = 0;
    block1ColorizeSaturationBand2Theta = 0;
    block1ColorizeBrightBand2Theta = 0;
    block1ColorizeHueBand3Theta = 0;
    block1ColorizeSaturationBand3Theta = 0;
    block1ColorizeBrightBand3Theta = 0;
    block1ColorizeHueBand4Theta = 0;
    block1ColorizeSaturationBand4Theta = 0;
    block1ColorizeBrightBand4Theta = 0;
    block1ColorizeHueBand5Theta = 0;
    block1ColorizeSaturationBand5Theta = 0;
    block1ColorizeBrightBand5Theta = 0;
    
    block2XDisplaceTheta = 0;
    block2YDisplaceTheta = 0;
    block2ZDisplaceTheta = 0;
    block2RotateTheta = 0;
    
    block2ShearMatrix1Theta = 0;
    block2ShearMatrix2Theta = 0;
    block2ShearMatrix3Theta = 0;
    block2ShearMatrix4Theta = 0;
    block2KaleidoscopeSliceTheta = 0;
    
    block2ColorizeHueBand1Theta = 0;
    block2ColorizeSaturationBand1Theta = 0;
    block2ColorizeBrightBand1Theta = 0;
    block2ColorizeHueBand2Theta = 0;
    block2ColorizeSaturationBand2Theta = 0;
    block2ColorizeBrightBand2Theta = 0;
    block2ColorizeHueBand3Theta = 0;
    block2ColorizeSaturationBand3Theta = 0;
    block2ColorizeBrightBand3Theta = 0;
    block2ColorizeHueBand4Theta = 0;
    block2ColorizeSaturationBand4Theta = 0;
    block2ColorizeBrightBand4Theta = 0;
    block2ColorizeHueBand5Theta = 0;
    block2ColorizeSaturationBand5Theta = 0;
    block2ColorizeBrightBand5Theta = 0;
    
    matrixMixBgRedIntoFgRedTheta = 0;
    matrixMixBgGreenIntoFgRedTheta = 0;
    matrixMixBgBlueIntoFgRedTheta = 0;
    matrixMixBgRedIntoFgGreenTheta = 0;
    matrixMixBgGreenIntoFgGreenTheta = 0;
    matrixMixBgBlueIntoFgGreenTheta = 0;
    matrixMixBgRedIntoFgBlueTheta = 0;
    matrixMixBgGreenIntoFgBlueTheta = 0;
    matrixMixBgBlueIntoFgBlueTheta = 0;
    
    finalMixAmountTheta = 0;
    finalKeyThresholdTheta = 0;
    finalKeySoftTheta = 0;
}

//--------------------------------------------------------------
void ofApp::keyPressed(int key){
    // 'f' key to toggle fullscreen on output window
    if (key == 'f' || key == 'F') {
        if (mainWindow) {
            isOutputFullscreen = !isOutputFullscreen;
            mainWindow->setFullscreen(isOutputFullscreen);
            ofLogNotice("ofApp") << "Output window fullscreen: " << (isOutputFullscreen ? "ON" : "OFF");
        }
    }
    
    // F10 to toggle window decoration
    if (key == OF_KEY_F10) {
        auto glfwWindow = dynamic_cast<ofAppGLFWWindow*>(mainWindow.get());
        if (glfwWindow) {
            auto win = glfwWindow->getGLFWWindow();
            bool decorated = glfwGetWindowAttrib(win, GLFW_DECORATED);
            glfwSetWindowAttrib(win, GLFW_DECORATED, !decorated);
        }
    }
}

//--------------------------------------------------------------
void ofApp::keyReleased(int key){
}

//--------------------------------------------------------------
void ofApp::exit(){
    ofLogNotice("ofApp") << "exit() called - beginning cleanup...";
    
    // IMPORTANT: Close singletons first before saving settings
    // to prevent them from accessing destroyed resources
    
    // 1. Close ParameterManager (OSC/MIDI) - prevents callbacks during shutdown
    ParameterManager::getInstance().close();
    ofLogNotice("ofApp") << "ParameterManager closed";
    
    // Clean up preview panel before pipeline is destroyed
    if (previewPanel) {
        previewPanel.reset();
        ofLogNotice("ofApp") << "PreviewPanel cleaned up";
    }
    
    // 2. Save settings on exit (after closing OSC to prevent race conditions)
    ofLogNotice("ofApp") << "Saving settings on exit...";
    
    // Sync current GUI values to SettingsManager
    if (gui) {
        syncGuiToSettingsManager();
    }
    
    // Save to config.json
    SettingsManager::getInstance().save();
    
    // Also save to settings.json for backward compatibility
    if (gui) {
        gui->saveVideoOscSettings();
    }
    
    ofLogNotice("ofApp") << "Settings saved successfully";
    
    // 3. Clean up modular components in reverse order of creation
    // This ensures proper cleanup of GPU resources and NDI/Spout
    ofLogNotice("ofApp") << "Cleaning up modular components...";
    
    // Audio analyzer - close sound stream before reset
    if (audioAnalyzer) {
        audioAnalyzer->close();
    }
    audioAnalyzer.reset();
    ofLogNotice("ofApp") << "AudioAnalyzer cleaned up";
    
    // Tempo manager
    tempoManager.reset();
    ofLogNotice("ofApp") << "TempoManager cleaned up";
    
    // Geometry manager depends on OpenGL - clean up before pipeline
    geometryManager.reset();
    ofLogNotice("ofApp") << "GeometryManager cleaned up";
    
    // Output manager uses NDI/Spout - explicitly close before reset to ensure
    // proper cleanup order and prevent NDI thread termination issues
    if (outputManager) {
        ofLogNotice("ofApp") << "Closing OutputManager...";
        outputManager->close();
        ofLogNotice("ofApp") << "OutputManager closed";
        
        // Add a longer delay to let NDI threads settle before destroying the sender objects
        // This is critical - destroying NDI senders while their internal threads are active
        // causes crashes that cannot be caught with try-catch
        ofLogNotice("ofApp") << "Waiting for NDI cleanup...";
        ofSleepMillis(200);
        
        // Now it's safe to destroy the OutputManager
        outputManager.reset();
        ofLogNotice("ofApp") << "OutputManager cleaned up";
    }
    
    // Pipeline contains FBOs and shaders
    pipeline.reset();
    ofLogNotice("ofApp") << "PipelineManager cleaned up";
    
    // Input manager last (may have active camera/NDI sources)
    inputManager.reset();
    ofLogNotice("ofApp") << "InputManager cleaned up";
    
    // 4. Stop legacy OSC receiver
    oscReceiver.stop();
    ofLogNotice("ofApp") << "Legacy OSC receiver stopped";
    
    ofLogNotice("ofApp") << "exit() completed successfully";
}

//==============================================================================
// Legacy OSC Functions (Stubs for backward compatibility with GuiApp)
//==============================================================================

void ofApp::setupOsc() {
    // OSC setup is handled by ParameterManager
    auto& pm = dragonwaves::ParameterManager::getInstance();
    pm.setup(dragonwaves::SettingsManager::getInstance().getOsc());
    oscEnabled = true;
}

void ofApp::processOscMessages() {
    // OSC processing is handled by ParameterManager
    auto& pm = dragonwaves::ParameterManager::getInstance();
    pm.update();
}

void ofApp::sendOscParameter(string address, float value) {
    auto& pm = dragonwaves::ParameterManager::getInstance();
    pm.sendParameter(address, value);
}

void ofApp::sendOscString(string address, string value) {
    auto& pm = dragonwaves::ParameterManager::getInstance();
    pm.sendString(address, value);
}

void ofApp::sendAllOscParameters() {
    auto& pm = dragonwaves::ParameterManager::getInstance();
    pm.sendAllParameters();
}

void ofApp::reloadOscSettings() {
    auto& pm = dragonwaves::ParameterManager::getInstance();
    pm.reloadOscSettings();
}

// Legacy block-specific OSC senders (stubs)
void ofApp::sendOscBlock1Ch1() {}
void ofApp::sendOscBlock1Ch2() {}
void ofApp::sendOscBlock1Fb1() {}
void ofApp::sendOscBlock2Input() {}
void ofApp::sendOscBlock2Fb2() {}
void ofApp::sendOscBlock3B1() {}
void ofApp::sendOscBlock3B2() {}
void ofApp::sendOscBlock3MatrixAndFinal() {}

//--------------------------------------------------------------
// Audio and Tempo OSC Parameters
//--------------------------------------------------------------
void ofApp::registerAudioTempoOscParams() {
    using namespace dragonwaves;
    auto& pm = ParameterManager::getInstance();
    
    // Audio parameter group
    auto audioGroup = std::make_shared<ParameterGroup>("Audio", "/gravity/audio");
    
    // Audio enable
    audioGroup->addParameter(std::make_shared<Parameter<bool>>(
        "enabled", "/gravity/audio/enabled", &audioAnalyzer->settings.enabled));
    
    // FFT bands (read-only outputs)
    static float fftBands[8] = {0};
    for (int i = 0; i < 8; i++) {
        audioGroup->addParameter(std::make_shared<Parameter<float>>(
            "fftBand" + std::to_string(i), "/gravity/audio/fftBand" + std::to_string(i), &fftBands[i], 0.0f, 1.0f));
    }
    
    // Audio controls
    audioGroup->addParameter(std::make_shared<Parameter<float>>(
        "amplitude", "/gravity/audio/amplitude", &audioAnalyzer->settings.amplitude, 0.0f, 10.0f));
    audioGroup->addParameter(std::make_shared<Parameter<float>>(
        "smoothing", "/gravity/audio/smoothing", &audioAnalyzer->settings.smoothing, 0.0f, 0.99f));
    audioGroup->addParameter(std::make_shared<Parameter<bool>>(
        "normalization", "/gravity/audio/normalization", &audioAnalyzer->settings.normalization));
    
    pm.registerGroup(audioGroup);
    
    // Tempo parameter group
    auto tempoGroup = std::make_shared<ParameterGroup>("Tempo", "/gravity/tempo");
    
    // BPM
    tempoGroup->addParameter(std::make_shared<Parameter<float>>(
        "bpm", "/gravity/tempo/bpm", &tempoManager->settings.bpm, 20.0f, 300.0f));
    
    // Tempo controls
    tempoGroup->addParameter(std::make_shared<Parameter<bool>>(
        "enabled", "/gravity/tempo/enabled", &tempoManager->settings.enabled));
    tempoGroup->addParameter(std::make_shared<Parameter<bool>>(
        "play", "/gravity/tempo/play", nullptr));  // Trigger only
    
    // Beat phase (read-only output)
    static float beatPhase = 0.0f;
    tempoGroup->addParameter(std::make_shared<Parameter<float>>(
        "beatPhase", "/gravity/tempo/beatPhase", &beatPhase, 0.0f, 1.0f));
    
    pm.registerGroup(tempoGroup);
    
    ofLogNotice("ofApp") << "Audio and Tempo OSC parameters registered";
}

bool ofApp::processOscAudioParams(const string& address, float value) {
    if (!audioAnalyzer) return false;
    
    if (address == "/gravity/audio/enabled") {
        audioAnalyzer->setEnabled(value > 0.5f);
        return true;
    }
    else if (address == "/gravity/audio/amplitude") {
        audioAnalyzer->setAmplitude(value);
        return true;
    }
    else if (address == "/gravity/audio/smoothing") {
        audioAnalyzer->setSmoothing(value);
        return true;
    }
    else if (address == "/gravity/audio/normalization") {
        audioAnalyzer->setNormalization(value > 0.5f);
        return true;
    }
    
    return false;
}

bool ofApp::processOscTempoParams(const string& address, float value) {
    if (!tempoManager) return false;
    
    if (address == "/gravity/tempo/bpm") {
        tempoManager->setBpm(value);
        return true;
    }
    else if (address == "/gravity/tempo/enabled") {
        tempoManager->setEnabled(value > 0.5f);
        return true;
    }
    else if (address == "/gravity/tempo/play") {
        tempoManager->setPlaying(value > 0.5f);
        return true;
    }
    else if (address == "/gravity/tempo/tap") {
        tempoManager->tap();
        return true;
    }
    
    return false;
}

//--------------------------------------------------------------
// Apply audio/BPM modulations from GUI to Block3Shader
//--------------------------------------------------------------
void ofApp::applyAudioModulationToParam(int blockNum, const std::string& paramName, bool enabled, int fftBand, float amount, float rangeScale) {
    if (!pipeline) return;
    
    ParamModulation* mod = nullptr;
    std::string blockName;
    
    switch (blockNum) {
        case 1:
            mod = pipeline->getBlock1().getModulation(paramName);
            blockName = "Block1";
            break;
        case 2:
            mod = pipeline->getBlock2().getModulation(paramName);
            blockName = "Block2";
            break;
        case 3:
        default:
            mod = pipeline->getBlock3().getModulation(paramName);
            blockName = "Block3";
            break;
    }
    
    if (mod) {
        mod->audio.enabled = enabled;
        mod->audio.fftBand = fftBand;
        mod->audio.amount = amount;
        mod->audio.rangeScale = rangeScale;
        ofLogNotice("ofApp") << "Applied audio modulation to " << blockName << "." << paramName 
                             << ": enabled=" << enabled << ", band=" << fftBand 
                             << ", amount=" << amount << ", rangeScale=" << rangeScale;
    } else {
        ofLogWarning("ofApp") << "Could not find modulation for " << blockName << ":" << paramName;
    }
}

void ofApp::applyBpmModulationToParam(const std::string& paramName, bool enabled, int division, int waveform, float phase, float minVal, float maxVal) {
    if (!pipeline) return;
    
    auto& block3 = pipeline->getBlock3();
    auto* mod = block3.getModulation(paramName);
    
    if (mod) {
        mod->bpm.enabled = enabled;
        mod->bpm.divisionIndex = division;
        mod->bpm.waveform = waveform;
        mod->bpm.phase = phase;
        mod->bpm.minValue = minVal;
        mod->bpm.maxValue = maxVal;
        ofLogNotice("ofApp") << "Applied BPM modulation to " << paramName 
                             << ": enabled=" << enabled << ", division=" << division << ", waveform=" << waveform;
    } else {
        ofLogWarning("ofApp") << "Could not find modulation for parameter: " << paramName;
    }
}

float ofApp::getModulatedValue(int blockNum, const std::string& paramName) const {
    if (!pipeline) return 0.0f;
    return pipeline->getModulatedValue(blockNum, paramName);
}
