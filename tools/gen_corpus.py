#!/usr/bin/env python3
"""Generate a deterministic synthetic source corpus for throughput benchmarking.

The corpus mixes languages, file sizes and "noise" directories (node_modules,
target, dist, .git) so that both the traversal path and the line-counting path
are exercised the way a real repository would exercise them.

Two properties matter for the numbers to mean anything:

*Shape.* Real repositories hold many files per directory. A tree with one file
per directory measures `readdir` and `getattr`, not line counting -- on such a
corpus `find` and `rg --files` are as slow as we are, and any counting
improvement disappears into traversal noise. `--shape realistic` (the default)
places 4-16 files in each directory; `--shape deep` reproduces the pathological
one-file-per-directory case on purpose, as a traversal stress test.

*Determinism.* The same --files/--seed/--shape always produces byte-identical
output, so before/after timings and the golden line counts in CORPUS.txt are
comparable across runs and across machines.
"""

from __future__ import annotations

import argparse
import json
import random
import shutil
import sys
from pathlib import Path

# (extension, single-line comment, doc comment prefix, block open, block close)
LANGS = [
    ("rs", "//", "///", "/*", "*/"),
    ("py", "#", None, '"""', '"""'),
    ("js", "//", "/**", "/*", "*/"),
    ("ts", "//", "/**", "/*", "*/"),
    ("go", "//", None, "/*", "*/"),
    ("java", "//", "/**", "/*", "*/"),
    ("c", "//", "/**", "/*", "*/"),
    ("rb", "#", None, "=begin", "=end"),
    ("sh", "#", None, None, None),
    ("css", None, None, "/*", "*/"),
]

# Directories the tool must skip in their entirety. Every name here is pruned
# during traversal, so files below them cost nothing beyond one name check.
NOISE_DIRS = ["node_modules", "target", ".git", "__pycache__", "dist", "vendor"]

FILES_PER_DIR = (4, 16)
DIR_DEPTH = (1, 4)


def make_body(lang, rng, lines: int) -> tuple[str, dict]:
    """Build file content plus the exact expected line-category counts."""
    _, single, doc, _, _ = lang
    out: list[str] = []
    counts = {"code": 0, "comment": 0, "doc": 0, "blank": 0}

    while len(out) < lines:
        roll = rng.random()
        if roll < 0.12:
            out.append("")
            counts["blank"] += 1
        elif roll < 0.28 and single:
            # Indented, like a comment inside a function body. Go documents
            # declarations with the same `//`, so indentation is what tells a
            # body comment apart from a doc comment.
            out.append(f"    {single} explanatory note {len(out)}")
            counts["comment"] += 1
        elif roll < 0.36 and doc:
            # A block-style doc marker has to be closed on the same line.
            # `/** text` with no `*/` is an unterminated block comment, and
            # every following line of the file belongs to it -- which would
            # make the golden totals describe a bug rather than a corpus.
            out.append(f"{doc} documented behaviour {len(out)} */" if doc == "/**" else f"{doc} documented behaviour {len(out)}")
            counts["doc"] += 1
        else:
            out.append(f"    value_{len(out)} = compute({len(out)});")
            counts["code"] += 1

    return "\n".join(out) + "\n", counts


def source_dirs(rng, count: int, shape: str) -> list[list[str]]:
    """Enough directory paths to hold `count` files, one entry per directory."""
    if shape == "deep":
        # One directory per file: a traversal stress test, not a realistic tree.
        return [[f"pkg{rng.randint(0, 40)}" for _ in range(rng.randint(1, 5))] for _ in range(count)]

    needed = count // (sum(FILES_PER_DIR) // 2) + 2
    # Breadth-first over a bounded tree: broad at the leaves, shallow overall,
    # which is the shape of a real repository.
    dirs: list[list[str]] = []
    frontier: list[list[str]] = []
    crates = 0
    while len(dirs) < needed:
        if not frontier:
            frontier.append([f"crate{crates}"])
            crates += 1
        parent = frontier.pop(0)
        dirs.append(parent)
        if len(parent) <= DIR_DEPTH[1]:
            frontier.extend(parent + [f"mod{child}"] for child in range(3))
    return dirs


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("dest", type=Path)
    ap.add_argument("--files", type=int, default=12000)
    ap.add_argument("--seed", type=int, default=1337)
    ap.add_argument("--max-lines", type=int, default=400)
    ap.add_argument("--noise-ratio", type=float, default=0.35)
    ap.add_argument("--shape", choices=("realistic", "deep"), default="realistic")
    args = ap.parse_args()

    if args.dest.exists():
        shutil.rmtree(args.dest)
    args.dest.mkdir(parents=True)

    rng = random.Random(args.seed)
    totals = {"files": 0, "lines": 0, "code": 0, "comment": 0, "doc": 0, "blank": 0}
    by_extension: dict[str, dict[str, int]] = {}

    n_noise = int(args.files * args.noise_ratio)
    n_source = args.files - n_noise

    written = 0
    directories = source_dirs(rng, n_source, args.shape)
    for parts in directories:
        if written >= n_source:
            break
        d = args.dest.joinpath(*parts)
        d.mkdir(parents=True, exist_ok=True)

        here = 1 if args.shape == "deep" else rng.randint(*FILES_PER_DIR)
        for _ in range(min(here, n_source - written)):
            lang = LANGS[written % len(LANGS)]
            ext = lang[0]
            body, counts = make_body(lang, rng, rng.randint(5, args.max_lines))
            (d / f"mod_{written}.{ext}").write_text(body, encoding="utf-8")

            lines = sum(counts.values())
            totals["files"] += 1
            totals["lines"] += lines
            per_ext = by_extension.setdefault(
                ext, {"files": 0, "lines": 0, "code": 0, "comment": 0, "doc": 0, "blank": 0}
            )
            per_ext["files"] += 1
            per_ext["lines"] += lines
            for key, value in counts.items():
                totals[key] += value
                per_ext[key] += value
            written += 1

    if written < n_source:
        print(
            f"warning: only {written} of {n_source} source files placed; "
            f"the {args.shape} shape ran out of directories",
            file=sys.stderr,
        )

    # Noise: files a correct implementation must skip entirely. They are the
    # reason directory pruning matters, so there are enough of them to notice.
    for i in range(n_noise):
        nd = args.dest / NOISE_DIRS[i % len(NOISE_DIRS)] / f"sub{i % 20}"
        nd.mkdir(parents=True, exist_ok=True)
        lang = LANGS[i % len(LANGS)]
        body, _ = make_body(lang, rng, rng.randint(20, 120))
        (nd / f"vendored_{i}.{lang[0]}").write_text(body, encoding="utf-8")

    manifest = {
        "seed": args.seed,
        "shape": args.shape,
        "noise_files": n_noise,
        "totals": totals,
        "by_extension": dict(sorted(by_extension.items())),
    }
    # The manifest lives *beside* the corpus, not inside it: a .json file in the
    # tree would be counted, and the golden totals would no longer describe what
    # the tool reports.
    manifest_path = args.dest.with_name(args.dest.name + ".manifest.json")
    manifest_path.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")

    print(f"corpus at {args.dest} ({args.shape}), manifest at {manifest_path}")
    for key, value in sorted(totals.items()):
        print(f"  {key}={value}")
    print(f"  noise_files={n_noise}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
