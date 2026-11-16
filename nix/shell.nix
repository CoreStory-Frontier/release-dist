{ pkgs, ... }:
{
  devShells.default =
    with pkgs;
    mkShell {
      packages = [
        age
        gh
        git
        nixd
        nix-output-monitor
        sops
      ];
    };
}
