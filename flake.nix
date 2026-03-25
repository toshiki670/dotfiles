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
        toolchain = [
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

        lintLocal = pkgs.writeShellApplication {
          name = "lint";
          runtimeInputs = toolchain;
          text = ''
            exec bash ${./nix/lint.sh} fix
          '';
        };

        checkCmd = pkgs.writeShellApplication {
          name = "check";
          runtimeInputs = toolchain;
          text = ''
            exec bash ${./nix/lint.sh} check
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

        devShells.default = pkgs.mkShell {
          packages = [
            lintLocal
            checkCmd
          ];
        };

        checks.check = pkgs.runCommand "check" {
          nativeBuildInputs = toolchain;
          src = self;
        } ''
          set -euo pipefail
          cd "$src"
          ${checkCmd}/bin/check
          touch $out
        '';
      });
}
