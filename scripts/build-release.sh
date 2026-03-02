#!/bin/bash
# Build release script for RustJay Waaaves

set -e

echo "Building RustJay Waaaves Release..."

# Get version from Cargo.toml
VERSION=$(grep '^version' Cargo.toml | head -1 | cut -d'"' -f2)
echo "Version: $VERSION"

# Clean previous builds
echo "Cleaning previous builds..."
cargo clean

# Build optimized release
echo "Building optimized release binary..."
RUSTFLAGS="-C target-cpu=native" cargo build --release

# Create release directory
RELEASE_DIR="release/rustjay_waaaves-${VERSION}"
mkdir -p "$RELEASE_DIR"

# Copy binary
echo "Copying binary..."
if [[ "$OSTYPE" == "darwin"* ]]; then
    cp target/release/rustjay_waaaves "$RELEASE_DIR/"
    chmod +x "$RELEASE_DIR/rustjay_waaaves"
elif [[ "$OSTYPE" == "msys" || "$OSTYPE" == "cygwin" ]]; then
    cp target/release/rustjay_waaaves.exe "$RELEASE_DIR/"
else
    cp target/release/rustjay_waaaves "$RELEASE_DIR/"
    chmod +x "$RELEASE_DIR/rustjay_waaaves"
fi

# Copy assets
echo "Copying assets..."
cp -r presets "$RELEASE_DIR/"

# Copy documentation
cp README.md "$RELEASE_DIR/" 2>/dev/null || true
cp LICENSE "$RELEASE_DIR/" 2>/dev/null || true

# Create default config if it doesn't exist
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

echo "Release built in: $RELEASE_DIR"
echo ""
echo "Binary size:"
ls -lh "$RELEASE_DIR"/rustjay_waaaves*
