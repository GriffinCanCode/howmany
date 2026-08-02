#!/usr/bin/env bash
# Install the crate the way a user would and check that it works.
#
# Everything the unit and integration suites cannot see lives here: whether the
# published package actually contains what it needs, whether the binary runs on a
# machine that has never had it before, whether it degrades gracefully with no
# optional `sherlock` detector on PATH, and whether it still counts a project
# that happens to live under a directory named like build output -- the bug that
# made the published 2.1.0 report "0 files, 0 lines" for anything under /tmp.
#
# Usage: tools/smoke_fresh_install.sh
set -euo pipefail

crate_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

root="$work/install"
# Deliberately hostile: a path containing "build" and "tmp", the words that used
# to make every exclusion pattern match and empty the result.
project="$work/tmp/build/myproject"

pass=0
fail=0
check() {
    local label=$1 expected=$2 actual=$3
    if [[ "$actual" == *"$expected"* ]]; then
        printf '  ok   %s\n' "$label"
        pass=$((pass + 1))
    else
        printf '  FAIL %s\n       expected to contain: %s\n       got: %s\n' \
            "$label" "$expected" "$actual"
        fail=$((fail + 1))
    fi
}

echo "==> packaging and installing from the crate, not the working tree"
cargo install --path "$crate_dir" --root "$root" --locked --force >/dev/null 2>&1
binary="$root/bin/howmany"
test -x "$binary"

echo "==> building a fixture under $project"
mkdir -p "$project/src" "$project/node_modules/dep" "$project/target/debug"
printf 'fn main() {\n    println!("hi");\n}\n' >"$project/src/main.rs"
printf '// note\npub fn f() {}\n' >"$project/src/lib.rs"
printf '# Title\n\nProse.\n' >"$project/README.md"
printf 'MIT License\n\nboilerplate\n' >"$project/LICENSE"
printf 'module.exports = 1;\n' >"$project/node_modules/dep/index.js"
printf 'fn generated() {}\n' >"$project/target/debug/gen.rs"

# A PATH with nothing on it but the installed binary: no `sherlock`, no shell
# utilities the tool might have been leaning on without saying so.
bare_path="$root/bin"

echo "==> running with an empty environment and no optional detector"
export HOWMANY_CACHE_DIR="$work/cache"
counts=$(env -i PATH="$bare_path" HOME="$work" HOWMANY_CACHE_DIR="$work/cache" \
    "$binary" "$project" --cli 2>&1)
# The fixture has six files. Three are countable source or prose; LICENSE is
# boilerplate, node_modules is a dependency, target is build output.
check "counts a project under tmp/build" "3 files, 8 lines" "$counts"

json=$(env -i PATH="$bare_path" HOME="$work" HOWMANY_CACHE_DIR="$work/cache" \
    "$binary" "$project" -o json --no-cache 2>/dev/null)
check "emits parseable JSON" '"basic"' "$json"
python3 -c 'import json,sys; json.load(sys.stdin)' <<<"$json"
echo "  ok   JSON parses"
pass=$((pass + 1))

# Naming the extensions proves the exclusions rather than trusting a total that
# could match by coincidence.
extensions=$(python3 -c \
    'import json,sys; print(",".join(sorted(json.load(sys.stdin)["basic"]["stats_by_extension"])))' \
    <<<"$json")
check "counts only source and prose" "md,rs" "$extensions"

functions=$(python3 -c 'import json,sys; print(json.load(sys.stdin)["complexity"]["function_count"])' <<<"$json")
if [[ "$functions" -gt 0 ]]; then
    echo "  ok   complexity was measured ($functions functions)"
    pass=$((pass + 1))
else
    echo "  FAIL complexity reported zero functions"
    fail=$((fail + 1))
fi

echo "==> checking the answer does not depend on the optional detector"
with_detector=$("$binary" "$project" --cli --no-cache 2>/dev/null)
without=$(env -i PATH="$bare_path" HOME="$work" HOWMANY_CACHE_DIR="$work/cache" \
    "$binary" "$project" --cli --no-cache 2>/dev/null)
check "same counts with and without sherlock on PATH" "$without" "$with_detector"

echo "==> checking reports land where asked and the run is repeatable"
"$binary" "$project" -o html --output-file "$work/report.html" >/dev/null 2>&1
test -s "$work/report.html" && {
    echo "  ok   HTML report written"
    pass=$((pass + 1))
}

first=$("$binary" "$project" -o json --reproducible 2>/dev/null | python3 -c \
    'import json,sys; d=json.load(sys.stdin); print(json.dumps(d["basic"], sort_keys=True))')
second=$("$binary" "$project" -o json --reproducible 2>/dev/null | python3 -c \
    'import json,sys; d=json.load(sys.stdin); print(json.dumps(d["basic"], sort_keys=True))')
check "two reproducible runs agree" "$first" "$second"

echo "==> checking the cache is scoped per project and never changes the answer"
# A second project, so the cache has history. A single shared cache file made
# every run pay to parse every file the machine had ever analysed, which cost
# more than it saved once a few repositories had been scanned.
other="$work/other"
mkdir -p "$other"
for i in 1 2 3 4 5 6 7 8; do
    printf 'fn f%d() {\n    let _ = %d;\n}\n' "$i" "$i" >"$other/f$i.rs"
done
"$binary" "$other" --cli >/dev/null 2>&1
"$binary" "$project" --cli >/dev/null 2>&1

scopes=$(find "$work/cache" -name '*.json' | wc -l | tr -d ' ')
if [[ "$scopes" -ge 2 ]]; then
    echo "  ok   each project has its own cache file ($scopes files)"
    pass=$((pass + 1))
else
    echo "  FAIL projects share one cache file ($scopes found)"
    fail=$((fail + 1))
fi

cached=$("$binary" "$project" --cli 2>/dev/null)
uncached=$("$binary" "$project" --cli --no-cache 2>/dev/null)
check "cached and uncached runs agree" "$uncached" "$cached"

echo "==> checking a missing path fails loudly instead of reporting zero"
if env -i PATH="$bare_path" HOME="$work" "$binary" "$work/nope" --cli >/dev/null 2>&1; then
    echo "  FAIL a missing path exited successfully"
    fail=$((fail + 1))
else
    echo "  ok   a missing path exits non-zero"
    pass=$((pass + 1))
fi

echo
printf '%d passed, %d failed\n' "$pass" "$fail"
[[ "$fail" -eq 0 ]]
