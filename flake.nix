{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    fenix.url = "github:nix-community/fenix";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, fenix, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        target = "aarch64-unknown-linux-musl";
        crossPkgs = pkgs.pkgsCross.aarch64-multiplatform-musl;
        crossCC = crossPkgs.stdenv.cc;

        toolchain = with fenix.packages.${system}; combine [
          stable.cargo
          stable.rustc
          stable.clippy
          stable.rustfmt
          stable.rust-src
          targets.${target}.stable.rust-std
        ];
      in {
        devShells = {
          default = pkgs.mkShell {
            nativeBuildInputs = [ toolchain pkgs.pkg-config ];
            buildInputs = [
              pkgs.openssl
              (pkgs.postgresql.withPackages (ps: [ ps.pgvector ]))
            ];
            LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [ pkgs.openssl ];
          };

          cross = pkgs.mkShell {
            nativeBuildInputs = [ toolchain pkgs.pkg-config crossCC ];
            buildInputs = [
              (pkgs.postgresql.withPackages (ps: [ ps.pgvector ]))
            ];

            CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER =
              "${crossCC}/bin/${crossCC.targetPrefix}cc";
            CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_RUSTFLAGS =
              "-C target-feature=+crt-static";

            CC_aarch64_unknown_linux_musl  = "${crossCC}/bin/${crossCC.targetPrefix}cc";
            CXX_aarch64_unknown_linux_musl = "${crossCC}/bin/${crossCC.targetPrefix}c++";
            AR_aarch64_unknown_linux_musl  = "${crossCC}/bin/${crossCC.targetPrefix}ar";

            AARCH64_UNKNOWN_LINUX_MUSL_OPENSSL_STATIC      = "1";
            AARCH64_UNKNOWN_LINUX_MUSL_OPENSSL_DIR         = "${crossPkgs.pkgsStatic.openssl.dev}";
            AARCH64_UNKNOWN_LINUX_MUSL_OPENSSL_LIB_DIR     = "${crossPkgs.pkgsStatic.openssl.out}/lib";
            AARCH64_UNKNOWN_LINUX_MUSL_OPENSSL_INCLUDE_DIR = "${crossPkgs.pkgsStatic.openssl.dev}/include";
          };
        };
      });
}
