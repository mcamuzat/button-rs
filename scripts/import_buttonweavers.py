#!/usr/bin/env python3
"""Convertit data.button.sql (buttonweavers) en JSON pour buttonmen-rs.

Usage : import_buttonweavers.py <data.button.sql> <sortie.json>
"""
import json
import os
import re
import sys

# (id, 'Nom', 'recette', btn_special, tourn_legal, (SELECT id FROM buttonset WHERE name="Set"))
ROW = re.compile(
    r"""\(\s*\d+\s*,\s*'((?:[^'\\]|\\.)*)'\s*,\s*'((?:[^'\\]|\\.)*)'\s*,\s*([01])\s*,\s*[01]\s*,"""
    r"""\s*\(SELECT id FROM buttonset\s+WHERE name="([^"]*)"\)"""
)


def unescape(text: str) -> str:
    return re.sub(r"\\(.)", r"\1", text)


def main(source: str, target: str) -> None:
    with open(source, encoding="utf-8") as f:
        text = f.read()
    buttons = [
        {"name": unescape(name), "recipe": unescape(recipe), "set": button_set, "special": special == "1"}
        for name, recipe, special, button_set in ROW.findall(text)
    ]
    if not buttons:
        sys.exit("aucun bouton trouvé : le format de data.button.sql a-t-il changé ?")
    os.makedirs(os.path.dirname(target) or ".", exist_ok=True)
    with open(target, "w", encoding="utf-8") as f:
        json.dump(buttons, f, ensure_ascii=False, indent=1)
    print(f"{len(buttons)} boutons écrits dans {target}")


if __name__ == "__main__":
    if len(sys.argv) != 3:
        sys.exit(__doc__)
    main(sys.argv[1], sys.argv[2])
