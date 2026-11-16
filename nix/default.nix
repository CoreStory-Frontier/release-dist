{
  imports = [ ./ci ];
  perSystem.imports = [
    ./pkgs
    ./shell.nix
  ];
}
