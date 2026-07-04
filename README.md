<a href="LICENSE" alt="MIT License"><img alt="GitHub" src="https://img.shields.io/github/license/toshiki670/dotfiles?style=flat-square"></a>
<a href="https://github.com/toshiki670/dotfiles/releases"><img alt="GitHub Release date" src="https://img.shields.io/github/release-date/toshiki670/dotfiles?style=flat-square"></a>
<a href="https://github.com/toshiki670/dotfiles/releases"><img alt="GitHub Releases" src="https://img.shields.io/github/v/tag/toshiki670/dotfiles?label=release&style=flat-square"></a>

# Overview

Personal dotfiles for macOS, managed by a self-contained Rust **`dotfiles`** CLI: one `cargo install` plus one `dotfiles apply` sets up a machine. **Fish** is the shell (`~/.config/fish/conf.d/`), with **[Starship](https://starship.rs/)** as the interactive prompt. Also included: **Neovim**, **Git** tooling (split config + delta), **mise**, optional **Ghostty** / **Zellij** configs, a few scripts under `bin/`, and small Rust CLI commands (`git-upstream`, `gcm`, `clip`, …) built as bins of the same package (see [Rust commands](#rust-commands)).

# Prerequisites

## Required Tools

These cover the Fish-first workflow and shared tooling (Git, editor, mise, CLI utilities).

```bash
brew install git gh fish nvim mise eza bat fd ripgrep starship zoxide fzf git-delta gitleaks
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
- `gitleaks` - Scans staged changes for secrets; used by the global `pre-commit` hook (see [Configuration notes](#configuration-notes)). Skipped with a warning if not installed.

## Optional Tools

```bash
brew install ffmpeg marp-cli gitui ghostty zellij smartmontools rtk
```

### Optional Homebrew tool descriptions

- `ffmpeg` - Multimedia framework (required for video/audio processing)
- `marp-cli` - Markdown to PDF/PowerPoint converter
- `gitui` - Terminal UI for git commands
- `ghostty` - Terminal emulator; config lives under `~/.config/ghostty/` (see [Configuration notes](#configuration-notes))
- `zellij` - Terminal multiplexer; config under `~/.config/zellij/`
- `smartmontools` - S.M.A.R.T. disk health monitoring (`smartctl`)
- `rtk` - CLI proxy that reduces LLM token usage by 60-90% ([rtk-ai/rtk](https://github.com/rtk-ai/rtk)); after install, run `rtk init -g` to configure Claude Code hooks

After installing `rtk`, initialize the Claude Code hook:

```bash
rtk init -g
```

# Installation

## 1. Install the `dotfiles` CLI

```bash
cargo install --git https://github.com/toshiki670/dotfiles
```

The `configs/` tree is embedded in the binary, so this single command is enough (a local clone's working tree is used automatically when present). The Rust toolchain is supplied by mise (`mise install`); see [Rust commands](#rust-commands).

## 2. Review destinations (optional)

```bash
dotfiles list
```

Preview what would be placed and where, without writing anything.

## 3. Apply the dotfiles

```bash
dotfiles apply
```

Places everything under `configs/`. If a machine-local value (e.g. git email/name) isn't stored yet, apply prompts for it interactively (sensitive values are read without echo) and saves it for next time; see [Daily usage](#daily-usage).

## 4. Restart Shell

```bash
exec fish -l
```

## 5. Set login shell (recommended for Fish)

```bash
chsh -s "$(which fish)"
```

# Daily usage

## Applying changes

Edit files under `configs/`, then re-run:

```bash
dotfiles apply
```

From a local clone, the working tree is used automatically; otherwise the copy embedded in the installed binary is used.

## Machine-local values

Some configs need a value that differs per machine (e.g. git email/name). Store one explicitly:

```bash
dotfiles local set <name> <value>
```

Values are kept in `~/.config/dotfiles/local.toml` and injected during `apply`. Run `dotfiles doctor` to see which declared names are still unset.

## Machine profile

```bash
dotfiles profile private
```

Opts a machine into private-only configs. The default is **not-private**, so new or work machines never receive them unless explicitly opted in. Run `dotfiles profile` with no argument to show the current value.

## Colors & themes

```bash
dotfiles color sample
```

Prints an ANSI color check table (16 + 256 colors). Theme configuration across tools is indexed in [COLOR.md](COLOR.md).

## Health check

```bash
dotfiles doctor
```

Reports machine-local values that are declared by a config but not yet set.

# Configuration notes

## Shell (Fish)

Modular config under `~/.config/fish/conf.d/`. Interactive sessions run `starship init fish` from `config.fish`; prompt styling lives in `~/.config/starship.toml`.

## Ghostty (macOS)

On macOS, `dotfiles apply` runs a hook that symlinks Ghostty's expected config path to `~/.config/ghostty/config`. If you use Ghostty, install it separately (see [Optional Tools](#optional-tools)). Ghostty works well as the terminal for a Fish-centric setup.

## Git hooks (gitleaks)

Git is configured with a global `core.hooksPath = ~/.config/git/hooks`, so the managed hooks run for **every** repository on the host. A single dispatcher script (`dispatch`) is symlinked under every client-side hook name (`pre-commit`, `commit-msg`, `prepare-commit-msg`, `pre-push`, `post-checkout`, `post-merge`, `post-commit`, `pre-rebase`, `pre-merge-commit`, `post-rewrite`, `applypatch-msg`, `pre-applypatch`, `post-applypatch`) and dispatches on `basename "$0"` (`dispatch` itself is not a hook name, so Git never runs it directly). It does two things:

1. **Secret scan (pre-commit only)** — runs `gitleaks git --staged` on the staged diff. If a likely secret is found, the commit is **blocked**; secret values are redacted in the output. False positives can be silenced with a `.gitleaks.toml` allowlist or an inline `gitleaks:allow` comment. If `gitleaks` is not installed, the scan is **skipped with a warning** (the commit is not blocked).
2. **Chaining (all hook types)** — because a global `core.hooksPath` makes Git stop looking at each repo's `.git/hooks`, the dispatcher explicitly invokes the repository-local `.git/hooks/<hook>` afterwards (if present and executable), forwarding arguments and stdin, so per-project hooks keep working.

Bypass everything for a single commit with `git commit --no-verify`.

**Note:** Repositories managed by husky / lefthook set their own `core.hooksPath` locally, which overrides the global one — in those repos these hooks (and the gitleaks scan) do not run.

## Claude Code

`~/.claude/settings.json` is placed by `dotfiles apply` (base `settings.json` plus a conditional `rtk.json` overlay when `rtk` is on `PATH`). It merges into the live file so keys the app writes itself (`model`, `theme`, `effortLevel`, …) are preserved, while dotfiles-owned shared settings (`hooks`, `statusLine`, `language`, `voiceEnabled`) are always enforced.

`PreToolUse` / `Bash` hooks provide two safety rails:

- **`rm` guard** — denies commands invoking `rm` and tells Claude to use `trash` instead. This keeps file deletion recoverable by default.
- **force-push guard** — denies `git push` commands that include `--force`, `-f`, `--force-with-lease`, or `--no-verify` so history rewrites and hook bypasses are blocked before execution.

Requires the `trash` CLI (bundled with macOS 15+). Both guards are intentionally string-based and simple: they are meant as practical pre-execution safety rails, not full shell parsers.

## Platform-Specific Notes

### macOS

- Homebrew configurations will be applied automatically
- Scripts under `bin/` (`$DOTFILES/bin`) and Rust commands (`~/.cargo/bin`) are added to PATH
- Ghostty config symlink is set up as described above

# Rust commands

All distributable commands live in the single root `dotfiles` package as multiple bins (a **Cargo workspace** at the repository root, whose only other members are the dev/maintenance tools under `tools/`). Install them into `~/.cargo/bin` with one `cargo install --git https://github.com/toshiki670/dotfiles` (or `cargo install --path .` from a clone; the tools under `tools/` are not installed). The Rust toolchain and the lint tools are supplied by **mise** (`mise.toml`), so a fresh machine bootstraps as: `mise install` (rust) → `cargo install` (commands) → `dotfiles apply` (configs).

Design and internals (manifest schema, apply pipeline, `locals` resolution, `when` gates, …) are not documented here — they live in the **[Rustdoc](https://toshiki670.github.io/dotfiles/)** (module docs of `dotfiles::core`), rebuilt on every push to `main`. Treat it as the architecture reference.

| Command | Description |
| --- | --- |
| `dotfiles` | dotfiles core; version entry point (`dotfiles --version`) plus `apply` (place `configs/` via per-directory `manifest.toml`; resolves & injects machine-local `locals` values), `list` (overview of every config's destination), `local set <name> <value>` (store a machine-local value in `~/.config/dotfiles/local.toml`), `profile [<name>]` (set or show the machine profile, e.g. `private`), `color sample` (print an ANSI color table — 16 + 256 colors), and `doctor` (report unset `locals`) |
| `git-upstream` | Merge `upstream/master` / initialize the upstream remote |
| `gcm` | AI-powered git commit with Conventional Commits (`claude -p`) |
| `clip` | Copy a file to the clipboard — `obj` (Finder object) / `text` (contents) / `path` (absolute path); macOS |
| `gh-clone` | `gh repo clone` + `ghq migrate`, printing the migrated path |
| `fzf-gh` | Pick a GitHub Issue/PR with fzf and build a `gh` command (Fish shim inserts it into the command line) |
| `fzf-ghq-cd` | Pick a ghq repo / linked worktree with fzf, printing the selected path (Fish shim cds) |
| `fzf-worktree-remove` | Pick a linked git worktree with fzf and remove it (Fish shim cds out if needed) |
| `cdabbr` | Expand a prompt_pwd-style abbreviated path and pick a directory with fzf (Fish shim cds) |
| `cleanup-env` | Prompt-then-cleanup caches / unused versions for brew / mise / cargo (`-n/--dry-run`) |
| `upgrade-env` | Upgrade all installed package managers (brew / mise / cargo) |
| `daily-check-worker`, `git-background-fetch-worker` | Background workers started from Fish `conf.d` hooks |

Every command binary supports `--help` / `--version`, except the env-driven background workers. `gh-clone` and the fzf-picker binaries (e.g. `fzf-ghq-cd`) keep a thin Fish shim for the part that must change the parent shell (`cd`), with the logic in the Rust binary.

Not installed (development / maintenance only, under `tools/`):

- `dotfiles-lint` — lint/format orchestrator, run via `mise run lint` / `mise run check`.
- `v-sync` — sync Neovim plugins and write `lazy-lock.json` back into the dotfiles source, run via `mise run v-sync`.

The root `dotfiles` package is the single distributable, versioned via release-plz with tags `v<version>`. See [Development](#development) for the release process.

# Development

See [CONTRIBUTING.md](CONTRIBUTING.md) for the development setup, test policy, and release process. To trigger a release directly: [Run Release Prepare Workflow](https://github.com/toshiki670/dotfiles/actions/workflows/release-prepare.yml).
