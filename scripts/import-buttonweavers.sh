#!/usr/bin/env bash
# Importe les boutons de buttonweavers dans data/buttonweavers/buttons.json (gitignoré).
# Usage local et privé uniquement : les noms des personnages appartiennent à Cheapass Games.
#   ./scripts/import-buttonweavers.sh [chemin/vers/un/clone/de/buttonmen]
set -euo pipefail
cd "$(dirname "$0")/.."
SRC="${1:-}"
if [ -z "$SRC" ]; then
  SRC="$(mktemp -d)/buttonmen"
  git clone --depth 1 https://github.com/buttonmen-dev/buttonmen.git "$SRC"
fi
python3 -I scripts/import_buttonweavers.py "$SRC/deploy/database/data.button.sql" data/buttonweavers/buttons.json
# Le client le charge depuis public/ (gitignoré aussi).
mkdir -p client/public/buttonweavers
cp data/buttonweavers/buttons.json client/public/buttonweavers/buttons.json
