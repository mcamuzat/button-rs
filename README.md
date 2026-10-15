# button-rs

Un duel de dés dans le navigateur, d'après Button Men de James Ernest (Cheapass Games) : moteur de règles en Rust compilé en WebAssembly, interface React. Le site est publié sur GitHub Pages à chaque push sur `main` (`.github/workflows/pages.yml`).

> Projet de fan non officiel, sans lien avec Cheapass Games. Les 27 personnages livrés (`data/buttons.toml`) sont originaux ; Button Men est une marque de Cheapass Games.

Règles de référence : [Button Men Online / buttonweavers](https://github.com/buttonmen-dev/buttonmen), reformulées dans `docs/RULES.md`.

Voir `docs/superpowers/specs/` pour la conception.

## Boutons buttonweavers (facultatif, usage privé)

`./scripts/import-buttonweavers.sh` importe les ~880 boutons de Button Men Online dans `data/buttonweavers/buttons.json` (gitignoré). Les noms des personnages appartiennent à Cheapass Games : ce fichier n'est pas versionné ni diffusé. Le build (`pnpm build`) ne copie jamais `client/public/` : ces boutons restent en local, servis par `pnpm dev` seulement. Les boutons qui utilisent des compétences hors v1 apparaissent « non jouables ». L'import retient aussi la catégorie de chaque édition (officielle, Button Men Online, fans, Fanatics) : relancez-le si l'écran de choix range tout dans « Autres ».

## Lancer en local

```bash
cargo install wasm-bindgen-cli --version 0.2.121 --locked   # une fois
./scripts/build-wasm.sh dev
cd client && pnpm install && pnpm dev
```

Après une modification des types Rust exposés au client : `./scripts/gen-types.sh`.

## Tests

```bash
cargo test --workspace
cd client && pnpm test && pnpm type-check
```

## Licence

MIT (voir `LICENSE`). Les noms et personnages de Button Men appartiennent à Cheapass Games et ne sont pas couverts par cette licence.
