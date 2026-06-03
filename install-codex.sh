#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
SKILLS_DIR="${CODEX_HOME:-$HOME/.codex}/skills"

mkdir -p "$SKILLS_DIR"

for skill in alg-spec alg; do
  src="$SCRIPT_DIR/skills/$skill"
  dst="$SKILLS_DIR/$skill"
  if [ -e "$dst" ]; then
    echo "skip: $dst already exists (remove it first to reinstall)"
  else
    ln -s "$src" "$dst"
    echo "linked: $dst -> $src"
  fi
done

echo "done. Skills installed to $SKILLS_DIR"
