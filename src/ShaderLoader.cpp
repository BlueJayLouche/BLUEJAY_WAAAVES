//
//  ShaderLoader.cpp
//  Gravity
//
//  Created by alpha on 12/3/2025.
//

#include "ShaderLoader.h"

//--------------------------------------------------------------
std::string ShaderLoader::detectShaderDirectory() {
    bool useGLES = false;
    bool useGL32 = false;
    
    #if defined(TARGET_OPENGLES)
        useGLES = true;
    #endif
    
    // Check GL version string for ES
    const char* glVersionCStr = reinterpret_cast<const char*>(glGetString(GL_VERSION));
    std::string glVersionStr = glVersionCStr ? std::string(glVersionCStr) : std::string();
    if (glVersionStr.find("OpenGL ES") != std::string::npos) {
        useGLES = true;
    }
    
    // Check renderer version
    if (!useGLES && ofGetGLRenderer()) {
        int majorVersion = ofGetGLRenderer()->getGLVersionMajor();
        int minorVersion = ofGetGLRenderer()->getGLVersionMinor();
        
        ofLogNotice("ShaderLoader") << "GL Version: " << majorVersion << "." << minorVersion 
                                    << " (" << glVersionStr << ")";
        
        // If GL version is 3.x (but not 4.x), use GL3.2 shaders
        if (majorVersion == 3) {
            useGL32 = true;
        }
        // If GL version is less than 3, fall back to GLES2
        else if (majorVersion < 3) {
            useGLES = true;
        }
        // Otherwise use GL4 (majorVersion >= 4)
    }
    
    std::string shaderDir;
    if (useGLES) {
        shaderDir = "shadersGLES2/";
        ofLogNotice("ShaderLoader") << "Using GLES2 shaders";
    } else if (useGL32) {
        shaderDir = "shadersGL3/";
        ofLogNotice("ShaderLoader") << "Using GL3.2 shaders";
    } else {
        shaderDir = "shadersGL4/";
        ofLogNotice("ShaderLoader") << "Using GL4 shaders";
    }
    
    return shaderDir;
}

//--------------------------------------------------------------
std::string ShaderLoader::getShaderDirectory() {
    return detectShaderDirectory();
}

//--------------------------------------------------------------
bool ShaderLoader::load(ofShader& shader, const std::string& shaderName) {
    std::string shaderDir = getShaderDirectory();
    std::string fullPath = shaderDir + shaderName;
    
    bool success = shader.load(fullPath);
    
    if (success) {
        // Bind default OF attributes (position, texcoord, color, normal)
        shader.bindDefaults();
        ofLogNotice("ShaderLoader") << "Successfully loaded shader: " << fullPath;
    } else {
        ofLogError("ShaderLoader") << "Failed to load shader: " << fullPath;
    }
    
    return success;
}

//--------------------------------------------------------------
bool ShaderLoader::loadFromPaths(ofShader& shader, const std::string& vertPath, const std::string& fragPath) {
    bool success = shader.load(vertPath, fragPath);
    
    if (success) {
        // Bind default OF attributes (position, texcoord, color, normal)
        shader.bindDefaults();
        ofLogNotice("ShaderLoader") << "Successfully loaded shader from paths: " << vertPath << ", " << fragPath;
    } else {
        ofLogError("ShaderLoader") << "Failed to load shader from paths: " << vertPath << ", " << fragPath;
    }
    
    return success;
}
