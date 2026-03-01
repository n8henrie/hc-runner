{
  lib,
  stdenv,
  apple-sdk,
  perl,
  rustPlatform,
  openssl,
  pkg-config,
}:
rustPlatform.buildRustPackage {
  inherit ((fromTOML (builtins.readFile ./Cargo.toml)).package) name version;
  src = lib.cleanSource ./.;
  cargoLock.lockFile = ./Cargo.lock;
  nativeBuildInputs = [ pkg-config ] ++ (lib.optionals (stdenv.isLinux && stdenv.isAarch64) [ perl ]);
  buildInputs = [
    openssl
  ]
  ++ lib.optionals stdenv.isDarwin [ apple-sdk ];
  doCheck = stdenv.isLinux;
  meta = with lib; {
    mainProgram = "hc-runner";
    homepage = "https://github.com/n8henrie/hc-runner";
    license = licenses.mit;
    maintainers = [ maintainers.n8henrie ];
    platforms = with platforms; linux ++ darwin;
  };
}
