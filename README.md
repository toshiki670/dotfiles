<a href="LICENSE" alt="MIT License"><img alt="GitHub" src="https://img.shields.io/github/license/toshiki670/dotfiles?style=flat-square"></a>
<a href="https://github.com/toshiki670/dotfiles/releases"><img alt="GitHub Release date" src="https://img.shields.io/github/release-date/toshiki670/dotfiles?style=flat-square"></a>
<a href="https://github.com/toshiki670/dotfiles/releases"><img alt="GitHub Releases" src="https://img.shields.io/github/v/tag/toshiki670/dotfiles?label=release&style=flat-square"></a>

# Overview

- Simplification of environment construction
- Unification of environment across multiple platforms

This repository is managed with [chezmoi](https://www.chezmoi.io/). **Fish** is the primary shell going forward (`~/.config/fish/conf.d/`), with **[Starship](https://starship.rs/)** as the interactive prompt. **Zsh** remains fully maintained (Sheldon, zeno, fzf, etc.). Also included: **Neovim**, **Git** (split config + delta), **mise**, optional **Ghostty** / **Zellij** configs, and scripts under `bin/`.

# Prerequisites

## Required Tools

These cover the Fish-first workflow and shared tooling (Git, editor, mise, CLI utilities).

```bash
brew install git gh fish nvim mise eza bat fd ripgrep starship zoxide fzf git-delta
```

### Required Homebrew tool descriptions

- `git` - Version control system
- `gh` - GitHub CLI (used by shell prompts and aliases)
- `fish` - Fish shell (**primary shell** for this dotfiles set)
- `nvim` - Neovim text editor
- `mise` - Runtime version manager
- `eza` - Modern replacement for ls
- `bat` - Modern replacement for cat with syntax highlighting
- `fd` - Fast and user-friendly alternative to find
- `ripgrep` - Fast search tool (rg command)
- `starship` - Minimal, fast prompt ([starship.rs](https://starship.rs/)); Fish loads it from `config.fish`, config at `~/.config/starship.toml`
- `zoxide` - Smarter cd command that learns your habits
- `fzf` - Command-line fuzzy finder (used with zoxide and Fish/Zsh key bindings)
- `git-delta` - Syntax-highlighting pager for git, diff, and grep output

## Zsh-only dependencies

Install these if you use the **Zsh** configuration (Sheldon, zeno snippet expansion, etc.).

```bash
brew install zsh sheldon deno
```

- `zsh` - Z shell
- `sheldon` - Zsh plugin manager
- `deno` - JavaScript/TypeScript runtime (required for zeno.zsh)

## Optional Tools

```bash
brew install ffmpeg marp-cli gitui ghostty zellij smartmontools
```

### Optional Homebrew tool descriptions

- `ffmpeg` - Multimedia framework (required for video/audio processing)
- `marp-cli` - Markdown to PDF/PowerPoint converter
- `gitui` - Terminal UI for git commands
- `ghostty` - Terminal emulator; config lives under `~/.config/ghostty/` (see [Configuration](#configuration))
- `zellij` - Terminal multiplexer; config under `~/.config/zellij/`
- `smartmontools` - S.M.A.R.T. disk health monitoring (`smartctl`)

# Installation

## Using chezmoi (Recommended)

### 1. Install chezmoi

```bash
brew install chezmoi
```

### 2. Initialize with this repository

```bash
chezmoi init --ssh toshiki670
```

### 3. Preview changes (optional)

```bash
chezmoi diff
```

### 4. Apply the dotfiles

```bash
chezmoi apply
```

### 5. Restart Shell

```bash
exec fish -l
```

If you use Zsh instead: `exec zsh -l` (or `exec $SHELL -l` after `chsh`).

### 6. Set login shell (recommended for Fish)

```bash
chsh -s "$(which fish)"
```

# Configuration

## Shells (Fish and Zsh)

- **Fish (preferred)** — Modular config under `~/.config/fish/conf.d/`. Interactive sessions run `starship init fish` from `config.fish`; prompt styling lives in `~/.config/starship.toml`. New work and day-to-day usage should favor Fish.- **Zsh** — Entry point is `~/.config/zsh/` (via `dot_zshrc.tmpl`), with Sheldon and modular includes under `configs/`. Install [Zsh-only dependencies](#zsh-only-dependencies) if you use this stack.

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
exec $SHELL -l
```

## Ghostty (macOS)

On macOS, `chezmoi apply` runs a hook that symlinks Ghostty’s expected config path to `~/.config/ghostty/config`. If you use Ghostty, install it separately (see [Optional Tools](#optional-tools)). Ghostty works well as the terminal for a Fish-centric setup.

## Platform-Specific Notes

### macOS

- Homebrew configurations will be applied automatically
- Custom binaries in `bin/` will be added to PATH
- Ghostty config symlink is set up as described above

# Development

See [CONTRIBUTING.md](CONTRIBUTING.md) for lint, test, and release instructions. To trigger a release directly: [Run Release Workflow](https://github.com/toshiki670/dotfiles/actions/workflows/release.yml).
