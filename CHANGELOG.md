# Changelog

All notable changes to RustJay Waaaves will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial release

### Changed
- 

### Fixed
- HSB value clamping to prevent color distortion with brightness/saturation invert
- Added missing temporal filter controls for Block 2 FB2

## [0.1.0] - 2025-03-02

### Added
- Core VJ engine with 3-block shader pipeline
- Block 1: Channel mixing with FB1 feedback
- Block 2: Secondary processing with FB2 feedback  
- Block 3: Matrix mixer and final output
- ImGui-based control interface
- Webcam input support (macOS/Windows/Linux)
- Audio reactivity with FFT analysis
- LFO modulation system with tempo sync
- Preset system with save/load
- OpenFrameworks preset compatibility
- Temporal filtering for feedback smoothing
- Feedback delay (0-120 frames)
- Recording to MP4 via FFmpeg
- OSC address expose for controller integration
- Pop-out tabs for modular UI layout

### Technical
- wgpu-based rendering (Vulkan/Metal/DX12)
- Dual-window architecture (output + control)
- Modular shader architecture for Block 1
- Cross-platform support (macOS, Windows, Linux)

[Unreleased]: https://github.com/yourusername/rustjay_waaaves/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/yourusername/rustjay_waaaves/releases/tag/v0.1.0
