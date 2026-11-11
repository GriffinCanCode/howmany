---
doc_radar:
  sentinels:
    - file: Boxfile
      contains: ['version = "3.1.0"', 'source = "target/box/payload"', 'out = "target/box/howmany-3.1.0.box"']
    - file: packaging/box-editors.toml
      contains: ['id = "GriffinCanCode.howmany"', 'version = "2.0.1"', 'editors = ["vscode", "cursor"]']
---

# packaging

Everything that turns a commit into something a user can install: the version
bump, the git tag, and the Homebrew formula.

| File | What it does |
|---|---|
| `release.sh` | Bump the version in `Cargo.toml`, tag, and push. |
| `howmany.rb` | The Homebrew formula for the `howmany` package. |
| `homebrew.md` | How the formula reaches the tap, and what breaks when it doesn't. |

These live inside the crate rather than beside it because every path they use is
relative to the crate root — `Cargo.toml`, `packaging/howmany.rb`. `release.sh`
resolves its own directory and `cd`s to the parent, so it works from any working
directory, but only as long as that parent is the crate.

## Cutting a release

```bash
./packaging/release.sh patch --dry-run   # print the plan, change nothing
./packaging/release.sh patch             # bump, tag, push
```

The script refuses to run with a dirty working tree, so commit or stash first. It
takes `patch`, `minor`, or `major`. It bumps `Cargo.toml` and nothing else — the
formula's version is stamped in by the release workflow, from the tag, once the
tarball it has to checksum exists.

Before releasing, confirm the packaged crate is actually installable — the test
suites all run inside the source tree and cannot see a packaging mistake:

```bash
cargo package                    # warns about anything missing from `include`
./tools/smoke_fresh_install.sh   # installs from the crate and runs the binary
```

## Homebrew

`homebrew.md` has the full procedure. The short version: the formula here is the
source of truth, the release workflow copies it into
[the tap](https://github.com/GriffinCanCode/homebrew-howmany) with the tag's
checksum stamped in, and users install it as
`brew install GriffinCanCode/howmany/howmany`. A bare `brew install howmany`
resolves against homebrew-core, where we are not published, and always fails.

Homebrew 6 will not install a formula from a loose path, so test the candidate
through the local tap checkout — `homebrew.md` has the three commands.

The tap job authenticates with the `HOMEBREW_TAP_TOKEN` secret. It fails without
failing the release, so a red Homebrew job means the formula did not publish
even though the tag, the binaries, and crates.io all did.

## The Box

We package the GitHub release executables and the official Marketplace VSIX.
Run this from the checkout root with Box, the GitHub CLI, and Python 3.11 or later.

```bash
set -euo pipefail
tag=v3.1.0
stage=target/box
assets="$stage/assets"
payload="$stage/payload"
mac=howmany-aarch64-apple-darwin
linux=howmany-x86_64-unknown-linux-gnu
vsix=GriffinCanCode.howmany-2.0.1.vsix

mkdir -p "$assets" "$payload/bin/macos-aarch64" "$payload/bin/linux-x86_64" \
  "$payload/share/box/editors"
gh release download "$tag" --dir "$assets" --clobber --pattern "$mac" --pattern "$linux"
gh release view "$tag" --json assets --jq '
  .assets[]
  | select(.name == "howmany-aarch64-apple-darwin" or .name == "howmany-x86_64-unknown-linux-gnu")
  | [(.digest | ltrimstr("sha256:")), .name] | join("  ")
' > "$stage/asset-sha256"
(cd "$assets" && shasum -a 256 -c ../asset-sha256)

curl --compressed --fail --silent --show-error --location --max-time 60 \
  'https://marketplace.visualstudio.com/_apis/public/gallery/publishers/GriffinCanCode/vsextensions/howmany/2.0.1/vspackage' \
  --output "$assets/$vsix"
python3 - "$assets/$vsix" <<'PY'
import hashlib
from pathlib import Path
import sys
import tomllib

vsix = Path(sys.argv[1])
editor = tomllib.loads(Path("packaging/box-editors.toml").read_text())
if vsix.name != editor["file"] or hashlib.sha256(vsix.read_bytes()).hexdigest() != editor["sha256"]:
    raise SystemExit("Marketplace VSIX does not match packaging/box-editors.toml")
PY

install -m 0755 "$assets/$mac" "$payload/bin/macos-aarch64/howmany"
install -m 0755 "$assets/$linux" "$payload/bin/linux-x86_64/howmany"
install -m 0644 "$assets/$vsix" "$payload/share/box/editors/$vsix"
install -m 0644 packaging/box-editors.toml "$payload/share/box/editors.toml"
git show "${tag}:LICENSE" > "$payload/LICENSE"
box pack --dry-run
box pack
```

Test the archive on both declared targets before uploading it with
`box store target/box/howmany-3.1.0.box`. For a new CLI version, update the
Boxfile and release tag in the recipe. Update the VSIX pin when the editor
extension changes.
