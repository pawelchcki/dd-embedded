{
  description = "KTG";
  nixConfig.bash-prompt-prefix = "\[ktg\] ";

  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    treefmt-nix.url = "github:numtide/treefmt-nix";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    treefmt-nix,
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = nixpkgs.legacyPackages.${system};
      treefmt = treefmt-nix.lib.evalModule pkgs ./treefmt.nix;
      devtools = pkgs.symlinkJoin {
        name = "devtools";
        paths = [
          pkgs.espflash
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
          packages =
            [
              devtools
            ];
        };
      checks = {
        formatting = treefmt.config.build.check self;
      };
    });
}
