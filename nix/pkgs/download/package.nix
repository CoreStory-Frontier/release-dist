{
  gh,
  writeShellApplication,
}:
writeShellApplication {
  name = "download";
  runtimeInputs = [ gh ];
  text = ''
    if [ $# -ne 2 ]; then
      echo "usage: download <tag> <dir>" >&2
      exit 64
    fi
    tag=$1
    dir=$2
    repo=CoreStory-Frontier/plot

    gh release download "$tag" -R "$repo" -D "$dir"
    gh release view "$tag" -R "$repo" --json body -q .body > "$dir/../notes.md"
  '';
}
