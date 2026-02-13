#include "InputSource.h"

namespace dragonwaves {

void InputSource::drawToFbo(ofFbo& fbo) {
    if (!isInitialized()) return;
    
    fbo.begin();
    ofViewport(0, 0, fbo.getWidth(), fbo.getHeight());
    ofSetupScreenOrtho(fbo.getWidth(), fbo.getHeight());
    ofClear(0, 0, 0, 255);
    
    getTexture().draw(0, 0, fbo.getWidth(), fbo.getHeight());
    
    fbo.end();
}

} // namespace dragonwaves
