{
  lib,
  stdenv,
  apple-sdk,
  perl,
  rustPlatform,
  openssl,
  pkg-config,
  cacert,
}:
rustPlatform.buildRustPackage {
  inherit ((lib.importTOML ./Cargo.toml).package) name version;
  src = lib.cleanSource ./.;
  cargoLock.lockFile = ./Cargo.lock;
  nativeBuildInputs = [
    pkg-config
  ]
  ++ (lib.optionals (with stdenv.hostPlatform; isLinux && isAarch64) [ perl ]);
  buildInputs = [
    openssl
  ]
  ++ lib.optionals stdenv.hostPlatform.isDarwin [ apple-sdk ];
  doCheck = stdenv.hostPlatform.isLinux;
  dontUseCargoParallelTests = true;
  nativeCheckInputs = [ cacert ];
  meta = with lib; {
    mainProgram = "hc-runner";
    homepage = "https://github.com/n8henrie/hc-runner";
    license = licenses.mit;
    maintainers = [ maintainers.n8henrie ];
    platforms = with platforms; linux ++ darwin;
  };
}
