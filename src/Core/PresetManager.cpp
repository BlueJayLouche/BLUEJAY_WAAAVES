#include "PresetManager.h"
#include <algorithm>

namespace dragonwaves {

//==============================================================================
// PresetData
//==============================================================================
PresetData::PresetData() {
    // Initialize arrays with size 16 (PARAMETER_ARRAY_LENGTH)
    ch1Adjust.resize(16, 0.0f);
    ch2MixAndKey.resize(16, 0.0f);
    ch2Adjust.resize(16, 0.0f);
    ch1AdjustLfo.resize(16, 0.0f);
    ch2MixAndKeyLfo.resize(16, 0.0f);
    ch2AdjustLfo.resize(16, 0.0f);
    fb1MixAndKey.resize(16, 0.0f);
    fb1Geo1.resize(16, 0.0f);
    fb1Color1.resize(16, 0.0f);
    fb1Filters.resize(16, 0.0f);
    fb1MixAndKeyLfo.resize(16, 0.0f);
    fb1Geo1Lfo1.resize(16, 0.0f);
    fb1Geo1Lfo2.resize(16, 0.0f);
    fb1Color1Lfo1.resize(16, 0.0f);
    
    block2InputAdjust.resize(16, 0.0f);
    block2InputAdjustLfo.resize(16, 0.0f);
    fb2MixAndKey.resize(16, 0.0f);
    fb2Geo1.resize(16, 0.0f);
    fb2Color1.resize(16, 0.0f);
    fb2Filters.resize(16, 0.0f);
    fb2MixAndKeyLfo.resize(16, 0.0f);
    fb2Geo1Lfo1.resize(16, 0.0f);
    fb2Geo1Lfo2.resize(16, 0.0f);
    fb2Color1Lfo1.resize(16, 0.0f);
    
    block1Geo.resize(16, 0.0f);
    block1Colorize.resize(16, 0.0f);
    block1Filters.resize(16, 0.0f);
    block1Geo1Lfo1.resize(16, 0.0f);
    block1Geo1Lfo2.resize(16, 0.0f);
    block1ColorizeLfo1.resize(16, 0.0f);
    block1ColorizeLfo2.resize(16, 0.0f);
    block1ColorizeLfo3.resize(16, 0.0f);
    
    block2Geo.resize(16, 0.0f);
    block2Colorize.resize(16, 0.0f);
    block2Filters.resize(16, 0.0f);
    block2Geo1Lfo1.resize(16, 0.0f);
    block2Geo1Lfo2.resize(16, 0.0f);
    block2ColorizeLfo1.resize(16, 0.0f);
    block2ColorizeLfo2.resize(16, 0.0f);
    block2ColorizeLfo3.resize(16, 0.0f);
    
    matrixMix.resize(16, 0.0f);
    finalMixAndKey.resize(16, 0.0f);
    matrixMixLfo1.resize(16, 0.0f);
    matrixMixLfo2.resize(16, 0.0f);
    finalMixAndKeyLfo.resize(16, 0.0f);
}

//==============================================================================
// PresetManager
//==============================================================================
void PresetManager::setup() {
    migrateOldSaveStates();
    scanBanks();
    
    // Set default bank
    if (banks.find("Default") != banks.end()) {
        currentBank = "Default";
    } else if (!banks.empty()) {
        currentBank = banks.begin()->first;
    }
    
    indexPresets();
    
    ofLogNotice("PresetManager") << "Setup complete. Banks: " << banks.size() 
                                  << ", Current: " << currentBank;
}

void PresetManager::scanBanks() {
    banks.clear();
    
    ofDirectory presetsDir(basePath);
    if (!presetsDir.exists()) {
        ofDirectory::createDirectory(basePath, false, true);
        ofLogNotice("PresetManager") << "Created presets directory";
    }
    
    presetsDir.listDir();
    
    for (int i = 0; i < presetsDir.size(); i++) {
        if (presetsDir.getFile(i).isDirectory()) {
            std::string name = presetsDir.getName(i);
            PresetBank bank;
            bank.name = name;
            bank.path = basePath + name + "/";
            banks[name] = bank;
        }
    }
    
    // Ensure Default bank exists
    if (banks.find("Default") == banks.end()) {
        createBank("Default");
    }
    
    ofLogNotice("PresetManager") << "Scanned " << banks.size() << " banks";
}

std::vector<std::string> PresetManager::getBankNames() const {
    std::vector<std::string> names;
    for (const auto& pair : banks) {
        names.push_back(pair.first);
    }
    std::sort(names.begin(), names.end());
    return names;
}

void PresetManager::switchBank(const std::string& bankName) {
    if (banks.find(bankName) != banks.end()) {
        currentBank = bankName;
        indexPresets();
        ofLogNotice("PresetManager") << "Switched to bank: " << bankName;
    } else {
        ofLogWarning("PresetManager") << "Bank not found: " << bankName;
    }
}

void PresetManager::createBank(const std::string& bankName) {
    std::string path = basePath + bankName + "/";
    ofDirectory::createDirectory(path, false, true);
    
    PresetBank bank;
    bank.name = bankName;
    bank.path = path;
    banks[bankName] = bank;
    
    ofLogNotice("PresetManager") << "Created bank: " << bankName;
}

void PresetManager::indexPresets() {
    auto it = banks.find(currentBank);
    if (it == banks.end()) return;
    
    PresetBank& bank = it->second;
    bank.presetFiles.clear();
    bank.presetDisplayNames.clear();
    
    ofDirectory dir(bank.path);
    dir.allowExt("json");
    dir.listDir();
    dir.sort();
    
    for (int i = 0; i < dir.size(); i++) {
        std::string filename = dir.getName(i);
        bank.presetFiles.push_back(filename);
        bank.presetDisplayNames.push_back(cleanDisplayName(filename));
    }
    
    ofLogNotice("PresetManager") << "Indexed " << bank.presetFiles.size() 
                                  << " presets in " << currentBank;
}

std::vector<std::string> PresetManager::getPresetNames() const {
    auto it = banks.find(currentBank);
    if (it != banks.end()) {
        return it->second.presetDisplayNames;
    }
    return std::vector<std::string>();
}

bool PresetManager::savePreset(const std::string& name, const PresetData& data) {
    auto it = banks.find(currentBank);
    if (it == banks.end()) return false;
    
    std::string filename = generatePresetFilename(name);
    std::string fullPath = it->second.path + filename;
    
    ofJson json;
    
    // Block 1
    json["block1"]["ch1Adjust"] = data.ch1Adjust;
    json["block1"]["ch2MixAndKey"] = data.ch2MixAndKey;
    json["block1"]["ch2Adjust"] = data.ch2Adjust;
    json["block1"]["ch1AdjustLfo"] = data.ch1AdjustLfo;
    json["block1"]["ch2MixAndKeyLfo"] = data.ch2MixAndKeyLfo;
    json["block1"]["ch2AdjustLfo"] = data.ch2AdjustLfo;
    json["block1"]["fb1MixAndKey"] = data.fb1MixAndKey;
    json["block1"]["fb1Geo1"] = data.fb1Geo1;
    json["block1"]["fb1Color1"] = data.fb1Color1;
    json["block1"]["fb1Filters"] = data.fb1Filters;
    json["block1"]["fb1MixAndKeyLfo"] = data.fb1MixAndKeyLfo;
    json["block1"]["fb1Geo1Lfo1"] = data.fb1Geo1Lfo1;
    json["block1"]["fb1Geo1Lfo2"] = data.fb1Geo1Lfo2;
    json["block1"]["fb1Color1Lfo1"] = data.fb1Color1Lfo1;
    json["block1"]["fb1DelayTime"] = data.fb1DelayTime;
    
    // Block 2
    json["block2"]["block2InputAdjust"] = data.block2InputAdjust;
    json["block2"]["block2InputAdjustLfo"] = data.block2InputAdjustLfo;
    json["block2"]["fb2MixAndKey"] = data.fb2MixAndKey;
    json["block2"]["fb2Geo1"] = data.fb2Geo1;
    json["block2"]["fb2Color1"] = data.fb2Color1;
    json["block2"]["fb2Filters"] = data.fb2Filters;
    json["block2"]["fb2MixAndKeyLfo"] = data.fb2MixAndKeyLfo;
    json["block2"]["fb2Geo1Lfo1"] = data.fb2Geo1Lfo1;
    json["block2"]["fb2Geo1Lfo2"] = data.fb2Geo1Lfo2;
    json["block2"]["fb2Color1Lfo1"] = data.fb2Color1Lfo1;
    json["block2"]["fb2DelayTime"] = data.fb2DelayTime;
    
    // Block 3
    json["block3"]["block1Geo"] = data.block1Geo;
    json["block3"]["block1Colorize"] = data.block1Colorize;
    json["block3"]["block1Filters"] = data.block1Filters;
    json["block3"]["block1Geo1Lfo1"] = data.block1Geo1Lfo1;
    json["block3"]["block1Geo1Lfo2"] = data.block1Geo1Lfo2;
    json["block3"]["block1ColorizeLfo1"] = data.block1ColorizeLfo1;
    json["block3"]["block1ColorizeLfo2"] = data.block1ColorizeLfo2;
    json["block3"]["block1ColorizeLfo3"] = data.block1ColorizeLfo3;
    json["block3"]["block2Geo"] = data.block2Geo;
    json["block3"]["block2Colorize"] = data.block2Colorize;
    json["block3"]["block2Filters"] = data.block2Filters;
    json["block3"]["block2Geo1Lfo1"] = data.block2Geo1Lfo1;
    json["block3"]["block2Geo1Lfo2"] = data.block2Geo1Lfo2;
    json["block3"]["block2ColorizeLfo1"] = data.block2ColorizeLfo1;
    json["block3"]["block2ColorizeLfo2"] = data.block2ColorizeLfo2;
    json["block3"]["block2ColorizeLfo3"] = data.block2ColorizeLfo3;
    json["block3"]["matrixMix"] = data.matrixMix;
    json["block3"]["finalMixAndKey"] = data.finalMixAndKey;
    json["block3"]["matrixMixLfo1"] = data.matrixMixLfo1;
    json["block3"]["matrixMixLfo2"] = data.matrixMixLfo2;
    json["block3"]["finalMixAndKeyLfo"] = data.finalMixAndKeyLfo;
    
    // Switches
    json["switches"]["ch1InputSelect"] = data.ch1InputSelect;
    json["switches"]["ch2InputSelect"] = data.ch2InputSelect;
    json["switches"]["block2InputSelect"] = data.block2InputSelect;
    
    ofSaveJson(fullPath, json);
    
    indexPresets();
    
    if (onPresetSaved) {
        onPresetSaved(name);
    }
    
    ofLogNotice("PresetManager") << "Saved preset: " << name << " to " << fullPath;
    return true;
}

bool PresetManager::loadPreset(const std::string& name, PresetData& data) {
    auto it = banks.find(currentBank);
    if (it == banks.end()) return false;
    
    // Find preset file by display name
    auto& bank = it->second;
    int index = -1;
    for (int i = 0; i < bank.presetDisplayNames.size(); i++) {
        if (bank.presetDisplayNames[i] == name) {
            index = i;
            break;
        }
    }
    
    if (index < 0) {
        // Try as filename
        for (int i = 0; i < bank.presetFiles.size(); i++) {
            if (cleanDisplayName(bank.presetFiles[i]) == name) {
                index = i;
                break;
            }
        }
    }
    
    if (index < 0) {
        ofLogWarning("PresetManager") << "Preset not found: " << name;
        return false;
    }
    
    std::string fullPath = bank.path + bank.presetFiles[index];
    ofJson json = ofLoadJson(fullPath);
    
    if (json.is_null()) {
        ofLogError("PresetManager") << "Failed to load JSON: " << fullPath;
        return false;
    }
    
    // Helper to load array
    auto loadArray = [](ofJson& j, const std::string& key, std::vector<float>& arr) {
        if (j.contains(key) && j[key].is_array()) {
            for (int i = 0; i < std::min((int)arr.size(), (int)j[key].size()); i++) {
                arr[i] = j[key][i].get<float>();
            }
        }
    };
    
    // Block 1
    if (json.contains("block1")) {
        auto& b1 = json["block1"];
        loadArray(b1, "ch1Adjust", data.ch1Adjust);
        loadArray(b1, "ch2MixAndKey", data.ch2MixAndKey);
        loadArray(b1, "ch2Adjust", data.ch2Adjust);
        loadArray(b1, "ch1AdjustLfo", data.ch1AdjustLfo);
        loadArray(b1, "ch2MixAndKeyLfo", data.ch2MixAndKeyLfo);
        loadArray(b1, "ch2AdjustLfo", data.ch2AdjustLfo);
        loadArray(b1, "fb1MixAndKey", data.fb1MixAndKey);
        loadArray(b1, "fb1Geo1", data.fb1Geo1);
        loadArray(b1, "fb1Color1", data.fb1Color1);
        loadArray(b1, "fb1Filters", data.fb1Filters);
        loadArray(b1, "fb1MixAndKeyLfo", data.fb1MixAndKeyLfo);
        loadArray(b1, "fb1Geo1Lfo1", data.fb1Geo1Lfo1);
        loadArray(b1, "fb1Geo1Lfo2", data.fb1Geo1Lfo2);
        loadArray(b1, "fb1Color1Lfo1", data.fb1Color1Lfo1);
        if (b1.contains("fb1DelayTime")) data.fb1DelayTime = b1["fb1DelayTime"];
    }
    
    // Block 2
    if (json.contains("block2")) {
        auto& b2 = json["block2"];
        loadArray(b2, "block2InputAdjust", data.block2InputAdjust);
        loadArray(b2, "block2InputAdjustLfo", data.block2InputAdjustLfo);
        loadArray(b2, "fb2MixAndKey", data.fb2MixAndKey);
        loadArray(b2, "fb2Geo1", data.fb2Geo1);
        loadArray(b2, "fb2Color1", data.fb2Color1);
        loadArray(b2, "fb2Filters", data.fb2Filters);
        loadArray(b2, "fb2MixAndKeyLfo", data.fb2MixAndKeyLfo);
        loadArray(b2, "fb2Geo1Lfo1", data.fb2Geo1Lfo1);
        loadArray(b2, "fb2Geo1Lfo2", data.fb2Geo1Lfo2);
        loadArray(b2, "fb2Color1Lfo1", data.fb2Color1Lfo1);
        if (b2.contains("fb2DelayTime")) data.fb2DelayTime = b2["fb2DelayTime"];
    }
    
    // Block 3
    if (json.contains("block3")) {
        auto& b3 = json["block3"];
        loadArray(b3, "block1Geo", data.block1Geo);
        loadArray(b3, "block1Colorize", data.block1Colorize);
        loadArray(b3, "block1Filters", data.block1Filters);
        loadArray(b3, "block1Geo1Lfo1", data.block1Geo1Lfo1);
        loadArray(b3, "block1Geo1Lfo2", data.block1Geo1Lfo2);
        loadArray(b3, "block1ColorizeLfo1", data.block1ColorizeLfo1);
        loadArray(b3, "block1ColorizeLfo2", data.block1ColorizeLfo2);
        loadArray(b3, "block1ColorizeLfo3", data.block1ColorizeLfo3);
        loadArray(b3, "block2Geo", data.block2Geo);
        loadArray(b3, "block2Colorize", data.block2Colorize);
        loadArray(b3, "block2Filters", data.block2Filters);
        loadArray(b3, "block2Geo1Lfo1", data.block2Geo1Lfo1);
        loadArray(b3, "block2Geo1Lfo2", data.block2Geo1Lfo2);
        loadArray(b3, "block2ColorizeLfo1", data.block2ColorizeLfo1);
        loadArray(b3, "block2ColorizeLfo2", data.block2ColorizeLfo2);
        loadArray(b3, "block2ColorizeLfo3", data.block2ColorizeLfo3);
        loadArray(b3, "matrixMix", data.matrixMix);
        loadArray(b3, "finalMixAndKey", data.finalMixAndKey);
        loadArray(b3, "matrixMixLfo1", data.matrixMixLfo1);
        loadArray(b3, "matrixMixLfo2", data.matrixMixLfo2);
        loadArray(b3, "finalMixAndKeyLfo", data.finalMixAndKeyLfo);
    }
    
    // Switches
    if (json.contains("switches")) {
        auto& sw = json["switches"];
        if (sw.contains("ch1InputSelect")) data.ch1InputSelect = sw["ch1InputSelect"];
        if (sw.contains("ch2InputSelect")) data.ch2InputSelect = sw["ch2InputSelect"];
        if (sw.contains("block2InputSelect")) data.block2InputSelect = sw["block2InputSelect"];
    }
    
    if (onPresetLoaded) {
        onPresetLoaded(name);
    }
    
    ofLogNotice("PresetManager") << "Loaded preset: " << name << " from " << fullPath;
    return true;
}

bool PresetManager::renamePreset(int index, const std::string& newName) {
    auto it = banks.find(currentBank);
    if (it == banks.end()) return false;
    
    auto& bank = it->second;
    if (index < 0 || index >= bank.presetFiles.size()) return false;
    
    std::string oldFilename = bank.presetFiles[index];
    std::string newFilename = generatePresetFilename(newName);
    
    if (oldFilename == newFilename) return true;
    
    std::string oldPath = bank.path + oldFilename;
    std::string newPath = bank.path + newFilename;
    
    ofFile oldFile(oldPath);
    if (oldFile.exists()) {
        if (oldFile.renameTo(newPath)) {
            indexPresets();
            return true;
        }
    }
    
    return false;
}

bool PresetManager::deletePreset(int index) {
    auto it = banks.find(currentBank);
    if (it == banks.end()) return false;
    
    auto& bank = it->second;
    if (index < 0 || index >= bank.presetFiles.size()) return false;
    
    std::string path = bank.path + bank.presetFiles[index];
    ofFile file(path);
    
    if (file.exists()) {
        file.remove();
        indexPresets();
        return true;
    }
    
    return false;
}

void PresetManager::migrateOldSaveStates() {
    ofDirectory oldDir("saveStates");
    ofDirectory presetsDir("presets");
    
    if (presetsDir.exists()) return;  // Already migrated
    
    ofDirectory::createDirectory("presets", false, true);
    ofDirectory::createDirectory("presets/Default", false, true);
    
    if (oldDir.exists()) {
        oldDir.allowExt("json");
        oldDir.listDir();
        
        for (int i = 0; i < oldDir.size(); i++) {
            std::string srcPath = oldDir.getPath(i);
            std::string filename = oldDir.getName(i);
            std::string dstPath = "presets/Default/" + filename;
            
            ofFile srcFile(srcPath);
            srcFile.copyTo(dstPath);
        }
        
        ofLogNotice("PresetManager") << "Migrated " << oldDir.size() << " presets";
    }
}

std::string PresetManager::generatePresetFilename(const std::string& displayName) {
    // Find highest existing prefix number
    auto it = banks.find(currentBank);
    int maxPrefix = 0;
    
    if (it != banks.end()) {
        for (const auto& filename : it->second.presetFiles) {
            size_t pos = 0;
            while (pos < filename.length() && isdigit(filename[pos])) pos++;
            if (pos > 0 && pos < filename.length() && filename[pos] == '_') {
                int prefix = std::stoi(filename.substr(0, pos));
                maxPrefix = std::max(maxPrefix, prefix);
            }
        }
    }
    
    // Sanitize name
    std::string sanitized = displayName;
    for (char& c : sanitized) {
        if (c == '/' || c == '\\' || c == ':' || c == '*' ||
            c == '?' || c == '"' || c == '<' || c == '>' || c == '|') {
            c = '_';
        }
    }
    
    char prefixStr[8];
    snprintf(prefixStr, sizeof(prefixStr), "%03d_", maxPrefix + 1);
    return std::string(prefixStr) + sanitized + ".json";
}

std::string PresetManager::cleanDisplayName(const std::string& filename) {
    std::string name = filename;
    
    // Remove .json
    size_t dotPos = name.find_last_of(".");
    if (dotPos != std::string::npos) {
        name = name.substr(0, dotPos);
    }
    
    // Remove numeric prefix (###_)
    size_t pos = 0;
    while (pos < name.length() && isdigit(name[pos])) pos++;
    if (pos > 0 && pos < name.length() && name[pos] == '_') {
        name = name.substr(pos + 1);
    }
    
    // Remove legacy gwSaveState prefix
    if (name.find("gwSaveState") == 0) {
        size_t gwPos = 11;
        while (gwPos < name.length() && isdigit(name[gwPos])) gwPos++;
        name = name.substr(gwPos);
    }
    
    if (name.empty()) {
        name = "Preset";
    }
    
    return name;
}

} // namespace dragonwaves
