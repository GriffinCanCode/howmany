#!/usr/bin/env python3
"""Delete the items rustc reports as never used.

Spans come from `cargo clippy --message-format=json`, so the deletion is driven
by the compiler rather than by matching source text. Each pass removes whole
items -- with their doc comments and attributes -- then the caller rebuilds; a
removal can expose the next layer of dead code, so the script is run until it
reports nothing left to do.

Everything works on bytes, never on `str`. Rustc spans are byte offsets, and
using them as character indices silently mangles any file containing non-ASCII
text -- which the interactive dashboard's sources do, since their labels carry
emoji.
"""

from __future__ import annotations

import json
import subprocess
import sys
from collections import defaultdict
from pathlib import Path

# "not a member of trait" is the cascade: removing a dead method from a trait
# orphans every implementation of it, and those are dead for the same reason.
DEAD = ("is never used", "are never used", "is not a member of trait")
ATTACHED_PREFIXES = (b"///", b"//!", b"#[", b"//")


def dead_spans() -> dict[Path, list[tuple[int, int]]]:
    """Byte ranges of never-used item identifiers, keyed by file."""
    proc = subprocess.run(
        ["cargo", "clippy", "--all-targets", "--message-format=json"],
        capture_output=True,
        text=True,
    )
    # Deduplicated: `--all-targets` compiles the library once per target, so the
    # same dead item is reported several times. Acting on each report separately
    # deletes one item and then a second, unrelated span at the same offsets.
    spans: dict[Path, set[tuple[int, int]]] = defaultdict(set)
    for raw in proc.stdout.splitlines():
        try:
            record = json.loads(raw)
        except json.JSONDecodeError:
            continue
        message = record.get("message") or {}
        if not any(marker in message.get("message", "") for marker in DEAD):
            continue
        # Primary spans point at the identifiers; the `span` on a "help" child is
        # a suggestion, not an item.
        for span in message.get("spans", []):
            if span.get("is_primary"):
                spans[Path(span["file_name"])].add(
                    (span["byte_start"], span["byte_end"])
                )
    return {path: sorted(found) for path, found in spans.items()}


def close_brace(source: bytes, open_at: int) -> int | None:
    """Offset of the brace matching the one at `open_at`.

    Skips string, byte-string, raw-string and character literals as well as both
    comment forms, because a `"{"` in a format string does not open a block.
    """
    depth = 0
    index = open_at
    end = len(source)
    while index < end:
        char = source[index : index + 1]
        pair = source[index : index + 2]
        if pair == b"//":
            index = source.find(b"\n", index)
            if index == -1:
                return None
            continue
        if pair == b"/*":
            index = source.find(b"*/", index)
            if index == -1:
                return None
            index += 2
            continue
        if char == b"r" and source[index + 1 : index + 2] in (b'"', b"#"):
            hashes = 0
            probe = index + 1
            while source[probe : probe + 1] == b"#":
                hashes += 1
                probe += 1
            if source[probe : probe + 1] == b'"':
                terminator = b'"' + b"#" * hashes
                closed = source.find(terminator, probe + 1)
                if closed == -1:
                    return None
                index = closed + len(terminator)
                continue
        if char in (b'"', b"'"):
            quote = char
            index += 1
            while index < end:
                current = source[index : index + 1]
                if current == b"\\":
                    index += 2
                    continue
                if current == quote:
                    break
                # A lifetime or label (`'a`) is not a character literal, so stop
                # before consuming the rest of the file looking for a closer.
                if quote == b"'" and current in (b" ", b"\t", b"\n", b",", b">", b")", b"{", b";"):
                    index -= 1
                    break
                index += 1
            index += 1
            continue
        if char == b"{":
            depth += 1
        elif char == b"}":
            depth -= 1
            if depth == 0:
                return index
        index += 1
    return None


def item_bounds(source: bytes, name_start: int) -> tuple[int, int] | None:
    """Extend an identifier span to the whole item, doc comments included."""
    line_start = source.rfind(b"\n", 0, name_start) + 1

    start = line_start
    while start > 0:
        previous = source.rfind(b"\n", 0, start - 1) + 1
        if previous >= start:
            break
        if source[previous:start].strip().startswith(ATTACHED_PREFIXES):
            start = previous
            continue
        break

    # A signature can span lines; the body starts at the first brace after it.
    brace = source.find(b"{", name_start)
    semicolon = source.find(b";", name_start)
    if brace == -1 or (semicolon != -1 and semicolon < brace):
        if semicolon == -1:
            return None
        # A trait method declaration with no body.
        newline = source.find(b"\n", semicolon)
        return start, len(source) if newline == -1 else newline + 1

    closing = close_brace(source, brace)
    if closing is None:
        return None
    newline = source.find(b"\n", closing)
    return start, len(source) if newline == -1 else newline + 1


def main() -> int:
    spans = dead_spans()
    if not spans:
        print("no dead items reported")
        return 0

    removed = 0
    for path, identifiers in spans.items():
        source = path.read_bytes()
        cuts = {
            bounds
            for name_start, _ in identifiers
            if (bounds := item_bounds(source, name_start))
        }
        if not cuts:
            continue

        # Delete from the end so earlier offsets stay valid, and drop any span
        # that overlaps one already removed.
        applied = 0
        boundary = len(source) + 1
        for start, end in sorted(cuts, reverse=True):
            if end > boundary:
                continue
            source = source[:start] + source[end:]
            boundary = start
            applied += 1
        path.write_bytes(source)
        removed += applied
        print(f"{path}: removed {applied} item(s)")

    print(f"removed {removed} item(s) total")
    return 0


if __name__ == "__main__":
    sys.exit(main())
