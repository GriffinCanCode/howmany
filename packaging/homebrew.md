# Homebrew

`howmany` ships from its own tap, [GriffinCanCode/homebrew-howmany][tap], not
from homebrew-core. Users install it with a qualified name:

```bash
brew install GriffinCanCode/howmany/howmany
```

Homebrew taps the repository as part of that command, so there is no separate
`brew tap` step. **A bare `brew install howmany` cannot work** — that name
resolves against homebrew-core only, and homebrew-core requires a notability bar
— roughly 75 stars, 30 forks, or 30 watchers — that this project does not yet
clear. Every place we document installation has to use the qualified name;
documenting the bare one is what produced [issue
#1](https://github.com/GriffinCanCode/howmany/issues/1).

## How the tap gets updated

`packaging/howmany.rb` is the source of truth. The `update-homebrew-formula`
job in `.github/workflows/release.yml` runs on every `v*` tag: it copies that
file into the tap, stamps in the tag's tarball url and its sha256, checks the
result parses as Ruby, and pushes. Nothing in the tap is hand-maintained, so
the published formula cannot drift from the crate.

The job authenticates with `HOMEBREW_TAP_TOKEN`, a token with write access to
the tap. When it expired, the tap stayed on v2.0.0 while three newer releases
shipped. Preflight now requires a working writer or proof that the tap already
publishes the requested tag with its exact archive checksum.

The second route supports local publication when the CI credential expires.
Preflight reads the live formula and hashes the tagged archive before accepting
that route. Build and publishing jobs always check out the requested tag.

## Testing a formula change

Homebrew 6 refuses to install a formula given as a loose path — it has to live
in a tap — so test by dropping the candidate into the local tap checkout, which
is a clone and pushes nothing:

```bash
cp packaging/howmany.rb "$(brew --repository GriffinCanCode/howmany)/Formula/"
brew install --build-from-source GriffinCanCode/howmany/howmany
brew test howmany
brew audit --strict --formula GriffinCanCode/howmany/howmany
```

## Publishing out of band

If CI cannot publish and the formula has to go out by hand:

```bash
VERSION=v3.1.0
URL="https://github.com/GriffinCanCode/howmany/archive/refs/tags/$VERSION.tar.gz"
SHA=$(curl -fsSL "$URL" | shasum -a 256 | cut -d' ' -f1)

git clone https://github.com/GriffinCanCode/homebrew-howmany.git /tmp/tap
sed -e "s|^  url \".*\"|  url \"$URL\"|" \
    -e "s|^  sha256 \".*\"|  sha256 \"$SHA\"|" \
    packaging/howmany.rb > /tmp/tap/Formula/howmany.rb
git -C /tmp/tap commit -am "howmany ${VERSION#v}" && git -C /tmp/tap push
```

That is the same transformation the workflow performs. After publishing the
formula, resume the release with the workflow on `main`; its builds remain pinned
to the requested tag:

```bash
gh workflow run release.yml --ref main -f version=v3.1.0
```

Rotate the CI token before the next automatic tap update.

[tap]: https://github.com/GriffinCanCode/homebrew-howmany
