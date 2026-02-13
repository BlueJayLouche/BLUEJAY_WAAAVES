#pragma once

#include "ofMain.h"
#include "SettingsManager.h"

namespace dragonwaves {

//==============================================================================
// Preset data structure matching the original format
//==============================================================================
struct PresetData {
    // Block 1
    std::vector<float> ch1Adjust;
    std::vector<float> ch2MixAndKey;
    std::vector<float> ch2Adjust;
    std::vector<float> ch1AdjustLfo;
    std::vector<float> ch2MixAndKeyLfo;
    std::vector<float> ch2AdjustLfo;
    std::vector<float> fb1MixAndKey;
    std::vector<float> fb1Geo1;
    std::vector<float> fb1Color1;
    std::vector<float> fb1Filters;
    std::vector<float> fb1MixAndKeyLfo;
    std::vector<float> fb1Geo1Lfo1;
    std::vector<float> fb1Geo1Lfo2;
    std::vector<float> fb1Color1Lfo1;
    int fb1DelayTime = 1;
    
    // Block 2
    std::vector<float> block2InputAdjust;
    std::vector<float> block2InputAdjustLfo;
    std::vector<float> fb2MixAndKey;
    std::vector<float> fb2Geo1;
    std::vector<float> fb2Color1;
    std::vector<float> fb2Filters;
    std::vector<float> fb2MixAndKeyLfo;
    std::vector<float> fb2Geo1Lfo1;
    std::vector<float> fb2Geo1Lfo2;
    std::vector<float> fb2Color1Lfo1;
    int fb2DelayTime = 1;
    
    // Block 3
    std::vector<float> block1Geo;
    std::vector<float> block1Colorize;
    std::vector<float> block1Filters;
    std::vector<float> block1Geo1Lfo1;
    std::vector<float> block1Geo1Lfo2;
    std::vector<float> block1ColorizeLfo1;
    std::vector<float> block1ColorizeLfo2;
    std::vector<float> block1ColorizeLfo3;
    std::vector<float> block2Geo;
    std::vector<float> block2Colorize;
    std::vector<float> block2Filters;
    std::vector<float> block2Geo1Lfo1;
    std::vector<float> block2Geo1Lfo2;
    std::vector<float> block2ColorizeLfo1;
    std::vector<float> block2ColorizeLfo2;
    std::vector<float> block2ColorizeLfo3;
    std::vector<float> matrixMix;
    std::vector<float> finalMixAndKey;
    std::vector<float> matrixMixLfo1;
    std::vector<float> matrixMixLfo2;
    std::vector<float> finalMixAndKeyLfo;
    
    // Switches and discrete values
    int ch1InputSelect = 0;
    int ch2InputSelect = 1;
    int block2InputSelect = 0;
    
    PresetData();
};

//==============================================================================
// Bank management
//==============================================================================
struct PresetBank {
    std::string name;
    std::string path;
    std::vector<std::string> presetFiles;
    std::vector<std::string> presetDisplayNames;
};

//==============================================================================
// Centralized preset management
//==============================================================================
class PresetManager {
public:
    static PresetManager& getInstance() {
        static PresetManager instance;
        return instance;
    }
    
    // Initialize and scan banks
    void setup();
    
    // Bank management
    void scanBanks();
    std::vector<std::string> getBankNames() const;
    void switchBank(const std::string& bankName);
    void createBank(const std::string& bankName);
    
    // Preset listing
    std::vector<std::string> getPresetNames() const;
    
    // Save/Load
    bool savePreset(const std::string& name, const PresetData& data);
    bool loadPreset(const std::string& name, PresetData& data);
    bool renamePreset(int index, const std::string& newName);
    bool deletePreset(int index);
    
    // Current bank
    std::string getCurrentBankName() const { return currentBank; }
    void setCurrentBank(const std::string& bank) { currentBank = bank; indexPresets(); }
    
    // Legacy migration
    void migrateOldSaveStates();
    
    // Callbacks
    void setOnPresetLoaded(std::function<void(const std::string&)> callback) {
        onPresetLoaded = callback;
    }
    void setOnPresetSaved(std::function<void(const std::string&)> callback) {
        onPresetSaved = callback;
    }
    
private:
    PresetManager() = default;
    ~PresetManager() = default;
    PresetManager(const PresetManager&) = delete;
    PresetManager& operator=(const PresetManager&) = delete;
    
    std::map<std::string, PresetBank> banks;
    std::string currentBank = "Default";
    std::string basePath = "presets/";
    
    std::function<void(const std::string&)> onPresetLoaded;
    std::function<void(const std::string&)> onPresetSaved;
    
    void indexPresets();
    std::string generatePresetFilename(const std::string& displayName);
    std::string cleanDisplayName(const std::string& filename);
};

} // namespace dragonwaves

// Backwards compatibility
typedef dragonwaves::PresetManager PresetManager;
typedef dragonwaves::PresetData PresetData;
