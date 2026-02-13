# Dragon Waves Refactoring Roadmap

## Current State Analysis

The existing codebase is approximately 6,000+ lines across:
- `ofApp.h/cpp` - Main application (~2000+ lines)
- `GuiApp.h/cpp` - GUI with ImGui (~4000+ lines)
- Dense parameter arrays, tight coupling, difficult to maintain

## Target Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                         GuiApp                              │
│  ┌──────────┐  ┌──────────┐  ┌─────────────────────────┐   │
│  │ ImGui UI │  │  MIDI    │  │      OSC Controls       │   │
│  └────┬─────┘  └────┬─────┘  └───────────┬─────────────┘   │
│       └─────────────┴────────────────────┘                 │
│                         │                                   │
│                         ▼                                   │
│              ┌─────────────────────┐                       │
│              │   ParameterManager  │                       │
│              └──────────┬──────────┘                       │
└─────────────────────────┼───────────────────────────────────┘
                          │
┌─────────────────────────┼───────────────────────────────────┐
│                         ▼                                   │
│                        ofApp                                │
│  ┌──────────────┬───────┴───────┬────────────────────────┐ │
│  │              │               │                        │ │
│  ▼              ▼               ▼                        ▼ │
│ ┌─────────┐  ┌─────────────┐  ┌─────────────┐  ┌────────┐│ │
│ │ Settings│  │   Input     │  │   Shader    │  │ Output ││ │
│ │ Manager │  │   Manager   │  │   Pipeline  │  │Manager ││ │
│ └────┬────┘  └──────┬──────┘  └──────┬──────┘  └───┬────┘│ │
│      │              │                │              │     │ │
│      │    ┌─────────┼────────────────┼─────────┐    │     │ │
│      │    │         ▼                ▼         │    │     │ │
│      │    │   ┌───────────────────────────┐    │    │     │ │
│      │    │   │     Geometry Manager      │    │    │     │ │
│      │    │   └───────────────────────────┘    │    │     │ │
│      │    └────────────────────────────────────┘    │     │ │
│      │                    │                         │     │ │
│      └────────────────────┼─────────────────────────┘     │ │
│                           ▼                               │ │
│                    ┌─────────────┐                       │ │
│                    │   Preset    │                       │ │
│                    │   Manager   │                       │ │
│                    └─────────────┘                       │ │
└───────────────────────────────────────────────────────────┘
```

## Phase 1: Foundation (Core Module) ✅

### Completed:
- ✅ `SettingsManager` - XML-based configuration
- ✅ `PresetManager` - Preset bank management
- ✅ Cross-platform macros (`OFAPP_HAS_SPOUT`)

### Benefits:
- Display settings (resolution, FPS) configurable post-compilation
- Centralized settings persistence
- Backward-compatible preset system

## Phase 2: Input System ✅

### Completed:
- ✅ `InputSource` base class
- ✅ `WebcamInput`, `NdiInput`, `SpoutInput`, `VideoFileInput`
- ✅ `InputManager` - unified input switching

### Benefits:
- Clean abstraction over different input types
- Easy to add new input sources
- Video file support with looping

## Phase 3: Shader Pipeline ✅

### Completed:
- ✅ `Block1Shader`, `Block2Shader`, `Block3Shader`
- ✅ `PipelineManager` - orchestrates all blocks
- ✅ `DelayBuffer` - efficient frame delay/feedback

### Benefits:
- Clean separation of shader stages
- Parameter structures for type safety
- Reusable delay buffers

## Phase 4: Output System ✅

### Completed:
- ✅ `OutputManager` - unified output handling
- ✅ `AsyncPixelTransfer` - PBO-based efficient transfer
- ✅ `NdiOutputSender`, `SpoutOutputSender`

### Benefits:
- Async PBO transfer for NDI (better performance)
- Unified output control
- Platform abstraction

## Phase 5: Geometry System ✅

### Completed:
- ✅ `GeometricPattern` base class
- ✅ `HypercubePattern`, `LinePattern`, etc.
- ✅ `GeometryManager` - pattern coordination

### Benefits:
- Easy to add new patterns
- Clean pattern lifecycle management
- Separated from shader logic

## Phase 6: Parameter System ✅

### Completed:
- ✅ `Parameter<T>` template
- ✅ `ParameterGroup` - OSC address organization
- ✅ `ParameterManager` - centralized OSC/MIDI

### Benefits:
- Type-safe parameters
- Automated OSC addressing
- MIDI mapping with latching

## Implementation Strategy

### Step 1: Create New Modules
1. Copy new header files to `src/` subdirectories
2. Implement `.cpp` files incrementally
3. Test each module independently

### Step 2: Refactor ofApp
1. Replace input handling with `InputManager`
2. Replace shader code with `PipelineManager`
3. Replace output code with `OutputManager`
4. Keep existing OSC message handlers (address compatibility)

### Step 3: Refactor GuiApp
1. Replace parameter arrays with `ParameterGroup` references
2. Use `PresetManager` for save/load
3. Use `SettingsManager` for configuration UI
4. Keep existing ImGui structure

### Step 4: Testing
1. Verify preset save/load compatibility
2. Test OSC addressing matches original
3. Test MIDI mapping
4. Cross-platform build testing

## CMakeLists.txt Template

```cmake
cmake_minimum_required(VERSION 3.1)
project(dragonWaves)

# OF paths
set(OF_DIRECTORY /path/to/of_v0.12.0)

# Sources
file(GLOB SOURCES 
    src/*.cpp
    src/Core/*.cpp
    src/Inputs/*.cpp
    src/ShaderPipeline/*.cpp
    src/Output/*.cpp
    src/Geometry/*.cpp
    src/Parameters/*.cpp
)

file(GLOB HEADERS
    src/*.h
    src/Core/*.h
    src/Inputs/*.h
    src/ShaderPipeline/*.h
    src/Output/*.h
    src/Geometry/*.h
    src/Parameters/*.h
)

# Platform detection
if(WIN32)
    add_definitions(-DTARGET_WIN32)
    add_definitions(-DOFAPP_HAS_SPOUT=1)
elseif(APPLE)
    add_definitions(-DTARGET_OSX)
    add_definitions(-DOFAPP_HAS_SPOUT=0)
elseif(UNIX)
    add_definitions(-DTARGET_LINUX)
    add_definitions(-DOFAPP_HAS_SPOUT=0)
endif()

# Addons
include(${OF_DIRECTORY}/addons/ofxImGui/ofxImGui.cmake)
include(${OF_DIRECTORY}/addons/ofxMidi/ofxMidi.cmake)
include(${OF_DIRECTORY}/addons/ofxOsc/ofxOsc.cmake)
include(${OF_DIRECTORY}/addons/ofxNDI/ofxNDI.cmake)

if(WIN32)
    include(${OF_DIRECTORY}/addons/ofxSpout/ofxSpout.cmake)
endif()

# Target
add_executable(${PROJECT_NAME} ${SOURCES} ${HEADERS})
target_link_libraries(${PROJECT_NAME} ${OPENFRAMEWORKS_LIBRARIES})
```

## Project Generator Setup

1. Generate new project with:
   - ofxImGui
   - ofxMidi
   - ofxOsc
   - ofxNDI
   - ofxSpout (Windows only)

2. Add new source directories to `config.make` or IDE project

3. Copy shader folders to `bin/data/`

## Performance Targets

1. **Frame Time**: < 33ms @ 30fps with 1080p internal resolution
2. **Memory**: < 1GB GPU memory for all FBOs
3. **Latency**: < 2 frames for NDI output
4. **Startup**: < 5 seconds to ready state

## Known Considerations

1. **Shader Compatibility**: Ensure `shadersGL3/` exists for macOS
2. **NDI Runtime**: NDI runtime required on target system
3. **Spout**: Windows only, requires compatible GPU
4. **Video Files**: Use GPU-accelerated codecs (HAP, ProRes)

## Migration Checklist

- [ ] All original OSC addresses preserved
- [ ] Preset JSON format unchanged
- [ ] MIDI CC mappings preserved
- [ ] Dual-window architecture maintained
- [ ] GUI layout preserved
- [ ] Shader behavior unchanged
- [ ] Cross-platform builds working
- [ ] Performance equal or better

## Future Enhancements (Post-Refactor)

1. **Video Playlist** - Multiple video files with transitions
2. **Parameter Modulation** - Internal LFO routing matrix
3. **Shader Hot-Reload** - Edit shaders without restart
4. **Syphon Support** - macOS texture sharing
5. **Recording** - Save output to video file
6. **Network Control** - Web-based remote control
