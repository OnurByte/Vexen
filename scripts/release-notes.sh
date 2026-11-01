#!/usr/bin/env bash
# Print the GitHub release notes for a tag: what changed (from CHANGELOG.md),
# how to install, how to verify. Used by the release workflow and for backfill.
#
#   scripts/release-notes.sh v0.2.5 > notes.md
set -euo pipefail

tag="${1:?usage: release-notes.sh vMAJOR.MINOR.PATCH}"
ver="${tag#v}"
repo="${GITHUB_REPOSITORY:-OnurByte/Vexen}"
changelog="$(dirname "$0")/../CHANGELOG.md"

# The section under "## <version>", up to the next "## " heading, without leading blank lines.
changes="$(awk -v v="$ver" '$0 == "## " v { f = 1; next } /^## / { f = 0 } f' "$changelog" | sed -e '/./,$!d')"
if [ -z "$changes" ]; then
  changes="This version has no changelog entry. See the commit history for what changed."
fi

printf '%s\n' "## What changed in $ver" ""
printf '%s\n' "$changes" ""
printf '%s\n' "## Install" ""
printf '%s\n' '- **npx (Node 18 or newer, no Rust needed):** `npx @onurbyte/vexen@'"$ver"' init` inside any Git repository. The launcher downloads the archive below for your platform, checks its SHA-256 and runs it.'
printf '%s\n' '- **Prebuilt binary:** download the archive for your platform from the assets below (`vexen-'"$ver"'-<target>.zip` on Windows, `.tar.gz` elsewhere; x64 and arm64 for Linux and macOS, x64 for Windows), unpack it and put `vexen` on your `PATH`.'
printf '%s\n' '- **From source:** `cargo install --git https://github.com/'"$repo"' --tag '"$tag"' --locked`.'
printf '%s\n' "" "## Verify" ""
printf '%s\n' '- Every archive has a `.sha256` file next to it: `sha256sum -c vexen-'"$ver"'-<target>.tar.gz.sha256` (on Windows compare it with `certutil -hashfile <archive> SHA256`).'
printf '%s\n' '- Releases from 0.2.2 on also carry a GitHub build-provenance attestation: `gh attestation verify <archive> --repo '"$repo"'`. Earlier releases have checksums only.'
printf '%s\n' "" "## Status" ""
printf '%s\n' "Early development, so behavior can still change between 0.x versions. [Where Vexen stands](https://github.com/$repo#where-vexen-stands) lists what works, what has been tested and what has not; the full history is in [CHANGELOG.md](https://github.com/$repo/blob/$tag/CHANGELOG.md)."
