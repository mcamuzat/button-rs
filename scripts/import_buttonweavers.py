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


# Sections of the buttonset table (comment lines), first word -> SetCategory (crates/button-data).
CATEGORIES = {"official": "Official", "button": "Online", "unofficial": "Unofficial", "fanatics": "Fanatics", "special": "Special"}
SET = re.compile(r"""^\(\s*\d+\s*,\s*'((?:[^'\\]|\\.)*)'""")


def read_sets(text: str) -> dict:
    """Edition name -> (category, release order), from the buttonset INSERT."""
    start = text.index("INSERT INTO buttonset")
    end = text.index(";", start)
    sets, category = {}, "Other"
    for line in text[start:end].splitlines():
        line = line.strip()
        if line.startswith("#"):
            first = line.lstrip("# ").split(" ")[0].lower()
            category = CATEGORIES.get(first, "Other")
        elif m := SET.match(line):
            sets[unescape(m.group(1))] = (category, len(sets))
    return sets


def unescape(text: str) -> str:
    return re.sub(r"\\(.)", r"\1", text)


def main(source: str, target: str) -> None:
    with open(source, encoding="utf-8") as f:
        text = f.read()
    sets = read_sets(text)
    buttons = []
    for name, recipe, special, button_set in ROW.findall(text):
        category, order = sets.get(button_set, ("Other", len(sets)))
        buttons.append({
            "name": unescape(name), "recipe": unescape(recipe), "set": button_set,
            "category": category, "set_order": order, "special": special == "1",
        })
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
