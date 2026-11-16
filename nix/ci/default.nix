{ inputs, ... }:
{
  imports = [ inputs.github-actions.flakeModules.default ];

  perSystem =
    {
      config,
      pkgs,
      ...
    }:
    {
      githubActions = {
        enable = true;
        workflows = (pkgs.callPackage ./workflows.nix { }).workflows;
      };

      apps.write-workflows.program = pkgs.writeShellApplication {
        name = "write-workflows";
        runtimeInputs = [ pkgs.yq-go ];
        text = ''
          rm -rf .github/workflows && mkdir -p .github/workflows
          for f in ${config.githubActions.workflowsDir}/*.yml; do
            yq 'del(.. | select(. == null))' "$f" >".github/workflows/$(basename "$f")"
          done
        '';
      };
    };
}
