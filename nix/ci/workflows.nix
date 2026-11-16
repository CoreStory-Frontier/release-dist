{ ... }:
{
  workflows.release = {
    name = "release";
    runName = "Release \${{ inputs.release_tag }}";
    on.workflowDispatch = {
      inputs = {
        release_tag = {
          description = "release tag";
          required = true;
          type = "string";
        };
        download_tag = {
          description = "download tag";
          required = true;
          type = "string";
        };
      };
    };
    permissions.contents = "write";
    concurrency.group = "release-\${{ inputs.release_tag }}";
    jobs = import ./jobs.nix;
  };
}
