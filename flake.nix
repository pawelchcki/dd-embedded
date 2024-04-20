{
  description = "KTG";
  nixConfig.bash-prompt-prefix = "\[ktg\] ";

  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    treefmt-nix.url = "github:numtide/treefmt-nix";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    treefmt-nix,
    rust-overlay,
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      overlays = [ (import rust-overlay) ];
      pkgs = import nixpkgs {
        inherit system overlays;
      };

      rust = pkgs.rust-bin.nightly.latest.default.override {
        extensions = [ "rust-src" ];
        targets = [ "riscv32imc-unknown-none-elf" ];
      };

      treefmt = treefmt-nix.lib.evalModule pkgs ./treefmt.nix;
      devtools = pkgs.symlinkJoin {
        name = "devtools";
        paths = [
          pkgs.espflash
          rust
        ];
      };
    in {
      packages = {
        devtools = devtools;
        default = devtools;
      };
      formatter = treefmt.config.build.wrapper;
      devShells.default =
        pkgs.mkShell
        {
          packages = [
            devtools
          ];
        };
      checks = {
        formatting = treefmt.config.build.check self;
      };
    });
}
