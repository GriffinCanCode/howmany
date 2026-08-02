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

The job authenticates to the tap with the `HOMEBREW_TAP_TOKEN` secret, a
personal access token with write access to `homebrew-howmany`. **This is the
single point of failure.** When that token expires the job fails with
`Bad credentials`, the release itself still succeeds, and the tap silently
freezes at the last version that got through — which is exactly how the tap sat
on v2.0.0 while v2.1.0, v2.2.0, and v3.0.0 shipped. If a release's Homebrew job
goes red, rotate the token before assuming the formula published.

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
VERSION=v3.0.0
URL="https://github.com/GriffinCanCode/howmany/archive/refs/tags/$VERSION.tar.gz"
SHA=$(curl -fsSL "$URL" | shasum -a 256 | cut -d' ' -f1)

git clone https://github.com/GriffinCanCode/homebrew-howmany.git /tmp/tap
sed -e "s|^  url \".*\"|  url \"$URL\"|" \
    -e "s|^  sha256 \".*\"|  sha256 \"$SHA\"|" \
    packaging/howmany.rb > /tmp/tap/Formula/howmany.rb
git -C /tmp/tap commit -am "howmany ${VERSION#v}" && git -C /tmp/tap push
```

That is the same transformation the workflow performs. Prefer fixing the token.

[tap]: https://github.com/GriffinCanCode/homebrew-howmany
