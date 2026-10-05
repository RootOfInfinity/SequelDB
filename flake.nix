{
  description = "LexION: Lexing In Overdone Novelty";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
  };

  outputs =
    { self, nixpkgs }:
    let
      pkgs = nixpkgs.legacyPackages."x86_64-linux";
    in
    {

      devShells."x86_64-linux".default = pkgs.mkShell {
        buildInputs = with pkgs; [
          cargo
          rustc
          rustfmt
          clippy
          rust-analyzer
        ];
        nativeBuildInputs = [ pkgs.pkg-config ];
        env.RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
      };

      packages."x86_64-linux".default = pkgs.rustPlatform.buildRustPackage {
        name = "lexion";
        src = ./.;
        buildInputs = [ ];
        nativeBuildInputs = [ pkgs.pkg-config ];
        cargoHash = "sha256-F1Nd5aQCn8WiJdxIOZ7U4ogSOaiH2ys0m2NDU4N25P0=";
      };

    };
}
