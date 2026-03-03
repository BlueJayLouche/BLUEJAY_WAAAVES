#!/bin/bash
#
# Install dependencies for RustJay Waaaves on various Linux distributions
# This script detects the package manager and installs required dependencies
#

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}RustJay Waaaves - Dependency Installer${NC}"
echo "========================================"
echo ""

# Detect package manager
detect_package_manager() {
    if command -v pacman &> /dev/null; then
        echo "pacman"
    elif command -v apt-get &> /dev/null; then
        echo "apt"
    elif command -v dnf &> /dev/null; then
        echo "dnf"
    elif command -v yum &> /dev/null; then
        echo "yum"
    elif command -v zypper &> /dev/null; then
        echo "zypper"
    elif command -v apk &> /dev/null; then
        echo "apk"
    elif command -v emerge &> /dev/null; then
        echo "portage"
    elif command -v nix-env &> /dev/null; then
        echo "nix"
    else
        echo "unknown"
    fi
}

# Get distro name for better messaging
get_distro_name() {
    if [ -f /etc/os-release ]; then
        . /etc/os-release
        echo "$NAME"
    else
        echo "Unknown Linux distribution"
    fi
}

PKG_MANAGER=$(detect_package_manager)
DISTRO=$(get_distro_name)

echo -e "Detected distribution: ${GREEN}$DISTRO${NC}"
echo -e "Detected package manager: ${GREEN}$PKG_MANAGER${NC}"
echo ""

# Dependencies needed:
# - libclang: Required for bindgen (v4l2-sys-mit crate)
# - base-devel/build-essential: Basic build tools
# - v4l-utils: Video4Linux2 libraries (optional, for webcam support)

install_pacman() {
    echo -e "${YELLOW}Installing dependencies with pacman...${NC}"
    echo ""
    
    # Check if running as root or with sudo
    if [ "$EUID" -eq 0 ]; then
        pacman -S --needed --noconfirm \
            base-devel \
            clang \
            v4l-utils \
            pkg-config
    elif command -v sudo &> /dev/null; then
        sudo pacman -S --needed --noconfirm \
            base-devel \
            clang \
            v4l-utils \
            pkg-config
    else
        echo -e "${RED}Error: This script needs to run with root privileges or sudo.${NC}"
        echo "Please run: sudo $0"
        exit 1
    fi
}

install_apt() {
    echo -e "${YELLOW}Installing dependencies with apt...${NC}"
    echo ""
    
    # Update package list
    if [ "$EUID" -eq 0 ]; then
        apt-get update
        apt-get install -y \
            build-essential \
            libclang-dev \
            libv4l-dev \
            pkg-config
    elif command -v sudo &> /dev/null; then
        sudo apt-get update
        sudo apt-get install -y \
            build-essential \
            libclang-dev \
            libv4l-dev \
            pkg-config
    else
        echo -e "${RED}Error: This script needs to run with root privileges or sudo.${NC}"
        echo "Please run: sudo $0"
        exit 1
    fi
}

install_dnf() {
    echo -e "${YELLOW}Installing dependencies with dnf...${NC}"
    echo ""
    
    if [ "$EUID" -eq 0 ]; then
        dnf install -y \
            gcc \
            gcc-c++ \
            make \
            clang \
            clang-devel \
            libv4l-devel \
            pkgconfig
    elif command -v sudo &> /dev/null; then
        sudo dnf install -y \
            gcc \
            gcc-c++ \
            make \
            clang \
            clang-devel \
            libv4l-devel \
            pkgconfig
    else
        echo -e "${RED}Error: This script needs to run with root privileges or sudo.${NC}"
        echo "Please run: sudo $0"
        exit 1
    fi
}

install_yum() {
    echo -e "${YELLOW}Installing dependencies with yum...${NC}"
    echo ""
    
    if [ "$EUID" -eq 0 ]; then
        yum install -y \
            gcc \
            gcc-c++ \
            make \
            clang \
            clang-devel \
            libv4l-devel \
            pkgconfig
    elif command -v sudo &> /dev/null; then
        sudo yum install -y \
            gcc \
            gcc-c++ \
            make \
            clang \
            clang-devel \
            libv4l-devel \
            pkgconfig
    else
        echo -e "${RED}Error: This script needs to run with root privileges or sudo.${NC}"
        echo "Please run: sudo $0"
        exit 1
    fi
}

install_zypper() {
    echo -e "${YELLOW}Installing dependencies with zypper...${NC}"
    echo ""
    
    if [ "$EUID" -eq 0 ]; then
        zypper install -y \
            gcc \
            gcc-c++ \
            make \
            clang \
            libv4l-devel \
            pkg-config
    elif command -v sudo &> /dev/null; then
        sudo zypper install -y \
            gcc \
            gcc-c++ \
            make \
            clang \
            libv4l-devel \
            pkg-config
    else
        echo -e "${RED}Error: This script needs to run with root privileges or sudo.${NC}"
        echo "Please run: sudo $0"
        exit 1
    fi
}

install_apk() {
    echo -e "${YELLOW}Installing dependencies with apk...${NC}"
    echo ""
    
    if [ "$EUID" -eq 0 ]; then
        apk add \
            build-base \
            clang-dev \
            libv4l-dev \
            pkgconfig
    elif command -v sudo &> /dev/null; then
        sudo apk add \
            build-base \
            clang-dev \
            libv4l-dev \
            pkgconfig
    else
        echo -e "${RED}Error: This script needs to run with root privileges or sudo.${NC}"
        echo "Please run: sudo $0"
        exit 1
    fi
}

install_portage() {
    echo -e "${YELLOW}Installing dependencies with emerge...${NC}"
    echo ""
    echo -e "${YELLOW}Note: Gentoo users may need to ensure the following are in their USE flags:${NC}"
    echo "  - clang"
    echo "  - v4l"
    echo ""
    
    if [ "$EUID" -eq 0 ]; then
        emerge -av \
            sys-devel/clang \
            media-libs/libv4l \
            virtual/pkgconfig
    elif command -v sudo &> /dev/null; then
        sudo emerge -av \
            sys-devel/clang \
            media-libs/libv4l \
            virtual/pkgconfig
    else
        echo -e "${RED}Error: This script needs to run with root privileges or sudo.${NC}"
        echo "Please run: sudo $0"
        exit 1
    fi
}

install_nix() {
    echo -e "${YELLOW}Installing dependencies with nix...${NC}"
    echo ""
    echo -e "${YELLOW}Note: For NixOS, add the following to your configuration.nix or use nix-shell:${NC}"
    echo ""
    cat << 'EOF'
# shell.nix example:
{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = with pkgs; [
    clang
    libv4l
    pkg-config
    rustup
  ];
  
  LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
}
EOF
    echo ""
    echo "Then run: nix-shell"
}

# Main installation logic
case "$PKG_MANAGER" in
    pacman)
        install_pacman
        ;;
    apt)
        install_apt
        ;;
    dnf)
        install_dnf
        ;;
    yum)
        install_yum
        ;;
    zypper)
        install_zypper
        ;;
    apk)
        install_apk
        ;;
    portage)
        install_portage
        ;;
    nix)
        install_nix
        # Don't continue to success message for nix as we can't auto-install
        exit 0
        ;;
    unknown)
        echo -e "${RED}Error: Could not detect a supported package manager.${NC}"
        echo ""
        echo "Supported package managers:"
        echo "  - pacman (Arch, Manjaro, EndeavourOS)"
        echo "  - apt (Debian, Ubuntu, Linux Mint, Pop!_OS)"
        echo "  - dnf/yum (Fedora, RHEL, CentOS, AlmaLinux, Rocky Linux)"
        echo "  - zypper (openSUSE)"
        echo "  - apk (Alpine Linux)"
        echo "  - portage (Gentoo)"
        echo "  - nix (NixOS)"
        echo ""
        echo "Please install the following dependencies manually:"
        echo "  - clang / libclang-dev"
        echo "  - build-essential / base-devel / gcc"
        echo "  - v4l-utils / libv4l-dev (optional, for webcam support)"
        echo "  - pkg-config"
        exit 1
        ;;
esac

echo ""
echo -e "${GREEN}✓ Dependencies installed successfully!${NC}"
echo ""
echo "You can now build the project with:"
echo "  cargo build --release"
echo ""
echo "Or run in development mode with:"
echo "  cargo run"
