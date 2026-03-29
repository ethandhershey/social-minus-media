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
      in {
        devShells.default = pkgs.mkShell {
          nativeBuildInputs = [
            fenix.packages.${system}.stable.completeToolchain
            pkgs.pkg-config
          ];
          buildInputs = [
            pkgs.openssl
            (pkgs.postgresql.withPackages (ps: [ ps.pgvector ]))
          ];
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [ pkgs.openssl ];

          shellHook = ''
            export PGDATA=$PWD/.postgres
            export PGHOST=$PWD/.postgres
            export PGPORT=5432
            export DATABASE_URL="postgresql:///mydb?host=$PWD/.postgres&port=$PGPORT"
          '';
        };
      });
}
