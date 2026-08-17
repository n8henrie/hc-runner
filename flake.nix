{
  description = "github.com/n8henrie/hc-runner";

  inputs.nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";

  outputs =
    { self, nixpkgs }:
    let
      systems = [
        "aarch64-darwin"
        "x86_64-linux"
        "aarch64-linux"
      ];
      eachSystem =
        with nixpkgs.lib;
        f: foldAttrs mergeAttrs { } (map (s: mapAttrs (_: v: { ${s} = v; }) (f s)) systems);
      inherit ((nixpkgs.lib.importTOML ./Cargo.toml).package) name;
    in
    {
      overlays = {
        default = self.overlays.${name};
        ${name} = _: prev: {
          ${name} = self.packages.${prev.system}.${name};
        };
      };
    }
    // (eachSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in
      {

        packages = {
          default = self.packages.${system}.${name};
          ${name} = pkgs.callPackage ./package.nix { };
        };

        apps =
          let
            hc-runner = {
              type = "app";
              program = pkgs.lib.getExe self.packages.${system}.${name};
            };
          in
          {
            inherit hc-runner;
            default = hc-runner;
            macos-perms = {
              type = "app";
              program = pkgs.lib.getExe (
                pkgs.writeShellApplication {
                  name = "perms-script";
                  text = ''
                    set -x

                    TMPDIR=$(mktemp -d)
                    trap 'launchctl bootout gui/$UID/com.n8henrie.${name}_tmp' EXIT

                    launchctl submit \
                      -l com.n8henrie.${name}_tmp \
                      -o "$TMPDIR"/out.txt \
                      -e "$TMPDIR"/err.txt \
                      -- \
                      ${pkgs.lib.getExe self.outputs.packages.${system}.${name}} \
                      --slug hc-runner-setup-delete-me \
                      --url http://fake \
                      -- \
                      ls ~/Desktop ~/Downloads ~/Documents

                    until test -s "$TMPDIR"/out.txt
                      do sleep 0.1
                    done
                  '';
                }
              );
            };
          };

        devShells.default = pkgs.mkShell {
          inputsFrom = [ self.outputs.packages.${system}.${name} ];
          buildInputs = with pkgs; [
            bacon
            clippy
            rust-analyzer
            rustfmt
          ];
        };
      }
    ));
}
