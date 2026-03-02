#!/bin/bash
# Package RustJay Waaaves for Windows
# Run this on Windows or use cross-compilation

set -e

VERSION=$(grep '^version' Cargo.toml | head -1 | cut -d'"' -f2)
RELEASE_DIR="release/rustjay_waaaves-${VERSION}-windows"

mkdir -p "$RELEASE_DIR"

echo "Building Windows release..."

# Build for Windows
echo "Building release binary for Windows..."

# If on Windows (MSYS/Cygwin/Git Bash)
if [[ "$OSTYPE" == "msys" || "$OSTYPE" == "cygwin" || "$OSTYPE" == "win32" ]]; then
    RUSTFLAGS="-C target-cpu=native" cargo build --release
    cp target/release/rustjay_waaaves.exe "$RELEASE_DIR/"
else
    # Cross-compile from macOS/Linux
    echo "Cross-compiling for Windows..."
    rustup target add x86_64-pc-windows-gnu 2>/dev/null || true
    cargo build --release --target x86_64-pc-windows-gnu
    cp target/x86_64-pc-windows-gnu/release/rustjay_waaaves.exe "$RELEASE_DIR/"
fi

# Copy assets
echo "Copying assets..."
cp -r presets "$RELEASE_DIR/"

# Copy documentation
cp README.md "$RELEASE_DIR/" 2>/dev/null || true
cp LICENSE "$RELEASE_DIR/" 2>/dev/null || true

# Create start script
cat > "$RELEASE_DIR/Start RustJay Waaaves.bat" << 'EOF'
@echo off
echo Starting RustJay Waaaves...
start rustjay_waaaves.exe
EOF

# Create portable config
if [ ! -f "$RELEASE_DIR/config.toml" ]; then
    cat > "$RELEASE_DIR/config.toml" << 'EOF'
[resolution]
internal = { preset = "HD720", width = 1280, height = 720 }
output = { preset = "HD1080", width = 1920, height = 1080 }
input = { preset = "SD480", width = 640, height = 480 }

[pipeline]
internal_width = 1280
internal_height = 720

[window]
ui_scale = 1.0

[features]
audio_enabled = true
temporal_filter = true
EOF
fi

# Create ZIP archive
echo "Creating ZIP archive..."
if command -v zip &> /dev/null; then
    cd release
    zip -r "rustjay_waaaves-${VERSION}-windows.zip" "rustjay_waaaves-${VERSION}-windows"
    cd ..
else
    echo "zip command not found, using PowerShell..."
    powershell -Command "Compress-Archive -Path '$RELEASE_DIR' -DestinationPath 'release/rustjay_waaaves-${VERSION}-windows.zip'"
fi

echo "Windows package created: rustjay_waaaves-${VERSION}-windows.zip"
