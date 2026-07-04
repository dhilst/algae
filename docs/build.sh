#!/usr/bin/env bash
# Build the Algae documentation site end to end: compile the kernel to wasm,
# bundle the CodeMirror editor, stage the runtime assets into _static/, and run
# Sphinx. Mirrors the CI `docs` job so the site can be built and previewed
# locally. Requires: cargo + wasm-pack, node + npm, python3 with the packages in
# requirements.txt.
set -euo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$HERE/.." && pwd)"
STATIC="$HERE/_static"

echo "==> Building algae-wasm (wasm32, --target web)"
wasm-pack build "$ROOT/algae-wasm" --target web --release
cp "$ROOT/algae-wasm/pkg/algae_wasm.js" "$STATIC/"
cp "$ROOT/algae-wasm/pkg/algae_wasm_bg.wasm" "$STATIC/"

echo "==> Building the CodeMirror editor bundle"
( cd "$ROOT/editors/codemirror" && npm ci && npm run build )
cp "$ROOT/editors/codemirror/dist/algae-editor.js" "$STATIC/"

echo "==> Staging playground example sources"
mkdir -p "$STATIC/examples"
cp "$ROOT"/algae/stdlib/v1/*.alg "$STATIC/examples/"

echo "==> Running Sphinx"
python3 -m sphinx -b html "$HERE" "$HERE/_build/html"

echo "==> Done: $HERE/_build/html/index.html"
