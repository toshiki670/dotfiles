{
  description = "Nix-based lint/format environment for dotfiles";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
    let
      pkgs = import nixpkgs { inherit system; };
      starshipConfigSchema = pkgs.fetchurl {
        name = "starship-config-schema.json";
        url = "https://starship.rs/config-schema.json";
        hash = "sha256-pEHfJUFK0WZrTxGLiXSbvnT9Lp8Rqd3QAGhMJARa5kU=";
      };
      taploNixConfig = pkgs.writeText "taplo-nix.toml" ''
        [[rule]]
        include = ["home/dot_config/starship.toml"]

        [rule.schema]
        path = "${starshipConfigSchema}"
      '';
      pythonEnv = pkgs.python3.withPackages (ps: [
          ps.pathspec
          ps.pytest
          ps.mypy
        ]);
        toolchain = [
          pkgs.coreutils
          pkgs.diffutils
          pkgs.bash
          pythonEnv
          pkgs.chezmoi
          pkgs.shellcheck
          pkgs.shfmt
          pkgs.ruff
          pkgs.stylua
          pkgs.taplo
          pkgs.markdownlint-cli2
          pkgs.fish
          pkgs.zsh
        ];

        lintLocal = pkgs.writeShellApplication {
          name = "lint";
          runtimeInputs = toolchain;
          text = ''
            tmp_home="$(mktemp -d)"
            ruff_cache_dir="$(mktemp -d)"
            export HOME="$tmp_home"
            export XDG_DATA_HOME="$tmp_home/.local/share"
            export XDG_CONFIG_HOME="$tmp_home/.config"
            export RUFF_CACHE_DIR="$ruff_cache_dir"
            mkdir -p "$XDG_DATA_HOME" "$XDG_CONFIG_HOME"
            export TAPLO_CONFIG=${taploNixConfig}
            export PYTHONPATH="$PWD/nix"
            exec ${pythonEnv}/bin/python -m lint.cli fix "$@"
          '';
        };

        checkCmd = pkgs.writeShellApplication {
          name = "check";
          runtimeInputs = toolchain;
          text = ''
            tmp_home="$(mktemp -d)"
            ruff_cache_dir="$(mktemp -d)"
            export HOME="$tmp_home"
            export XDG_DATA_HOME="$tmp_home/.local/share"
            export XDG_CONFIG_HOME="$tmp_home/.config"
            export RUFF_CACHE_DIR="$ruff_cache_dir"
            mkdir -p "$XDG_DATA_HOME" "$XDG_CONFIG_HOME"
            export TAPLO_CONFIG=${taploNixConfig}
            export PYTHONPATH="$PWD/nix"
            exec ${pythonEnv}/bin/python -m lint.cli check "$@"
          '';
        };

        lintTestsCmd = pkgs.writeShellApplication {
          name = "lint-tests";
          runtimeInputs = toolchain;
          text = ''
            cd ${./.}
            export TAPLO_CONFIG=${taploNixConfig}
            pytest_cache_dir="$(mktemp -d)"
            export PYTEST_ADDOPTS="-o cache_dir=$pytest_cache_dir"
            exec ${pythonEnv}/bin/python -m pytest tests/lint "$@"
          '';
        };

        lintTypecheckCmd = pkgs.writeShellApplication {
          name = "lint-typecheck";
          runtimeInputs = toolchain;
          text = ''
            cd ${./.}
            mypy_cache_dir="$(mktemp -d)"
            export MYPYPATH="nix"
            exec ${pythonEnv}/bin/python -m mypy --cache-dir "$mypy_cache_dir" "$@"
          '';
        };

        lintStylecheckCmd = pkgs.writeShellApplication {
          name = "lint-stylecheck";
          runtimeInputs = toolchain;
          text = ''
            cd ${./.}
            ruff_cache_dir="$(mktemp -d)"
            export RUFF_CACHE_DIR="$ruff_cache_dir"
            exec ruff check nix/lint.py nix/lint tests/lint "$@"
          '';
        };
      in
      {
        apps.lint = {
          type = "app";
          program = "${lintLocal}/bin/lint";
        };

        apps.check = {
          type = "app";
          program = "${checkCmd}/bin/check";
        };

        apps.lint-tests = {
          type = "app";
          program = "${lintTestsCmd}/bin/lint-tests";
        };

        apps.lint-typecheck = {
          type = "app";
          program = "${lintTypecheckCmd}/bin/lint-typecheck";
        };

        apps.lint-stylecheck = {
          type = "app";
          program = "${lintStylecheckCmd}/bin/lint-stylecheck";
        };

        devShells.default = pkgs.mkShell {
          packages = [
            lintLocal
            checkCmd
            lintTestsCmd
            lintTypecheckCmd
            lintStylecheckCmd
          ];
          shellHook = ''
            export TAPLO_CONFIG=${taploNixConfig}
          '';
        };

        checks.check = pkgs.runCommand "check" {
          nativeBuildInputs = toolchain;
          src = ./.;
        } ''
          set -euo pipefail
          cd "$src"
          ${checkCmd}/bin/check --summary
          touch $out
        '';

        checks.lint-tests = pkgs.runCommand "lint-tests" {
          nativeBuildInputs = toolchain;
          src = ./.;
        } ''
          set -euo pipefail
          cd "$src"
          ${lintTestsCmd}/bin/lint-tests
          touch $out
        '';

        checks.lint-typecheck = pkgs.runCommand "lint-typecheck" {
          nativeBuildInputs = toolchain;
          src = ./.;
        } ''
          set -euo pipefail
          cd "$src"
          ${lintTypecheckCmd}/bin/lint-typecheck
          touch $out
        '';

        checks.lint-stylecheck = pkgs.runCommand "lint-stylecheck" {
          nativeBuildInputs = toolchain;
          src = ./.;
        } ''
          set -euo pipefail
          cd "$src"
          ${lintStylecheckCmd}/bin/lint-stylecheck
          touch $out
        '';
      });
}
