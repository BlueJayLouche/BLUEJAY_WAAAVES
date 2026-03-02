#!/bin/bash
# Package RustJay Waaaves for Linux

set -e

VERSION=$(grep '^version' Cargo.toml | head -1 | cut -d'"' -f2)
RELEASE_DIR="release/rustjay_waaaves-${VERSION}-linux"

mkdir -p "$RELEASE_DIR"

echo "Building Linux release..."

# Build release
echo "Building release binary..."
RUSTFLAGS="-C target-cpu=native" cargo build --release

# Copy binary
cp target/release/rustjay_waaaves "$RELEASE_DIR/"
chmod +x "$RELEASE_DIR/rustjay_waaaves"

# Copy assets
echo "Copying assets..."
cp -r presets "$RELEASE_DIR/"

# Copy documentation
cp README.md "$RELEASE_DIR/" 2>/dev/null || true
cp LICENSE "$RELEASE_DIR/" 2>/dev/null || true

# Create start script
cat > "$RELEASE_DIR/rustjay_waaaves.sh" << 'EOF'
#!/bin/bash
# RustJay Waaaves Launcher

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Run the application
./rustjay_waaaves "$@"
EOF
chmod +x "$RELEASE_DIR/rustjay_waaaves.sh"

# Create desktop entry
cat > "$RELEASE_DIR/rustjay-waaaves.desktop" << EOF
[Desktop Entry]
Name=RustJay Waaaves
Comment=High-performance VJ application
Exec=$SCRIPT_DIR/rustjay_waaaves
Icon=$SCRIPT_DIR/rustjay-waaaves.png
Terminal=false
Type=Application
Categories=AudioVideo;Video;Graphics;
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

# Create AppImage (optional - requires appimagetool)
if command -v appimagetool &> /dev/null; then
    echo "Creating AppImage..."
    
    # Create AppDir structure
    APPDIR="release/AppDir"
    mkdir -p "$APPDIR/usr/bin"
    mkdir -p "$APPDIR/usr/share/applications"
    mkdir -p "$APPDIR/usr/share/icons/hicolor/256x256/apps"
    
    cp "$RELEASE_DIR/rustjay_waaaves" "$APPDIR/usr/bin/"
    cp -r "$RELEASE_DIR/presets" "$APPDIR/usr/bin/"
    cp "$RELEASE_DIR/rustjay-waaaves.desktop" "$APPDIR/usr/share/applications/"
    cp "$RELEASE_DIR/rustjay-waaaves.desktop" "$APPDIR/"
    
    # Create AppRun script
    cat > "$APPDIR/AppRun" << 'EOF'
#!/bin/bash
SELF=$(readlink -f "$0")
HERE=${SELF%/*}
export PATH="${HERE}/usr/bin:${PATH}"
cd "$HERE/usr/bin"
exec ./rustjay_waaaves "$@"
EOF
    chmod +x "$APPDIR/AppRun"
    
    appimagetool "$APPDIR" "release/rustjay_waaaves-${VERSION}-x86_64.AppImage"
    echo "AppImage created: rustjay_waaaves-${VERSION}-x86_64.AppImage"
fi

# Create tarball
echo "Creating tarball..."
cd release
tar -czf "rustjay_waaaves-${VERSION}-linux.tar.gz" "rustjay_waaaves-${VERSION}-linux"
cd ..

echo "Linux package created: rustjay_waaaves-${VERSION}-linux.tar.gz"
