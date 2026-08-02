# tools

Development tooling for the `howmany` crate. Nothing here ships to users — these
are the scripts that build, measure, and audit the crate during development.
None of them is on the `include` list in `Cargo.toml`, so they stay out of the
published package.

| File | What it does |
|---|---|
| `rebuild.sh` | Clean release build, then symlink the binary into `/usr/local/bin/howmany`. |
| `gen_corpus.py` | Generate a deterministic synthetic source corpus for throughput benchmarking. |
| `smoke_fresh_install.sh` | Install from the crate and verify the binary works on a machine that has never had it. |
| `shake_dead_code.py` | Delete items the compiler reports as never used, driven by clippy's JSON spans. |

Run everything from the crate root, not from this directory.

## Building

```bash
./tools/rebuild.sh
```

## Measuring throughput

`gen_corpus.py` exists because timings are only comparable against a fixed
corpus. It writes a golden manifest next to the tree, so a run's counts can be
checked against known-correct numbers rather than against a previous run of the
same possibly-wrong code.

```bash
python3 tools/gen_corpus.py /tmp/corpus --files 30000 --seed 7
cargo build --release
./target/release/howmany /tmp/corpus --cli            # compare to the manifest
```

Shape matters: `--shape realistic` (the default) puts 4-16 files per directory
the way real repositories do, so the numbers measure line counting. `--shape
deep` forces one file per directory, which measures directory traversal instead
— useful as a stress test, misleading as a headline.

The Criterion suite in `../benches/` covers the same paths in-process:

```bash
cargo bench
```

## Proving a fresh install works

The unit and integration suites all run inside the source tree, so none of them
can catch a packaging mistake: a missing file in `include`, a path that only
resolves on this machine, or a dependency on some tool that happens to be
installed here. This script installs from the crate and runs the binary under
`env -i` with nothing on `PATH` but the binary itself.

```bash
./tools/smoke_fresh_install.sh
```

It exits non-zero if any check fails, and prints one line per check.

## Removing dead code

`shake_dead_code.py` takes its spans from `cargo clippy --message-format=json`,
so the compiler decides what is dead rather than a text match. Removing one item
can expose the next layer, so run it until it reports nothing:

```bash
python3 tools/shake_dead_code.py     # repeat until "no dead items reported"
cargo test --all-targets             # confirm nothing live was removed
```

Always run the tests afterwards. The script deletes whole items, including their
doc comments and attributes, and a public item that only *appears* unused inside
the crate may still be part of the published API.
