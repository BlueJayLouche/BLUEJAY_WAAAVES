#pragma once

#include "ofMain.h"
#include "ofxOsc.h"

namespace dragonwaves {

//==============================================================================
// Parameter types
//==============================================================================
enum class ParamType {
    FLOAT,
    INT,
    BOOL
};

//==============================================================================
// Parameter change callback
//==============================================================================
typedef std::function<void()> ParamCallback;

//==============================================================================
// Base parameter class
//==============================================================================
class ParameterBase {
public:
    ParameterBase(const std::string& name, const std::string& oscAddress, ParamType type)
        : name(name), oscAddress(oscAddress), type(type) {}
    
    virtual ~ParameterBase() = default;
    
    // Getters
    const std::string& getName() const { return name; }
    const std::string& getOscAddress() const { return oscAddress; }
    ParamType getType() const { return type; }
    
    // Value access (implemented in template subclass)
    virtual float getAsFloat() const = 0;
    virtual void setFromFloat(float value) = 0;
    virtual int getAsInt() const = 0;
    virtual void setFromInt(int value) = 0;
    virtual bool getAsBool() const = 0;
    virtual void setFromBool(bool value) = 0;
    
    // Callback
    void setCallback(ParamCallback cb) { callback = cb; }
    void notifyChanged() { if (callback) callback(); }
    
protected:
    std::string name;
    std::string oscAddress;
    ParamType type;
    ParamCallback callback;
};

//==============================================================================
// Typed parameter template
//==============================================================================
template<typename T>
class Parameter : public ParameterBase {
public:
    Parameter(const std::string& name, const std::string& oscAddress, T* valuePtr, 
              T minVal = T(0), T maxVal = T(1))
        : ParameterBase(name, oscAddress, getParamType<T>()), 
          valuePtr(valuePtr), minVal(minVal), maxVal(maxVal) {}
    
    T get() const { return *valuePtr; }
    void set(T value) { 
        *valuePtr = glm::clamp(value, minVal, maxVal);
        notifyChanged();
    }
    
    float getAsFloat() const override { return static_cast<float>(*valuePtr); }
    void setFromFloat(float value) override { 
        set(static_cast<T>(glm::clamp(value, 0.0f, 1.0f) * (maxVal - minVal) + minVal));
    }
    
    int getAsInt() const override { return static_cast<int>(*valuePtr); }
    void setFromInt(int value) override { set(static_cast<T>(value)); }
    
    bool getAsBool() const override { return *valuePtr != T(0); }
    void setFromBool(bool value) override { set(value ? maxVal : minVal); }
    
private:
    T* valuePtr;
    T minVal;
    T maxVal;
    
    template<typename U>
    static ParamType getParamType() {
        if (std::is_same<U, float>::value || std::is_same<U, double>::value) {
            return ParamType::FLOAT;
        } else if (std::is_same<U, bool>::value) {
            return ParamType::BOOL;
        } else {
            return ParamType::INT;
        }
    }
};

// Specialization for bool
template<>
inline float Parameter<bool>::getAsFloat() const { return *valuePtr ? 1.0f : 0.0f; }

template<>
inline void Parameter<bool>::setFromFloat(float value) { set(value > 0.5f); }

template<>
inline int Parameter<bool>::getAsInt() const { return *valuePtr ? 1 : 0; }

} // namespace dragonwaves
