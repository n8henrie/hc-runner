{
  description = "github.com/n8henrie/hc-runner";

  inputs.nixpkgs.url = "github:nixos/nixpkgs";

  outputs =
    { self, nixpkgs }:
    let
      inherit (nixpkgs) lib;
      systems = [
        "aarch64-darwin"
        "x86_64-linux"
        "aarch64-linux"
      ];
      systemClosure =
        attrs: builtins.foldl' (acc: system: lib.recursiveUpdate acc (attrs system)) { } systems;
    in
    systemClosure (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ self.overlays.default ];
        };
        # Placeholder name allows one to enter `nix develop` prior to `Cargo.toml` existing
        name =
          if builtins.pathExists ./Cargo.toml then
            ((builtins.fromTOML (builtins.readFile ./Cargo.toml)).package).name
          else
            "placeholder";
      in
      {
        overlays = {
          default = self.overlays.${name};
          ${name} = _: prev: {
            # inherit doesn't work with dynamic attributes
            ${name} = self.packages.${prev.system}.${name};
          };
        };
        packages.${system} = {
          default = self.packages.${system}.${name};
          ${name} = pkgs.rustPlatform.buildRustPackage {
            inherit name;
            version =
              if builtins.pathExists ./Cargo.toml then
                ((builtins.fromTOML (builtins.readFile ./Cargo.toml)).package).version
              else
                "placeholder";
            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;
            nativeBuildInputs = [ pkgs.pkg-config ];
            buildInputs =
              with pkgs;
              ([ openssl ] ++ lib.optionals stdenv.isDarwin [ darwin.apple_sdk.frameworks.SystemConfiguration ]);
            dontUseCargoParallelTests = true;
            doCheck = pkgs.stdenv.isLinux;
          };
        };

        apps.${system}.default = {
          type = "app";
          program = "${self.packages.${system}.${name}}/bin/${name}";
        };

        devShells.${system}.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            cargo
            rust-analyzer
          ];
        };
      }
    );
}
