{
  description = "cgit static generation environment with Rust renderer";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        
        cgit-renderer = pkgs.rustPlatform.buildRustPackage {
          pname = "cgit-renderer";
          version = "0.1.0";
          src = ./renderer;
          cargoLock.lockFile = ./renderer/Cargo.lock;
        };
      in
      {
        packages.default = cgit-renderer;
        packages.cgit-renderer = cgit-renderer;

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            cgit
            git
            rustup
            cargo
            rustc
            pkg-config
          ];
          shellHook = ''
            export CGIT_BIN="${pkgs.cgit}/lib/cgit/cgit.cgi"
            export CGIT_SHARE="${pkgs.cgit}/share/cgit"
            echo "cgit binary: $CGIT_BIN"
          '';
        };
      }
    );
}
