{
  lib,
  stdenv,
  darwin,
  perl,
  rustPlatform,
  openssl,
  pkg-config,
}:
rustPlatform.buildRustPackage {
  inherit ((builtins.fromTOML (builtins.readFile ./Cargo.toml)).package) name version;
  src = lib.cleanSource ./.;
  cargoLock.lockFile = ./Cargo.lock;
  nativeBuildInputs = [ pkg-config ];
  buildInputs = [
    openssl
  ] ++ lib.optionals stdenv.isDarwin [ darwin.apple_sdk.frameworks.SystemConfiguration ];
  dontUseCargoParallelTests = true;
  doCheck = stdenv.isLinux;
}
