#!/usr/bin/env bash
# Verification oracle for the algae parser: pytest + CLI smoke checks.
set -euo pipefail
cd "$(dirname "$0")/.."

FIXTURES=(test/stack.alg test/kvstore.alg test/rbac.alg test/base/container.alg)

echo "== pytest =="
python -m pytest test/ -q

echo "== check =="
python algae.py check "${FIXTURES[@]}"

echo "== print =="
python algae.py print "${FIXTURES[@]}" > /dev/null

echo "== fmt idempotency + ascii round-trip =="
tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT
for f in "${FIXTURES[@]}"; do
    python algae.py fmt "$f" > "$tmpdir/once.alg"
    python algae.py fmt "$tmpdir/once.alg" > "$tmpdir/twice.alg"
    diff "$tmpdir/once.alg" "$tmpdir/twice.alg" > /dev/null || { echo "fmt not idempotent: $f"; exit 1; }
    python algae.py fmt --ascii "$f" > "$tmpdir/ascii.alg"
    python algae.py check "$tmpdir/ascii.alg" > /dev/null || { echo "ascii output does not re-parse: $f"; exit 1; }
done

echo "VERIFY OK"
