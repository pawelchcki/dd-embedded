# treefmt.nix
{pkgs, ...}: {
  # Used to find the project root
  projectRootFile = "flake.nix";

  programs.alejandra.enable = true;
  programs.rustfmt.enable = true;
}
