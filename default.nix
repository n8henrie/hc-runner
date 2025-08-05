{
  lib,
  stdenv,
  apple-sdk_12,
  perl,
  rustPlatform,
  openssl,
  pkg-config,
}:
rustPlatform.buildRustPackage {
  inherit ((builtins.fromTOML (builtins.readFile ./Cargo.toml)).package) name version;
  src = lib.cleanSource ./.;
  cargoLock.lockFile = ./Cargo.lock;
  nativeBuildInputs = [ pkg-config ] ++ (lib.optionals (stdenv.isLinux && stdenv.isAarch64) [ perl ]);
  buildInputs = [
    openssl
  ] ++ lib.optionals stdenv.isDarwin [ apple-sdk_12 ];
  doCheck = stdenv.isLinux;
}
