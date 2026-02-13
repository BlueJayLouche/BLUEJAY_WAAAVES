# Shader Loading Update - GL3.2 Support

## Overview
Updated the shader loading system to properly support GL3.2, GL4, and GLES2 shaders with automatic detection.

## What Changed

### ShaderLoader Class
A new cross-platform shader loader that automatically detects OpenGL version and loads the appropriate shaders:

- **GL4 (OpenGL 4.0+)**: Loads from `shadersGL4/` directory
- **GL3.2 (OpenGL 3.x)**: Loads from `shadersGL32/` directory  
- **GLES2 (OpenGL ES 2.0)**: Loads from `shadersGLES2/` directory

### ofApp.cpp Changes
Replaced the manual shader detection code (lines 72-102) with simple ShaderLoader calls:

**Before:**
```cpp
std::string shaderDir = "shadersGL4";
bool useGLES = false;
// ... 30 lines of version detection logic ...
shader1.load(shaderDir + "/shader1");
shader2.load(shaderDir + "/shader2");
shader3.load(shaderDir + "/shader3");
```

**After:**
```cpp
#include "ShaderLoader.h"
// ...
ShaderLoader::load(shader1, "shader1");
ShaderLoader::load(shader2, "shader2");
ShaderLoader::load(shader3, "shader3");
```

## File Structure

Your project should have the following directory structure:

```
bin/data/
├── shadersGL4/           # OpenGL 4.0+ shaders (#version 460)
│   ├── shader1.vert
│   ├── shader1.frag
│   ├── shader2.vert
│   ├── shader2.frag
│   ├── shader3.vert
│   └── shader3.frag
├── shadersGL32/          # OpenGL 3.2 shaders (#version 150)
│   ├── shader1.vert
│   ├── shader1.frag
│   ├── shader2.vert
│   ├── shader2.frag
│   ├── shader3.vert
│   └── shader3.frag
└── shadersGLES2/         # OpenGL ES 2.0 shaders (if needed)
    ├── shader1.vert
    ├── shader1.frag
    ├── shader2.vert
    ├── shader2.frag
    ├── shader3.vert
    └── shader3.frag
```

## Installation

1. Copy `ShaderLoader.h` and `ShaderLoader.cpp` to your `src/` directory
2. Replace your existing `ofApp.cpp` with the updated version
3. Create the `shadersGL32/` directory in `bin/data/`
4. Copy the converted GL3.2 shaders (shader1_gl32.vert, etc.) to `bin/data/shadersGL32/`
5. Rename them to remove the `_gl32` suffix:
   - `shader1_gl32.vert` → `shader1.vert`
   - `shader1_gl32.frag` → `shader1.frag`
   - etc.

## Benefits

1. **Automatic Detection**: No manual version checking needed
2. **Cross-Platform**: Works on Windows, Mac, Linux, and mobile
3. **Clean Code**: Reduced from ~30 lines to 3 lines
4. **Maintainable**: All shader loading logic in one place
5. **Better Logging**: Clear messages about which shader version is being used

## Console Output

You'll see messages like:
```
[notice ] ShaderLoader: GL Version: 3.3 (3.3.0 NVIDIA 536.23)
[notice ] ShaderLoader: Using GL3.2 shaders
[notice ] ShaderLoader: Successfully loaded shader: shadersGL32/shader1
[notice ] ShaderLoader: Successfully loaded shader: shadersGL32/shader2
[notice ] ShaderLoader: Successfully loaded shader: shadersGL32/shader3
```

## Notes

- The ShaderLoader automatically detects GL version on startup
- GL 3.x versions (3.0-3.9) will use the GL3.2 shader directory
- GL 4.x and above will use the GL4 shader directory
- OpenGL ES will use the GLES2 shader directory
- All logging goes through ofLog for consistency with openFrameworks
