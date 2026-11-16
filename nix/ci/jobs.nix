{
  release = {
    runsOn = "ubuntu-latest";
    env.DL_TAG = "\${{ inputs.download_tag }}";
    env.RELEASE_TAG = "\${{ inputs.release_tag }}";
    steps = [
      { uses = "actions/checkout@v4"; }
      { uses = "DeterminateSystems/nix-installer-action@main"; }
      {
        id = "secrets";
        name = "decrypt-github-app-secrets";
        env.SOPS_AGE_KEY = "\${{ secrets.SOPS_AGE_KEY }}";
        run = ''
          decrypt() {
            nix shell --inputs-from . nixpkgs#sops -c sops decrypt --extract "[\"$1\"]" secrets/default.yaml
          }
          client_id=$(decrypt app_client_id)
          private_key=$(decrypt app_private_key)
          while IFS= read -r line; do
            [ -n "$line" ] && echo "::add-mask::$line"
          done <<<"$private_key"
          delim=$(openssl rand -hex 16)
          {
            echo "client_id=$client_id"
            echo "private_key<<$delim"
            echo "$private_key"
            echo "$delim"
          } >>"$GITHUB_OUTPUT"
        '';
      }
      {
        id = "app";
        name = "github-app-token";
        uses = "actions/create-github-app-token@v2";
        with_ = {
          app-id = "\${{ steps.secrets.outputs.client_id }}";
          private-key = "\${{ steps.secrets.outputs.private_key }}";
          owner = "CoreStory-Frontier";
          repositories = "plot";
        };
      }
      {
        name = "download";
        env.GH_TOKEN = "\${{ steps.app.outputs.token }}";
        run = ''nix run .#download -- "$DL_TAG" dist'';
      }
      {
        name = "latest-json";
        run = ''
          nix run .#latest-json -- \
            --tag "$RELEASE_TAG" \
            --base-url "https://github.com/''${{ github.repository }}/releases/download" \
            --dist dist
        '';
      }
      {
        name = "release-create";
        env.GH_TOKEN = "\${{ github.token }}";
        run = ''
          gh release create "$RELEASE_TAG" dist/* \
            -R "''${{ github.repository }}" \
            --title "$RELEASE_TAG" \
            --notes-file notes.md
        '';
      }
    ];
  };
}
