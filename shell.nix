let
  # Rolling updates, not deterministic.
  pkgs = import (fetchTarball ("channel:nixpkgs-unstable")) { };
in
pkgs.callPackage (
  {
    mkShell,
    cargo,
    rustc,
  }:
  mkShell {
    strictDeps = true;
    nativeBuildInputs = [
      cargo
      rustc
    ];
  }
) { }
