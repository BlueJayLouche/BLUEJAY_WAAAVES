# App Icons

This directory should contain the application icons:

- `32x32.png` - Small icon for window title bar
- `128x128.png` - Medium icon for display
- `icon.icns` - macOS icon file (contains multiple sizes)

## Creating macOS Icon Files

You can create an `.icns` file from PNG images using:

```bash
# Using iconutil (recommended)
mkdir icon.iconset
sips -z 16 16     icon_1024x1024.png --out icon.iconset/icon_16x16.png
sips -z 32 32     icon_1024x1024.png --out icon.iconset/icon_16x16@2x.png
sips -z 32 32     icon_1024x1024.png --out icon.iconset/icon_32x32.png
sips -z 64 64     icon_1024x1024.png --out icon.iconset/icon_32x32@2x.png
sips -z 128 128   icon_1024x1024.png --out icon.iconset/icon_128x128.png
sips -z 256 256   icon_1024x1024.png --out icon.iconset/icon_128x128@2x.png
sips -z 256 256   icon_1024x1024.png --out icon.iconset/icon_256x256.png
sips -z 512 512   icon_1024x1024.png --out icon.iconset/icon_256x256@2x.png
sips -z 512 512   icon_1024x1024.png --out icon.iconset/icon_512x512.png
cp icon_1024x1024.png icon.iconset/icon_512x512@2x.png
iconutil -c icns icon.iconset
mv icon.icns icons/
```

Or use a tool like:
- [IconKit](https://itunes.apple.com/app/iconkit/id507135296) (GUI app)
- [makeicns](https://github.com/awr1/makeicns) (command line)
