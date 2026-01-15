<a href="LICENSE" alt="MIT License"><img alt="GitHub" src="https://img.shields.io/github/license/toshiki670/dotfiles?style=flat-square"></a>
<a href="https://github.com/toshiki670/dotfiles/releases"><img alt="GitHub Release date" src="https://img.shields.io/github/release-date/toshiki670/dotfiles?style=flat-square"></a>
<a href="https://github.com/toshiki670/dotfiles/releases"><img alt="GitHub Releases" src="https://img.shields.io/github/v/tag/toshiki670/dotfiles?label=release&style=flat-square"></a>

# Overview

- Simplification of environment construction
- Unification of environment across multiple platforms

# Release

To create a new release, see [How to Execute a Release](CONTRIBUTING.md#リリースの実行方法) in the Contributing guide.

**Quick Access:**

- 📖 [Release Guide](CONTRIBUTING.md#リリースの実行方法) - Detailed instructions on how to create a release
- 🚀 [Run Release Workflow](https://github.com/toshiki670/dotfiles/actions/workflows/release.yml) - Direct link to GitHub Actions release workflow

# Prerequisites

## Required Tools

```bash
$ brew install git gh zsh nvim mise sheldon eza bat fd ripgrep zoxide fzf git-delta
```

### Tool Descriptions

- `git` - Version control system
- `gh` - GitHub CLI
- `zsh` - Z shell (recommended shell)
- `nvim` - Neovim text editor
- `mise` - Runtime version manager
- `sheldon` - Zsh plugin manager
- `eza` - Modern replacement for ls
- `bat` - Modern replacement for cat with syntax highlighting
- `fd` - Fast and user-friendly alternative to find
- `ripgrep` - Fast search tool (rg command)
- `zoxide` - Smarter cd command that learns your habits
- `fzf` - Command-line fuzzy finder (required for zoxide's zi command)
- `git-delta` - Syntax-highlighting pager for git, diff, and grep output

## Optional Tools

```bash
$ brew install ffmpeg marp-cli gitui
```

### Optional Tool Descriptions

- `ffmpeg` - Multimedia framework (required for video/audio processing)
- `marp-cli` - Markdown to PDF/PowerPoint converter
- `gitui` - Terminal UI for git commands

## Rust Tools

### Necessary

```bash
$ cargo install cargo-audit cargo-cache cargo-edit cargo-llvm-cov cargo-make cargo-modules cargo-outdated cargo-tree cargo-update cargo-watch
```

#### Tool Descriptions

- `cargo-audit` - Scan dependencies for known security vulnerabilities
- `cargo-cache` - Manage Cargo cache directory, display size, and clean unnecessary files
- `cargo-edit` - Add, remove, and update dependencies in Cargo.toml from command line
- `cargo-llvm-cov` - Measure code coverage and generate reports
- `cargo-make` - Task runner/build tool for automating complex build flows and tasks
- `cargo-modules` - Visualize project module structure
- `cargo-outdated` - List available dependency updates
- `cargo-tree` - Display dependency tree structure
- `cargo-update` - Batch update installed Cargo binary crates
- `cargo-watch` - Monitor source code changes and automatically run commands (build, test, etc.) on change

### Optional

```bash
$ cargo install cargo-release tauri-cli create-tauri-app
```

#### Optional Tool Descriptions

- `cargo-release` - Automate the release process for new versions
- `tauri-cli` - Tauri application development CLI
- `create-tauri-app` - Scaffolding tool for Tauri applications

### Additional Tools

The following tools may already be installed in your environment:

- `sccache` (v0.12.0) - Cache compilation results to reduce build times
- `sea-orm-cli` (v1.1.19) - SeaORM CLI for migrations and entity generation
- `taplo-cli` (v0.10.0) - TOML formatter and linter
- `trunk` (v0.21.14) - Build tool for Rust + WebAssembly applications
- `wasm-bindgen-cli` - Generate bindings between Rust and JavaScript for WebAssembly

# Installation

## Using chezmoi (Recommended)

### 1. Install chezmoi

```bash
$ brew install chezmoi
```

### 2. Initialize with this repository

```bash
$ chezmoi init --ssh toshiki670
```

### 3. Preview changes (optional)

```bash
$ chezmoi diff
```

### 4. Apply the dotfiles

```bash
$ chezmoi apply
```

### 5. Restart Shell

```bash
$ exec $SHELL -l
```

## Old Installation

### 1. Clone Repository

```bash
$ cd ~
$ git clone https://github.com/toshiki670/dotfiles.git
```

### 2. Run Install Script

```bash
$ cd ~/dotfiles
$ ./install
```

### 3. Restart Shell

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
