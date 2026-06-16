<a href="LICENSE" alt="MIT License"><img alt="GitHub" src="https://img.shields.io/github/license/toshiki670/dotfiles?style=flat-square"></a>
<a href="https://github.com/toshiki670/dotfiles/releases"><img alt="GitHub Release date" src="https://img.shields.io/github/release-date/toshiki670/dotfiles?style=flat-square"></a>
<a href="https://github.com/toshiki670/dotfiles/releases"><img alt="GitHub Releases" src="https://img.shields.io/github/v/tag/toshiki670/dotfiles?label=release&style=flat-square"></a>

# Overview

- Simplification of environment construction
- Unification of environment across multiple platforms

This repository is managed with [chezmoi](https://www.chezmoi.io/). **Fish** is the shell (`~/.config/fish/conf.d/`), with **[Starship](https://starship.rs/)** as the interactive prompt. Also included: **Neovim**, **Git** (split config + delta), **mise**, optional **Ghostty** / **Zellij** configs, a few scripts under `bin/`, and small **Rust** CLI commands (`color`, `git-upstream`, `gcm`, `copy-obj`, `v-sync`, …) built from the repository-root crate and installed via `cargo install` into `~/.cargo/bin` (see [Rust commands](#rust-commands)).

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
- `fzf` - Command-line fuzzy finder (used with zoxide and Fish key bindings)
- `git-delta` - Syntax-highlighting pager for git, diff, and grep output

## Optional Tools

```bash
brew install ffmpeg marp-cli gitui ghostty zellij smartmontools rtk
```

### Optional Homebrew tool descriptions

- `ffmpeg` - Multimedia framework (required for video/audio processing)
- `marp-cli` - Markdown to PDF/PowerPoint converter
- `gitui` - Terminal UI for git commands
- `ghostty` - Terminal emulator; config lives under `~/.config/ghostty/` (see [Configuration](#configuration))
- `zellij` - Terminal multiplexer; config under `~/.config/zellij/`
- `smartmontools` - S.M.A.R.T. disk health monitoring (`smartctl`)
- `rtk` - CLI proxy that reduces LLM token usage by 60–90% ([rtk-ai/rtk](https://github.com/rtk-ai/rtk)); after install, run `rtk init -g` to configure Claude Code hooks

After installing `rtk`, initialize the Claude Code hook:

```bash
rtk init -g
```

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

### 6. Set login shell (recommended for Fish)

```bash
chsh -s "$(which fish)"
```

# Configuration

## Shell (Fish)

Modular config under `~/.config/fish/conf.d/`. Interactive sessions run `starship init fish` from `config.fish`; prompt styling lives in `~/.config/starship.toml`.

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
- Scripts under `bin/` (`$DOTFILES/bin`) and Rust commands (`~/.cargo/bin`) are added to PATH
- Ghostty config symlink is set up as described above

# Rust commands

The CLI commands form a **Cargo workspace** at the repository root. The root package is `dotfiles` itself (core), and each individual command is an independent crate under `crates/*`. On `chezmoi apply`, a hook (`run_onchange_after_cargo-install`) installs the *distributable* crates into `~/.cargo/bin` via `cargo install` (the support library and the lint tool are not installed). The Rust toolchain and the lint tools are supplied by **mise** (`mise.toml`), so a fresh machine bootstraps as: `mise install` (rust) → `chezmoi apply` (cargo install).

| Command | Crate | Description |
| --- | --- | --- |
| `dotfiles` | (root) | dotfiles core; currently a version / `--help` entry point (`dotfiles --version`) |
| `color` | `crates/color` | Print an ANSI color table (16 + 256 colors) |
| `git-upstream` | `crates/git-upstream` | Merge `upstream/master` / initialize the upstream remote |
| `gcm` | `crates/gcm` | AI-powered git commit with Conventional Commits (`claude -p`) |
| `copy-obj` | `crates/copy-obj` | Copy a file as a Finder-pasteable file object (macOS) |
| `v-sync` | `crates/v-sync` | Sync Neovim plugins and re-add `lazy-lock.json` into chezmoi |
| `gh-clone` | `crates/gh-clone` | `gh repo clone` + `ghq migrate`, printing the migrated path |
| `fzf-ghq-cd` | `crates/fzf-picker` | Pick a ghq repo / linked worktree with fzf, printing the selected path (Fish shim cds) |
| `fzf-worktree-remove` | `crates/fzf-picker` | Pick a linked git worktree with fzf and remove it (Fish shim cds out if needed) |
| `cdabbr` | `crates/fzf-picker` | Expand a prompt_pwd-style abbreviated path and pick a directory with fzf (Fish shim cds) |
| `cleanup-env` | `crates/env-tools` | Prompt-then-cleanup caches / unused versions for brew / mise / cargo (`-n/--dry-run`) |
| `upgrade-env` | `crates/env-tools` | Upgrade all installed package managers (brew / mise / cargo) |
| `daily-check-worker`, `git-background-fetch-worker` | `crates/dotfiles-workers` | Background workers started from Fish `conf.d` hooks |

Every command binary supports `--help` / `--version`, except the env-driven background workers. `gh-clone` and the `fzf-picker` binaries (e.g. `fzf-ghq-cd`) keep a thin Fish shim for the part that must change the parent shell (`cd`), with the logic in the Rust binary.

Not installed (development only):

- `tools/dotfiles-lint` — lint/format orchestrator, run via `mise run lint` / `mise run check`.

Each distributable crate is versioned independently via release-plz: per-package tags `<crate>-v<version>`, while the root `dotfiles` keeps `v<version>`. See [CONTRIBUTING.md](CONTRIBUTING.md) for the release process.

# Development

See [CONTRIBUTING.md](CONTRIBUTING.md) for lint, test, and release instructions. To trigger a release directly: [Run Release Prepare Workflow](https://github.com/toshiki670/dotfiles/actions/workflows/release-prepare.yml).
