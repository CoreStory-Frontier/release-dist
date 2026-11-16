{ rustPlatform }:
let
  src = ../../../bin/latest-json;
in
rustPlatform.buildRustPackage {
  pname = "latest-json";
  version = "0.1.0";
  inherit src;
  cargoLock.lockFile = src + "/Cargo.lock";
  meta.mainProgram = "latest-json";
}
