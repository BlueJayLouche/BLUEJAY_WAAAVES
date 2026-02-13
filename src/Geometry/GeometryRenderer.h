#pragma once

#include "ofMain.h"

namespace dragonwaves {

//==============================================================================
// Base class for geometric patterns
//==============================================================================
class GeometricPattern {
public:
    GeometricPattern(const std::string& name);
    virtual ~GeometricPattern() = default;
    
    virtual void setup() {}
    virtual void update() {}
    virtual void draw(int width, int height) = 0;
    
    const std::string& getName() const { return name; }
    bool isEnabled() const { return enabled; }
    void setEnabled(bool enable) { enabled = enable; }
    
protected:
    std::string name;
    bool enabled = false;
};

//==============================================================================
// Hypercube pattern
//==============================================================================
class HypercubePattern : public GeometricPattern {
public:
    HypercubePattern();
    
    void setup() override;
    void update() override;
    void draw(int width, int height) override;
    
    // Parameters
    float thetaRate = 0.01f;
    float phiRate = 0.01f;
    float size = 1.0f;
    
private:
    float theta = 0.0f;
    float phi = 0.0f;
    float r = 0.0f;
    float x[8];
    float y[8];
    float z[8];
    float colorTheta = 0.0f;
};

//==============================================================================
// Line/Wireframe pattern
//==============================================================================
class LinePattern : public GeometricPattern {
public:
    LinePattern();
    
    void update() override;
    void draw(int width, int height) override;
    
    // Parameters
    float thetaRate = 0.01f;
    float phiRate = 0.01f;
    float etaRate = 0.01f;
    
private:
    float theta = 0.0f;
    float phi = 0.0f;
    float eta = 0.0f;
};

//==============================================================================
// Seven Star pattern
//==============================================================================
class SevenStarPattern : public GeometricPattern {
public:
    SevenStarPattern();
    
    void setup() override;
    void update() override;
    void draw(int width, int height) override;
    
    // Parameters
    float acceleration1 = 0.002f;
    float acceleration2 = 0.00125f;
    float threshold = 0.125f;
    float hueInc1 = 0.021257f;
    float hueInc2 = 0.083713f;
    float saturationInc1 = 0.00612374f;
    float chaosInc = 0.0001f;
    
private:
    static constexpr int REPS = 7;
    
    ofVec2f points[REPS];
    ofVec2f points1[REPS + 1];
    ofVec2f position1;
    float increment1 = 0.0f;
    int index1 = 0;
    
    ofVec2f points2[REPS];
    ofVec2f position2;
    float increment2 = 0.0f;
    int index2 = 0;
    
    float thetaHue1 = 0.0f;
    float thetaHue2 = 0.0f;
    float thetaSaturation1 = 0.0f;
    float thetaChaos = 0.0f;
};

//==============================================================================
// Spiral Ellipse (LissaBall) pattern
//==============================================================================
class SpiralEllipsePattern : public GeometricPattern {
public:
    SpiralEllipsePattern();
    
    void update() override;
    void draw(int width, int height) override;
    
    // Parameters
    float radius1Inc = 0.75f;
    float theta1Inc = 0.07f;
    float radius2Inc = 0.55f;
    float theta2Inc = 0.08f;
    float radius3Inc = 0.65f;
    float theta3Inc = 0.05f;
    
private:
    float spiralTheta1 = 0.0f;
    float spiralRadius1 = 0.0f;
    float spiralTheta2 = 0.0f;
    float spiralRadius2 = 0.0f;
    float spiralTheta3 = 0.0f;
    float spiralRadius3 = 0.0f;
    float spiralTheta1Inc = 0.01f;
    float spiralTheta2Inc = 0.01f;
    float spiralTheta3Inc = 0.01f;
};

//==============================================================================
// Lissajous Curve pattern
//==============================================================================
class LissajousPattern : public GeometricPattern {
public:
    LissajousPattern(const std::string& name);
    
    void update() override;
    void draw(int width, int height) override;
    
    // Base parameters
    float xFreq = 0.1f;
    float yFreq = 0.2f;
    float zFreq = 0.3f;
    float xAmp = 1.0f;
    float yAmp = 1.0f;
    float zAmp = 0.5f;
    float xPhase = 0.0f;
    float yPhase = 0.0f;
    float zPhase = 0.0f;
    float xOffset = 0.5f;
    float yOffset = 0.5f;
    float speed = 0.5f;
    float size = 0.5f;
    float numPoints = 0.5f;
    float lineWidth = 0.2f;
    float colorSpeed = 0.5f;
    float hue = 0.5f;
    float hueSpread = 1.0f;
    
    // Waveshapes (0=Sine, 1=Triangle, 2=Ramp, 3=Saw, 4=Square)
    int xShape = 0;
    int yShape = 0;
    int zShape = 0;
    
    // LFO parameters (amplitude and rate for each parameter)
    struct LFO {
        float amp = 0.0f;
        float rate = 0.0f;
        int shape = 0;
    };
    
    LFO xFreqLfo;
    LFO yFreqLfo;
    LFO zFreqLfo;
    LFO xAmpLfo;
    LFO yAmpLfo;
    LFO zAmpLfo;
    LFO xPhaseLfo;
    LFO yPhaseLfo;
    LFO zPhaseLfo;
    LFO xOffsetLfo;
    LFO yOffsetLfo;
    LFO speedLfo;
    LFO sizeLfo;
    LFO numPointsLfo;
    LFO lineWidthLfo;
    LFO colorSpeedLfo;
    LFO hueLfo;
    LFO hueSpreadLfo;
    
private:
    float theta = 0.0f;
    float colorTheta = 0.0f;
    float xFreqLfoTheta = 0.0f;
    float yFreqLfoTheta = 0.0f;
    float speedLfoTheta = 0.0f;
    
    float lissajousWave(float theta, int shape);
};

//==============================================================================
// Geometry manager - handles all patterns
//==============================================================================
class GeometryManager {
public:
    GeometryManager();
    ~GeometryManager();
    
    void setup();
    void update();
    
    // Draw patterns (after shader processing)
    void drawPatterns(int width, int height);
    
    // Get patterns
    HypercubePattern& getHypercube() { return hypercube; }
    LinePattern& getLine() { return line; }
    SevenStarPattern& getSevenStar() { return sevenStar; }
    SpiralEllipsePattern& getSpiralEllipse() { return spiralEllipse; }
    LissajousPattern& getLissajous1() { return lissajous1; }
    LissajousPattern& getLissajous2() { return lissajous2; }
    
private:
    HypercubePattern hypercube;
    LinePattern line;
    SevenStarPattern sevenStar;
    SpiralEllipsePattern spiralEllipse;
    LissajousPattern lissajous1;
    LissajousPattern lissajous2;
};

} // namespace dragonwaves

// Backwards compatibility
typedef dragonwaves::GeometryManager GeometryManager;
