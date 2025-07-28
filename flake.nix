{
  description = "minecraft-rs development environment";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      nixpkgs,
      fenix,
      ...
    }:
    let
      inherit (nixpkgs) lib;

      supportedSystems = [
        "aarch64-darwin"
        "aarch64-linux"
        "i686-linux"
        "x86_64-darwin"
        "x86_64-linux"
      ];

      forAllSystems =
        systems: f:
        lib.genAttrs systems (
          system:
          f (
            import nixpkgs {
              inherit system;
              overlays = [ fenix.overlays.default ];
            }
          )
        );
    in
    {
      formatter = forAllSystems supportedSystems (pkgs: pkgs.nixfmt-rfc-style);

      devShells = forAllSystems supportedSystems (pkgs: {
        default = pkgs.mkShell {
          buildInputs = [
            # stable components
            pkgs.fenix.stable.rustc
            pkgs.fenix.stable.cargo
            pkgs.fenix.stable.rust-std
            pkgs.fenix.stable.clippy
            pkgs.fenix.stable.rust-src
            pkgs.fenix.stable.rust-docs

            # nightly rustfmt
            pkgs.fenix.latest.rustfmt

            # rust-analyzer
            pkgs.rust-analyzer
          ];
        };
      });
    };
}
