# Packaging Guide for RustJay Waaaves

This guide explains how to build and package RustJay Waaaves for distribution on different platforms.

## Prerequisites

### All Platforms
- Rust toolchain (1.75+): https://rustup.rs/
- Git

### Platform-Specific

**macOS:**
- Xcode Command Line Tools
- (Optional) `create-dmg` for nice DMG creation: `brew install create-dmg`

**Windows:**
- Visual Studio Build Tools or MinGW-w64
- (Optional) 7-Zip or PowerShell for creating archives

**Linux:**
- build-essential / base-devel
- (Optional) `appimagetool` for AppImage creation

---

## Quick Start

### 1. Basic Release Build

```bash
# Run the build script
./scripts/build-release.sh
```

This creates a release directory with the binary and assets in `release/rustjay_waaaves-{version}/`.

---

## Platform-Specific Packaging

### macOS (.app bundle + DMG)

```bash
./scripts/package-macos.sh
```

**Output:**
- `release/RustJayWaaaves.app` - macOS app bundle
- `release/rustjay_waaaves-{version}-macos.dmg` - Installable disk image

**Features:**
- Native .app bundle with Info.plist
- Camera and microphone permission descriptions
- Ad-hoc code signing
- DMG for easy distribution

**Installation:**
Users drag the .app to their Applications folder.

---

### Windows (.zip)

```bash
# From Windows (Git Bash, MSYS2, or Cygwin)
./scripts/package-windows.sh
```

**Output:**
- `release/rustjay_waaaves-{version}-windows/` - Portable folder
- `release/rustjay_waaaves-{version}-windows.zip` - Zipped distribution

**Features:**
- Portable (no installation required)
- Includes presets and default config
- Start batch script included

**Cross-compilation from macOS/Linux:**
```bash
# Install cross-compilation target
rustup target add x86_64-pc-windows-gnu

# Install MinGW (macOS)
brew install mingw-w64

# Build
./scripts/package-windows.sh
```

---

### Linux (.tar.gz + AppImage)

```bash
./scripts/package-linux.sh
```

**Output:**
- `release/rustjay_waaaves-{version}-linux/` - Portable folder
- `release/rustjay_waaaves-{version}-linux.tar.gz` - Compressed archive
- `release/rustjay_waaaves-{version}-x86_64.AppImage` - Universal binary (if appimagetool installed)

**Features:**
- Portable folder distribution
- Desktop entry file
- Shell launcher script
- Optional AppImage (works on most distributions)

**Dependencies for end users:**
The binary is mostly self-contained, but may need:
- GPU drivers (Vulkan, Metal, or DX12)
- Camera permissions (for webcam input)

---

## What's Included in a Release

```
rustjay_waaaves-{version}/
├── rustjay_waaaves          # Main binary (or .exe on Windows)
├── rustjay_waaaves.sh       # Launcher script (Linux)
├── Start RustJay Waaaves.bat # Launcher script (Windows)
├── rustjay-waaaves.desktop   # Desktop entry (Linux)
├── config.toml              # Default configuration
├── presets/                 # Preset banks
│   └── Default/
│       ├── astart.json
│       ├── xfiles.json
│       └── ...
├── README.md                # Documentation
└── LICENSE                  # License file
```

---

## Optimization Settings

The release profile in `Cargo.toml` includes:

```toml
[profile.release]
opt-level = 3        # Maximum optimization
lto = "fat"          # Full Link Time Optimization
codegen-units = 1    # Single codegen unit for better optimization
panic = "abort"      # Smaller binary, no unwinding
```

This produces an optimized binary at the cost of longer compile times.

---

## Binary Size Optimization (Optional)

If you want smaller binaries:

```bash
# Install strip tool for Rust
cargo install cargo-strip

# Build and strip symbols
cargo build --release
cargo strip

# Or use UPX for compression (macOS/Linux)
brew install upx  # macOS
upx --best target/release/rustjay_waaaves
```

**Note:** Stripping symbols removes debug info, making crash reports less useful.

---

## Code Signing (Recommended for Distribution)

### macOS

```bash
# With Apple Developer Certificate
codesign --force --deep --sign "Developer ID Application: Your Name" RustJayWaaaves.app

# Notarize (for distribution outside App Store)
xcrun altool --notarize-app --primary-bundle-id "com.rustjay.waaaves" \
  --username "your@email.com" --password "@keychain:AC_PASSWORD" \
  --file rustjay_waaaves-{version}-macos.dmg
```

### Windows

Use `signtool` from Windows SDK or a tool like `osslsigncode`:

```bash
# With code signing certificate
osslsigncode sign -pkcs12 certificate.p12 -n "RustJay Waaaves" \
  -i https://rustjay.dev -in rustjay_waaaves.exe -out rustjay_waaaves-signed.exe
```

---

## Automated Releases with GitHub Actions

Create `.github/workflows/release.yml`:

```yaml
name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  build-macos:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-action@stable
      - run: ./scripts/package-macos.sh
      - uses: actions/upload-artifact@v3
        with:
          name: macos-release
          path: release/*.dmg

  build-windows:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-action@stable
      - run: ./scripts/package-windows.sh
        shell: bash
      - uses: actions/upload-artifact@v3
        with:
          name: windows-release
          path: release/*.zip

  build-linux:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-action@stable
      - run: ./scripts/package-linux.sh
      - uses: actions/upload-artifact@v3
        with:
          name: linux-release
          path: release/*.tar.gz
```

---

## Testing a Release

Before publishing, test the release package:

1. **Clean environment test:**
   ```bash
   # Move to a clean machine or VM
   # Extract the package
   # Run without having Rust installed
   ```

2. **Test presets load correctly:**
   - Launch the app
   - Load a few presets
   - Verify they apply correctly

3. **Test webcam input:**
   - Connect a camera
   - Select it as input
   - Verify video displays

4. **Test audio reactivity:**
   - Enable audio input
   - Play some music
   - Verify visualization reacts

---

## Troubleshooting

### "Library not found" errors
The wgpu crate uses system GPU drivers. Ensure end users have:
- **macOS**: macOS 11.0+ (Metal support)
- **Windows**: DirectX 12 or Vulkan compatible GPU
- **Linux**: Mesa drivers or proprietary GPU drivers

### Webcam not working
On macOS, the app needs camera permissions. The Info.plist includes the required `NSCameraUsageDescription`.

### Large binary size
The binary includes all shader code and is statically linked. This is normal for ~20-50MB.

### Slow startup
First launch may be slow due to shader compilation caching. Subsequent launches will be faster.

---

## Version Numbering

Update the version in `Cargo.toml` before each release:

```toml
[package]
version = "0.1.1"  # Bump this
```

Follow [Semantic Versioning](https://semver.org/):
- **MAJOR**: Breaking changes
- **MINOR**: New features (backward compatible)
- **PATCH**: Bug fixes

---

## Distribution Checklist

- [ ] Version bumped in `Cargo.toml`
- [ ] `CHANGELOG.md` updated
- [ ] Git tag created: `git tag v0.1.1`
- [ ] All platforms built and tested
- [ ] Presets included in package
- [ ] README and LICENSE included
- [ ] (Optional) Code signed
- [ ] (Optional) GitHub Release created with binaries

---

## Questions?

For packaging issues, check:
1. The build output for errors
2. That all assets are in the release folder
3. That the binary runs on a clean system
