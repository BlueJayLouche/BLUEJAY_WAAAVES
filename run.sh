#!/bin/bash
# Development wrapper script for RustJay Waaaves
# This script provides helpful output and common options for development

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}╔══════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  RustJay Waaaves - Development Launcher                  ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════════════════════╝${NC}"
echo ""

# Function to check if local Syphon framework exists
check_syphon() {
    local local_framework="../crates/syphon/syphon-lib/Syphon.framework"
    
    if [ -d "$local_framework" ]; then
        echo -e "${GREEN}✅ Local Syphon.framework found${NC}"
        return 0
    else
        echo -e "${RED}❌ Local Syphon.framework not found at:${NC}"
        echo "   $local_framework"
        echo ""
        echo "   Please ensure the framework exists:"
        echo "   1. Download from: https://github.com/Syphon/Syphon-Framework/releases"
        echo "   2. Extract and copy to: crates/syphon/syphon-lib/Syphon.framework"
        echo ""
        return 1
    fi
}

# Check for cargo
check_cargo() {
    if ! command -v cargo &> /dev/null; then
        echo -e "${RED}❌ Rust/Cargo not found${NC}"
        echo "   Please install Rust: https://rustup.rs/"
        exit 1
    fi
}

# Parse arguments
BUILD_TYPE="debug"
FEATURES="ipc-syphon"
CHECK_SYPHON=1

while [[ $# -gt 0 ]]; do
    case $1 in
        --release)
            BUILD_TYPE="release"
            shift
            ;;
        --no-syphon)
            FEATURES=""
            shift
            ;;
        --no-check)
            CHECK_SYPHON=0
            shift
            ;;
        --help|-h)
            echo "Usage: ./run.sh [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --release       Build and run in release mode (optimized)"
            echo "  --no-syphon     Run without Syphon support"
            echo "  --no-check      Skip Syphon framework check"
            echo "  --help, -h      Show this help message"
            echo ""
            echo "Examples:"
            echo "  ./run.sh                    # Run in debug mode with Syphon"
            echo "  ./run.sh --release          # Run in release mode"
            echo "  ./run.sh --no-syphon        # Run without Syphon (for testing)"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# Check prerequisites
check_cargo

# Check Syphon if enabled
if [ $CHECK_SYPHON -eq 1 ] && [ -n "$FEATURES" ]; then
    check_syphon
    if [ $? -ne 0 ]; then
        echo -e "${YELLOW}⚠️  Continuing without Syphon checks (--no-check to skip)${NC}"
        echo ""
    fi
fi

# Build and run
echo -e "${BLUE}🚀 Building RustJay Waaaves...${NC}"
echo "   Mode: $BUILD_TYPE"
echo "   Features: ${FEATURES:-none}"
echo ""

if [ "$BUILD_TYPE" = "release" ]; then
    if [ -n "$FEATURES" ]; then
        cargo run --release --features "$FEATURES"
    else
        cargo run --release
    fi
else
    if [ -n "$FEATURES" ]; then
        cargo run --features "$FEATURES"
    else
        cargo run
    fi
fi

# Capture exit code
EXIT_CODE=$?

if [ $EXIT_CODE -ne 0 ]; then
    echo ""
    echo -e "${RED}❌ Application exited with error code $EXIT_CODE${NC}"
    echo ""
    echo "Common issues:"
    echo "  - Missing local Syphon.framework (see check above)"
    echo "  - Missing system frameworks (should be present on macOS)"
    echo "  - GPU compatibility issues (check logs above)"
fi

exit $EXIT_CODE
