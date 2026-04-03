{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      crane,
      rust-overlay,
      flake-utils,
      treefmt-nix,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        # Rust toolchain with thumbv7em-none-eabi target for no_std testing
        rustToolchain = pkgs.rust-bin.nightly.latest.default.override {
          extensions = [
            "clippy"
            "rust-src"
            "rustfmt"
          ];
          targets = [ "thumbv7em-none-eabi" ];
        };

        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        # Common source filtering
        src = craneLib.cleanCargoSource ./.;

        # Common args for all builds
        commonArgs = {
          inherit src;
          strictDeps = true;
        };

        # Build linea-proof-core for thumbv7em-none-eabi (no_std check)
        linea-proof-core-no-std = craneLib.buildPackage (
          commonArgs
          // {
            pname = "linea-proof-core";
            cargoExtraArgs = "-p linea-proof-core --target thumbv7em-none-eabi";
            # No std library available for this target
            RUSTFLAGS = "-C panic=abort";
            # Skip tests - can't run on embedded target
            doCheck = false;
          }
        );

        # Build verify-header-demo for native target
        verify-header-demo = craneLib.buildPackage (
          commonArgs
          // {
            pname = "verify-header-demo";
            cargoExtraArgs = "-p verify-header-demo";
            nativeBuildInputs = with pkgs; [
              pkg-config
            ];
            buildInputs =
              with pkgs;
              [
                openssl
              ]
              ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
                pkgs.darwin.apple_sdk.frameworks.Security
                pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
              ];
          }
        );

        # treefmt configuration
        treefmtEval = treefmt-nix.lib.evalModule pkgs {
          projectRootFile = "flake.nix";
          programs.nixfmt.enable = true;
          programs.rustfmt = {
            enable = true;
            package = rustToolchain;
          };
        };

        # Build dependencies for clippy
        cargoArtifacts = craneLib.buildDepsOnly (
          commonArgs
          // {
            nativeBuildInputs = with pkgs; [
              pkg-config
            ];
            buildInputs =
              with pkgs;
              [
                openssl
              ]
              ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
                pkgs.darwin.apple_sdk.frameworks.Security
                pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
              ];
          }
        );

        # Clippy check
        clippy = craneLib.cargoClippy (
          commonArgs
          // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--all-targets -- -D warnings";
            nativeBuildInputs = with pkgs; [
              pkg-config
            ];
            buildInputs =
              with pkgs;
              [
                openssl
              ]
              ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
                pkgs.darwin.apple_sdk.frameworks.Security
                pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
              ];
          }
        );

      in
      {
        checks = {
          inherit
            linea-proof-core-no-std
            verify-header-demo
            clippy
            ;
          formatting = treefmtEval.config.build.check self;
        };

        packages = {
          inherit linea-proof-core-no-std verify-header-demo;
          default = verify-header-demo;
        };

        formatter = treefmtEval.config.build.wrapper;

        devShells.default = craneLib.devShell {
          checks = self.checks.${system};
          packages =
            with pkgs;
            [
              pkg-config
              openssl
            ]
            ++ [ treefmtEval.config.build.wrapper ];
        };
      }
    );
}
