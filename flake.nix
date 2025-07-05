{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    nci = {
      url = "github:yusdacra/nix-cargo-integration";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };
    home-manager.url = "github:nix-community/home-manager";
  };

  outputs = inputs @ {
    self,
    parts,
    nci,
    ...
  }:
    parts.lib.mkFlake {inherit inputs;} {
      systems = ["x86_64-linux" "aarch64-linux"];
      imports = [
        inputs.home-manager.flakeModules.home-manager
        nci.flakeModule
        ./crates.nix
      ];
      perSystem = {
        config,
        pkgs,
        ...
      }: let
        outputs = config.nci.outputs;
      in {
        devShells.default = outputs."surrealdb-extras".devShell.overrideAttrs (old: {
          packages =
            (old.packages or [])
            ++ (with pkgs; [
              cargo-expand
              cargo-edit
              cargo-machete
              cargo-features-manager
            ]);
        });
      };
    };
}
