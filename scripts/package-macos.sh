#!/bin/bash
# Package RustJay Waaaves for macOS as an .app bundle

set -e

VERSION=$(grep '^version' Cargo.toml | head -1 | cut -d'"' -f2)
APP_NAME="RustJay Waaaves"
APP_BUNDLE="RustJayWaaaves.app"
RELEASE_DIR="release/rustjay_waaaves-${VERSION}"

mkdir -p "$RELEASE_DIR"

echo "Building macOS app bundle..."

# Build release
echo "Building release binary..."
RUSTFLAGS="-C target-cpu=native" cargo build --release

# Create app bundle structure
APP_CONTENTS="$RELEASE_DIR/$APP_BUNDLE/Contents"
mkdir -p "$APP_CONTENTS/MacOS"
mkdir -p "$APP_CONTENTS/Resources"

# Copy binary
cp target/release/rustjay_waaaves "$APP_CONTENTS/MacOS/"

# Copy resources
cp -r presets "$APP_CONTENTS/Resources/"
cp README.md "$APP_CONTENTS/Resources/" 2>/dev/null || true

# Create Info.plist
cat > "$APP_CONTENTS/Info.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleName</key>
    <string>${APP_NAME}</string>
    <key>CFBundleDisplayName</key>
    <string>${APP_NAME}</string>
    <key>CFBundleIdentifier</key>
    <string>com.rustjay.waaaves</string>
    <key>CFBundleVersion</key>
    <string>${VERSION}</string>
    <key>CFBundleShortVersionString</key>
    <string>${VERSION}</string>
    <key>CFBundleExecutable</key>
    <string>rustjay_waaaves</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>LSMinimumSystemVersion</key>
    <string>11.0</string>
    <key>NSCameraUsageDescription</key>
    <string>RustJay Waaaves needs camera access for video input.</string>
    <key>NSMicrophoneUsageDescription</key>
    <string>RustJay Waaaves needs microphone access for audio reactivity.</string>
    <key>LSBackgroundOnly</key>
    <false/>
</dict>
</plist>
EOF

# Sign the app (optional, ad-hoc signing)
echo "Signing app bundle..."
codesign --force --deep --sign - "$RELEASE_DIR/$APP_BUNDLE" 2>/dev/null || true

# Create DMG
echo "Creating DMG..."
DMG_NAME="rustjay_waaaves-${VERSION}-macos.dmg"

# Check if create-dmg is installed
if command -v create-dmg &> /dev/null; then
    create-dmg \
        --volname "${APP_NAME}" \
        --window-pos 200 120 \
        --window-size 800 400 \
        --icon-size 100 \
        --app-drop-link 600 185 \
        "$RELEASE_DIR/../$DMG_NAME" \
        "$RELEASE_DIR/$APP_BUNDLE"
else
    echo "create-dmg not found, creating simple DMG with hdiutil..."
    hdiutil create -volname "${APP_NAME}" -srcfolder "$RELEASE_DIR/$APP_BUNDLE" -ov -format UDZO "$RELEASE_DIR/../$DMG_NAME"
fi

echo "macOS package created: $DMG_NAME"
