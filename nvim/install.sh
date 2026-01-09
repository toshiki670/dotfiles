#!/usr/bin/env bash

# Modern Neovim Configuration Installer
# Installs and sets up the Lua + lazy.nvim configuration

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
NVIM_CONFIG_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/nvim"

echo "================================"
echo "Modern Neovim Configuration Installer"
echo "================================"
echo ""

# Check if Neovim is installed
if ! command -v nvim &> /dev/null; then
    echo "❌ Neovim is not installed."
    echo "Please install Neovim 0.8+ first:"
    echo ""
    echo "  macOS:    brew install neovim"
    echo "  Arch:     sudo pacman -S neovim"
    echo "  Ubuntu:   sudo apt install neovim"
    echo ""
    exit 1
fi

# Check Neovim version
NVIM_VERSION=$(nvim --version | head -n1 | cut -d' ' -f2 | cut -d'v' -f2)
REQUIRED_VERSION="0.8.0"

if [ "$(printf '%s\n' "$REQUIRED_VERSION" "$NVIM_VERSION" | sort -V | head -n1)" != "$REQUIRED_VERSION" ]; then
    echo "⚠️  Warning: Neovim version $NVIM_VERSION is older than $REQUIRED_VERSION"
    echo "Some features may not work correctly."
    echo ""
fi

echo "✓ Neovim version: $NVIM_VERSION"
echo ""

# Check dependencies
echo "Checking dependencies..."
MISSING_DEPS=()

if ! command -v rg &> /dev/null; then
    MISSING_DEPS+=("ripgrep")
fi

if ! command -v fd &> /dev/null; then
    MISSING_DEPS+=("fd")
fi

if ! command -v node &> /dev/null; then
    MISSING_DEPS+=("node")
fi

if [ ${#MISSING_DEPS[@]} -gt 0 ]; then
    echo "⚠️  Missing optional dependencies: ${MISSING_DEPS[*]}"
    echo "Install them for better experience:"
    echo ""
    echo "  macOS:    brew install ${MISSING_DEPS[*]}"
    echo "  Arch:     sudo pacman -S ${MISSING_DEPS[*]}"
    echo "  Ubuntu:   sudo apt install ${MISSING_DEPS[*]}"
    echo ""
    read -p "Continue anyway? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
else
    echo "✓ All dependencies are installed"
fi

echo ""

# Backup existing configuration
if [ -d "$NVIM_CONFIG_DIR" ] || [ -L "$NVIM_CONFIG_DIR" ]; then
    BACKUP_DIR="${NVIM_CONFIG_DIR}.backup.$(date +%Y%m%d_%H%M%S)"
    echo "📦 Backing up existing Neovim config to: $BACKUP_DIR"
    mv "$NVIM_CONFIG_DIR" "$BACKUP_DIR"
    echo "✓ Backup created"
    echo ""
fi

# Create symlink
echo "🔗 Creating symlink: $SCRIPT_DIR -> $NVIM_CONFIG_DIR"
ln -sf "$SCRIPT_DIR" "$NVIM_CONFIG_DIR"
echo "✓ Symlink created"
echo ""

# Clean old data (optional)
read -p "Clean old Neovim data/cache? (y/N): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo "🧹 Cleaning old data..."
    rm -rf "${XDG_DATA_HOME:-$HOME/.local/share}/nvim"
    rm -rf "${XDG_STATE_HOME:-$HOME/.local/state}/nvim"
    rm -rf "${XDG_CACHE_HOME:-$HOME/.cache}/nvim"
    echo "✓ Data cleaned"
    echo ""
fi

echo "================================"
echo "✅ Installation complete!"
echo "================================"
echo ""
echo "Next steps:"
echo "  1. Start Neovim: nvim"
echo "  2. Lazy.nvim will auto-install plugins"
echo "  3. Run :checkhealth to verify setup"
echo "  4. Run :Mason to install LSP servers"
echo ""
echo "Key mappings:"
echo "  <Space>t  - Toggle file tree"
echo "  <Space>df - Find files"
echo "  <Space>dg - Live grep"
echo "  <Space>db - List buffers"
echo "  <Space>ch - Show cheatsheet"
echo ""
echo "For more info, see: $SCRIPT_DIR/README.md"
echo ""
