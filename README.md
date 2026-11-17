# release-dist

Repository housing official release downloads for CoreStory, an application that turns source code into searchable, source-grounded intelligence for developers and agents alike.

## Download

Navigate to the [Releases](https://github.com/CoreStory-Frontier/release-dist/releases) page and select an artifact suitable for your platform.  A CLI and desktop GUI are available
for both Apple silicon and x86 Linux architectures, and the CLI is available for x86-64 Windows systems as well.  Build provenance attestations are signed and can be verified using
[cosign](https://docs.sigstore.dev/cosign/verifying/verify/) for CLI archives and the accompanying Sigstore bundle.  Desktop applications are signed for their platform's updater, and
on macOS for Gatekeeper as well.  The macOS app is signed with an Apple Developer ID certificate and notarized by Apple.

## Support

Please report any problem with installation, download, or use by opening an [issue](https://github.com/CoreStory-Frontier/release-dist/issues/new).  Be sure to include the release version,
platform, and any relevant error output.

## License

Use of CoreStory is subject to the terms supplied with the release.  Public availability of downloads does not grant an open-source license.
