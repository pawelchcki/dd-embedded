# treefmt.nix
{pkgs, ...}: {
  # Used to find the project root
  projectRootFile = "flake.nix";
  # Enable the Nix formatter "alejandra"
  programs.alejandra.enable = true;
  # Format C sources
  programs.clang-format.enable = true;
  # Format py sources
  programs.black.enable = true;
  # Format .sh scripts
  programs.shfmt.enable = true;
  # Format Go sources
  programs.gofmt.enable = true;
  settings.formatter.gofmt.excludes = ["**/vendor/**"];
  # Format Jsonnet sources
  programs.jsonnetfmt.enable = true;
  # Format bazel files
  programs.buildifier.enable = true;
  settings.formatter.buildifier.excludes = ["tools/experiment_gen/generator/templates/**" "tools/experiment_gen/tests/**/*.bzl"];
}
