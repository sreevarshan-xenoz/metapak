#!/bin/bash
set -e

# Color codes
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

VERSION="0.1.0"
INSTALL_DIR="${HOME}/.local"
BIN_DIR="${INSTALL_DIR}/bin"
SHARE_DIR="${INSTALL_DIR}/share"

usage() {
    echo "metapak Installer v${VERSION}"
    echo ""
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  -i, --install      Install metapak (default)"
    echo "  -u, --uninstall    Uninstall metapak"
    echo "  -U, --update       Update and rebuild"
    echo "  -h, --help         Show this help message"
    echo ""
}

install() {
    echo -e "${BLUE}=== metapak Installer ===${NC}"

    # 1. Check dependencies
    echo -e "${GREEN}[1/6] Checking dependencies...${NC}"

    if ! command -v cargo &> /dev/null; then
        echo "Rust/Cargo is not installed. Installing via rustup..."
        if command -v pacman &> /dev/null; then
            sudo pacman -S --noconfirm rustup
        else
            echo "Error: Cannot install Rust - pacman not found"
            exit 1
        fi
        rustup default stable
    fi

    if ! command -v pacman &> /dev/null; then
        echo -e "${RED}Error: 'pacman' not found. Is this an Arch Linux system?${NC}"
        exit 1
    fi

    # Check for AUR helper
    if command -v paru &> /dev/null; then
        echo "Found AUR helper: paru"
    elif command -v yay &> /dev/null; then
        echo "Found AUR helper: yay"
    else
        echo -e "${YELLOW}Warning: No AUR helper (paru/yay) found. AUR functionality limited.${NC}"
    fi

    # 2. Build Release
    echo -e "${GREEN}[2/6] Building release binary...${NC}"
    cargo build --release

    # 3. Install Binary
    echo -e "${GREEN}[3/6] Installing binary to ${BIN_DIR}...${NC}"
    mkdir -p "${BIN_DIR}"
    cp target/release/metapak "${BIN_DIR}/"
    chmod +x "${BIN_DIR}/metapak"

    # 4. Install Config
    echo -e "${GREEN}[4/6] Installing default config...${NC}"
    mkdir -p "${INSTALL_DIR}/config/metapak"
    if [ ! -f "${INSTALL_DIR}/config/metapak/config.toml" ]; then
        cp config.example.toml "${INSTALL_DIR}/config/metapak/config.toml"
    fi

    # 5. Install Desktop Entry
    echo -e "${GREEN}[5/6] Installing desktop entry...${NC}"
    mkdir -p "${SHARE_DIR}/applications"
    cp metapak.desktop "${SHARE_DIR}/applications/"
    update-desktop-database "${SHARE_DIR}/applications/" 2>/dev/null || true

    # 6. Finalize
    echo -e "${GREEN}[6/6] Installation Complete!${NC}"
    echo ""
    echo -e "${GREEN}To run metapak:${NC}"
    echo "  ${BIN_DIR}/metapak"
    echo ""
    echo "Or add to PATH by adding to ~/.bashrc or ~/.zshrc:"
    echo "  export PATH=\"\${HOME}/.local/bin:\$PATH\""
    echo ""
    echo -e "Press ${GREEN}?${NC} in the app for keyboard shortcuts"
}

uninstall() {
    echo -e "${BLUE}=== Uninstalling metapak ===${NC}"
    
    if [ -f "${BIN_DIR}/metapak" ]; then
        rm -f "${BIN_DIR}/metapak"
        echo "Removed binary"
    fi
    
    if [ -d "${INSTALL_DIR}/config/metapak" ]; then
        rm -rf "${INSTALL_DIR}/config/metapak"
        echo "Removed config"
    fi
    
    if [ -f "${SHARE_DIR}/applications/metapak.desktop" ]; then
        rm -f "${SHARE_DIR}/applications/metapak.desktop"
        echo "Removed desktop entry"
    fi
    
    echo -e "${GREEN}Uninstallation complete!${NC}"
}

update() {
    echo -e "${BLUE}=== Updating metapak ===${NC}"
    cargo pull 2>/dev/null || true
    cargo build --release
    mkdir -p "${BIN_DIR}"
    cp target/release/metapak "${BIN_DIR}/"
    chmod +x "${BIN_DIR}/metapak"
    echo -e "${GREEN}Update complete!${NC}"
}

# Parse arguments
ACTION="install"

while [[ $# -gt 0 ]]; do
    case $1 in
        -i|--install)
            ACTION="install"
            shift
            ;;
        -u|--uninstall)
            ACTION="uninstall"
            shift
            ;;
        -U|--update)
            ACTION="update"
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            usage
            exit 1
            ;;
    esac
done

case $ACTION in
    install)
        install
        ;;
    uninstall)
        uninstall
        ;;
    update)
        update
        ;;
esac
