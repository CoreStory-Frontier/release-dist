{ pkgs, ... }:
{
  devShells = {
    default =
      with pkgs;
      mkShell {
        packages =
          let
            sops-decrypt = writeShellApplication {
              name = "sops-decrypt";
              runtimeInputs = [ sops ];
              text = ''sops decrypt --extract "[\"$1\"]" secrets/default.yaml'';
            };
          in
          [
            age
            gh
            git
            nixd
            nix-output-monitor
            sops-decrypt
          ];
      };
  };
}
