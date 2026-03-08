#!/bin/bash
# Fix Syphon framework install name
# Run this script with: ./fix_syphon.sh (or use sudo if needed)

echo "Checking Syphon.framework install name..."

CURRENT_NAME=$(otool -D /Library/Frameworks/Syphon.framework/Syphon 2>/dev/null | tail -1)

echo "Current install name: $CURRENT_NAME"

if echo "$CURRENT_NAME" | grep -q "@loader_path"; then
    echo ""
    echo "⚠️  Syphon.framework has incorrect install name!"
    echo "   This will cause 'Library not loaded' errors."
    echo ""
    echo "Fixing with install_name_tool..."
    
    sudo install_name_tool -id \
        /Library/Frameworks/Syphon.framework/Versions/A/Syphon \
        /Library/Frameworks/Syphon.framework/Syphon
    
    if [ $? -eq 0 ]; then
        echo "✅ Fixed! Verifying..."
        otool -D /Library/Frameworks/Syphon.framework/Syphon
    else
        echo "❌ Failed to fix. You may need to run this script with sudo:"
        echo "   sudo ./fix_syphon.sh"
        exit 1
    fi
else
    echo "✅ Syphon.framework install name is correct!"
fi
