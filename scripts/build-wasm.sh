#!/usr/bin/env bash
# Compile buttonmen-wasm et génère la colle JS dans client/src/wasm/.
#   ./scripts/build-wasm.sh        build optimisé (profil wasm-release + wasm-opt)
#   ./scripts/build-wasm.sh dev    build rapide, sans optimisation
set -euo pipefail
cd "$(dirname "$0")/.."

PROFILE="wasm-release"
[ "${1:-}" = "dev" ] && PROFILE="dev"
OUT="client/src/wasm"
TARGET_DIR="${CARGO_TARGET_DIR:-target}"

cargo build -p buttonmen-wasm --target wasm32-unknown-unknown --profile "$PROFILE"

# Le profil "dev" écrit dans target/.../debug/.
PROFILE_DIR="$PROFILE"
[ "$PROFILE" = "dev" ] && PROFILE_DIR="debug"

wasm-bindgen --target web --out-dir "$OUT" --out-name buttonmen \
  "$TARGET_DIR/wasm32-unknown-unknown/$PROFILE_DIR/buttonmen_wasm.wasm"

if [ "$PROFILE" = "wasm-release" ] && command -v wasm-opt >/dev/null; then
  wasm-opt -Oz --enable-bulk-memory --enable-nontrapping-float-to-int \
    --enable-sign-ext --enable-mutable-globals --enable-reference-types --enable-multivalue \
    "$OUT/buttonmen_bg.wasm" -o "$OUT/buttonmen_bg.wasm"
fi

ls -lh "$OUT"/buttonmen_bg.wasm
