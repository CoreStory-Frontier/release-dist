{ pkgs, ... }:
{
  packages = {
    download = pkgs.callPackage ./download/package.nix { };
    latest-json = pkgs.callPackage ./latest-json/package.nix { };
  };
}
