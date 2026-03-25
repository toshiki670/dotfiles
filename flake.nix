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

        lintLocal = pkgs.writeShellApplication {
          name = "lint";
          runtimeInputs = [
            pkgs.coreutils
            pkgs.git
            pkgs.diffutils
            pkgs.bash
            pkgs.shellcheck
            pkgs.shfmt
            pkgs.stylua
            pkgs.taplo
            pkgs.markdownlint-cli2
            pkgs.fish
            pkgs.zsh
          ];
          text = ''
            exec bash ${./nix/lint.sh} fix
          '';
        };

        lintCI = pkgs.writeShellApplication {
          name = "lint-ci";
          runtimeInputs = lintLocal.runtimeInputs;
          text = ''
            exec bash ${./nix/lint.sh} check
          '';
        };
      in
      {
        devShells.default = pkgs.mkShell {
          packages = [
            lintLocal
            lintCI
          ];
        };

        checks.lint = pkgs.runCommand "lint-ci" {
          nativeBuildInputs = lintCI.runtimeInputs;
          src = self;
        } ''
          set -euo pipefail
          cd "$src"
          lint-ci
          touch $out
        '';
      });
}
