#!/usr/bin/env bash
# Régénère les types TypeScript du client (client/src/engine/generated/) depuis les types Rust.
# ts-rs écrit un fichier par type quand on lance ses tests `export_bindings_*`.
set -euo pipefail
cd "$(dirname "$0")/.."
rm -rf client/src/engine/generated
for crate in buttonmen-engine buttonmen-data buttonmen-ai buttonmen-wasm; do
  cargo test -p "$crate" --features ts export_bindings --quiet
done
ls client/src/engine/generated
