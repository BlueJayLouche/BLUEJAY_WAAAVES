#include "PreviewRenderer.h"

namespace dragonwaves {

PreviewRenderer::PreviewRenderer() {}

PreviewRenderer::~PreviewRenderer() {}

void PreviewRenderer::setup(int width, int height) {
    previewWidth = width;
    previewHeight = height;
    
    // Allocate preview texture
    previewTexture.allocate(previewWidth, previewHeight, GL_RGBA);
    
    // Allocate pixels buffer for copying
    previewPixels.allocate(previewWidth, previewHeight, OF_PIXELS_RGBA);
    
    initialized = true;
    
    ofLogNotice("PreviewRenderer") << "Setup complete: " << previewWidth << "x" << previewHeight;
}

void PreviewRenderer::update(PipelineManager& pipeline, int drawMode) {
    if (!enabled || !initialized) return;
    
    auto startTime = ofGetElapsedTimeMicros();
    
    // Get source texture based on draw mode
    ofTexture* sourceTex = nullptr;
    switch (drawMode) {
        case 0:  // BLOCK1
            sourceTex = &pipeline.getBlock1Output();
            break;
        case 1:  // BLOCK2
            sourceTex = &pipeline.getBlock2Output();
            break;
        case 2:  // BLOCK3
        case 3:  // ALL BLOCKS
        default:
            sourceTex = &pipeline.getFinalOutput();
            break;
    }
    
    if (sourceTex && sourceTex->isAllocated()) {
        // Copy source texture to our shareable preview texture
        int srcW = sourceTex->getWidth();
        int srcH = sourceTex->getHeight();
        
        // Read pixels from source
        ofPixels srcPixels;
        sourceTex->readToPixels(srcPixels);
        
        // Ensure our preview texture matches source size
        bool needsAlloc = !previewTexture.isAllocated();
        bool wrongSize = previewTexture.getWidth() != srcW || 
                         previewTexture.getHeight() != srcH;
        
        if (needsAlloc || wrongSize) {
            ofLogNotice("PreviewRenderer") << "Allocating texture: " << srcW << "x" << srcH 
                                           << (needsAlloc ? " (first time)" : " (resize)");
            previewTexture.allocate(srcW, srcH, GL_RGBA);
            // Update preview dimensions for color picker
            previewWidth = srcW;
            previewHeight = srcH;
        }
        
        // Load into our texture
        previewTexture.loadData(srcPixels);
        
        // Store for color picker (at source resolution)
        previewPixels = srcPixels;
    }
    
    auto endTime = ofGetElapsedTimeMicros();
    lastUpdateTimeMs = (endTime - startTime) / 1000.0f;
}

void PreviewRenderer::draw(int x, int y, int w, int h) {
    if (!enabled || !initialized) return;
    
    if (!previewTexture.isAllocated()) return;
    
    int drawW = (w > 0) ? w : previewWidth;
    int drawH = (h > 0) ? h : previewHeight;
    
    previewTexture.draw(x, y, drawW, drawH);
}

ofTexture& PreviewRenderer::getBlockTexture(PipelineManager& pipeline, int blockNum) {
    switch (blockNum) {
        case 0: return pipeline.getBlock1Output();
        case 1: return pipeline.getBlock2Output();
        case 2: 
        default: return pipeline.getFinalOutput();
    }
}

ofColor PreviewRenderer::pickColor(int x, int y) {
    if (!enabled || !initialized) return ofColor::black;
    
    // Use actual pixel buffer dimensions
    int w = previewPixels.getWidth();
    int h = previewPixels.getHeight();
    
    // Clamp to bounds
    x = ofClamp(x, 0, w - 1);
    y = ofClamp(y, 0, h - 1);
    
    // Read color from pixels
    if (previewPixels.size() > 0) {
        lastPickedColor = previewPixels.getColor(x, y);
    }
    
    return lastPickedColor;
}

} // namespace dragonwaves
