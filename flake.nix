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

        apps.default = {
          type = "app";
          program = pkgs.lib.getExe self.packages.${system}.${name};
        };

        devShells.default = pkgs.mkShell {
          inputsFrom = [ self.outputs.packages.${system}.${name} ];
          buildInputs = with pkgs; [
            bacon
            rust-analyzer
          ];
        };
      }
    ));
}
