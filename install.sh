#!/usr/bin/env bash

# metapak robust universal installer
# Supports: Linux, macOS

set -euo pipefail

# Color codes
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

REPO_URL="https://github.com/sreevarshan-xenoz/metapak.git"
INSTALL_DIR="${HOME}/.local/bin"
SHARE_DIR="${HOME}/.local/share"
CONFIG_DIR="${HOME}/.config/metapak"
TMP_DIR=$(mktemp -d -t metapak-install-XXXXXX)

# Ensure cleanup runs on exit or interrupt
cleanup() {
    local exit_code=$?
    if [ -d "$TMP_DIR" ]; then
        rm -rf "$TMP_DIR"
    fi
    if [ $exit_code -ne 0 ]; then
        echo -e "${RED}Installation failed or was interrupted.${NC}"
    fi
    exit $exit_code
}
trap cleanup EXIT INT TERM

echo -e "${BLUE}=== metapak Installer ===${NC}"

# 1. Dependency Validation
echo -e "${GREEN}[1/6] Checking dependencies...${NC}"

if ! command -v curl >/dev/null 2>&1; then
    echo -e "${RED}Error: curl is required to download dependencies. Please install it and try again.${NC}"
    exit 1
fi

if ! command -v git >/dev/null 2>&1; then
    echo -e "${RED}Error: git is required to clone the repository. Please install it and try again.${NC}"
    exit 1
fi

# Rust installation / detection
if ! command -v cargo >/dev/null 2>&1; then
    echo -e "${YELLOW}Rust/Cargo not found. Installing via rustup...${NC}"
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    
    # Safely source the environment for the current script
    if [ -f "$HOME/.cargo/env" ]; then
        source "$HOME/.cargo/env"
    else
        echo -e "${RED}Failed to source Cargo environment. You may need to restart your terminal and try again.${NC}"
        exit 1
    fi
else
    echo "Found Rust: $(cargo --version)"
fi

# 2. Clone Repository
echo -e "${GREEN}[2/6] Downloading source code...${NC}"
git clone --depth 1 "$REPO_URL" "$TMP_DIR"
cd "$TMP_DIR"

# 3. Build Release
echo -e "${GREEN}[3/6] Building release binary (this may take a few minutes)...${NC}"
cargo build --release

# 4. Install Binary
echo -e "${GREEN}[4/6] Installing binary to ${INSTALL_DIR}...${NC}"
mkdir -p "$INSTALL_DIR"

# Idempotence check
if [ -f "${INSTALL_DIR}/metapak" ]; then
    echo -e "${YELLOW}metapak is already installed. Updating binary...${NC}"
    rm -f "${INSTALL_DIR}/metapak"
fi

cp target/release/metapak "${INSTALL_DIR}/"
chmod +x "${INSTALL_DIR}/metapak"

# 5. Install Config & Desktop Entry
echo -e "${GREEN}[5/6] Setting up configuration...${NC}"
mkdir -p "${CONFIG_DIR}"
if [ ! -f "${CONFIG_DIR}/config.toml" ]; then
    cp config.example.toml "${CONFIG_DIR}/config.toml"
    echo "Created default config at ${CONFIG_DIR}/config.toml"
else
    echo "Config already exists. Skipping..."
fi

if [ "$(uname)" = "Linux" ]; then
    echo "Setting up desktop entry..."
    mkdir -p "${SHARE_DIR}/applications"
    cp metapak.desktop "${SHARE_DIR}/applications/"
    
    if command -v update-desktop-database >/dev/null 2>&1; then
        update-desktop-database "${SHARE_DIR}/applications/" 2>/dev/null || true
    fi
fi

# 6. PATH Injection & Finalize
echo -e "${GREEN}[6/6] Finalizing installation...${NC}"

# Detect shell and check PATH
SHELL_RC=""
if [[ "$SHELL" == *"zsh"* ]]; then
    SHELL_RC="$HOME/.zshrc"
elif [[ "$SHELL" == *"bash"* ]]; then
    SHELL_RC="$HOME/.bashrc"
    # Some bash environments use .bash_profile
    if [ ! -f "$SHELL_RC" ] && [ -f "$HOME/.bash_profile" ]; then
        SHELL_RC="$HOME/.bash_profile"
    fi
elif [[ "$SHELL" == *"fish"* ]]; then
    SHELL_RC="$HOME/.config/fish/config.fish"
fi

if [ -n "$SHELL_RC" ] && [ -f "$SHELL_RC" ]; then
    if ! grep -q "$INSTALL_DIR" "$SHELL_RC"; then
        echo -e "${YELLOW}Adding ${INSTALL_DIR} to PATH in ${SHELL_RC}${NC}"
        echo "" >> "$SHELL_RC"
        echo "# Added by metapak installer" >> "$SHELL_RC"
        if [[ "$SHELL" == *"fish"* ]]; then
            echo "set -gx PATH \"$INSTALL_DIR\" \$PATH" >> "$SHELL_RC"
        else
            echo "export PATH=\"$INSTALL_DIR:\$PATH\"" >> "$SHELL_RC"
        fi
        echo -e "${YELLOW}Note: Please restart your terminal or run 'source ${SHELL_RC}' to update your PATH.${NC}"
    fi
else
    echo -e "${YELLOW}Could not automatically detect your shell configuration.${NC}"
    echo -e "Please ensure ${INSTALL_DIR} is in your PATH."
fi

echo ""
echo -e "${GREEN}✓ metapak Installation Complete!${NC}"
echo -e "Run ${BLUE}metapak${NC} to get started."
