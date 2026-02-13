//
//  ShaderLoader.h
//  Gravity
//
//  Created by alpha on 12/3/2025.
//

#ifndef ShaderLoader_h
#define ShaderLoader_h

#include "ofMain.h"

/**
 * @class ShaderLoader
 * @brief Cross-platform shader loading helper
 *
 * This class handles loading the appropriate shader version based on
 * the current platform and OpenGL context. It automatically selects
 * between GL4, GL3.2, and GLES2 shader versions.
 */
class ShaderLoader {
public:
    /**
     * Load the appropriate shader version for the current platform
     *
     * @param shader The shader object to load into
     * @param shaderName Base name of the shader (without extension or directory)
     * @return True if loading was successful, false otherwise
     */
    static bool load(ofShader& shader, const std::string& shaderName);
    
    /**
     * Load the appropriate shader version from explicit vertex and fragment paths
     *
     * @param shader The shader object to load into
     * @param vertPath Path to the vertex shader
     * @param fragPath Path to the fragment shader
     * @return True if loading was successful, false otherwise
     */
    static bool loadFromPaths(ofShader& shader, const std::string& vertPath, const std::string& fragPath);
    
    /**
     * Get recommended shader directory based on platform and OpenGL version
     * @return Path to the recommended shader directory
     */
    static std::string getShaderDirectory();
    
    /**
     * Detect OpenGL version and return appropriate shader directory
     * @return Path to shader directory (e.g., "shadersGL4/", "shadersGL32/", "shadersGLES2/")
     */
    static std::string detectShaderDirectory();
};

#endif /* ShaderLoader_h */
