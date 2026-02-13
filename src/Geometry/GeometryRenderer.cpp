#include "GeometryRenderer.h"

namespace dragonwaves {

//==============================================================================
// GeometricPattern
//==============================================================================
GeometricPattern::GeometricPattern(const std::string& n)
    : name(n) {
}

//==============================================================================
// HypercubePattern
//==============================================================================
HypercubePattern::HypercubePattern()
    : GeometricPattern("Hypercube") {
}

void HypercubePattern::setup() {
}

void HypercubePattern::update() {
    theta += thetaRate;
    phi += phiRate;
    colorTheta += 0.01f;
}

void HypercubePattern::draw(int width, int height) {
    if (!enabled) return;
    
    float centerX = width * 0.5f;
    float centerY = height * 0.5f;
    float scale = (width < height ? width : height) * 0.4f * size;
    
    // Project 4D hypercube to 2D
    float c1 = cos(theta);
    float s1 = sin(theta);
    float c2 = cos(phi);
    float s2 = sin(phi);
    
    // Hypercube vertices (4D)
    float vertices4D[16][4] = {
        {-1, -1, -1, -1}, {1, -1, -1, -1}, {1, 1, -1, -1}, {-1, 1, -1, -1},
        {-1, -1, 1, -1}, {1, -1, 1, -1}, {1, 1, 1, -1}, {-1, 1, 1, -1},
        {-1, -1, -1, 1}, {1, -1, -1, 1}, {1, 1, -1, 1}, {-1, 1, -1, 1},
        {-1, -1, 1, 1}, {1, -1, 1, 1}, {1, 1, 1, 1}, {-1, 1, 1, 1}
    };
    
    // Project to 2D
    ofVec2f projected[16];
    for (int i = 0; i < 16; i++) {
        float x = vertices4D[i][0];
        float y = vertices4D[i][1];
        float z = vertices4D[i][2];
        float w4 = vertices4D[i][3];
        
        // 4D rotation
        float y1 = y * c1 - z * s1;
        float z1 = y * s1 + z * c1;
        float x1 = x * c2 - w4 * s2;
        float w1 = x * s2 + w4 * c2;
        
        // Project to 3D then 2D
        float dist = 3.0f;
        float factor = scale / (dist - z1 * 0.3f);
        projected[i].x = centerX + x1 * factor;
        projected[i].y = centerY + y1 * factor;
    }
    
    // Draw edges
    ofSetLineWidth(2);
    
    // Define edges of hypercube
    int edges[][2] = {
        // Inner cube
        {0,1}, {1,2}, {2,3}, {3,0},
        {4,5}, {5,6}, {6,7}, {7,4},
        {0,4}, {1,5}, {2,6}, {3,7},
        // Outer cube
        {8,9}, {9,10}, {10,11}, {11,8},
        {12,13}, {13,14}, {14,15}, {15,12},
        {8,12}, {9,13}, {10,14}, {11,15},
        // Connections
        {0,8}, {1,9}, {2,10}, {3,11},
        {4,12}, {5,13}, {6,14}, {7,15}
    };
    
    for (int i = 0; i < 32; i++) {
        int v1 = edges[i][0];
        int v2 = edges[i][1];
        
        float hue = fmod(colorTheta + i * 0.03f, 1.0f);
        ofSetColor(ofFloatColor::fromHsb(hue, 0.8f, 1.0f));
        
        ofDrawLine(projected[v1].x, projected[v1].y, projected[v2].x, projected[v2].y);
    }
    
}

//==============================================================================
// LinePattern
//==============================================================================
LinePattern::LinePattern()
    : GeometricPattern("Line") {
}

void LinePattern::update() {
    theta += thetaRate;
    phi += phiRate;
    eta += etaRate;
}

void LinePattern::draw(int width, int height) {
    if (!enabled) return;
    
    float centerX = width * 0.5f;
    float centerY = height * 0.5f;
    float radius = (width < height ? width : height) * 0.4f;
    
    ofSetLineWidth(3);
    
    int numLines = 12;
    for (int i = 0; i < numLines; i++) {
        float t = ofMap(i, 0, numLines, 0, TWO_PI);
        
        float x1 = centerX + cos(t + theta) * radius * 0.3f;
        float y1 = centerY + sin(t + theta) * radius * 0.3f;
        float x2 = centerX + cos(t + phi) * radius;
        float y2 = centerY + sin(t + phi) * radius;
        
        float hue = fmod((float)i / numLines + eta, 1.0f);
        ofSetColor(ofFloatColor::fromHsb(hue, 0.9f, 1.0f));
        
        ofDrawLine(x1, y1, x2, y2);
    }
    
}

//==============================================================================
// SevenStarPattern
//==============================================================================
SevenStarPattern::SevenStarPattern()
    : GeometricPattern("SevenStar") {
}

void SevenStarPattern::setup() {
    // Initialize points on a circle
    for (int i = 0; i < REPS; i++) {
        float angle = TWO_PI * i / REPS - PI * 0.5f;
        points[i] = ofVec2f(cos(angle), sin(angle));
        points1[i] = ofVec2f(cos(angle), sin(angle));
    }
    points1[REPS] = points1[0];
    
    for (int i = 0; i < REPS; i++) {
        points2[i] = points[i];
    }
}

void SevenStarPattern::update() {
    thetaHue1 += hueInc1;
    thetaHue2 += hueInc2;
    thetaSaturation1 += saturationInc1;
    thetaChaos += chaosInc;
}

void SevenStarPattern::draw(int width, int height) {
    if (!enabled) return;
    
    float centerX = width * 0.5f;
    float centerY = height * 0.5f;
    float radius = (width < height ? width : height) * 0.4f;
    
    
    // First pattern (increment 3)
    position1 += acceleration1;
    if (position1.x >= 1.0f || position1.x <= 0.0f) {
        index1 = (index1 + 3) % REPS;
        position1 = ofVec2f(0, 0);
    }
    
    ofVec2f target1 = points1[(index1 + 3) % (REPS + 1)];
    ofVec2f current1 = points1[index1] + (target1 - points1[index1]) * position1.x;
    
    // Second pattern (increment 2)
    position2 += acceleration2;
    if (position2.x >= 1.0f || position2.x <= 0.0f) {
        index2 = (index2 + 2) % REPS;
        position2 = ofVec2f(0, 0);
    }
    
    ofVec2f target2 = points2[(index2 + 2) % REPS];
    ofVec2f current2 = points2[index2] + (target2 - points2[index2]) * position2.x;
    
    // Draw connections
    float hue1 = fmod(thetaHue1, 1.0f);
    float hue2 = fmod(thetaHue2, 1.0f);
    
    ofSetColor(ofFloatColor::fromHsb(hue1, 0.5f + sin(thetaSaturation1) * 0.5f, 1.0f));
    ofSetLineWidth(2);
    ofDrawLine(centerX + current1.x * radius, centerY + current1.y * radius,
               centerX + current2.x * radius, centerY + current2.y * radius);
    
}

//==============================================================================
// SpiralEllipsePattern
//==============================================================================
SpiralEllipsePattern::SpiralEllipsePattern()
    : GeometricPattern("SpiralEllipse") {
}

void SpiralEllipsePattern::update() {
    spiralTheta1 += spiralTheta1Inc * 0.1f;
    spiralRadius1 += radius1Inc;
    if (spiralRadius1 > 200) spiralRadius1 = 0;
    
    spiralTheta2 += spiralTheta2Inc * 0.1f;
    spiralRadius2 += radius2Inc;
    if (spiralRadius2 > 150) spiralRadius2 = 0;
    
    spiralTheta3 += spiralTheta3Inc * 0.1f;
    spiralRadius3 += radius3Inc;
    if (spiralRadius3 > 100) spiralRadius3 = 0;
}

void SpiralEllipsePattern::draw(int width, int height) {
    if (!enabled) return;
    
    float centerX = width * 0.5f;
    float centerY = height * 0.5f;
    
    ofSetLineWidth(2);
    
    // Draw three spiral ellipses
    for (int i = 0; i < 100; i++) {
        float t = i * 0.1f;
        
        float x1 = centerX + cos(spiralTheta1 + t) * spiralRadius1 * (1 + t * 0.02f);
        float y1 = centerY + sin(spiralTheta1 + t) * spiralRadius1 * (0.6f + t * 0.01f);
        
        float hue = fmod(t * 0.01f, 1.0f);
        ofSetColor(ofFloatColor::fromHsb(hue, 0.8f, 1.0f, 0.5f));
        ofDrawCircle(x1, y1, 3);
    }
}

//==============================================================================
// LissajousPattern
//==============================================================================
LissajousPattern::LissajousPattern(const std::string& name)
    : GeometricPattern(name) {
}

float LissajousPattern::lissajousWave(float theta, int shape) {
    switch (shape) {
        case 0: return sin(theta);                    // Sine
        case 1: return (2.0f / PI) * asin(sin(theta)); // Triangle
        case 2: return (2.0f / TWO_PI) * fmod(theta + PI, TWO_PI) - 1.0f;  // Ramp
        case 3: return 1.0f - (2.0f / TWO_PI) * fmod(theta + PI, TWO_PI);  // Saw
        case 4: return (sin(theta) >= 0.0f) ? 1.0f : -1.0f;  // Square
        default: return sin(theta);
    }
}

void LissajousPattern::update() {
    // Apply LFOs
    float xFreqMod = xFreq + lissajousWave(xFreqLfoTheta, xFreqLfo.shape) * xFreqLfo.amp;
    float yFreqMod = yFreq + lissajousWave(yFreqLfoTheta, yFreqLfo.shape) * yFreqLfo.amp;
    float speedMod = speed + lissajousWave(speedLfoTheta, speedLfo.shape) * speedLfo.amp;
    
    xFreqLfoTheta += xFreqLfo.rate * 0.1f;
    yFreqLfoTheta += yFreqLfo.rate * 0.1f;
    speedLfoTheta += speedLfo.rate * 0.1f;
    
    // Update theta
    theta += speedMod * 0.1f;
    colorTheta += colorSpeed * 0.01f;
}

void LissajousPattern::draw(int width, int height) {
    if (!enabled) return;
    
    float centerX = width * xOffset;
    float centerY = height * yOffset;
    float baseSize = (width < height ? width : height) * 0.4f * size;
    
    // Calculate actual number of points
    int nPoints = ofMap(numPoints, 0, 1, 10, 2000);
    float lw = ofMap(lineWidth, 0, 1, 1, 10);
    
    ofSetLineWidth(lw);
    
    ofMesh mesh;
    mesh.setMode(OF_PRIMITIVE_LINE_STRIP);
    
    for (int i = 0; i < nPoints; i++) {
        float t = ofMap(i, 0, nPoints, 0, TWO_PI * 4);  // 4 cycles
        
        float x = xAmp * lissajousWave(t * xFreq + xPhase + theta, xShape);
        float y = yAmp * lissajousWave(t * yFreq + yPhase + theta, yShape);
        float z = zAmp * lissajousWave(t * zFreq + zPhase + theta, zShape);
        
        float px = centerX + x * baseSize;
        float py = centerY + y * baseSize;
        
        // Color based on Z depth and position
        float currentHue = fmod(this->hue + hueSpread * (i / (float)nPoints) + colorTheta, 1.0f);
        ofFloatColor color = ofFloatColor::fromHsb(currentHue, 0.8f, 1.0f);
        color.a = ofMap(z, -1, 1, 0.2f, 1.0f);
        
        mesh.addVertex(ofVec3f(px, py, z * baseSize));
        mesh.addColor(color);
    }
    
    mesh.draw();
}

//==============================================================================
// GeometryManager
//==============================================================================
GeometryManager::GeometryManager()
    : lissajous1("Lissajous1"),
      lissajous2("Lissajous2") {
}

GeometryManager::~GeometryManager() {
}

void GeometryManager::setup() {
    hypercube.setup();
    sevenStar.setup();
}

void GeometryManager::update() {
    hypercube.update();
    line.update();
    sevenStar.update();
    spiralEllipse.update();
    lissajous1.update();
    lissajous2.update();
}

void GeometryManager::drawPatterns(int width, int height) {
    // Batch blend mode operations - enable once for all patterns that need it
    bool blendEnabled = hypercube.isEnabled() || line.isEnabled() || sevenStar.isEnabled() || 
                        spiralEllipse.isEnabled() || lissajous1.isEnabled() || lissajous2.isEnabled();
    
    if (blendEnabled) {
        ofEnableBlendMode(OF_BLENDMODE_ADD);
    }
    
    // Draw all patterns
    hypercube.draw(width, height);
    line.draw(width, height);
    sevenStar.draw(width, height);
    spiralEllipse.draw(width, height);
    lissajous1.draw(width, height);
    lissajous2.draw(width, height);
    
    // Disable blend mode once at the end
    if (blendEnabled) {
        ofDisableBlendMode();
    }
}

} // namespace dragonwaves
