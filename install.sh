#!/bin/bash
set -e

# Color codes
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}=== Arch TUI Installer ===${NC}"

# 1. Check dependencies
echo -e "${GREEN}[1/5] Checking dependencies...${NC}"

if ! command -v cargo &> /dev/null; then
    echo "Rust/Cargo is not installed. Installing via rustup..."
    sudo pacman -S --noconfirm rustup
    rustup default stable
fi

if ! command -v pacman &> /dev/null; then
    echo "Error: 'pacman' not found. Is this an Arch Linux system?"
    exit 1
fi

# Check for AUR helper
if command -v paru &> /dev/null; then
    echo "Found AUR helper: paru"
elif command -v yay &> /dev/null; then
    echo "Found AUR helper: yay"
else
    echo "Warning: No AUR helper (paru/yay) found. AUR functionality might be limited."
    read -p "Do you want to install 'paru-bin' (requires sudo)? [y/N] " install_paru
    if [[ "$install_paru" =~ ^[Yy]$ ]]; then
        echo "Installing paru-bin..."
        # Basic manual build fallback if no helper
        git clone https://aur.archlinux.org/paru-bin.git /tmp/paru-bin
        cd /tmp/paru-bin
        makepkg -si --noconfirm
        cd -
    fi
fi

# 2. Build Release
echo -e "${GREEN}[2/5] Building release binary...${NC}"
cargo build --release

# 3. Install Binary
echo -e "${GREEN}[3/5] Installing binary to ~/.local/bin/...${NC}"
mkdir -p ~/.local/bin
cp target/release/arch-tui ~/.local/bin/
chmod +x ~/.local/bin/arch-tui

# 4. Install Desktop Entry
echo -e "${GREEN}[4/5] Installing desktop entry...${NC}"
mkdir -p ~/.local/share/applications
cp arch-tui.desktop ~/.local/share/applications/

# 5. Finalize
echo -e "${GREEN}[5/5] Installation Complete!${NC}"
echo ""
echo "Please ensure ~/.local/bin is in your PATH."
echo "You can launch the app by typing 'arch-tui' in terminal"
echo "or by searching 'Arch TUI' in your application menu."
