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
