# packaging

Everything that turns a commit into something a user can install: the version
bump, the git tag, and the Homebrew formula.

| File | What it does |
|---|---|
| `release.sh` | Bump the version in `Cargo.toml`, update the formula, tag, and push. |
| `howmany.rb` | The Homebrew formula for the `howmany` package. |
| `create-homebrew-tap.sh` | Scaffold the `homebrew-howmany` tap repository from the formula. |
| `homebrew.md` | How to publish and update the formula in the tap. |

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
takes `patch`, `minor`, or `major`, and updates both `Cargo.toml` and the
formula's version so the two cannot drift apart.

Before releasing, confirm the packaged crate is actually installable — the test
suites all run inside the source tree and cannot see a packaging mistake:

```bash
cargo package                    # warns about anything missing from `include`
./tools/smoke_fresh_install.sh   # installs from the crate and runs the binary
```

## Homebrew

`homebrew.md` has the full procedure. The short version: the formula here is the
source of truth, and publishing means copying it into the tap repository.

```bash
brew install --build-from-source packaging/howmany.rb   # test locally
brew audit --strict packaging/howmany.rb                # before publishing
```
