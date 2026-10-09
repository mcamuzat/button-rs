# buttonmen-rs

Button Men (le jeu de dés de James Ernest / Cheapass Games) dans le navigateur : moteur de règles en Rust compilé en WebAssembly, interface React.

Règles de référence : [Button Men Online / buttonweavers](https://github.com/buttonmen-dev/buttonmen), reformulées dans `docs/RULES.md`.

Voir `docs/superpowers/specs/` pour la conception.

## Boutons buttonweavers (facultatif, usage privé)

`./scripts/import-buttonweavers.sh` importe les ~880 boutons de Button Men Online dans `data/buttonweavers/buttons.json` (gitignoré). Les noms des personnages appartiennent à Cheapass Games : ce fichier n'est pas versionné ni diffusé. Les boutons qui utilisent des compétences hors v1 apparaissent « non jouables ».

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
