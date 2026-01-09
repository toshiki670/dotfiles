<a href="LICENSE" alt="MIT License"><img alt="GitHub" src="https://img.shields.io/github/license/toshiki670/dotfiles?style=flat-square"></a>
<a href="https://github.com/toshiki670/dotfiles/releases"><img alt="GitHub Release date" src="https://img.shields.io/github/release-date/toshiki670/dotfiles?style=flat-square"></a>
<a href="https://github.com/toshiki670/dotfiles/releases"><img alt="GitHub Releases" src="https://img.shields.io/github/v/tag/toshiki670/dotfiles?label=release&style=flat-square"></a>

# Overview

- Simplification of environment construction
- Unification of environment across multiple platforms

# Prerequisites

## Required Tools

```bash
$ brew install git gh zsh nvim mise sheldon eza
```

### Tool Descriptions

- `git` - Version control system
- `gh` - GitHub CLI
- `zsh` - Z shell (recommended shell)
- `nvim` - Neovim text editor
- `mise` - Runtime version manager
- `sheldon` - Zsh plugin manager
- `eza` - Modern replacement for ls

## Optional Tools

```bash
$ brew install ffmpeg marp-cli
```

### Optional Tool Descriptions

- `ffmpeg` - Multimedia framework (required for video/audio processing)
- `marp-cli` - Markdown to PDF/PowerPoint converter

# Installation

## 1. Clone Repository

```bash
$ cd ~
$ git clone https://github.com/toshiki670/dotfiles.git
```

## 2. Run Install Script

```bash
$ cd ~/dotfiles
$ ./install
```

## 3. Restart Shell

```bash
$ exec $SHELL -l
```

# Configuration

## Environment Variables (Using Mise)

Create global configuration file: `~/.config/mise/config.toml`

```toml
[env]
# yt-dlp browser selection
# Options: "chrome:Default", "chrome:Profile 1", "firefox", "safari", "edge"
YT_BROWSER = "chrome:Default"
```

**Note:** To check your Chrome profile name, visit `chrome://version/` and look for the "Profile Path".

Apply changes:

```bash
$ exec $SHELL -l
```

## Platform-Specific Notes

### macOS

- Homebrew configurations will be applied automatically
- Custom binaries in `bin/` will be added to PATH

### Linux (Arch)

- GNOME autostart configurations available in `linux/gnome/autostart/`
- systemd service files available in `linux/systemd/`

### Windows

- Run `install_win.ps1` for PowerShell configuration
- Windows Terminal settings available in `win/windows_terminal/`
