# Dragon Waves - Modular Architecture

## Overview

This project has been refactored into a modular architecture to improve maintainability, performance, and cross-platform compatibility while preserving existing functionality including presets, OSC addressing, and dual-window architecture.

## Architecture

```
src/
├── Core/               # Settings, configuration, utilities
├── Inputs/             # Input source management
├── ShaderPipeline/     # Shader processing pipeline
├── Output/             # NDI/Spout output
├── Geometry/           # Geometric patterns
├── Parameters/         # OSC/MIDI parameter system
├── Presets/            # Preset management
└── [legacy files]      # Original ofApp, GuiApp
```

## Module Descriptions

### Core Module (`src/Core/`)

**SettingsManager** - Centralized configuration management
- `DisplaySettings` - Resolution, FPS (configurable via XML)
- `OscSettings` - OSC ports and addresses
- `MidiSettings` - MIDI device configuration
- `InputSourceSettings` - Input source types and indices

Usage:
```cpp
auto& settings = dragonwaves::SettingsManager::getInstance();
settings.load();  // Load from settings.xml

// Access display settings
int width = settings.getDisplay().internalWidth;
settings.getDisplay().targetFPS = 60;
settings.save();  // Save to settings.xml
```

### Inputs Module (`src/Inputs/`)

**InputManager** - Manages all input sources
- `WebcamInput` - Camera capture
- `NdiInput` - NDI receiver
- `SpoutInput` - Spout receiver (Windows only)
- `VideoFileInput` - Video file playback with looping

Usage:
```cpp
dragonwaves::InputManager inputManager;
inputManager.setup(settings.getDisplay());

// Configure inputs
inputManager.configureInput1(dragonwaves::InputType::NDI, 0);
inputManager.configureInput2(dragonwaves::InputType::WEBCAM, 1);

// In update()
inputManager.update();
ofTexture& tex1 = inputManager.getInput1Texture();
```

### ShaderPipeline Module (`src/ShaderPipeline/`)

**PipelineManager** - Manages the 3-block shader pipeline
- `Block1Shader` - Channel mixing, FB1 feedback, geometric patterns
- `Block2Shader` - Block2 input processing, FB2 feedback
- `Block3Shader` - Matrix mixing, final output
- `DelayBuffer` - Frame delay/feedback buffers

Usage:
```cpp
dragonwaves::PipelineManager pipeline;
pipeline.setup(settings.getDisplay());

// Set inputs
pipeline.setInput1Texture(inputManager.getInput1Texture());
pipeline.setInput2Texture(inputManager.getInput2Texture());

// Configure parameters via shader blocks
pipeline.getBlock1().params.ch1MixAmount = 0.5f;
pipeline.getBlock1().params.ch1XDisplace = 0.2f;

// Process frame
pipeline.processFrame();

// Get output
ofTexture& output = pipeline.getFinalOutput();
```

### Output Module (`src/Output/`)

**OutputManager** - Handles NDI/Spout output
- `NdiOutputSender` - Async PBO-based NDI sending
- `SpoutOutputSender` - Spout texture sharing (Windows)
- `AsyncPixelTransfer` - Efficient GPU->CPU transfer

Usage:
```cpp
dragonwaves::OutputManager outputManager;
outputManager.setup(settings.getDisplay());

// Enable outputs
outputManager.setNdiBlock3Enabled(true);
outputManager.setSpoutBlock3Enabled(true);

// Send frames
outputManager.sendBlock3(pipeline.getFinalOutput());
```

### Geometry Module (`src/Geometry/`)

**GeometryManager** - Geometric pattern generation
- `HypercubePattern`
- `LinePattern`
- `SevenStarPattern`
- `SpiralEllipsePattern`
- `LissajousPattern`

Usage:
```cpp
dragonwaves::GeometryManager geometry;
geometry.setup();

// Enable patterns
geometry.getHypercube().setEnabled(true);
geometry.getHypercube().thetaRate = 0.02f;

// In update()
geometry.update();

// Draw (called within framebuffer begin/end)
geometry.drawPatterns(width, height);
```

### Parameters Module (`src/Parameters/`)

**ParameterManager** - Centralized OSC/MIDI parameter management
- `Parameter<T>` - Typed parameter wrapper
- `ParameterGroup` - Organize related parameters
- OSC send/receive
- MIDI mapping with latching

Usage:
```cpp
auto& paramManager = dragonwaves::ParameterManager::getInstance();
paramManager.setup(settings.getOsc());

// Create parameter group
auto block1Params = std::make_shared<ParameterGroup>("Block1", "/gravity/block1");
block1Params->addParameter(std::make_shared<Parameter<float>>(
    "ch1Mix", "/gravity/block1/ch1/mix", &pipeline.getBlock1().params.ch1MixAmount));

paramManager.registerGroup(block1Params);

// MIDI mapping
paramManager.addMidiMapping(1, "/gravity/block1/ch1/mix", 0.0f, 1.0f);
```

## Cross-Platform Compatibility

The code uses `#ifdef` guards for platform-specific features:

```cpp
#if defined(TARGET_WIN32)
    // Spout code here
#endif

#if defined(TARGET_OPENGLES)
    // GLES-specific code
#endif
```

## OSC Addressing

The original OSC addressing scheme is preserved:

```
/gravity/block1/ch1/xDisplace      - Channel 1 X displacement
/gravity/block1/ch2/mixAmount      - Channel 2 mix amount
/gravity/block1/fb1/delayTime      - FB1 delay time
/gravity/block2/input/zDisplace    - Block2 input Z displacement
/gravity/block3/matrix/mixBgRed    - Matrix mixer
/gravity/preset/save               - Save preset trigger
/gravity/preset/load               - Load preset trigger
```

## Settings XML Format

```xml
<settings>
    <display>
        <input1Width>640</input1Width>
        <input1Height>480</input1Height>
        <internalWidth>1280</internalWidth>
        <internalHeight>720</internalHeight>
        <targetFPS>30</targetFPS>
    </display>
    <osc>
        <enabled>1</enabled>
        <receivePort>7000</receivePort>
        <sendIP>127.0.0.1</sendIP>
        <sendPort>7001</sendPort>
    </osc>
    <midi>
        <selectedPort>0</selectedPort>
        <enabled>1</enabled>
    </midi>
</settings>
```

## Preset System

The existing JSON preset format is preserved. Presets are stored in `bin/data/presets/<bank>/`.

## Building

### Project Generator
Add these addons:
- ofxImGui
- ofxMidi
- ofxOsc
- ofxNDI
- ofxSpout (Windows only)

### CMake
See `CMakeLists.txt` for module configuration.

## Performance Optimizations

1. **GPU-only FBOs** - No CPU backing where not needed
2. **Async PBO transfer** - Non-blocking NDI sending
3. **Lazy initialization** - Resources created on-demand
4. **Shared input sources** - Webcam/NDI/Spout instances reused
5. **Unified shader uniforms** - Batch uniform updates

## Migration from Legacy Code

1. Replace direct input handling with `InputManager`
2. Replace shader direct calls with `PipelineManager`
3. Move OSC/MIDI to `ParameterManager`
4. Use `SettingsManager` for configuration
5. Delegate geometry to `GeometryManager`
6. Use `OutputManager` for NDI/Spout

See `docs/MIGRATION.md` for detailed migration steps.

## Future Enhancements

- Video file browser/playlist
- Additional geometric patterns
- Syphon support (macOS)
- Shader hot-reloading
- Parameter modulation matrix
