{
  inputs = {
    nixpkgs = {
      url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    };
    rust-overlay.url = "github:oxalica/rust-overlay";
    utils.url = "github:numtide/flake-utils";
    mzoon = {
      url = "github:JoyOfHardware/mzoon_nixos";
    };
    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };
  };

  outputs =
    inputs:
    inputs.utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import inputs.nixpkgs {
          localSystem = system;
          overlays = [
            inputs.rust-overlay.overlays.default
            (final: prev: {
              mzoon = inputs.mzoon.packages."${system}".default;

              tryit = prev.callPackage (
                {
                  stdenv,
                  rustPlatform,
                  binaryen,
                  mzoon,
                }:
                stdenv.mkDerivation {
                  pname = "tryit";
                  version = "0.1.0";

                  cargoDeps = pkgs.rustPlatform.importCargoLock {
                    lockFile = ./Cargo.lock;
                    allowBuiltinFetchGit = true;
                  };

                  src = inputs.self;

                  nativeBuildInputs = [
                    binaryen
                    mzoon
                    mzoon.passthru.rust # Use the rust version pinned in mzoon_nixos
                    mzoon.passthru.bindgen-cli # Use the rust version pinned in mzoon_nixos
                    rustPlatform.cargoSetupHook
                  ];

                  buildPhase = ''
                    runHook preBuild

                    mzoon build -r

                    runHook postBuild
                  '';

                  installPhase = ''
                    runHook preBuild

                    mkdir -p $out/bin
                    cp ./target/release/backend $out/bin/app
                    cp -rf frontend $out/bin/
                    cp -rf public $out/bin/

                    runHook postBuild
                  '';
                }
              ) { };
            })
          ];
        };
      in
      {
        packages = {
          default = inputs.self.packages."${system}".tryit;
          tryit = pkgs.tryit;
        };

        devShells.default =
          with pkgs;
          mkShell {
            inputsFrom = [ tryit ];
          };
      }
    );
}
