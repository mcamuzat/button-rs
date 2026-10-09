# Jalon 1 — Button Men jouable en hotseat : plan d'implémentation

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Une partie complète de Button Men (3 manches gagnantes, cœur des règles + Swing/Option/Twin + 15 compétences) se joue à deux sur le même écran dans le navigateur, sur un moteur Rust compilé en WASM.

**Architecture:** `crates/engine` est un réducteur pur `apply(&GameState, PlayerId, Action) -> Result<(GameState, Vec<GameEvent>), RuleError>` ; les compétences sont des variantes de `DieSkill` interrogées à des points d'accroche (`hooks.rs`) par des `match` exhaustifs. `crates/button-data` parse les recettes (nom 8) et charge les boutons ; `crates/engine-wasm` garde l'état dans un objet `WasmMatch` et n'échange que du JSON (`GameView`, `Action`, événements). `client/` (React) n'affiche que la vue fournie par le moteur via une interface `GameAdapter`.

**Tech Stack:** Rust stable (édition 2024), serde, thiserror 2, nom 8, ts-rs 12 (types TypeScript générés), proptest 1, toml 1, wasm-bindgen =0.2.121 ; React 19, Vite 7, TypeScript 5.9, Tailwind CSS 4 (`@tailwindcss/vite`), motion 12, Vitest 3, pnpm.

**Spec:** `docs/superpowers/specs/2026-10-09-buttonmen-rs-design.md`

## Global Constraints

- Le moteur porte toute la logique ; WASM et client n'en contiennent aucune. Le client n'affiche que des valeurs fournies par le moteur (`GameView`, `LegalActions`, `SkillInfo`).
- Toute règle implémentée est annotée `// R x.y: <résumé>` et `R x.y` existe dans `docs/RULES.md`.
- `match` exhaustifs sur `DieSkill` dans chaque accroche, sans joker `_`.
- Tout hasard passe par `SeededRng` dans `GameState`.
- Bornes Swing : P 1–30, R 2–16, S 6–20, T 2–12, U 8–30, V 6–12, W 4–12, X 4–20, Y 1–20, Z 4–30.
- Match en `max_wins` manches gagnantes, 3 par défaut.
- Lettres de compétences (buttonweavers) : Berserk `B`, Chance `c`, Fire `F`, Focus `f`, Konstant `k`, Null `n`, Poison `p`, Queer `q`, Reserve `r`, Shadow `s`, Speed `z`, Stealth `d`, Stinger `g`, Trip `t`, Value `v`.
- Scores internes en **demi-points** (`i32`) ; l'affichage divise par 2.
- `data/buttonweavers/` est gitignoré ; le jeu fonctionne sans lui.
- Interface en français.
- Commits : messages conventionnels, terminés par `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.

## Review Focus

1. **Égalités d'initiative** (mêmes valeurs des deux côtés, ou un joueur sans dé éligible parce qu'il n'a que des Trip/Stinger) : la partie doit toujours désigner un premier joueur, jamais bloquer. → test `initiative_with_no_eligible_dice_still_decides` (Tâche 5).
2. **Pass alors qu'une attaque n'est possible qu'avec l'aide de Fire** : passer doit être refusé (on doit attaquer si on le peut). → test `pass_refused_when_only_fire_assisted_attack_exists` (Tâche 9).
3. **Recettes buttonweavers exotiques** (`+`, `?`, `{`, `C`, dés à 0 face, espaces multiples) : le parseur renvoie une erreur typée, il ne panique jamais, et le bouton apparaît « non jouable ». → test `parser_never_panics_on_arbitrary_input` (Tâche 4, proptest).
4. **Hotseat et Swing cachée** : quand le joueur 1 a choisi sa Swing, la vue du joueur 2 ne doit pas la révéler avant que les deux aient choisi. → test `pending_swing_hidden_from_opponent` (Tâche 12).
5. **Manche qui finit sur deux passes successives avec des dés restants** : le score compte les dés gardés à moitié, et la manche suivante démarre bien (Swing du perdant redemandée). → test `two_consecutive_passes_end_round` (Tâche 8).

---

## Structure des fichiers

```
buttonmen-rs/
├── Cargo.toml                       # workspace
├── rust-toolchain.toml
├── .cargo/config.toml               # TS_RS_EXPORT_DIR
├── .gitignore
├── README.md
├── docs/RULES.md                    # règles numérotées (R x.y)
├── data/buttons.toml                # boutons originaux
├── scripts/
│   ├── build-wasm.sh
│   ├── gen-types.sh
│   ├── import-buttonweavers.sh
│   └── import_buttonweavers.py
├── crates/
│   ├── engine/
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs               # ré-exports
│   │   │   ├── ids.rs               # PlayerId, DieId
│   │   │   ├── rng.rs               # SeededRng (splitmix64)
│   │   │   ├── skill.rs             # DieSkill, SkillSet, SkillInfo
│   │   │   ├── die.rs               # SwingLetter, DieShape, TwinPart, DieSpec, Sides, Die
│   │   │   ├── state.rs             # GameState, PlayerState, DieSlot, WaitingFor…
│   │   │   ├── action.rs            # Action, DeclaredAttack, AttackKind, SpecChoice
│   │   │   ├── event.rs             # GameEvent
│   │   │   ├── error.rs             # RuleError + sous-erreurs
│   │   │   ├── hooks.rs             # points d'accroche des compétences
│   │   │   ├── setup.rs             # new_game, Swing/Option, Reserve, début de manche
│   │   │   ├── initiative.rs        # initiative + Focus/Chance
│   │   │   ├── attack.rs            # validation des attaques, sommes Skill
│   │   │   ├── resolve.rs           # capture, relance, Berserk, Trip
│   │   │   ├── fire.rs              # aide Fire
│   │   │   ├── scoring.rs           # points en demi-points
│   │   │   ├── round.rs             # fin de tour, de manche, de match
│   │   │   ├── apply.rs             # réducteur
│   │   │   ├── legal.rs             # legal_actions, candidate_actions
│   │   │   ├── view.rs              # GameView, view_for
│   │   │   └── testing.rs           # constructeur de scénarios (feature "testing")
│   │   └── tests/
│   │       ├── common/mod.rs
│   │       ├── setup.rs
│   │       ├── initiative.rs
│   │       ├── attacks.rs
│   │       ├── rounds.rs
│   │       ├── skills.rs
│   │       ├── fire.rs
│   │       ├── view.rs
│   │       └── properties.rs
│   ├── button-data/
│   │   ├── Cargo.toml
│   │   └── src/{lib.rs, recipe.rs, catalog.rs}
│   └── engine-wasm/
│       ├── Cargo.toml
│       └── src/lib.rs
└── client/
    ├── package.json, vite.config.ts, tsconfig.json, index.html
    └── src/
        ├── main.tsx, App.tsx, styles.css
        ├── engine/generated/        # types générés par ts-rs (versionnés)
        ├── engine/wasm.ts           # chargement du module WASM
        ├── engine/format.ts         # affichage des demi-points
        ├── adapters/GameAdapter.ts
        ├── adapters/LocalAdapter.ts
        ├── screens/{Home,ButtonPicker,Table,MatchEnd}.tsx
        └── table/{DieChip,PlayerArea,AttackPanel,SpecifyDicePanel,InitiativePanel,FirePanel,ReservePanel,Handoff,selection}.ts(x)
```

**Note sur les tests Rust :** les tests d'intégration de `crates/engine/tests/` utilisent `buttonmen_engine::testing`, activé par une dev-dependency de la crate sur elle-même (`buttonmen-engine = { path = ".", features = ["testing"] }`, ajoutée à la Tâche 5). Un simple `cargo test -p buttonmen-engine` suffit. Ce dépôt n'a pas de Tilt : les commandes cargo et pnpm se lancent directement.

---
### Task 1: Workspace, identifiants et RNG seedé

**Files:**
- Create: `Cargo.toml`, `rust-toolchain.toml`, `.cargo/config.toml`, `.gitignore`, `README.md`
- Create: `crates/engine/Cargo.toml`, `crates/engine/src/lib.rs`, `crates/engine/src/ids.rs`, `crates/engine/src/rng.rs`

**Interfaces:**
- Produces: `PlayerId { One, Two }` avec `ALL`, `index()`, `other()` ; `DieId(pub u16)` ; `SeededRng::new(u64)`, `next_u64()`, `roll(sides: u8) -> u8` (uniforme 1..=sides), `coin() -> bool`.

- [ ] **Step 1: Créer le workspace**

`Cargo.toml` :
```toml
[workspace]
members = ["crates/engine"]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2024"
license = "MIT"

[workspace.dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
nom = "8"
toml = "1"
ts-rs = "12"
proptest = "1"
buttonmen-engine = { path = "crates/engine" }
buttonmen-data = { path = "crates/button-data" }

[profile.wasm-release]
inherits = "release"
opt-level = "z"
lto = true
codegen-units = 1
panic = "abort"
strip = true
```

`rust-toolchain.toml` :
```toml
[toolchain]
channel = "stable"
components = ["rustfmt", "clippy"]
targets = ["wasm32-unknown-unknown"]
```

`.cargo/config.toml` :
```toml
[env]
# ts-rs écrit les types TypeScript générés ici (cargo test --features ts).
TS_RS_EXPORT_DIR = { value = "client/src/engine/generated", relative = true }
```

`.gitignore` :
```
/target
/data/buttonweavers/
/client/public/buttonweavers/
/client/node_modules
/client/dist
/client/src/wasm/
```

`README.md` :
```markdown
# buttonmen-rs

Button Men (le jeu de dés de James Ernest / Cheapass Games) dans le navigateur : moteur de règles en Rust compilé en WebAssembly, interface React.

Règles de référence : [Button Men Online / buttonweavers](https://github.com/buttonmen-dev/buttonmen), reformulées dans `docs/RULES.md`.

Voir `docs/superpowers/specs/` pour la conception.
```

`crates/engine/Cargo.toml` :
```toml
[package]
name = "buttonmen-engine"
version.workspace = true
edition.workspace = true
license.workspace = true

[features]
# Génère les types TypeScript (cargo test -p buttonmen-engine --features ts).
ts = ["dep:ts-rs"]
# Expose le module `testing` (constructeur de scénarios) aux tests et aux autres crates.
testing = []

[dependencies]
serde = { workspace = true }
thiserror = { workspace = true }
ts-rs = { workspace = true, optional = true }

[dev-dependencies]
proptest = { workspace = true }
serde_json = { workspace = true }
```

- [ ] **Step 2: Écrire les tests du RNG et des identifiants**

`crates/engine/src/ids.rs` :
```rust
use serde::{Deserialize, Serialize};

/// A seat at the table. Button Men is played by two players (R 2.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub enum PlayerId {
    One,
    Two,
}

impl PlayerId {
    pub const ALL: [PlayerId; 2] = [PlayerId::One, PlayerId::Two];

    pub fn index(self) -> usize {
        match self {
            PlayerId::One => 0,
            PlayerId::Two => 1,
        }
    }

    pub fn other(self) -> PlayerId {
        match self {
            PlayerId::One => PlayerId::Two,
            PlayerId::Two => PlayerId::One,
        }
    }
}

/// Stable identifier of a die for the whole match: it survives rounds and captures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct DieId(pub u16);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn other_is_an_involution() {
        for p in PlayerId::ALL {
            assert_ne!(p, p.other());
            assert_eq!(p, p.other().other());
        }
        assert_eq!(PlayerId::One.index(), 0);
        assert_eq!(PlayerId::Two.index(), 1);
    }
}
```

`crates/engine/src/rng.rs` (tests d'abord, implémentation à l'étape 4) :
```rust
use serde::{Deserialize, Serialize};

/// The only source of randomness of a match (splitmix64). Serializable, so a match
/// can be saved and replayed exactly.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SeededRng {
    state: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_same_sequence() {
        let mut a = SeededRng::new(42);
        let mut b = SeededRng::new(42);
        for _ in 0..100 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn roll_stays_in_range_and_hits_every_face() {
        let mut rng = SeededRng::new(7);
        let mut seen = [false; 7];
        for _ in 0..10_000 {
            let v = rng.roll(6);
            assert!((1..=6).contains(&v));
            seen[v as usize] = true;
        }
        assert!(seen[1..=6].iter().all(|&s| s));
    }

    #[test]
    fn one_sided_die_always_rolls_one() {
        let mut rng = SeededRng::new(1);
        for _ in 0..50 {
            assert_eq!(rng.roll(1), 1);
        }
    }

    #[test]
    fn coin_produces_both_sides() {
        let mut rng = SeededRng::new(3);
        let heads = (0..1000).filter(|_| rng.coin()).count();
        assert!((350..650).contains(&heads));
    }
}
```

`crates/engine/src/lib.rs` :
```rust
//! Button Men rules engine. Rule references (`R x.y`) point to `docs/RULES.md`.

pub mod ids;
pub mod rng;

pub use ids::{DieId, PlayerId};
pub use rng::SeededRng;
```

- [ ] **Step 3: Lancer les tests, vérifier qu'ils échouent**

Run: `cargo test -p buttonmen-engine`
Expected: échec de compilation, `no function or associated item named 'new' found for struct 'SeededRng'`.

- [ ] **Step 4: Implémenter le RNG**

Ajouter dans `rng.rs`, au-dessus du module de tests :
```rust
impl SeededRng {
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    /// splitmix64.
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// A fair roll in `1..=sides`, by rejection sampling (no modulo bias).
    pub fn roll(&mut self, sides: u8) -> u8 {
        assert!(sides >= 1, "a die has at least one side");
        let n = u64::from(sides);
        let zone = u64::MAX - (u64::MAX % n);
        loop {
            let x = self.next_u64();
            if x < zone {
                return (x % n) as u8 + 1;
            }
        }
    }

    pub fn coin(&mut self) -> bool {
        self.next_u64() & 1 == 1
    }
}
```

- [ ] **Step 5: Lancer les tests, vérifier qu'ils passent**

Run: `cargo test -p buttonmen-engine && cargo clippy -p buttonmen-engine -- -D warnings && cargo fmt --all --check`
Expected: 5 tests PASS, clippy et fmt sans remarque.

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml rust-toolchain.toml .cargo .gitignore README.md crates/engine
git commit -m "feat(engine): workspace, identifiants et RNG seedé"
```

---

### Task 2: Règles de référence `docs/RULES.md`

Les tâches suivantes annotent le code avec `R x.y` ; ce fichier doit exister avant. Le contenu est une reformulation en français de `HowToPlay.js` et des `BMSkill*.php` de buttonweavers (licence BSD-3).

**Files:**
- Create: `docs/RULES.md`

**Interfaces:**
- Produces: les numéros de section `R 1.1` … `R 7.4` cités par toutes les tâches suivantes.

- [ ] **Step 1: Écrire `docs/RULES.md`**

```markdown
# Règles de Button Men (référence du moteur)

Source : Button Men Online / buttonweavers (`github.com/buttonmen-dev/buttonmen`, licence BSD-3) — `src/ui/js/HowToPlay.js` pour les règles de base, `src/engine/BMSkill*.php` pour les compétences, `BMDieSwing.php` pour les bornes Swing, `BMGame.php` pour l'initiative. Chaque section est citée dans le code sous la forme `R x.y`.

## R 1 — Matériel

- **R 1.1 Recette.** Un bouton est une liste de dés écrits `<compétences>(<forme>)<compétences>`, séparés par des espaces. Les compétences sont des lettres (voir R 6). Exemple : `p(4) s(10) (X) (6/10) z(4,4)`.
- **R 1.2 Swing.** Une lettre P R S T U V W X Y Z dans les parenthèses désigne un dé Swing dont le joueur choisit la taille dans la plage de la lettre : P 1–30, R 2–16, S 6–20, T 2–12, U 8–30, V 6–12, W 4–12, X 4–20, Y 1–20, Z 4–30. Tous les dés d'un même bouton portant la même lettre ont la même taille.
- **R 1.3 Option.** `(a/b)` : le joueur choisit l'une des deux tailles, a ou b.
- **R 1.4 Twin.** `(a,b)` : deux dés lancés ensemble qui comptent comme un seul. Chaque moitié est un nombre ou une lettre Swing. Taille = somme des tailles ; valeur = somme des deux jets ; valeur minimale 2.
- **R 1.5 Valeurs.** Un dé à n faces montre une valeur de 1 à n (de 2 à la taille pour un Twin).

## R 2 — Match

- **R 2.1** Deux joueurs, chacun avec un bouton.
- **R 2.2** Un match se joue en manches ; le premier joueur à gagner `max_wins` manches (3 par défaut) gagne le match.
- **R 2.3** Après une manche, le perdant peut changer ses tailles Swing et Option ; le gagnant garde les siennes. Après une égalité, personne ne change.
- **R 2.4** Un joueur peut abandonner à tout moment ; son adversaire gagne le match.

## R 3 — Début de manche

- **R 3.1** Au premier tour, chaque joueur choisit ses tailles Swing et Option, en secret ; les choix sont révélés ensemble.
- **R 3.2** Tous les dés en jeu sont lancés. Les dés capturés à la manche précédente reviennent à leur propriétaire ; chaque manche repart de la recette (hors dés Reserve ajoutés, R 6.11).
- **R 3.3 Initiative.** On compare les dés de chaque joueur, triés par valeur croissante, en ignorant ceux que leurs compétences excluent (R 6.14 Trip, R 6.13 Stinger). Le joueur dont le plus petit dé est le plus bas commence ; à égalité on compare le suivant, et ainsi de suite. Si les dés d'un joueur sont épuisés alors que l'autre en a encore, celui qui en a encore commence.
- **R 3.4** Égalité complète : le premier joueur est tiré au sort.
- **R 3.5 Réactions.** Le joueur qui n'a pas l'initiative peut réagir avec ses dés Focus (R 6.4) ou Chance (R 6.2), ou laisser l'autre commencer. S'il prend l'initiative, l'autre peut réagir à son tour, et ainsi de suite.

## R 4 — Tour de jeu

- **R 4.1** À son tour, un joueur doit attaquer s'il le peut. Il ne passe que s'il n'a aucune attaque possible (y compris avec l'aide de Fire, R 6.3).
- **R 4.2 Power.** Un dé capture un dé adverse dont la valeur est inférieure ou égale à la sienne.
- **R 4.3 Skill.** Un ou plusieurs dés capturent un dé adverse dont la valeur est exactement la somme des leurs.
- **R 4.4 Après l'attaque.** Les dés capturés passent dans la pile de score de l'attaquant ; les dés attaquants sont relancés.
- **R 4.5 Fin de manche.** La manche s'arrête quand un joueur n'a plus de dés, ou quand les deux joueurs passent l'un après l'autre.

## R 5 — Score

- **R 5.1** Un dé gardé vaut la moitié de sa taille ; un dé capturé vaut sa taille. Les compétences Poison (R 6.7), Null (R 6.6) et Value (R 6.15) modifient ce calcul.
- **R 5.2** Le joueur au score le plus haut gagne la manche. À égalité, la manche est nulle, sans départage.

## R 6 — Compétences

- **R 6.1 Berserk (B).** Ne participe pas aux attaques Skill. Fait des attaques Berserk : comme une attaque Speed (un dé capture plusieurs dés dont la somme égale exactement sa valeur). Après une attaque Berserk, le dé perd Berserk et devient un dé à la moitié de ses faces, arrondie au supérieur (un Twin divise chacune de ses moitiés).
- **R 6.2 Chance (c).** Si le joueur n'a pas l'initiative en début de manche, il peut relancer un de ses dés Chance. S'il prend ainsi l'initiative, l'adversaire peut relancer un de ses dés Chance, et ainsi de suite, jusqu'à ce qu'un joueur échoue à prendre l'initiative ou laisse l'autre commencer. Un dé Chance relancé peut attaquer au premier tour.
- **R 6.3 Fire (F).** Ne fait pas d'attaque Power. Peut aider une attaque Power ou Skill : avant l'attaque, on augmente la valeur de dés attaquants et on diminue d'autant la valeur de dés Fire qui ne participent pas à l'attaque. Aucun dé ne sort de sa plage de valeurs. Fire n'aide aucun autre type d'attaque.
- **R 6.4 Focus (f).** Si le joueur n'a pas l'initiative en début de manche, il peut baisser la valeur d'un ou plusieurs de ses dés Focus, à condition de prendre ainsi l'initiative. L'adversaire peut alors faire de même, et ainsi de suite. Si le joueur commence, ses dés Focus baissés ne peuvent pas servir à sa première attaque ; le second joueur n'a pas cette restriction.
- **R 6.5 Konstant (k).** Ne se relance pas après une attaque (ni après un Trip ou une relance Chance) : il garde sa valeur. Ne fait pas d'attaque Power, ne fait pas d'attaque Skill seul, mais dans une attaque Skill à plusieurs dés il peut ajouter ou soustraire sa valeur. Si une compétence change sa valeur sans le relancer (Focus, Fire), la nouvelle valeur reste.
- **R 6.6 Null (n).** Quand un dé Null participe à une attaque, les dés capturés valent zéro point. Un dé Null vaut lui-même zéro point.
- **R 6.7 Poison (p).** Un dé Poison gardé retire sa taille entière au score ; un dé Poison capturé retire la moitié de sa taille.
- **R 6.8 Queer (q).** Se comporte comme un dé normal quand il montre une valeur paire, et comme un dé Shadow (R 6.10) quand il montre une valeur impaire.
- **R 6.9 Speed (z).** Peut aussi faire des attaques Speed : un dé Speed capture un ou plusieurs dés dont la somme égale exactement sa valeur.
- **R 6.10 Shadow (s).** Ne fait pas d'attaque Power ; fait des attaques Shadow : il capture un dé dont la valeur est supérieure ou égale à la sienne et inférieure ou égale à sa taille.
- **R 6.11 Reserve (r).** Un dé Reserve n'est pas en jeu au départ. Chaque fois qu'un joueur perd une manche, il peut choisir un de ses dés Reserve, qui entre en jeu pour toutes les manches suivantes (et perd Reserve).
- **R 6.12 Stealth (d).** Ne fait que des attaques Skill à plusieurs dés (au moins deux dés). Ne peut être capturé que par une attaque Skill à plusieurs dés.
- **R 6.13 Stinger (g).** Dans une attaque Skill, compte pour n'importe quelle valeur entre sa valeur minimale et sa valeur actuelle. Ne compte pas pour l'initiative.
- **R 6.14 Trip (t).** Peut faire des attaques Trip : on choisit un dé adverse, on relance les deux dés ; si le dé Trip montre alors une valeur supérieure ou égale, la cible est capturée, sinon les deux dés gardent leurs nouvelles valeurs. Une attaque Trip sans aucune chance de capturer (taille du Trip inférieure à la valeur minimale de la cible) est illégale. Ne compte pas pour l'initiative.
- **R 6.15 Value (v).** Un dé Value compte comme si sa taille était égale à sa valeur actuelle. Quand un dé Value participe à une attaque, les dés capturés comptent pour la valeur qu'ils montraient au moment de la capture.

## R 7 — Interactions retenues

- **R 7.1** Null l'emporte sur Value et Poison : un dé capturé par une attaque avec un dé Null, ou un dé Null lui-même, vaut zéro.
- **R 7.2** Poison et Value se combinent : la base du calcul Poison est la valeur affichée.
- **R 7.3** Konstant et Stinger se combinent : le dé compte pour toute valeur de ±1 à ±sa valeur.
- **R 7.4** Un dé Focus baissé par le premier joueur reste inutilisable jusqu'à la fin de son premier tour.
```

- [ ] **Step 2: Commit**

```bash
git add docs/RULES.md
git commit -m "docs: règles de référence numérotées (R x.y)"
```

---
### Task 3: Compétences et dés (types)

**Files:**
- Create: `crates/engine/src/skill.rs`, `crates/engine/src/die.rs`
- Modify: `crates/engine/src/lib.rs`

**Interfaces:**
- Consumes: `DieId`, `SeededRng` (Tâche 1).
- Produces:
  - `DieSkill` (15 variantes) avec `ALL`, `letter() -> char`, `from_letter(char) -> Option<DieSkill>` ; `SkillSet` avec `has`, `insert`, `remove`, `iter`, `FromIterator<DieSkill>` ; `SkillInfo { skill, letter, name, rule, description }` et `skill_info(DieSkill) -> SkillInfo`.
  - `SwingLetter` (10 variantes) avec `ALL`, `range() -> RangeInclusive<u8>`, `letter()`, `from_letter()` ; `TwinPart { Fixed(u8), Swing(SwingLetter) }` ; `DieShape { Fixed(u8), Swing(SwingLetter), Option(u8, u8), Twin(TwinPart, TwinPart) }` ; `DieSpec { shape, skills }` avec `DieSpec::new(shape)`, `with(skill)`, `notation() -> String`.
  - `Sides { Single(u8), Twin(u8, u8) }` avec `max()`, `min()`, `roll(&mut SeededRng)`, `halved()`.
  - `Die { id, skills, sides, value, dizzy }` avec `max()`, `min()`, `has(DieSkill)`.

- [ ] **Step 1: Écrire les tests**

Créer `crates/engine/src/skill.rs` avec seulement les tests pour l'instant :
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn letters_round_trip_and_are_unique() {
        let mut letters: Vec<char> = DieSkill::ALL.iter().map(|s| s.letter()).collect();
        for s in DieSkill::ALL {
            assert_eq!(DieSkill::from_letter(s.letter()), Some(s));
        }
        letters.sort();
        letters.dedup();
        assert_eq!(letters.len(), DieSkill::ALL.len());
        assert_eq!(DieSkill::from_letter('o'), None); // Ornery : hors v1
    }

    #[test]
    fn skill_set_basics() {
        let mut set: SkillSet = [DieSkill::Poison, DieSkill::Shadow].into_iter().collect();
        assert!(set.has(DieSkill::Poison));
        set.remove(DieSkill::Poison);
        assert!(!set.has(DieSkill::Poison));
        set.insert(DieSkill::Berserk);
        assert_eq!(set.iter().collect::<Vec<_>>(), vec![DieSkill::Berserk, DieSkill::Shadow]);
    }

    #[test]
    fn every_skill_has_french_info_and_a_rule() {
        for s in DieSkill::ALL {
            let info = skill_info(s);
            assert_eq!(info.letter, s.letter());
            assert!(info.rule.starts_with("R 6."));
            assert!(!info.description.is_empty());
        }
    }
}
```

Créer `crates/engine/src/die.rs` avec seulement les tests :
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::skill::DieSkill;

    #[test]
    fn swing_ranges_match_buttonweavers() {
        assert_eq!(SwingLetter::X.range(), 4..=20);
        assert_eq!(SwingLetter::R.range(), 2..=16);
        assert_eq!(SwingLetter::P.range(), 1..=30);
        assert_eq!(SwingLetter::from_letter('Z'), Some(SwingLetter::Z));
        assert_eq!(SwingLetter::from_letter('Q'), None);
    }

    #[test]
    fn sides_bounds() {
        assert_eq!(Sides::Single(6).max(), 6);
        assert_eq!(Sides::Single(6).min(), 1);
        assert_eq!(Sides::Twin(4, 6).max(), 10);
        assert_eq!(Sides::Twin(4, 6).min(), 2);
    }

    #[test]
    fn halved_rounds_up_and_halves_twins_per_half() {
        assert_eq!(Sides::Single(7).halved(), Sides::Single(4));
        assert_eq!(Sides::Single(1).halved(), Sides::Single(1));
        assert_eq!(Sides::Twin(5, 4).halved(), Sides::Twin(3, 2));
    }

    #[test]
    fn twin_roll_is_sum_of_halves() {
        let mut rng = SeededRng::new(9);
        for _ in 0..500 {
            let v = Sides::Twin(1, 1).roll(&mut rng);
            assert_eq!(v, 2);
            let w = Sides::Twin(6, 6).roll(&mut rng);
            assert!((2..=12).contains(&w));
        }
    }

    #[test]
    fn notation_follows_recipe_syntax() {
        let spec = DieSpec::new(DieShape::Swing(SwingLetter::X))
            .with(DieSkill::Poison)
            .with(DieSkill::Speed);
        assert_eq!(spec.notation(), "pz(X)");
        assert_eq!(DieSpec::new(DieShape::Option(6, 10)).notation(), "(6/10)");
        assert_eq!(
            DieSpec::new(DieShape::Twin(TwinPart::Fixed(4), TwinPart::Swing(SwingLetter::Y))).notation(),
            "(4,Y)"
        );
    }
}
```

Dans `lib.rs`, ajouter `pub mod skill; pub mod die;`.

- [ ] **Step 2: Lancer les tests, vérifier qu'ils échouent**

Run: `cargo test -p buttonmen-engine`
Expected: échec de compilation (`DieSkill`, `SwingLetter`… introuvables).

- [ ] **Step 3: Implémenter `skill.rs`**

Au-dessus du module de tests :
```rust
use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

/// Die skills supported in v1 (R 6). The order is the display order of notations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub enum DieSkill {
    Berserk,
    Chance,
    Fire,
    Focus,
    Konstant,
    Null,
    Poison,
    Queer,
    Reserve,
    Shadow,
    Speed,
    Stealth,
    Stinger,
    Trip,
    Value,
}

impl DieSkill {
    pub const ALL: [DieSkill; 15] = [
        DieSkill::Berserk,
        DieSkill::Chance,
        DieSkill::Fire,
        DieSkill::Focus,
        DieSkill::Konstant,
        DieSkill::Null,
        DieSkill::Poison,
        DieSkill::Queer,
        DieSkill::Reserve,
        DieSkill::Shadow,
        DieSkill::Speed,
        DieSkill::Stealth,
        DieSkill::Stinger,
        DieSkill::Trip,
        DieSkill::Value,
    ];

    /// R 1.1: recipe letter, as in buttonweavers' skill abbreviation table.
    pub fn letter(self) -> char {
        match self {
            DieSkill::Berserk => 'B',
            DieSkill::Chance => 'c',
            DieSkill::Fire => 'F',
            DieSkill::Focus => 'f',
            DieSkill::Konstant => 'k',
            DieSkill::Null => 'n',
            DieSkill::Poison => 'p',
            DieSkill::Queer => 'q',
            DieSkill::Reserve => 'r',
            DieSkill::Shadow => 's',
            DieSkill::Speed => 'z',
            DieSkill::Stealth => 'd',
            DieSkill::Stinger => 'g',
            DieSkill::Trip => 't',
            DieSkill::Value => 'v',
        }
    }

    pub fn from_letter(c: char) -> Option<DieSkill> {
        DieSkill::ALL.into_iter().find(|s| s.letter() == c)
    }
}

/// The skills of one die.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct SkillSet(BTreeSet<DieSkill>);

impl SkillSet {
    pub fn has(&self, skill: DieSkill) -> bool {
        self.0.contains(&skill)
    }

    pub fn insert(&mut self, skill: DieSkill) {
        self.0.insert(skill);
    }

    pub fn remove(&mut self, skill: DieSkill) {
        self.0.remove(&skill);
    }

    pub fn iter(&self) -> impl Iterator<Item = DieSkill> + '_ {
        self.0.iter().copied()
    }
}

impl FromIterator<DieSkill> for SkillSet {
    fn from_iter<I: IntoIterator<Item = DieSkill>>(iter: I) -> Self {
        SkillSet(iter.into_iter().collect())
    }
}

/// Display information for a skill, shown by the client as a tooltip.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct SkillInfo {
    pub skill: DieSkill,
    pub letter: char,
    pub name: String,
    pub rule: String,
    pub description: String,
}

pub fn skill_info(skill: DieSkill) -> SkillInfo {
    let (name, rule, description) = match skill {
        DieSkill::Berserk => (
            "Berserk",
            "R 6.1",
            "Pas d'attaque Skill. Attaque Berserk : capture des dés dont la somme égale sa valeur, puis perd Berserk et passe à la moitié de ses faces.",
        ),
        DieSkill::Chance => (
            "Chance",
            "R 6.2",
            "Sans l'initiative en début de manche, peut relancer un dé Chance pour tenter de la prendre.",
        ),
        DieSkill::Fire => (
            "Fire",
            "R 6.3",
            "Pas d'attaque Power. Peut baisser sa valeur pour augmenter d'autant des dés qui font une attaque Power ou Skill.",
        ),
        DieSkill::Focus => (
            "Focus",
            "R 6.4",
            "Sans l'initiative en début de manche, peut baisser sa valeur pour la prendre ; il ne peut alors pas servir à la première attaque.",
        ),
        DieSkill::Konstant => (
            "Konstant",
            "R 6.5",
            "Ne se relance jamais. Pas d'attaque Power ni Skill seul ; dans une attaque Skill à plusieurs dés, ajoute ou soustrait sa valeur.",
        ),
        DieSkill::Null => (
            "Null",
            "R 6.6",
            "Vaut zéro point, et les dés qu'il aide à capturer valent zéro point.",
        ),
        DieSkill::Poison => (
            "Poison",
            "R 6.7",
            "Points négatifs : gardé, retire sa taille ; capturé, retire la moitié de sa taille.",
        ),
        DieSkill::Queer => (
            "Queer",
            "R 6.8",
            "Normal sur une valeur paire, Shadow sur une valeur impaire.",
        ),
        DieSkill::Reserve => (
            "Reserve",
            "R 6.11",
            "Hors jeu au départ ; après une manche perdue, on peut le faire entrer en jeu pour la suite du match.",
        ),
        DieSkill::Shadow => (
            "Shadow",
            "R 6.10",
            "Pas d'attaque Power. Capture un dé de valeur supérieure ou égale à la sienne, sans dépasser sa taille.",
        ),
        DieSkill::Speed => (
            "Speed",
            "R 6.9",
            "Peut capturer plusieurs dés dont la somme égale exactement sa valeur.",
        ),
        DieSkill::Stealth => (
            "Stealth",
            "R 6.12",
            "N'attaque et ne peut être capturé que par des attaques Skill d'au moins deux dés.",
        ),
        DieSkill::Stinger => (
            "Stinger",
            "R 6.13",
            "En attaque Skill, compte pour toute valeur de son minimum à sa valeur actuelle. Ignoré pour l'initiative.",
        ),
        DieSkill::Trip => (
            "Trip",
            "R 6.14",
            "Attaque Trip : relance les deux dés, capture si le Trip fait au moins autant. Ignoré pour l'initiative.",
        ),
        DieSkill::Value => (
            "Value",
            "R 6.15",
            "Compte pour sa valeur au lieu de sa taille ; les dés qu'il capture comptent pour leur valeur au moment de la capture.",
        ),
    };
    SkillInfo {
        skill,
        letter: skill.letter(),
        name: name.to_string(),
        rule: rule.to_string(),
        description: description.to_string(),
    }
}
```

- [ ] **Step 4: Implémenter `die.rs`**

Au-dessus du module de tests :
```rust
use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};

use crate::ids::DieId;
use crate::rng::SeededRng;
use crate::skill::{DieSkill, SkillSet};

/// R 1.2: swing letters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub enum SwingLetter {
    P,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
}

impl SwingLetter {
    pub const ALL: [SwingLetter; 10] = [
        SwingLetter::P,
        SwingLetter::R,
        SwingLetter::S,
        SwingLetter::T,
        SwingLetter::U,
        SwingLetter::V,
        SwingLetter::W,
        SwingLetter::X,
        SwingLetter::Y,
        SwingLetter::Z,
    ];

    /// R 1.2: allowed sizes, from buttonweavers' `BMDieSwing::$swingRanges`.
    pub fn range(self) -> RangeInclusive<u8> {
        match self {
            SwingLetter::P => 1..=30,
            SwingLetter::R => 2..=16,
            SwingLetter::S => 6..=20,
            SwingLetter::T => 2..=12,
            SwingLetter::U => 8..=30,
            SwingLetter::V => 6..=12,
            SwingLetter::W => 4..=12,
            SwingLetter::X => 4..=20,
            SwingLetter::Y => 1..=20,
            SwingLetter::Z => 4..=30,
        }
    }

    pub fn letter(self) -> char {
        match self {
            SwingLetter::P => 'P',
            SwingLetter::R => 'R',
            SwingLetter::S => 'S',
            SwingLetter::T => 'T',
            SwingLetter::U => 'U',
            SwingLetter::V => 'V',
            SwingLetter::W => 'W',
            SwingLetter::X => 'X',
            SwingLetter::Y => 'Y',
            SwingLetter::Z => 'Z',
        }
    }

    pub fn from_letter(c: char) -> Option<SwingLetter> {
        SwingLetter::ALL.into_iter().find(|l| l.letter() == c)
    }
}

/// R 1.4: one half of a twin die.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub enum TwinPart {
    Fixed(u8),
    Swing(SwingLetter),
}

/// The shape of a die as written in a recipe (R 1.1–1.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub enum DieShape {
    Fixed(u8),
    Swing(SwingLetter),
    Option(u8, u8),
    Twin(TwinPart, TwinPart),
}

/// One die of a recipe: shape + skills.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct DieSpec {
    pub shape: DieShape,
    pub skills: SkillSet,
}

impl DieSpec {
    pub fn new(shape: DieShape) -> Self {
        Self {
            shape,
            skills: SkillSet::default(),
        }
    }

    pub fn with(mut self, skill: DieSkill) -> Self {
        self.skills.insert(skill);
        self
    }

    /// R 1.1: the recipe notation of this die, skills first.
    pub fn notation(&self) -> String {
        let skills: String = self.skills.iter().map(DieSkill::letter).collect();
        let part = |p: &TwinPart| match p {
            TwinPart::Fixed(n) => n.to_string(),
            TwinPart::Swing(l) => l.letter().to_string(),
        };
        let core = match &self.shape {
            DieShape::Fixed(n) => n.to_string(),
            DieShape::Swing(l) => l.letter().to_string(),
            DieShape::Option(a, b) => format!("{a}/{b}"),
            DieShape::Twin(a, b) => format!("{},{}", part(a), part(b)),
        };
        format!("{skills}({core})")
    }
}

/// The resolved number of sides of a die in play.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub enum Sides {
    Single(u8),
    Twin(u8, u8),
}

impl Sides {
    /// R 1.4–1.5: the highest value (the size).
    pub fn max(self) -> u8 {
        match self {
            Sides::Single(n) => n,
            Sides::Twin(a, b) => a + b,
        }
    }

    /// R 1.4–1.5: the lowest value.
    pub fn min(self) -> u8 {
        match self {
            Sides::Single(_) => 1,
            Sides::Twin(_, _) => 2,
        }
    }

    pub fn roll(self, rng: &mut SeededRng) -> u8 {
        match self {
            Sides::Single(n) => rng.roll(n),
            Sides::Twin(a, b) => rng.roll(a) + rng.roll(b),
        }
    }

    /// R 6.1: half the sides, rounded up; a twin halves each of its halves.
    pub fn halved(self) -> Sides {
        match self {
            Sides::Single(n) => Sides::Single(n.div_ceil(2)),
            Sides::Twin(a, b) => Sides::Twin(a.div_ceil(2), b.div_ceil(2)),
        }
    }
}

/// A die in play (or captured) during a round.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Die {
    pub id: DieId,
    pub skills: SkillSet,
    pub sides: Sides,
    pub value: u8,
    /// R 6.4 / R 7.4: a Focus die turned down by the starting player cannot attack
    /// until the end of that player's first turn.
    pub dizzy: bool,
}

impl Die {
    pub fn max(&self) -> u8 {
        self.sides.max()
    }

    pub fn min(&self) -> u8 {
        self.sides.min()
    }

    pub fn has(&self, skill: DieSkill) -> bool {
        self.skills.has(skill)
    }
}
```

Mettre `lib.rs` à jour :
```rust
//! Button Men rules engine. Rule references (`R x.y`) point to `docs/RULES.md`.

pub mod die;
pub mod ids;
pub mod rng;
pub mod skill;

pub use die::{Die, DieShape, DieSpec, Sides, SwingLetter, TwinPart};
pub use ids::{DieId, PlayerId};
pub use rng::SeededRng;
pub use skill::{DieSkill, SkillInfo, SkillSet, skill_info};
```

- [ ] **Step 5: Lancer les tests, vérifier qu'ils passent**

Run: `cargo test -p buttonmen-engine && cargo clippy -p buttonmen-engine -- -D warnings`
Expected: tous les tests PASS.

- [ ] **Step 6: Commit**

```bash
git add crates/engine
git commit -m "feat(engine): types des compétences et des dés"
```

---

### Task 4: Parseur de recettes (`crates/button-data`)

**Files:**
- Create: `crates/button-data/Cargo.toml`, `crates/button-data/src/lib.rs`, `crates/button-data/src/recipe.rs`
- Modify: `Cargo.toml` (membre du workspace)

**Interfaces:**
- Consumes: `DieSpec`, `DieShape`, `TwinPart`, `SwingLetter`, `DieSkill` (Tâche 3).
- Produces: `parse_recipe(&str) -> Result<Vec<DieSpec>, RecipeError>` ; `RecipeError { Syntax { rest: String }, UnsupportedSkill { letter: char }, UnsupportedDie { core: String }, InvalidSize { size: u8 }, Empty }` (implémente `Display` en français et `Serialize`).

- [ ] **Step 1: Déclarer la crate**

Dans `Cargo.toml` du workspace : `members = ["crates/engine", "crates/button-data"]`.

`crates/button-data/Cargo.toml` :
```toml
[package]
name = "buttonmen-data"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
buttonmen-engine = { workspace = true }
nom = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
thiserror = { workspace = true }
toml = { workspace = true }

[dev-dependencies]
proptest = { workspace = true }
```

`crates/button-data/src/lib.rs` :
```rust
//! Buttons: recipe parsing (R 1.1) and the button catalog.

pub mod recipe;

pub use recipe::{RecipeError, parse_recipe};
```

- [ ] **Step 2: Écrire les tests**

`crates/button-data/src/recipe.rs`, tests seulement :
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use buttonmen_engine::{DieShape, DieSkill, SwingLetter, TwinPart};
    use proptest::prelude::*;

    #[test]
    fn parses_plain_and_swing_dice() {
        let dice = parse_recipe("(4) (4) (10) (12) (X)").unwrap();
        assert_eq!(dice.len(), 5);
        assert_eq!(dice[0].shape, DieShape::Fixed(4));
        assert_eq!(dice[4].shape, DieShape::Swing(SwingLetter::X));
    }

    #[test]
    fn parses_option_and_twin() {
        let dice = parse_recipe("(6/10) (4,4) (X,X) (8,Y)").unwrap();
        assert_eq!(dice[0].shape, DieShape::Option(6, 10));
        assert_eq!(dice[1].shape, DieShape::Twin(TwinPart::Fixed(4), TwinPart::Fixed(4)));
        assert_eq!(
            dice[2].shape,
            DieShape::Twin(TwinPart::Swing(SwingLetter::X), TwinPart::Swing(SwingLetter::X))
        );
        assert_eq!(
            dice[3].shape,
            DieShape::Twin(TwinPart::Fixed(8), TwinPart::Swing(SwingLetter::Y))
        );
    }

    #[test]
    fn parses_prefix_and_suffix_skills() {
        let dice = parse_recipe("ps(10) (6)z  Bk(X)").unwrap();
        assert!(dice[0].skills.has(DieSkill::Poison) && dice[0].skills.has(DieSkill::Shadow));
        assert!(dice[1].skills.has(DieSkill::Speed));
        assert!(dice[2].skills.has(DieSkill::Berserk) && dice[2].skills.has(DieSkill::Konstant));
    }

    #[test]
    fn rejects_unsupported_skills_and_dice() {
        assert_eq!(parse_recipe("o(6)"), Err(RecipeError::UnsupportedSkill { letter: 'o' }));
        assert_eq!(parse_recipe("+(6)"), Err(RecipeError::UnsupportedSkill { letter: '+' }));
        assert_eq!(parse_recipe("(C)"), Err(RecipeError::UnsupportedDie { core: "C".into() }));
        assert_eq!(parse_recipe("(0)"), Err(RecipeError::InvalidSize { size: 0 }));
        assert_eq!(parse_recipe("(200,200)"), Err(RecipeError::InvalidSize { size: 200 }));
        assert_eq!(parse_recipe("   "), Err(RecipeError::Empty));
        assert!(matches!(parse_recipe("(6"), Err(RecipeError::Syntax { .. })));
        assert!(matches!(parse_recipe("(1/2/3)"), Err(RecipeError::Syntax { .. })));
    }

    #[test]
    fn notation_round_trips() {
        let text = "p(4) s(10) (X) (6/10) z(4,4) (X,X)";
        let dice = parse_recipe(text).unwrap();
        let back: Vec<String> = dice.iter().map(|d| d.notation()).collect();
        assert_eq!(back.join(" "), text);
    }

    proptest! {
        // Review Focus 3: exotic buttonweavers recipes never panic.
        #[test]
        fn parser_never_panics_on_arbitrary_input(s in "[ -~]{0,40}") {
            let _ = parse_recipe(&s);
        }

        #[test]
        fn parser_never_panics_on_recipe_like_input(s in "([a-zA-Z+?&%#`^!]{0,3}\\([0-9A-Z/,]{0,5}\\)[a-z!?]{0,2} ?){0,6}") {
            let _ = parse_recipe(&s);
        }
    }
}
```

- [ ] **Step 3: Lancer les tests, vérifier qu'ils échouent**

Run: `cargo test -p buttonmen-data`
Expected: échec de compilation (`parse_recipe` introuvable).

- [ ] **Step 4: Implémenter le parseur**

Au-dessus des tests dans `recipe.rs` :
```rust
use buttonmen_engine::{DieShape, DieSkill, DieSpec, SwingLetter, TwinPart};
use nom::branch::alt;
use nom::character::complete::{char, digit1, multispace0, multispace1, satisfy};
use nom::combinator::{all_consuming, map, map_res};
use nom::multi::{many0, separated_list1};
use nom::sequence::{delimited, separated_pair};
use nom::{IResult, Parser};
use serde::Serialize;

/// Why a recipe cannot be played with the v1 rules.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error, Serialize)]
#[serde(tag = "type")]
pub enum RecipeError {
    #[error("recette illisible près de « {rest} »")]
    Syntax { rest: String },
    #[error("compétence non prise en charge : « {letter} »")]
    UnsupportedSkill { letter: char },
    #[error("type de dé non pris en charge : « ({core}) »")]
    UnsupportedDie { core: String },
    #[error("taille de dé invalide : {size}")]
    InvalidSize { size: u8 },
    #[error("recette vide")]
    Empty,
}

/// Syntax tree of one die before validation: unknown letters are kept so that the
/// error can name them.
enum RawPart {
    Number(u8),
    Letter(char),
}

enum RawCore {
    Single(RawPart),
    Option(u8, u8),
    Twin(RawPart, RawPart),
}

struct RawDie {
    prefix: Vec<char>,
    core: RawCore,
    suffix: Vec<char>,
}

fn number(i: &str) -> IResult<&str, u8> {
    map_res(digit1, str::parse::<u8>).parse(i)
}

fn part(i: &str) -> IResult<&str, RawPart> {
    alt((
        map(number, RawPart::Number),
        map(satisfy(|c| c.is_ascii_alphabetic()), RawPart::Letter),
    ))
    .parse(i)
}

fn core(i: &str) -> IResult<&str, RawCore> {
    alt((
        map(separated_pair(number, char('/'), number), |(a, b)| RawCore::Option(a, b)),
        map(separated_pair(part, char(','), part), |(a, b)| RawCore::Twin(a, b)),
        map(part, RawCore::Single),
    ))
    .parse(i)
}

fn skill_char(i: &str) -> IResult<&str, char> {
    satisfy(|c| !c.is_whitespace() && c != '(' && c != ')').parse(i)
}

fn die(i: &str) -> IResult<&str, RawDie> {
    map(
        (many0(skill_char), delimited(char('('), core, char(')')), many0(skill_char)),
        |(prefix, core, suffix)| RawDie { prefix, core, suffix },
    )
    .parse(i)
}

fn recipe(i: &str) -> IResult<&str, Vec<RawDie>> {
    all_consuming(delimited(multispace0, separated_list1(multispace1, die), multispace0)).parse(i)
}

/// R 1.1: parses a button recipe such as `p(4) s(10) (X) (6/10) z(4,4)`.
pub fn parse_recipe(text: &str) -> Result<Vec<DieSpec>, RecipeError> {
    if text.trim().is_empty() {
        return Err(RecipeError::Empty);
    }
    let (_, raw) = recipe(text).map_err(|e| RecipeError::Syntax {
        rest: match e {
            nom::Err::Error(e) | nom::Err::Failure(e) => e.input.chars().take(12).collect(),
            nom::Err::Incomplete(_) => String::new(),
        },
    })?;
    raw.into_iter().map(validate).collect()
}

fn validate(raw: RawDie) -> Result<DieSpec, RecipeError> {
    let shape = match raw.core {
        RawCore::Single(RawPart::Number(n)) => DieShape::Fixed(size(n)?),
        RawCore::Single(RawPart::Letter(c)) => DieShape::Swing(swing(c)?),
        RawCore::Option(a, b) => DieShape::Option(size(a)?, size(b)?),
        RawCore::Twin(a, b) => DieShape::Twin(twin_part(a)?, twin_part(b)?),
    };
    let mut spec = DieSpec::new(shape);
    for c in raw.prefix.into_iter().chain(raw.suffix) {
        let skill = DieSkill::from_letter(c).ok_or(RecipeError::UnsupportedSkill { letter: c })?;
        spec.skills.insert(skill);
    }
    Ok(spec)
}

/// R 1.5: a die has 1 to 99 sides (a twin of two 99s, 198, still fits in a `u8`).
fn size(n: u8) -> Result<u8, RecipeError> {
    if (1..=99).contains(&n) { Ok(n) } else { Err(RecipeError::InvalidSize { size: n }) }
}

fn swing(c: char) -> Result<SwingLetter, RecipeError> {
    SwingLetter::from_letter(c).ok_or(RecipeError::UnsupportedDie { core: c.to_string() })
}

fn twin_part(p: RawPart) -> Result<TwinPart, RecipeError> {
    match p {
        RawPart::Number(n) => Ok(TwinPart::Fixed(size(n)?)),
        RawPart::Letter(c) => Ok(TwinPart::Swing(swing(c)?)),
    }
}
```

Note : la borne de 99 faces garantit que `Sides::max()` d'un Twin (au plus 198) ne déborde jamais d'un `u8`.

- [ ] **Step 5: Lancer les tests, vérifier qu'ils passent**

Run: `cargo test -p buttonmen-data && cargo clippy --workspace -- -D warnings`
Expected: PASS (dont les deux propriétés proptest).

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml crates/button-data
git commit -m "feat(data): parseur de recettes de boutons (nom)"
```

---
### Task 5: État, actions, mise en place d'une manche et initiative de base

**Files:**
- Create: `crates/engine/src/state.rs`, `action.rs`, `event.rs`, `error.rs`, `hooks.rs`, `setup.rs`, `initiative.rs`, `round.rs`, `apply.rs`, `testing.rs`
- Create: `crates/engine/tests/common/mod.rs`, `crates/engine/tests/setup.rs`
- Modify: `crates/engine/src/lib.rs`, `crates/engine/Cargo.toml`

**Interfaces:**
- Consumes: Tâches 1 et 3.
- Produces (utilisé par toutes les tâches suivantes) :
  - `Button { name: String, dice: Vec<DieSpec> }`, `MatchConfig { max_wins: u8 }` (défaut 3), `DieSlot { id, spec, in_play }`, `CaptureScoring { BySize, ByValue(u8), Zero }`, `CapturedDie { die, scoring }`, `Record { wins, losses, draws }`, `PlayerState { button, slots, choice, pending_choice, active, captured, record }` avec `die(id)`, `die_mut(id)`, `slot(id)`.
  - `WaitingFor { SpecifyDice { pending }, ChooseReserve { player }, ReactToInitiative { player }, Attack { player }, AdjustFire { player, attack }, GameOver { result } }`, `MatchResult { winner, reason }`, `EndReason { Rounds, Surrender }`.
  - `GameState { config, players, round, initiative, consecutive_passes, last_round_loser, waiting_for, rng }` avec `player(p)`, `player_mut(p)`, `actors() -> Vec<PlayerId>`.
  - `SpecChoice { swing: Vec<(SwingLetter, u8)>, options: Vec<(DieId, u8)> }` avec `swing_size(l)`, `option_size(id)` ; `AttackKind { Power, Skill, Shadow, Speed, Trip, Berserk }` ; `DeclaredAttack { kind, attackers, targets }` ; `Action` (10 variantes, voir code).
  - `GameEvent` (voir code), `RuleError`, `AttackError`, `ReactionError`, `FireError`.
  - `new_game([Button; 2], seed: u64, MatchConfig) -> GameState` ; `apply(&GameState, PlayerId, Action) -> Result<(GameState, Vec<GameEvent>), RuleError>`.
  - `pub(crate)` : `setup::{begin_round, ask_for_choices, start_round, required_letters, option_slots}`, `initiative::{initiative_values, compare, determine, start_attacks}`, `round::surrender`, `hooks::counts_for_initiative`.
  - `testing::{fixed, button, die, twin, Placed, attack_state, id}` (feature `testing`).

- [ ] **Step 1: Activer `testing` pour les tests d'intégration**

Dans `crates/engine/Cargo.toml`, section `[dev-dependencies]`, ajouter :
```toml
buttonmen-engine = { path = ".", features = ["testing"] }
```

- [ ] **Step 2: Écrire les types**

`crates/engine/src/action.rs` :
```rust
use serde::{Deserialize, Serialize};

use crate::die::SwingLetter;
use crate::ids::DieId;

/// R 1.2–1.3: the sizes chosen for swing letters and option dice.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct SpecChoice {
    pub swing: Vec<(SwingLetter, u8)>,
    pub options: Vec<(DieId, u8)>,
}

impl SpecChoice {
    pub fn swing_size(&self, letter: SwingLetter) -> Option<u8> {
        self.swing.iter().find(|(l, _)| *l == letter).map(|(_, s)| *s)
    }

    pub fn option_size(&self, die: DieId) -> Option<u8> {
        self.options.iter().find(|(d, _)| *d == die).map(|(_, s)| *s)
    }
}

/// R 4.2, R 4.3, R 6.1, R 6.9, R 6.10, R 6.14.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub enum AttackKind {
    Power,
    Skill,
    Shadow,
    Speed,
    Trip,
    Berserk,
}

impl AttackKind {
    pub const ALL: [AttackKind; 6] = [
        AttackKind::Power,
        AttackKind::Skill,
        AttackKind::Shadow,
        AttackKind::Speed,
        AttackKind::Trip,
        AttackKind::Berserk,
    ];
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct DeclaredAttack {
    pub kind: AttackKind,
    pub attackers: Vec<DieId>,
    pub targets: Vec<DieId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[serde(tag = "type")]
pub enum Action {
    /// R 3.1 / R 2.3.
    SpecifyDice { choice: SpecChoice },
    /// R 6.11: `None` declines.
    ChooseReserve { die: Option<DieId> },
    /// R 6.4: the new (lower) values.
    FocusTurnDown { dice: Vec<(DieId, u8)> },
    /// R 6.2.
    ChanceReroll { die: DieId },
    /// R 3.5: let the opponent start.
    PassInitiative,
    Attack { attack: DeclaredAttack },
    /// R 6.3: `fire` = amounts taken from Fire dice, `boosts` = amounts added to attackers.
    FireTurnDown { fire: Vec<(DieId, u8)>, boosts: Vec<(DieId, u8)> },
    CancelFire,
    /// R 4.1.
    Pass,
    /// R 2.4.
    Surrender,
}
```

`crates/engine/src/state.rs` :
```rust
use serde::{Deserialize, Serialize};

use crate::action::{DeclaredAttack, SpecChoice};
use crate::die::{Die, DieSpec};
use crate::ids::{DieId, PlayerId};
use crate::rng::SeededRng;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Button {
    pub name: String,
    pub dice: Vec<DieSpec>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct MatchConfig {
    pub max_wins: u8,
}

impl Default for MatchConfig {
    /// R 2.2: three round wins by default.
    fn default() -> Self {
        Self { max_wins: 3 }
    }
}

/// A die of the button, with its stable id. Reserve dice are out of play until chosen (R 6.11).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DieSlot {
    pub id: DieId,
    pub spec: DieSpec,
    pub in_play: bool,
}

/// How a captured die scores (R 5.1, R 6.6, R 6.15).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub enum CaptureScoring {
    BySize,
    ByValue(u8),
    Zero,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapturedDie {
    pub die: Die,
    pub scoring: CaptureScoring,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct Record {
    pub wins: u8,
    pub losses: u8,
    pub draws: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerState {
    pub button: String,
    pub slots: Vec<DieSlot>,
    /// Committed swing/option sizes (public once revealed).
    pub choice: SpecChoice,
    /// R 3.1: submitted during SpecifyDice, hidden until every player has chosen.
    pub pending_choice: Option<SpecChoice>,
    pub active: Vec<Die>,
    /// Dice captured from the opponent this round.
    pub captured: Vec<CapturedDie>,
    pub record: Record,
}

impl PlayerState {
    pub fn die(&self, id: DieId) -> Option<&Die> {
        self.active.iter().find(|d| d.id == id)
    }

    pub fn die_mut(&mut self, id: DieId) -> Option<&mut Die> {
        self.active.iter_mut().find(|d| d.id == id)
    }

    pub fn slot(&self, id: DieId) -> Option<&DieSlot> {
        self.slots.iter().find(|s| s.id == id)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub enum EndReason {
    Rounds,
    Surrender,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct MatchResult {
    pub winner: PlayerId,
    pub reason: EndReason,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[serde(tag = "type")]
pub enum WaitingFor {
    SpecifyDice { pending: Vec<PlayerId> },
    ChooseReserve { player: PlayerId },
    ReactToInitiative { player: PlayerId },
    Attack { player: PlayerId },
    AdjustFire { player: PlayerId, attack: DeclaredAttack },
    GameOver { result: MatchResult },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameState {
    pub config: MatchConfig,
    pub players: [PlayerState; 2],
    /// 1-based.
    pub round: u8,
    pub initiative: Option<PlayerId>,
    /// R 4.5: two consecutive passes end the round.
    pub consecutive_passes: u8,
    /// R 2.3: `None` in round 1 and after a draw.
    pub last_round_loser: Option<PlayerId>,
    pub waiting_for: WaitingFor,
    pub rng: SeededRng,
}

impl GameState {
    pub fn player(&self, p: PlayerId) -> &PlayerState {
        &self.players[p.index()]
    }

    pub fn player_mut(&mut self, p: PlayerId) -> &mut PlayerState {
        &mut self.players[p.index()]
    }

    /// The players who must act now (both during a simultaneous SpecifyDice).
    pub fn actors(&self) -> Vec<PlayerId> {
        match &self.waiting_for {
            WaitingFor::SpecifyDice { pending } => pending.clone(),
            WaitingFor::ChooseReserve { player }
            | WaitingFor::ReactToInitiative { player }
            | WaitingFor::Attack { player }
            | WaitingFor::AdjustFire { player, .. } => vec![*player],
            WaitingFor::GameOver { .. } => Vec::new(),
        }
    }
}
```

`crates/engine/src/event.rs` :
```rust
use serde::{Deserialize, Serialize};

use crate::action::{DeclaredAttack, SpecChoice};
use crate::die::Sides;
use crate::ids::{DieId, PlayerId};
use crate::state::MatchResult;

/// What happened during one `apply`, in order. Used for animations and the action log.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[serde(tag = "type")]
pub enum GameEvent {
    DiceSpecified { player: PlayerId, choice: SpecChoice },
    ReserveAdded { player: PlayerId, die: DieId },
    RoundStarted { round: u8 },
    DiceRolled { player: PlayerId, dice: Vec<(DieId, u8)> },
    InitiativeDecided { player: PlayerId, by_coin: bool },
    FocusTurnedDown { player: PlayerId, dice: Vec<(DieId, u8)> },
    ChanceRerolled { player: PlayerId, die: DieId, value: u8 },
    InitiativePassed { player: PlayerId },
    Attacked { player: PlayerId, attack: DeclaredAttack },
    FireUsed { player: PlayerId, fire: Vec<(DieId, u8)>, boosts: Vec<(DieId, u8)> },
    TripRolled { attacker: DieId, attacker_value: u8, target: DieId, target_value: u8, captured: bool },
    DieCaptured { by: PlayerId, die: DieId },
    DieRerolled { die: DieId, value: u8 },
    DieShrunk { die: DieId, sides: Sides },
    Passed { player: PlayerId },
    /// Scores in half-points.
    RoundEnded { round: u8, scores: [i32; 2], winner: Option<PlayerId> },
    MatchEnded { result: MatchResult },
}
```

`crates/engine/src/error.rs` :
```rust
use serde::{Deserialize, Serialize};

use crate::action::AttackKind;
use crate::die::SwingLetter;
use crate::ids::DieId;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[serde(tag = "type")]
pub enum RuleError {
    #[error("ce n'est pas à ce joueur d'agir")]
    NotYourTurn,
    #[error("cette action n'est pas possible maintenant")]
    WrongPhase,
    #[error("la partie est terminée")]
    GameOver,
    #[error("dé inconnu ou indisponible")]
    UnknownDie { die: DieId },
    #[error("taille Swing manquante, en double ou inutile pour {letter:?}")]
    SwingMismatch { letter: SwingLetter },
    #[error("taille {size} hors de la plage de la Swing {letter:?}")]
    SwingOutOfRange { letter: SwingLetter, size: u8 },
    #[error("choix d'Option manquant ou invalide")]
    InvalidOption { die: DieId },
    #[error("ce dé ne peut pas entrer en jeu depuis la réserve")]
    InvalidReserve { die: DieId },
    #[error("réaction d'initiative invalide : {reason}")]
    InvalidReaction { reason: ReactionError },
    #[error("attaque illégale : {reason}")]
    IllegalAttack { reason: AttackError },
    #[error("aide Fire invalide : {reason}")]
    InvalidFire { reason: FireError },
    #[error("passer est interdit : une attaque est possible")]
    PassNotAllowed,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[serde(tag = "type")]
pub enum AttackError {
    #[error("mauvais nombre de dés pour ce type d'attaque")]
    WrongDiceCount,
    #[error("ce dé n'est pas à vous")]
    NotYourDie { die: DieId },
    #[error("ce dé n'est pas à l'adversaire")]
    NotOpponentDie { die: DieId },
    #[error("un dé est cité deux fois")]
    DuplicateDie { die: DieId },
    #[error("ce dé ne peut pas faire ce type d'attaque")]
    KindUnavailable { die: DieId, kind: AttackKind },
    #[error("cette cible est protégée contre ce type d'attaque")]
    TargetProtected { die: DieId },
    #[error("ce dé Focus a été baissé et ne peut pas encore attaquer")]
    Dizzy { die: DieId },
    #[error("les valeurs ne permettent pas cette attaque")]
    ValuesDoNotMatch,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[serde(tag = "type")]
pub enum ReactionError {
    #[error("aucun dé choisi")]
    NoDice,
    #[error("ce dé n'est pas un dé Focus")]
    NotFocus { die: DieId },
    #[error("ce dé n'est pas un dé Chance")]
    NotChance { die: DieId },
    #[error("la nouvelle valeur doit être plus basse et dans la plage du dé")]
    ValueNotLowered { die: DieId },
    #[error("ce changement ne donne pas l'initiative")]
    DoesNotGainInitiative,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[serde(tag = "type")]
pub enum FireError {
    #[error("ce dé n'est pas un dé Fire disponible")]
    NotFire { die: DieId },
    #[error("seuls les dés attaquants peuvent être augmentés")]
    NotAttacker { die: DieId },
    #[error("le total retiré doit égaler le total ajouté")]
    AmountsDiffer,
    #[error("un dé sortirait de sa plage de valeurs")]
    OutOfRange { die: DieId },
    #[error("l'attaque reste impossible après cet ajustement")]
    AttackStillInvalid,
}
```

- [ ] **Step 3: Écrire le constructeur de scénarios**

`crates/engine/src/testing.rs` :
```rust
//! Scenario builders for tests and for other crates' tests (feature "testing").

use crate::die::{Die, DieShape, DieSpec, Sides, TwinPart};
use crate::ids::{DieId, PlayerId};
use crate::setup::new_game;
use crate::skill::DieSkill;
use crate::state::{Button, GameState, MatchConfig, WaitingFor};

pub fn fixed(n: u8) -> DieSpec {
    DieSpec::new(DieShape::Fixed(n))
}

pub fn button(name: &str, dice: Vec<DieSpec>) -> Button {
    Button {
        name: name.to_string(),
        dice,
    }
}

/// A die of a hand-made scenario, with a forced value.
#[derive(Debug, Clone)]
pub struct Placed {
    pub spec: DieSpec,
    pub value: u8,
}

impl Placed {
    pub fn with(mut self, skill: DieSkill) -> Self {
        self.spec.skills.insert(skill);
        self
    }
}

/// An `n`-sided die showing `value`.
pub fn die(n: u8, value: u8) -> Placed {
    Placed { spec: fixed(n), value }
}

/// An `(a,b)` twin die showing `value`.
pub fn twin(a: u8, b: u8, value: u8) -> Placed {
    Placed {
        spec: DieSpec::new(DieShape::Twin(TwinPart::Fixed(a), TwinPart::Fixed(b))),
        value,
    }
}

fn sides_of(spec: &DieSpec) -> Sides {
    match spec.shape {
        DieShape::Fixed(n) => Sides::Single(n),
        DieShape::Twin(TwinPart::Fixed(a), TwinPart::Fixed(b)) => Sides::Twin(a, b),
        DieShape::Swing(_) | DieShape::Option(..) | DieShape::Twin(..) => {
            panic!("scenario dice must have fixed sizes")
        }
    }
}

/// A round in the attack phase, `to_move` to play, with exactly these dice and values.
/// Die ids: player One's dice are 0.., player Two's follow.
pub fn attack_state(one: Vec<Placed>, two: Vec<Placed>, to_move: PlayerId) -> GameState {
    let specs = |v: &[Placed]| v.iter().map(|p| p.spec.clone()).collect();
    let mut state = new_game(
        [button("Un", specs(&one)), button("Deux", specs(&two))],
        1,
        MatchConfig::default(),
    );
    for (p, placed) in [(PlayerId::One, &one), (PlayerId::Two, &two)] {
        let ps = state.player_mut(p);
        for slot in &mut ps.slots {
            slot.in_play = true;
        }
        ps.active = ps
            .slots
            .iter()
            .zip(placed.iter())
            .map(|(slot, pl)| {
                let sides = sides_of(&pl.spec);
                assert!((sides.min()..=sides.max()).contains(&pl.value), "value out of range");
                Die {
                    id: slot.id,
                    skills: slot.spec.skills.clone(),
                    sides,
                    value: pl.value,
                    dizzy: false,
                }
            })
            .collect();
        ps.captured.clear();
    }
    state.initiative = Some(to_move);
    state.consecutive_passes = 0;
    state.waiting_for = WaitingFor::Attack { player: to_move };
    state
}

/// The id of the `index`-th die of `player`'s button.
pub fn id(state: &GameState, player: PlayerId, index: usize) -> DieId {
    state.player(player).slots[index].id
}
```

- [ ] **Step 4: Écrire les tests d'intégration**

`crates/engine/tests/common/mod.rs` :
```rust
#![allow(dead_code)]

use buttonmen_engine::{Action, GameEvent, GameState, PlayerId, RuleError, apply};

/// Applies an action that must succeed.
pub fn ok(state: &GameState, player: PlayerId, action: Action) -> (GameState, Vec<GameEvent>) {
    apply(state, player, action).unwrap_or_else(|e| panic!("action refused: {e}"))
}

/// Applies an action that must fail, and returns the error.
pub fn err(state: &GameState, player: PlayerId, action: Action) -> RuleError {
    apply(state, player, action).expect_err("action unexpectedly accepted")
}
```

`crates/engine/tests/setup.rs` :
```rust
mod common;

use buttonmen_engine::testing::{button, fixed};
use buttonmen_engine::{
    Action, DieShape, DieSkill, DieSpec, MatchConfig, PlayerId, RuleError, SpecChoice, SwingLetter,
    TwinPart, WaitingFor, new_game,
};
use common::{err, ok};
use PlayerId::{One, Two};

fn swing(l: SwingLetter) -> DieSpec {
    DieSpec::new(DieShape::Swing(l))
}

fn x_choice(size: u8) -> Action {
    Action::SpecifyDice {
        choice: SpecChoice { swing: vec![(SwingLetter::X, size)], options: vec![] },
    }
}

fn swing_match() -> buttonmen_engine::GameState {
    new_game(
        [
            button("Avis", vec![fixed(4), fixed(10), swing(SwingLetter::X)]),
            button("Hammer", vec![fixed(6), fixed(20), swing(SwingLetter::X)]),
        ],
        11,
        MatchConfig::default(),
    )
}

#[test]
fn swing_buttons_wait_for_both_players() {
    let state = swing_match();
    assert_eq!(state.waiting_for, WaitingFor::SpecifyDice { pending: vec![One, Two] });
    assert_eq!(state.round, 1);
}

#[test]
fn swing_size_must_be_in_range_and_present() {
    let state = swing_match();
    assert_eq!(
        err(&state, One, x_choice(21)),
        RuleError::SwingOutOfRange { letter: SwingLetter::X, size: 21 }
    );
    assert_eq!(
        err(&state, One, Action::SpecifyDice { choice: SpecChoice::default() }),
        RuleError::SwingMismatch { letter: SwingLetter::X }
    );
    let extra = Action::SpecifyDice {
        choice: SpecChoice { swing: vec![(SwingLetter::X, 8), (SwingLetter::Y, 3)], options: vec![] },
    };
    assert_eq!(err(&state, One, extra), RuleError::SwingMismatch { letter: SwingLetter::Y });
}

#[test]
fn a_player_cannot_specify_twice() {
    let (state, _) = ok(&swing_match(), One, x_choice(8));
    assert_eq!(state.waiting_for, WaitingFor::SpecifyDice { pending: vec![Two] });
    assert_eq!(err(&state, One, x_choice(9)), RuleError::NotYourTurn);
}

#[test]
fn after_both_choices_dice_are_rolled_with_chosen_sizes() {
    let (state, _) = ok(&swing_match(), One, x_choice(8));
    let (state, events) = ok(&state, Two, x_choice(20));
    assert!(matches!(state.waiting_for, WaitingFor::Attack { .. }));
    let one_x = &state.player(One).active[2];
    assert_eq!(one_x.max(), 8);
    assert!((1..=8).contains(&one_x.value));
    assert_eq!(state.player(Two).active[2].max(), 20);
    assert!(events.iter().any(|e| matches!(e, buttonmen_engine::GameEvent::InitiativeDecided { .. })));
}

#[test]
fn same_letter_twin_halves_share_one_size() {
    let twin_x = DieSpec::new(DieShape::Twin(
        TwinPart::Swing(SwingLetter::X),
        TwinPart::Swing(SwingLetter::X),
    ));
    let state = new_game(
        [button("A", vec![twin_x, swing(SwingLetter::X)]), button("B", vec![fixed(6)])],
        5,
        MatchConfig::default(),
    );
    assert_eq!(state.waiting_for, WaitingFor::SpecifyDice { pending: vec![One] });
    let (state, _) = ok(&state, One, x_choice(7));
    assert_eq!(state.player(One).active[0].max(), 14);
    assert_eq!(state.player(One).active[1].max(), 7);
}

#[test]
fn option_dice_take_one_of_their_two_sizes() {
    let state = new_game(
        [button("A", vec![DieSpec::new(DieShape::Option(6, 10))]), button("B", vec![fixed(6)])],
        5,
        MatchConfig::default(),
    );
    let die = state.player(One).slots[0].id;
    let bad = Action::SpecifyDice { choice: SpecChoice { swing: vec![], options: vec![(die, 8)] } };
    assert_eq!(err(&state, One, bad), RuleError::InvalidOption { die });
    let good = Action::SpecifyDice { choice: SpecChoice { swing: vec![], options: vec![(die, 10)] } };
    let (state, _) = ok(&state, One, good);
    assert_eq!(state.player(One).active[0].max(), 10);
}

#[test]
fn fixed_buttons_start_attacking_immediately() {
    let state = new_game(
        [button("A", vec![fixed(4), fixed(6)]), button("B", vec![fixed(8)])],
        3,
        MatchConfig::default(),
    );
    assert!(matches!(state.waiting_for, WaitingFor::Attack { .. }));
}

#[test]
fn reserve_dice_start_out_of_play() {
    let state = new_game(
        [button("A", vec![fixed(4), fixed(20).with(DieSkill::Reserve)]), button("B", vec![fixed(8)])],
        3,
        MatchConfig::default(),
    );
    assert_eq!(state.player(One).active.len(), 1);
    assert!(!state.player(One).slots[1].in_play);
}

#[test]
fn initiative_goes_to_the_lowest_die() {
    // One-sided dice always show 1, (1,1) twins always show 2.
    let state = new_game(
        [
            button("A", vec![fixed(1), fixed(1)]),
            button("B", vec![fixed(1), DieSpec::new(DieShape::Twin(TwinPart::Fixed(1), TwinPart::Fixed(1)))]),
        ],
        3,
        MatchConfig::default(),
    );
    // A = [1, 1], B = [1, 2]: equal first die, A's second die is lower.
    assert_eq!(state.initiative, Some(One));
}

#[test]
fn initiative_with_no_eligible_dice_still_decides() {
    // Review Focus 1: player One only has Trip dice (excluded from initiative).
    let state = new_game(
        [button("A", vec![fixed(6).with(DieSkill::Trip)]), button("B", vec![fixed(6)])],
        3,
        MatchConfig::default(),
    );
    assert_eq!(state.initiative, Some(Two));
    assert_eq!(state.waiting_for, WaitingFor::Attack { player: Two });
}

#[test]
fn surrender_ends_the_match() {
    let state = new_game(
        [button("A", vec![fixed(6)]), button("B", vec![fixed(6)])],
        3,
        MatchConfig::default(),
    );
    let (state, _) = ok(&state, One, Action::Surrender);
    assert!(matches!(
        state.waiting_for,
        WaitingFor::GameOver { result: buttonmen_engine::MatchResult { winner: Two, .. } }
    ));
    assert_eq!(err(&state, Two, Action::Pass), RuleError::GameOver);
}
```

- [ ] **Step 5: Lancer les tests, vérifier qu'ils échouent**

Run: `cargo test -p buttonmen-engine`
Expected: échec de compilation (`new_game`, `apply`… introuvables).

- [ ] **Step 6: Implémenter les accroches, l'initiative, la mise en place et le réducteur**

`crates/engine/src/hooks.rs` (les autres accroches arrivent aux Tâches 7 à 10) :
```rust
//! Skill hooks: the only places where skills change the rules (spec §5.4).
//! Each hook matches every `DieSkill` explicitly, so a new skill must decide each hook.

use crate::die::Die;
use crate::skill::DieSkill;

/// R 3.3 + R 6.13 + R 6.14: does this die count when comparing initiative?
pub(crate) fn counts_for_initiative(die: &Die) -> bool {
    die.skills.iter().all(|skill| match skill {
        // R 6.13 / R 6.14: Stinger and Trip dice are ignored for initiative.
        DieSkill::Stinger | DieSkill::Trip => false,
        DieSkill::Berserk
        | DieSkill::Chance
        | DieSkill::Fire
        | DieSkill::Focus
        | DieSkill::Konstant
        | DieSkill::Null
        | DieSkill::Poison
        | DieSkill::Queer
        | DieSkill::Reserve
        | DieSkill::Shadow
        | DieSkill::Speed
        | DieSkill::Stealth
        | DieSkill::Value => true,
    })
}
```

`crates/engine/src/initiative.rs` :
```rust
use std::cmp::Ordering;

use crate::event::GameEvent;
use crate::hooks;
use crate::ids::PlayerId;
use crate::state::{GameState, WaitingFor};

/// R 3.3: sorted values of `player`'s dice that count for initiative.
pub(crate) fn initiative_values(state: &GameState, player: PlayerId) -> Vec<u8> {
    let mut values: Vec<u8> = state
        .player(player)
        .active
        .iter()
        .filter(|d| hooks::counts_for_initiative(d))
        .map(|d| d.value)
        .collect();
    values.sort_unstable();
    values
}

/// R 3.3: `Less` when `a` goes first. Lowest die first, then the next one; if one
/// list runs out while the other goes on, the longer list goes first (buttonweavers
/// pads each list with a sentinel lower than "no die").
pub(crate) fn compare(a: &[u8], b: &[u8]) -> Ordering {
    for (x, y) in a.iter().zip(b) {
        if x != y {
            return x.cmp(y);
        }
    }
    b.len().cmp(&a.len())
}

/// R 3.3 + R 3.4: decides who starts the round.
pub(crate) fn determine(state: &mut GameState, events: &mut Vec<GameEvent>) {
    let order = compare(
        &initiative_values(state, PlayerId::One),
        &initiative_values(state, PlayerId::Two),
    );
    let (winner, by_coin) = match order {
        Ordering::Less => (PlayerId::One, false),
        Ordering::Greater => (PlayerId::Two, false),
        // R 3.4: complete tie, coin toss.
        Ordering::Equal => (if state.rng.coin() { PlayerId::One } else { PlayerId::Two }, true),
    };
    state.initiative = Some(winner);
    events.push(GameEvent::InitiativeDecided { player: winner, by_coin });
    start_attacks(state);
}

/// R 6.4 / R 7.4: only the starting player's lowered Focus dice stay dizzy.
pub(crate) fn start_attacks(state: &mut GameState) {
    let first = state.initiative.expect("initiative is decided before attacks");
    for die in &mut state.player_mut(first.other()).active {
        die.dizzy = false;
    }
    state.waiting_for = WaitingFor::Attack { player: first };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lowest_first_die_wins() {
        assert_eq!(compare(&[1, 9], &[2, 3]), Ordering::Less);
        assert_eq!(compare(&[3], &[2, 20]), Ordering::Greater);
    }

    #[test]
    fn ties_compare_the_next_die() {
        assert_eq!(compare(&[1, 4], &[1, 3]), Ordering::Greater);
        assert_eq!(compare(&[2, 2], &[2, 2]), Ordering::Equal);
    }

    #[test]
    fn longer_list_wins_an_equal_prefix() {
        assert_eq!(compare(&[1, 2], &[1, 2, 3]), Ordering::Greater);
        assert_eq!(compare(&[1, 2, 3], &[1, 2]), Ordering::Less);
        assert_eq!(compare(&[], &[5]), Ordering::Greater);
        assert_eq!(compare(&[], &[]), Ordering::Equal);
    }
}
```

`crates/engine/src/setup.rs` :
```rust
use std::collections::BTreeSet;

use crate::action::SpecChoice;
use crate::die::{Die, DieShape, Sides, SwingLetter, TwinPart};
use crate::error::RuleError;
use crate::event::GameEvent;
use crate::ids::{DieId, PlayerId};
use crate::initiative;
use crate::rng::SeededRng;
use crate::skill::DieSkill;
use crate::state::{
    Button, DieSlot, GameState, MatchConfig, PlayerState, Record, WaitingFor,
};

/// Creates a match and plays it forward until the first decision.
pub fn new_game(buttons: [Button; 2], seed: u64, config: MatchConfig) -> GameState {
    let mut next_id = 0u16;
    let players = buttons.map(|button| {
        let slots = button
            .dice
            .into_iter()
            .map(|spec| {
                let id = DieId(next_id);
                next_id += 1;
                // R 6.11: reserve dice start out of play.
                let in_play = !spec.skills.has(DieSkill::Reserve);
                DieSlot { id, spec, in_play }
            })
            .collect();
        PlayerState {
            button: button.name,
            slots,
            choice: SpecChoice::default(),
            pending_choice: None,
            active: Vec::new(),
            captured: Vec::new(),
            record: Record::default(),
        }
    });
    let mut state = GameState {
        config,
        players,
        round: 1,
        initiative: None,
        consecutive_passes: 0,
        last_round_loser: None,
        waiting_for: WaitingFor::SpecifyDice { pending: Vec::new() },
        rng: SeededRng::new(seed),
    };
    begin_round(&mut state, &mut Vec::new());
    state
}

/// R 1.2: swing letters used by `player`'s dice in play.
pub(crate) fn required_letters(player: &PlayerState) -> BTreeSet<SwingLetter> {
    let mut letters = BTreeSet::new();
    for slot in player.slots.iter().filter(|s| s.in_play) {
        match slot.spec.shape {
            DieShape::Swing(l) => {
                letters.insert(l);
            }
            DieShape::Twin(a, b) => {
                for part in [a, b] {
                    if let TwinPart::Swing(l) = part {
                        letters.insert(l);
                    }
                }
            }
            DieShape::Fixed(_) | DieShape::Option(..) => {}
        }
    }
    letters
}

/// R 1.3: option dice in play, with their two sizes.
pub(crate) fn option_slots(player: &PlayerState) -> Vec<(DieId, u8, u8)> {
    player
        .slots
        .iter()
        .filter(|s| s.in_play)
        .filter_map(|s| match s.spec.shape {
            DieShape::Option(a, b) => Some((s.id, a, b)),
            DieShape::Fixed(_) | DieShape::Swing(_) | DieShape::Twin(..) => None,
        })
        .collect()
}

/// R 1.2 + R 1.3: `choice` gives exactly one valid size per letter and per option die.
pub(crate) fn validate_choice(player: &PlayerState, choice: &SpecChoice) -> Result<(), RuleError> {
    let letters = required_letters(player);
    for &letter in &letters {
        let sizes: Vec<u8> = choice
            .swing
            .iter()
            .filter(|(l, _)| *l == letter)
            .map(|(_, s)| *s)
            .collect();
        match sizes.as_slice() {
            [size] if letter.range().contains(size) => {}
            [size] => return Err(RuleError::SwingOutOfRange { letter, size: *size }),
            _ => return Err(RuleError::SwingMismatch { letter }),
        }
    }
    if let Some((letter, _)) = choice.swing.iter().find(|(l, _)| !letters.contains(l)) {
        return Err(RuleError::SwingMismatch { letter: *letter });
    }
    let options = option_slots(player);
    for &(die, a, b) in &options {
        let picks: Vec<u8> = choice
            .options
            .iter()
            .filter(|(d, _)| *d == die)
            .map(|(_, s)| *s)
            .collect();
        match picks.as_slice() {
            [size] if *size == a || *size == b => {}
            _ => return Err(RuleError::InvalidOption { die }),
        }
    }
    if let Some((die, _)) = choice
        .options
        .iter()
        .find(|(d, _)| !options.iter().any(|(o, _, _)| o == d))
    {
        return Err(RuleError::InvalidOption { die: *die });
    }
    Ok(())
}

fn needs_choice(player: &PlayerState) -> bool {
    !required_letters(player).is_empty() || !option_slots(player).is_empty()
}

/// R 2.3 + R 6.11: what happens before the dice are rolled.
pub(crate) fn begin_round(state: &mut GameState, events: &mut Vec<GameEvent>) {
    if let Some(loser) = state.last_round_loser {
        let has_reserve = state
            .player(loser)
            .slots
            .iter()
            .any(|s| !s.in_play && s.spec.skills.has(DieSkill::Reserve));
        if has_reserve {
            state.waiting_for = WaitingFor::ChooseReserve { player: loser };
            return;
        }
    }
    ask_for_choices(state, events);
}

/// R 3.1 + R 2.3: round 1, both players choose; later, the loser may change, and anyone
/// whose dice are not all sized (a reserve swing die just added) must choose.
pub(crate) fn ask_for_choices(state: &mut GameState, events: &mut Vec<GameEvent>) {
    let pending: Vec<PlayerId> = PlayerId::ALL
        .into_iter()
        .filter(|&p| {
            let player = state.player(p);
            needs_choice(player)
                && (validate_choice(player, &player.choice).is_err()
                    || state.last_round_loser == Some(p))
        })
        .collect();
    if pending.is_empty() {
        start_round(state, events);
    } else {
        state.waiting_for = WaitingFor::SpecifyDice { pending };
    }
}

/// Handles `Action::SpecifyDice`.
pub(crate) fn specify_dice(
    state: &mut GameState,
    player: PlayerId,
    choice: SpecChoice,
    events: &mut Vec<GameEvent>,
) -> Result<(), RuleError> {
    let WaitingFor::SpecifyDice { pending } = &state.waiting_for else {
        return Err(RuleError::WrongPhase);
    };
    if !pending.contains(&player) {
        return Err(RuleError::NotYourTurn);
    }
    let remaining: Vec<PlayerId> = pending.iter().copied().filter(|&p| p != player).collect();
    validate_choice(state.player(player), &choice)?;
    state.player_mut(player).pending_choice = Some(choice);
    if !remaining.is_empty() {
        state.waiting_for = WaitingFor::SpecifyDice { pending: remaining };
        return Ok(());
    }
    // R 3.1: choices are revealed together.
    for p in PlayerId::ALL {
        if let Some(choice) = state.player_mut(p).pending_choice.take() {
            events.push(GameEvent::DiceSpecified { player: p, choice: choice.clone() });
            state.player_mut(p).choice = choice;
        }
    }
    start_round(state, events);
    Ok(())
}

fn resolve_sides(shape: DieShape, id: DieId, choice: &SpecChoice) -> Sides {
    let swing = |l: SwingLetter| choice.swing_size(l).expect("choices are validated before rolling");
    let part = |p: TwinPart| match p {
        TwinPart::Fixed(n) => n,
        TwinPart::Swing(l) => swing(l),
    };
    match shape {
        DieShape::Fixed(n) => Sides::Single(n),
        DieShape::Swing(l) => Sides::Single(swing(l)),
        DieShape::Option(..) => {
            Sides::Single(choice.option_size(id).expect("choices are validated before rolling"))
        }
        DieShape::Twin(a, b) => Sides::Twin(part(a), part(b)),
    }
}

/// R 3.2: every die in play is rebuilt from its recipe and rolled.
pub(crate) fn start_round(state: &mut GameState, events: &mut Vec<GameEvent>) {
    events.push(GameEvent::RoundStarted { round: state.round });
    state.consecutive_passes = 0;
    for p in PlayerId::ALL {
        let mut dice: Vec<Die> = {
            let player = state.player(p);
            player
                .slots
                .iter()
                .filter(|s| s.in_play)
                .map(|s| Die {
                    id: s.id,
                    skills: s.spec.skills.clone(),
                    sides: resolve_sides(s.spec.shape, s.id, &player.choice),
                    value: 0,
                    dizzy: false,
                })
                .collect()
        };
        for die in &mut dice {
            die.value = die.sides.roll(&mut state.rng);
        }
        events.push(GameEvent::DiceRolled {
            player: p,
            dice: dice.iter().map(|d| (d.id, d.value)).collect(),
        });
        let player = state.player_mut(p);
        player.active = dice;
        player.captured.clear();
    }
    initiative::determine(state, events);
}
```

`crates/engine/src/round.rs` :
```rust
use crate::event::GameEvent;
use crate::ids::PlayerId;
use crate::state::{EndReason, GameState, MatchResult, WaitingFor};

/// R 2.4: the opponent wins the match.
pub(crate) fn surrender(state: &mut GameState, player: PlayerId, events: &mut Vec<GameEvent>) {
    let result = MatchResult { winner: player.other(), reason: EndReason::Surrender };
    events.push(GameEvent::MatchEnded { result });
    state.waiting_for = WaitingFor::GameOver { result };
}
```

`crates/engine/src/apply.rs` :
```rust
use crate::action::Action;
use crate::error::RuleError;
use crate::event::GameEvent;
use crate::ids::PlayerId;
use crate::state::{GameState, WaitingFor};
use crate::{round, setup};

/// The rules reducer: a new state and what happened, or why the action is refused.
pub fn apply(
    state: &GameState,
    player: PlayerId,
    action: Action,
) -> Result<(GameState, Vec<GameEvent>), RuleError> {
    if let WaitingFor::GameOver { .. } = state.waiting_for {
        return Err(RuleError::GameOver);
    }
    let mut next = state.clone();
    let mut events = Vec::new();
    match action {
        Action::Surrender => round::surrender(&mut next, player, &mut events),
        Action::SpecifyDice { choice } => setup::specify_dice(&mut next, player, choice, &mut events)?,
        // Wired by later tasks: initiative reactions (6), attacks and pass (8),
        // fire (9), reserve (10).
        Action::ChooseReserve { .. }
        | Action::FocusTurnDown { .. }
        | Action::ChanceReroll { .. }
        | Action::PassInitiative
        | Action::Attack { .. }
        | Action::FireTurnDown { .. }
        | Action::CancelFire
        | Action::Pass => return Err(RuleError::WrongPhase),
    }
    Ok((next, events))
}
```

`crates/engine/src/lib.rs` :
```rust
//! Button Men rules engine. Rule references (`R x.y`) point to `docs/RULES.md`.

pub mod action;
mod apply;
pub mod die;
pub mod error;
pub mod event;
mod hooks;
pub mod ids;
mod initiative;
pub mod rng;
mod round;
mod setup;
pub mod skill;
pub mod state;
#[cfg(feature = "testing")]
pub mod testing;

pub use action::{Action, AttackKind, DeclaredAttack, SpecChoice};
pub use apply::apply;
pub use die::{Die, DieShape, DieSpec, Sides, SwingLetter, TwinPart};
pub use error::{AttackError, FireError, ReactionError, RuleError};
pub use event::GameEvent;
pub use ids::{DieId, PlayerId};
pub use rng::SeededRng;
pub use setup::new_game;
pub use skill::{DieSkill, SkillInfo, SkillSet, skill_info};
pub use state::{
    Button, CaptureScoring, CapturedDie, DieSlot, EndReason, GameState, MatchConfig, MatchResult,
    PlayerState, Record, WaitingFor,
};
```

- [ ] **Step 7: Lancer les tests, vérifier qu'ils passent**

Run: `cargo test -p buttonmen-engine && cargo clippy -p buttonmen-engine --all-targets --all-features -- -D warnings`
Expected: tous les tests PASS. Si `testing.rs` provoque un avertissement `dead_code` hors tests, c'est normal tant que la feature est inactive ; clippy est lancé avec `--all-features`.

- [ ] **Step 8: Commit**

```bash
git add crates/engine
git commit -m "feat(engine): état de partie, choix Swing/Option, jet et initiative"
```

---
### Task 6: Réactions d'initiative (Focus, Chance)

**Files:**
- Modify: `crates/engine/src/initiative.rs`, `crates/engine/src/hooks.rs`, `crates/engine/src/apply.rs`, `crates/engine/src/testing.rs`
- Create: `crates/engine/tests/initiative.rs`

**Interfaces:**
- Consumes: Tâche 5.
- Produces: `hooks::rerolls(&Die) -> bool`, `hooks::reroll(&mut Die, &mut SeededRng)` (R 6.5, réutilisé par la Tâche 7) ; `initiative::{offer_reaction, focus_turn_down, chance_reroll, pass_initiative}` ; `testing::opening_state(one, two) -> GameState` (dés aux valeurs forcées, initiative calculée, réactions proposées).

- [ ] **Step 1: Ajouter `opening_state` au module `testing`**

Dans `crates/engine/src/testing.rs`, ajouter :
```rust
/// Like `attack_state`, but the initiative is then decided from the forced values
/// (R 3.3–3.5), so the state may wait for an initiative reaction.
pub fn opening_state(one: Vec<Placed>, two: Vec<Placed>) -> GameState {
    let mut state = attack_state(one, two, PlayerId::One);
    state.initiative = None;
    crate::initiative::determine(&mut state, &mut Vec::new());
    state
}
```

- [ ] **Step 2: Écrire les tests**

`crates/engine/tests/initiative.rs` :
```rust
mod common;

use buttonmen_engine::testing::{die, id, opening_state};
use buttonmen_engine::{
    Action, DieSkill, PlayerId, ReactionError, RuleError, SeededRng, WaitingFor,
};
use common::{err, ok};
use PlayerId::{One, Two};

#[test]
fn player_without_initiative_may_react_with_focus() {
    let state = opening_state(vec![die(10, 3)], vec![die(10, 5).with(DieSkill::Focus)]);
    assert_eq!(state.initiative, Some(One));
    assert_eq!(state.waiting_for, WaitingFor::ReactToInitiative { player: Two });
    let focus = id(&state, Two, 0);
    let (state, _) = ok(&state, Two, Action::FocusTurnDown { dice: vec![(focus, 2)] });
    assert_eq!(state.initiative, Some(Two));
    // One has no Focus/Chance dice: Two attacks first, with a dizzy Focus die (R 6.4).
    assert_eq!(state.waiting_for, WaitingFor::Attack { player: Two });
    let d = state.player(Two).die(focus).unwrap();
    assert_eq!(d.value, 2);
    assert!(d.dizzy);
}

#[test]
fn focus_must_lower_the_die_and_gain_initiative() {
    let state = opening_state(vec![die(10, 3)], vec![die(10, 5).with(DieSkill::Focus)]);
    let focus = id(&state, Two, 0);
    let refused = |value| err(&state, Two, Action::FocusTurnDown { dice: vec![(focus, value)] });
    assert_eq!(
        refused(4),
        RuleError::InvalidReaction { reason: ReactionError::DoesNotGainInitiative }
    );
    // A tie is not a gain.
    assert_eq!(
        refused(3),
        RuleError::InvalidReaction { reason: ReactionError::DoesNotGainInitiative }
    );
    assert_eq!(
        refused(6),
        RuleError::InvalidReaction { reason: ReactionError::ValueNotLowered { die: focus } }
    );
    assert_eq!(
        refused(0),
        RuleError::InvalidReaction { reason: ReactionError::ValueNotLowered { die: focus } }
    );
    assert_eq!(
        err(&state, Two, Action::FocusTurnDown { dice: vec![] }),
        RuleError::InvalidReaction { reason: ReactionError::NoDice }
    );
}

#[test]
fn only_focus_dice_can_be_turned_down() {
    let state = opening_state(
        vec![die(10, 3)],
        vec![die(10, 5).with(DieSkill::Focus), die(10, 9)],
    );
    let plain = id(&state, Two, 1);
    assert_eq!(
        err(&state, Two, Action::FocusTurnDown { dice: vec![(plain, 1)] }),
        RuleError::InvalidReaction { reason: ReactionError::NotFocus { die: plain } }
    );
}

#[test]
fn the_holder_cannot_react() {
    let state = opening_state(vec![die(10, 3)], vec![die(10, 5).with(DieSkill::Focus)]);
    assert_eq!(err(&state, One, Action::PassInitiative), RuleError::NotYourTurn);
}

#[test]
fn reactions_alternate_until_someone_passes() {
    let state = opening_state(
        vec![die(10, 3).with(DieSkill::Focus)],
        vec![die(10, 5).with(DieSkill::Focus)],
    );
    let (a, b) = (id(&state, One, 0), id(&state, Two, 0));
    let (state, _) = ok(&state, Two, Action::FocusTurnDown { dice: vec![(b, 2)] });
    assert_eq!(state.waiting_for, WaitingFor::ReactToInitiative { player: One });
    let (state, _) = ok(&state, One, Action::FocusTurnDown { dice: vec![(a, 1)] });
    assert_eq!(state.waiting_for, WaitingFor::ReactToInitiative { player: Two });
    let (state, _) = ok(&state, Two, Action::PassInitiative);
    assert_eq!(state.waiting_for, WaitingFor::Attack { player: One });
    // R 7.4: only the starting player's lowered dice stay dizzy.
    assert!(state.player(One).die(a).unwrap().dizzy);
    assert!(!state.player(Two).die(b).unwrap().dizzy);
}

#[test]
fn failed_chance_reroll_ends_reactions() {
    // [1, 1] against [1]: One keeps the longer list (R 3.3). A d1 always rerolls 1.
    let state = opening_state(
        vec![die(1, 1), die(1, 1)],
        vec![die(1, 1).with(DieSkill::Chance)],
    );
    assert_eq!(state.waiting_for, WaitingFor::ReactToInitiative { player: Two });
    let chance = id(&state, Two, 0);
    let (state, _) = ok(&state, Two, Action::ChanceReroll { die: chance });
    assert_eq!(state.waiting_for, WaitingFor::Attack { player: One });
}

#[test]
fn chance_reroll_gives_initiative_only_when_lower() {
    let mut gained = 0;
    let mut kept = 0;
    for seed in 0..60 {
        let mut state = opening_state(vec![die(6, 2)], vec![die(6, 6).with(DieSkill::Chance)]);
        state.rng = SeededRng::new(seed);
        let chance = id(&state, Two, 0);
        let (after, _) = ok(&state, Two, Action::ChanceReroll { die: chance });
        let value = after.player(Two).die(chance).unwrap().value;
        assert!((1..=6).contains(&value));
        match value {
            1 => {
                assert_eq!(after.initiative, Some(Two));
                gained += 1;
            }
            3..=6 => {
                assert_eq!(after.initiative, Some(One));
                kept += 1;
            }
            _ => {} // 2: tie, decided by a coin (R 3.4)
        }
    }
    assert!(gained > 0 && kept > 0);
}

#[test]
fn chance_on_a_konstant_die_keeps_its_value() {
    let state = opening_state(
        vec![die(1, 1), die(1, 1)],
        vec![die(20, 17).with(DieSkill::Chance).with(DieSkill::Konstant)],
    );
    let chance = id(&state, Two, 0);
    let (state, _) = ok(&state, Two, Action::ChanceReroll { die: chance });
    assert_eq!(state.player(Two).die(chance).unwrap().value, 17);
}
```

Note : dans `failed_chance_reroll_ends_reactions`, One a la liste `[1, 1]` et Two `[1]` : One garde l'initiative (liste plus longue, R 3.3) et Two peut réagir avec Chance.

- [ ] **Step 3: Lancer les tests, vérifier qu'ils échouent**

Run: `cargo test -p buttonmen-engine --test initiative`
Expected: FAIL — `player_without_initiative_may_react_with_focus` attend `ReactToInitiative` mais obtient `Attack` ; les actions de réaction renvoient `WrongPhase`.

- [ ] **Step 4: Implémenter**

Dans `hooks.rs`, ajouter :
```rust
use crate::rng::SeededRng;

/// R 4.4 + R 6.5: does this die take a new value when it is rerolled?
pub(crate) fn rerolls(die: &Die) -> bool {
    die.skills.iter().all(|skill| match skill {
        // R 6.5: Konstant dice keep their value.
        DieSkill::Konstant => false,
        DieSkill::Berserk
        | DieSkill::Chance
        | DieSkill::Fire
        | DieSkill::Focus
        | DieSkill::Null
        | DieSkill::Poison
        | DieSkill::Queer
        | DieSkill::Reserve
        | DieSkill::Shadow
        | DieSkill::Speed
        | DieSkill::Stealth
        | DieSkill::Stinger
        | DieSkill::Trip
        | DieSkill::Value => true,
    })
}

/// Rerolls a die, honouring `rerolls`.
pub(crate) fn reroll(die: &mut Die, rng: &mut SeededRng) {
    if rerolls(die) {
        die.value = die.sides.roll(rng);
    }
}
```

Dans `initiative.rs`, remplacer l'appel final de `determine` (`start_attacks(state);`) par `offer_reaction(state, winner.other());`, puis ajouter :
```rust
use std::collections::BTreeSet;

use crate::error::{ReactionError, RuleError};
use crate::ids::DieId;
use crate::skill::DieSkill;

fn can_react(state: &GameState, player: PlayerId) -> bool {
    state
        .player(player)
        .active
        .iter()
        .any(|d| d.has(DieSkill::Focus) || d.has(DieSkill::Chance))
}

/// R 3.5: the player without initiative may react if they have Focus or Chance dice.
pub(crate) fn offer_reaction(state: &mut GameState, player: PlayerId) {
    if can_react(state, player) {
        state.waiting_for = WaitingFor::ReactToInitiative { player };
    } else {
        start_attacks(state);
    }
}

fn expect_reacting(state: &GameState, player: PlayerId) -> Result<(), RuleError> {
    match state.waiting_for {
        WaitingFor::ReactToInitiative { player: p } if p == player => Ok(()),
        WaitingFor::ReactToInitiative { .. } => Err(RuleError::NotYourTurn),
        _ => Err(RuleError::WrongPhase),
    }
}

fn reaction(reason: ReactionError) -> RuleError {
    RuleError::InvalidReaction { reason }
}

/// R 6.4: lower one or more Focus dice, only to gain the initiative.
pub(crate) fn focus_turn_down(
    state: &mut GameState,
    player: PlayerId,
    dice: Vec<(DieId, u8)>,
    events: &mut Vec<GameEvent>,
) -> Result<(), RuleError> {
    expect_reacting(state, player)?;
    if dice.is_empty() {
        return Err(reaction(ReactionError::NoDice));
    }
    let mut seen = BTreeSet::new();
    for &(id, value) in &dice {
        let die = state.player(player).die(id).ok_or(RuleError::UnknownDie { die: id })?;
        if !die.has(DieSkill::Focus) {
            return Err(reaction(ReactionError::NotFocus { die: id }));
        }
        if !seen.insert(id) || value >= die.value || value < die.min() {
            return Err(reaction(ReactionError::ValueNotLowered { die: id }));
        }
    }
    for &(id, value) in &dice {
        let die = state.player_mut(player).die_mut(id).expect("checked above");
        die.value = value;
        // R 6.4: unusable for the first attack if this player starts.
        die.dizzy = true;
    }
    let order = compare(
        &initiative_values(state, player),
        &initiative_values(state, player.other()),
    );
    if order != Ordering::Less {
        return Err(reaction(ReactionError::DoesNotGainInitiative));
    }
    events.push(GameEvent::FocusTurnedDown { player, dice });
    state.initiative = Some(player);
    offer_reaction(state, player.other());
    Ok(())
}

/// R 6.2: reroll one Chance die; a failed attempt ends the reactions.
pub(crate) fn chance_reroll(
    state: &mut GameState,
    player: PlayerId,
    id: DieId,
    events: &mut Vec<GameEvent>,
) -> Result<(), RuleError> {
    expect_reacting(state, player)?;
    let GameState { players, rng, .. } = state;
    let die = players[player.index()]
        .die_mut(id)
        .ok_or(RuleError::UnknownDie { die: id })?;
    if !die.has(DieSkill::Chance) {
        return Err(reaction(ReactionError::NotChance { die: id }));
    }
    hooks::reroll(die, rng);
    let value = die.value;
    events.push(GameEvent::ChanceRerolled { player, die: id, value });
    let order = compare(
        &initiative_values(state, player),
        &initiative_values(state, player.other()),
    );
    let (gains, by_coin) = match order {
        Ordering::Less => (true, false),
        Ordering::Greater => (false, false),
        // R 3.4: a tie is settled by a coin toss.
        Ordering::Equal => (state.rng.coin(), true),
    };
    if gains {
        state.initiative = Some(player);
        events.push(GameEvent::InitiativeDecided { player, by_coin });
        offer_reaction(state, player.other());
    } else {
        start_attacks(state);
    }
    Ok(())
}

/// R 3.5: let the opponent start.
pub(crate) fn pass_initiative(
    state: &mut GameState,
    player: PlayerId,
    events: &mut Vec<GameEvent>,
) -> Result<(), RuleError> {
    expect_reacting(state, player)?;
    events.push(GameEvent::InitiativePassed { player });
    start_attacks(state);
    Ok(())
}
```

Dans `apply.rs`, ajouter `initiative` aux imports (`use crate::{initiative, round, setup};`) et remplacer dans le bras `WrongPhase` les trois variantes de réaction par :
```rust
        Action::FocusTurnDown { dice } => initiative::focus_turn_down(&mut next, player, dice, &mut events)?,
        Action::ChanceReroll { die } => initiative::chance_reroll(&mut next, player, die, &mut events)?,
        Action::PassInitiative => initiative::pass_initiative(&mut next, player, &mut events)?,
```
(le bras `WrongPhase` garde `ChooseReserve`, `Attack`, `FireTurnDown`, `CancelFire`, `Pass`).

- [ ] **Step 5: Lancer les tests, vérifier qu'ils passent**

Run: `cargo test -p buttonmen-engine && cargo clippy -p buttonmen-engine --all-targets --all-features -- -D warnings`
Expected: PASS, y compris les tests de la Tâche 5.

- [ ] **Step 6: Commit**

```bash
git add crates/engine
git commit -m "feat(engine): réactions d'initiative Focus et Chance"
```

---
### Task 7: Légalité des attaques et accroches de compétences

**Files:**
- Create: `crates/engine/src/attack.rs`, `crates/engine/tests/attacks.rs`
- Modify: `crates/engine/src/hooks.rs`, `crates/engine/src/lib.rs`

**Interfaces:**
- Consumes: Tâches 5 et 6.
- Produces:
  - `pub fn check_attack(&GameState, PlayerId, &DeclaredAttack) -> Result<(), AttackError>` (exportée : le serveur et l'IA s'en servent).
  - `pub(crate) attack::skill_sums(&[&Die]) -> BTreeSet<i32>`.
  - Accroches : `hooks::attack_kinds(&Die) -> BTreeSet<AttackKind>`, `hooks::needs_partner(&Die) -> bool`, `hooks::can_be_captured(&Die, AttackKind, usize) -> bool`, `hooks::skill_values(&Die) -> Vec<i32>`.

- [ ] **Step 1: Écrire les tests**

`crates/engine/tests/attacks.rs` :
```rust
use buttonmen_engine::testing::{attack_state, die, id, twin, Placed};
use buttonmen_engine::{
    AttackError, AttackKind, DeclaredAttack, DieSkill, GameState, PlayerId, check_attack,
};
use AttackKind::{Berserk, Power, Shadow, Skill, Speed, Trip};
use PlayerId::{One, Two};

/// One's dice `mine` against Two's dice `theirs`; One to move.
fn table(mine: Vec<Placed>, theirs: Vec<Placed>) -> GameState {
    attack_state(mine, theirs, One)
}

/// Checks an attack by One, with attackers and targets given by index.
fn check(state: &GameState, kind: AttackKind, attackers: &[usize], targets: &[usize]) -> Result<(), AttackError> {
    let attack = DeclaredAttack {
        kind,
        attackers: attackers.iter().map(|&i| id(state, One, i)).collect(),
        targets: targets.iter().map(|&i| id(state, Two, i)).collect(),
    };
    check_attack(state, One, &attack)
}

#[test]
fn power_captures_an_equal_or_lower_die() {
    let s = table(vec![die(6, 4)], vec![die(6, 4), die(6, 5)]);
    assert_eq!(check(&s, Power, &[0], &[0]), Ok(()));
    assert_eq!(check(&s, Power, &[0], &[1]), Err(AttackError::ValuesDoNotMatch));
    assert_eq!(check(&s, Power, &[0], &[0, 1]), Err(AttackError::WrongDiceCount));
}

#[test]
fn skill_needs_an_exact_sum() {
    let s = table(vec![die(6, 2), die(6, 3)], vec![die(10, 5), die(10, 6), die(10, 2)]);
    assert_eq!(check(&s, Skill, &[0, 1], &[0]), Ok(()));
    assert_eq!(check(&s, Skill, &[0, 1], &[1]), Err(AttackError::ValuesDoNotMatch));
    assert_eq!(check(&s, Skill, &[0], &[2]), Ok(()));
}

#[test]
fn dice_must_belong_to_the_right_player_and_appear_once() {
    let s = table(vec![die(6, 4)], vec![die(6, 3)]);
    let theirs = id(&s, Two, 0);
    let mine = id(&s, One, 0);
    let wrong_side = DeclaredAttack { kind: Power, attackers: vec![theirs], targets: vec![theirs] };
    assert_eq!(check_attack(&s, One, &wrong_side), Err(AttackError::DuplicateDie { die: theirs }));
    let own_target = DeclaredAttack { kind: Power, attackers: vec![mine], targets: vec![mine] };
    assert_eq!(check_attack(&s, One, &own_target), Err(AttackError::DuplicateDie { die: mine }));
    let not_mine = DeclaredAttack { kind: Skill, attackers: vec![theirs], targets: vec![] };
    assert_eq!(check_attack(&s, One, &not_mine), Err(AttackError::NotYourDie { die: theirs }));
}

#[test]
fn shadow_captures_between_its_value_and_its_size() {
    let s = table(
        vec![die(10, 2).with(DieSkill::Shadow)],
        vec![die(4, 2), die(12, 10), die(12, 11), die(4, 1)],
    );
    assert_eq!(check(&s, Shadow, &[0], &[0]), Ok(()));
    assert_eq!(check(&s, Shadow, &[0], &[1]), Ok(()));
    assert_eq!(check(&s, Shadow, &[0], &[2]), Err(AttackError::ValuesDoNotMatch));
    assert_eq!(check(&s, Shadow, &[0], &[3]), Err(AttackError::ValuesDoNotMatch));
    let shadow = id(&s, One, 0);
    assert_eq!(
        check(&s, Power, &[0], &[3]),
        Err(AttackError::KindUnavailable { die: shadow, kind: Power })
    );
}

#[test]
fn queer_is_shadow_on_odd_values_only() {
    let odd = table(vec![die(10, 3).with(DieSkill::Queer)], vec![die(10, 5), die(10, 1)]);
    assert_eq!(check(&odd, Shadow, &[0], &[0]), Ok(()));
    assert!(matches!(check(&odd, Power, &[0], &[1]), Err(AttackError::KindUnavailable { .. })));
    let even = table(vec![die(10, 4).with(DieSkill::Queer)], vec![die(10, 5), die(10, 1)]);
    assert_eq!(check(&even, Power, &[0], &[1]), Ok(()));
    assert!(matches!(check(&even, Shadow, &[0], &[0]), Err(AttackError::KindUnavailable { .. })));
}

#[test]
fn speed_captures_several_dice_summing_to_its_value() {
    let s = table(vec![die(10, 7).with(DieSkill::Speed), die(10, 7)], vec![die(6, 3), die(6, 4), die(6, 3)]);
    assert_eq!(check(&s, Speed, &[0], &[0, 1]), Ok(()));
    assert_eq!(check(&s, Speed, &[0], &[0, 2]), Err(AttackError::ValuesDoNotMatch));
    assert!(matches!(check(&s, Speed, &[1], &[0, 1]), Err(AttackError::KindUnavailable { .. })));
    // A Speed die can still make ordinary attacks.
    assert_eq!(check(&s, Power, &[0], &[0]), Ok(()));
}

#[test]
fn berserk_attacks_like_speed_but_never_skill() {
    let s = table(vec![die(10, 7).with(DieSkill::Berserk)], vec![die(6, 3), die(6, 4), die(10, 7)]);
    assert_eq!(check(&s, Berserk, &[0], &[0, 1]), Ok(()));
    assert!(matches!(check(&s, Skill, &[0], &[2]), Err(AttackError::KindUnavailable { .. })));
}

#[test]
fn trip_is_illegal_only_without_any_chance() {
    let s = table(vec![die(4, 1).with(DieSkill::Trip), die(1, 1).with(DieSkill::Trip)], vec![die(20, 20), twin(1, 1, 2)]);
    assert_eq!(check(&s, Trip, &[0], &[0]), Ok(()));
    assert_eq!(check(&s, Trip, &[0], &[1]), Ok(()));
    // R 6.14: a 1-sided Trip die can never beat a twin (minimum 2).
    assert_eq!(check(&s, Trip, &[1], &[1]), Err(AttackError::ValuesDoNotMatch));
}

#[test]
fn stealth_only_fights_in_multi_die_skill_attacks() {
    let s = table(
        vec![die(6, 3).with(DieSkill::Stealth), die(6, 2), die(10, 5)],
        vec![die(6, 5).with(DieSkill::Stealth), die(6, 3)],
    );
    assert!(matches!(check(&s, Skill, &[0], &[1]), Err(AttackError::KindUnavailable { .. })));
    assert!(matches!(check(&s, Power, &[0], &[1]), Err(AttackError::KindUnavailable { .. })));
    let stealth_target = id(&s, Two, 0);
    assert_eq!(check(&s, Power, &[2], &[0]), Err(AttackError::TargetProtected { die: stealth_target }));
    assert_eq!(check(&s, Skill, &[2], &[0]), Err(AttackError::TargetProtected { die: stealth_target }));
    assert_eq!(check(&s, Skill, &[0, 1], &[0]), Ok(()));
}

#[test]
fn konstant_adds_or_subtracts_and_never_attacks_alone() {
    let s = table(
        vec![die(6, 5), die(4, 2).with(DieSkill::Konstant)],
        vec![die(6, 3), die(10, 7), die(4, 2)],
    );
    assert_eq!(check(&s, Skill, &[0, 1], &[0]), Ok(()));
    assert_eq!(check(&s, Skill, &[0, 1], &[1]), Ok(()));
    assert!(matches!(check(&s, Skill, &[1], &[2]), Err(AttackError::KindUnavailable { .. })));
    assert!(matches!(check(&s, Power, &[1], &[2]), Err(AttackError::KindUnavailable { .. })));
}

#[test]
fn stinger_counts_for_any_value_up_to_its_face() {
    // R 6.13 example: a die showing 4 and a Stinger showing 6 reach 5 through 10.
    let s = table(
        vec![die(6, 4), die(10, 6).with(DieSkill::Stinger)],
        vec![die(4, 4), die(6, 5), die(10, 10), die(12, 11)],
    );
    assert_eq!(check(&s, Skill, &[0, 1], &[0]), Err(AttackError::ValuesDoNotMatch));
    assert_eq!(check(&s, Skill, &[0, 1], &[1]), Ok(()));
    assert_eq!(check(&s, Skill, &[0, 1], &[2]), Ok(()));
    assert_eq!(check(&s, Skill, &[0, 1], &[3]), Err(AttackError::ValuesDoNotMatch));
}

#[test]
fn fire_dice_cannot_power_attack() {
    let s = table(vec![die(6, 6).with(DieSkill::Fire)], vec![die(6, 1)]);
    assert!(matches!(check(&s, Power, &[0], &[0]), Err(AttackError::KindUnavailable { .. })));
    assert_eq!(check(&s, Skill, &[0], &[0]), Err(AttackError::ValuesDoNotMatch));
}

#[test]
fn dizzy_dice_cannot_attack() {
    let mut s = table(vec![die(6, 4).with(DieSkill::Focus)], vec![die(6, 3)]);
    let focus = id(&s, One, 0);
    s.player_mut(One).die_mut(focus).unwrap().dizzy = true;
    assert_eq!(check(&s, Power, &[0], &[0]), Err(AttackError::Dizzy { die: focus }));
}
```

- [ ] **Step 2: Lancer les tests, vérifier qu'ils échouent**

Run: `cargo test -p buttonmen-engine --test attacks`
Expected: échec de compilation (`check_attack` introuvable).

- [ ] **Step 3: Implémenter les accroches**

Ajouter à `hooks.rs` :
```rust
use std::collections::BTreeSet;

use crate::action::AttackKind;

/// Which attacks this die can make (R 4.2, R 4.3, R 6.1, R 6.3, R 6.5, R 6.8–6.10,
/// R 6.12, R 6.14). Grants are applied first, then restrictions.
pub(crate) fn attack_kinds(die: &Die) -> BTreeSet<AttackKind> {
    let odd = die.value % 2 == 1;
    let mut kinds = BTreeSet::from([AttackKind::Power, AttackKind::Skill]);
    for skill in die.skills.iter() {
        match skill {
            // R 6.10.
            DieSkill::Shadow => {
                kinds.insert(AttackKind::Shadow);
            }
            // R 6.8: Shadow on odd values.
            DieSkill::Queer if odd => {
                kinds.insert(AttackKind::Shadow);
            }
            // R 6.9.
            DieSkill::Speed => {
                kinds.insert(AttackKind::Speed);
            }
            // R 6.14.
            DieSkill::Trip => {
                kinds.insert(AttackKind::Trip);
            }
            // R 6.1.
            DieSkill::Berserk => {
                kinds.insert(AttackKind::Berserk);
            }
            DieSkill::Queer
            | DieSkill::Chance
            | DieSkill::Fire
            | DieSkill::Focus
            | DieSkill::Konstant
            | DieSkill::Null
            | DieSkill::Poison
            | DieSkill::Reserve
            | DieSkill::Stealth
            | DieSkill::Stinger
            | DieSkill::Value => {}
        }
    }
    for skill in die.skills.iter() {
        match skill {
            // R 6.10 / R 6.8: Shadow replaces Power.
            DieSkill::Shadow => {
                kinds.remove(&AttackKind::Power);
            }
            DieSkill::Queer if odd => {
                kinds.remove(&AttackKind::Power);
            }
            // R 6.1: no Skill attacks.
            DieSkill::Berserk => {
                kinds.remove(&AttackKind::Skill);
            }
            // R 6.5 / R 6.3: no Power attacks.
            DieSkill::Konstant | DieSkill::Fire => {
                kinds.remove(&AttackKind::Power);
            }
            // R 6.12: multi-die Skill attacks only (the "multi-die" part is `needs_partner`).
            DieSkill::Stealth => kinds.retain(|k| *k == AttackKind::Skill),
            DieSkill::Queer
            | DieSkill::Chance
            | DieSkill::Focus
            | DieSkill::Null
            | DieSkill::Poison
            | DieSkill::Reserve
            | DieSkill::Speed
            | DieSkill::Stinger
            | DieSkill::Trip
            | DieSkill::Value => {}
        }
    }
    kinds
}

/// R 6.5 + R 6.12: this die only makes Skill attacks together with at least one other die.
pub(crate) fn needs_partner(die: &Die) -> bool {
    die.skills.iter().any(|skill| match skill {
        DieSkill::Konstant | DieSkill::Stealth => true,
        DieSkill::Berserk
        | DieSkill::Chance
        | DieSkill::Fire
        | DieSkill::Focus
        | DieSkill::Null
        | DieSkill::Poison
        | DieSkill::Queer
        | DieSkill::Reserve
        | DieSkill::Shadow
        | DieSkill::Speed
        | DieSkill::Stinger
        | DieSkill::Trip
        | DieSkill::Value => false,
    })
}

/// R 6.12: can `target` be captured by a `kind` attack made by `attackers` dice?
pub(crate) fn can_be_captured(target: &Die, kind: AttackKind, attackers: usize) -> bool {
    target.skills.iter().all(|skill| match skill {
        DieSkill::Stealth => kind == AttackKind::Skill && attackers >= 2,
        DieSkill::Berserk
        | DieSkill::Chance
        | DieSkill::Fire
        | DieSkill::Focus
        | DieSkill::Konstant
        | DieSkill::Null
        | DieSkill::Poison
        | DieSkill::Queer
        | DieSkill::Reserve
        | DieSkill::Shadow
        | DieSkill::Speed
        | DieSkill::Stinger
        | DieSkill::Trip
        | DieSkill::Value => true,
    })
}

/// R 4.3 + R 6.5 + R 6.13 + R 7.3: the values this die may stand for in a Skill attack.
pub(crate) fn skill_values(die: &Die) -> Vec<i32> {
    let mut stinger = false;
    let mut konstant = false;
    for skill in die.skills.iter() {
        match skill {
            DieSkill::Stinger => stinger = true,
            DieSkill::Konstant => konstant = true,
            DieSkill::Berserk
            | DieSkill::Chance
            | DieSkill::Fire
            | DieSkill::Focus
            | DieSkill::Null
            | DieSkill::Poison
            | DieSkill::Queer
            | DieSkill::Reserve
            | DieSkill::Shadow
            | DieSkill::Speed
            | DieSkill::Stealth
            | DieSkill::Trip
            | DieSkill::Value => {}
        }
    }
    let low = if stinger { die.min() } else { die.value };
    let mut values: Vec<i32> = (low..=die.value).map(i32::from).collect();
    if konstant {
        let negatives: Vec<i32> = values.iter().map(|v| -v).collect();
        values.extend(negatives);
    }
    values
}
```

- [ ] **Step 4: Implémenter `attack.rs`**

```rust
use std::collections::BTreeSet;

use crate::action::{AttackKind, DeclaredAttack};
use crate::die::Die;
use crate::error::AttackError;
use crate::hooks;
use crate::ids::PlayerId;
use crate::state::GameState;

struct Participants<'a> {
    attackers: Vec<&'a Die>,
    targets: Vec<&'a Die>,
}

fn gather<'a>(
    state: &'a GameState,
    player: PlayerId,
    attack: &DeclaredAttack,
) -> Result<Participants<'a>, AttackError> {
    let mut seen = BTreeSet::new();
    for &die in attack.attackers.iter().chain(&attack.targets) {
        if !seen.insert(die) {
            return Err(AttackError::DuplicateDie { die });
        }
    }
    let attackers = attack
        .attackers
        .iter()
        .map(|&die| state.player(player).die(die).ok_or(AttackError::NotYourDie { die }))
        .collect::<Result<Vec<_>, _>>()?;
    let targets = attack
        .targets
        .iter()
        .map(|&die| {
            state
                .player(player.other())
                .die(die)
                .ok_or(AttackError::NotOpponentDie { die })
        })
        .collect::<Result<Vec<_>, _>>()?;
    // R 6.4 / R 7.4.
    if let Some(d) = attackers.iter().find(|d| d.dizzy) {
        return Err(AttackError::Dizzy { die: d.id });
    }
    Ok(Participants { attackers, targets })
}

/// How many attackers and targets each kind takes (R 4.2, R 4.3, R 6.1, R 6.9, R 6.10, R 6.14).
fn check_counts(kind: AttackKind, p: &Participants) -> Result<(), AttackError> {
    let (a, t) = (p.attackers.len(), p.targets.len());
    let ok = match kind {
        AttackKind::Power | AttackKind::Shadow | AttackKind::Trip => a == 1 && t == 1,
        AttackKind::Skill => a >= 1 && t == 1,
        AttackKind::Speed | AttackKind::Berserk => a == 1 && t >= 1,
    };
    if ok { Ok(()) } else { Err(AttackError::WrongDiceCount) }
}

fn check_skills(kind: AttackKind, p: &Participants) -> Result<(), AttackError> {
    let n = p.attackers.len();
    for a in &p.attackers {
        let unavailable = !hooks::attack_kinds(a).contains(&kind)
            || (kind == AttackKind::Skill && n < 2 && hooks::needs_partner(a));
        if unavailable {
            return Err(AttackError::KindUnavailable { die: a.id, kind });
        }
    }
    for t in &p.targets {
        if !hooks::can_be_captured(t, kind, n) {
            return Err(AttackError::TargetProtected { die: t.id });
        }
    }
    Ok(())
}

/// R 4.3 + R 6.5 + R 6.13: every total the attackers can make together.
pub(crate) fn skill_sums(attackers: &[&Die]) -> BTreeSet<i32> {
    let mut sums = BTreeSet::from([0]);
    for die in attackers {
        let values = hooks::skill_values(die);
        sums = sums.iter().flat_map(|s| values.iter().map(move |v| s + v)).collect();
    }
    sums
}

fn values_allow(kind: AttackKind, p: &Participants) -> bool {
    let first_attacker = p.attackers[0];
    let first_target = p.targets[0];
    match kind {
        // R 4.2.
        AttackKind::Power => first_attacker.value >= first_target.value,
        // R 6.10.
        AttackKind::Shadow => {
            first_target.value >= first_attacker.value && first_target.value <= first_attacker.max()
        }
        // R 6.14: illegal only without any chance to capture.
        AttackKind::Trip => first_attacker.max() >= first_target.min(),
        // R 6.9 / R 6.1.
        AttackKind::Speed | AttackKind::Berserk => {
            p.targets.iter().map(|t| u32::from(t.value)).sum::<u32>() == u32::from(first_attacker.value)
        }
        // R 4.3.
        AttackKind::Skill => skill_sums(&p.attackers).contains(&i32::from(first_target.value)),
    }
}

/// Is `attack` legal for `player` right now (dice, counts, skills and values)?
pub fn check_attack(
    state: &GameState,
    player: PlayerId,
    attack: &DeclaredAttack,
) -> Result<(), AttackError> {
    let p = gather(state, player, attack)?;
    check_counts(attack.kind, &p)?;
    check_skills(attack.kind, &p)?;
    if values_allow(attack.kind, &p) { Ok(()) } else { Err(AttackError::ValuesDoNotMatch) }
}
```

Dans `lib.rs` : ajouter `mod attack;` et `pub use attack::check_attack;`.

- [ ] **Step 5: Lancer les tests, vérifier qu'ils passent**

Run: `cargo test -p buttonmen-engine && cargo clippy -p buttonmen-engine --all-targets --all-features -- -D warnings`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add crates/engine
git commit -m "feat(engine): légalité des attaques et accroches de compétences"
```

---
### Task 8: Résolution des attaques, passe, score, fin de manche et de match

**Files:**
- Create: `crates/engine/src/resolve.rs`, `crates/engine/src/scoring.rs`, `crates/engine/tests/rounds.rs`
- Modify: `crates/engine/src/attack.rs`, `crates/engine/src/hooks.rs`, `crates/engine/src/round.rs`, `crates/engine/src/apply.rs`, `crates/engine/src/lib.rs`

**Interfaces:**
- Consumes: Tâches 5 à 7.
- Produces:
  - `attack::possible_attacks(&GameState, PlayerId) -> Vec<DeclaredAttack>` (toutes les attaques légales sans aide Fire, ordre stable).
  - `hooks::capture_scoring(attackers: &[&Die], target: &Die) -> CaptureScoring`.
  - `resolve::declare_attack(&mut GameState, PlayerId, DeclaredAttack, &mut Vec<GameEvent>) -> Result<(), RuleError>` et `resolve::resolve_attack(...)` (sans contrôle).
  - `scoring::{Holding, die_points(&Die, Holding) -> i32, round_score(&GameState, PlayerId) -> i32}` (demi-points), `pub fn round_score` réexportée.
  - `round::{end_turn, pass}`.

- [ ] **Step 1: Écrire les tests**

Tests unitaires, à la fin de `crates/engine/src/scoring.rs` (créé à l'étape 3) :
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::die::Sides;
    use crate::ids::DieId;
    use crate::skill::DieSkill;

    fn d(sides: u8, value: u8, skills: &[DieSkill]) -> Die {
        Die {
            id: DieId(0),
            skills: skills.iter().copied().collect(),
            sides: Sides::Single(sides),
            value,
            dizzy: false,
        }
    }

    #[test]
    fn kept_dice_score_half_and_captured_dice_score_full() {
        assert_eq!(die_points(&d(7, 3, &[]), Holding::Kept), 7); // 3.5 points
        assert_eq!(die_points(&d(7, 3, &[]), Holding::Captured(CaptureScoring::BySize)), 14);
    }

    #[test]
    fn poison_is_negative() {
        let p = d(6, 2, &[DieSkill::Poison]);
        assert_eq!(die_points(&p, Holding::Kept), -12);
        assert_eq!(die_points(&p, Holding::Captured(CaptureScoring::BySize)), -6);
    }

    #[test]
    fn null_is_worth_nothing() {
        let n = d(20, 2, &[DieSkill::Null, DieSkill::Poison]);
        assert_eq!(die_points(&n, Holding::Kept), 0);
        assert_eq!(die_points(&d(20, 2, &[]), Holding::Captured(CaptureScoring::Zero)), 0);
    }

    #[test]
    fn value_dice_score_their_face() {
        let v = d(20, 3, &[DieSkill::Value]);
        assert_eq!(die_points(&v, Holding::Kept), 3);
        assert_eq!(die_points(&v, Holding::Captured(CaptureScoring::BySize)), 6);
        assert_eq!(die_points(&d(20, 5, &[]), Holding::Captured(CaptureScoring::ByValue(5))), 10);
        // R 7.2: Poison + Value.
        assert_eq!(die_points(&d(20, 3, &[DieSkill::Value, DieSkill::Poison]), Holding::Kept), -6);
    }
}
```

`crates/engine/tests/rounds.rs` :
```rust
mod common;

use buttonmen_engine::testing::{attack_state, button, die, fixed, id};
use buttonmen_engine::{
    Action, AttackError, AttackKind, CaptureScoring, DeclaredAttack, DieShape, DieSkill, DieSpec,
    EndReason, GameEvent, GameState, MatchConfig, MatchResult, PlayerId, RuleError, SeededRng,
    SpecChoice, Sides, SwingLetter, WaitingFor, new_game,
};
use common::{err, ok};
use PlayerId::{One, Two};

fn attack(state: &GameState, kind: AttackKind, attackers: &[usize], targets: &[usize]) -> Action {
    Action::Attack {
        attack: DeclaredAttack {
            kind,
            attackers: attackers.iter().map(|&i| id(state, One, i)).collect(),
            targets: targets.iter().map(|&i| id(state, Two, i)).collect(),
        },
    }
}

#[test]
fn power_attack_captures_rerolls_and_passes_the_turn() {
    let s = attack_state(vec![die(6, 4)], vec![die(6, 3), die(20, 20)], One);
    let (after, events) = ok(&s, One, attack(&s, AttackKind::Power, &[0], &[0]));
    assert_eq!(after.player(Two).active.len(), 1);
    assert_eq!(after.player(One).captured.len(), 1);
    assert_eq!(after.player(One).captured[0].scoring, CaptureScoring::BySize);
    assert!((1..=6).contains(&after.player(One).active[0].value));
    assert_eq!(after.waiting_for, WaitingFor::Attack { player: Two });
    assert!(events.iter().any(|e| matches!(e, GameEvent::DieCaptured { by: One, .. })));
}

#[test]
fn illegal_or_out_of_turn_attacks_are_refused() {
    let s = attack_state(vec![die(6, 2)], vec![die(6, 3)], One);
    assert_eq!(
        err(&s, One, attack(&s, AttackKind::Power, &[0], &[0])),
        RuleError::IllegalAttack { reason: AttackError::ValuesDoNotMatch }
    );
    assert_eq!(err(&s, Two, Action::Pass), RuleError::NotYourTurn);
}

#[test]
fn konstant_attackers_keep_their_value() {
    let s = attack_state(
        vec![die(6, 5), die(4, 2).with(DieSkill::Konstant)],
        vec![die(6, 3), die(20, 20)],
        One,
    );
    let (after, _) = ok(&s, One, attack(&s, AttackKind::Skill, &[0, 1], &[0]));
    assert_eq!(after.player(One).active[1].value, 2);
}

#[test]
fn berserk_halves_and_loses_berserk() {
    let s = attack_state(
        vec![die(10, 7).with(DieSkill::Berserk)],
        vec![die(6, 3), die(6, 4), die(20, 20)],
        One,
    );
    let (after, _) = ok(&s, One, attack(&s, AttackKind::Berserk, &[0], &[0, 1]));
    let b = &after.player(One).active[0];
    assert_eq!(b.sides, Sides::Single(5));
    assert!(!b.has(DieSkill::Berserk));
    assert!((1..=5).contains(&b.value));
}

#[test]
fn trip_rerolls_both_and_captures_on_greater_or_equal() {
    let mut captures = 0;
    let mut misses = 0;
    for seed in 0..40 {
        let mut s = attack_state(
            vec![die(6, 1).with(DieSkill::Trip), die(20, 20)],
            vec![die(6, 6), die(20, 20)],
            One,
        );
        s.rng = SeededRng::new(seed);
        let (after, events) = ok(&s, One, attack(&s, AttackKind::Trip, &[0], &[0]));
        let rolled = events.iter().find_map(|e| match e {
            GameEvent::TripRolled { attacker_value, target_value, captured, .. } => {
                Some((*attacker_value, *target_value, *captured))
            }
            _ => None,
        });
        let (a, t, captured) = rolled.expect("a trip event");
        assert_eq!(captured, a >= t);
        if captured {
            captures += 1;
            assert_eq!(after.player(Two).active.len(), 1);
        } else {
            misses += 1;
            assert_eq!(after.player(Two).active[0].value, t);
        }
        assert_eq!(after.player(One).active[0].value, a);
    }
    assert!(captures > 0 && misses > 0);
}

#[test]
fn null_and_value_attackers_change_capture_scoring() {
    let s = attack_state(vec![die(6, 4).with(DieSkill::Null)], vec![die(6, 3), die(20, 20)], One);
    let (after, _) = ok(&s, One, attack(&s, AttackKind::Power, &[0], &[0]));
    assert_eq!(after.player(One).captured[0].scoring, CaptureScoring::Zero);

    let s = attack_state(vec![die(6, 4).with(DieSkill::Value)], vec![die(20, 3), die(20, 20)], One);
    let (after, _) = ok(&s, One, attack(&s, AttackKind::Power, &[0], &[0]));
    assert_eq!(after.player(One).captured[0].scoring, CaptureScoring::ByValue(3));
}

#[test]
fn pass_is_refused_when_an_attack_exists() {
    let s = attack_state(vec![die(6, 4)], vec![die(6, 3)], One);
    assert_eq!(err(&s, One, Action::Pass), RuleError::PassNotAllowed);
}

#[test]
fn round_ends_when_a_player_runs_out_of_dice() {
    let s = attack_state(vec![die(6, 4), die(10, 1)], vec![die(6, 3)], One);
    let (after, events) = ok(&s, One, attack(&s, AttackKind::Power, &[0], &[0]));
    // One: kept 6 and 10 (3 + 5 points) + captured 6 (6 points) = 14 points = 28 half-points.
    assert!(events.contains(&GameEvent::RoundEnded { round: 1, scores: [28, 0], winner: Some(One) }));
    assert_eq!(after.round, 2);
    assert_eq!(after.player(One).record.wins, 1);
    assert_eq!(after.player(Two).record.losses, 1);
    assert_eq!(after.last_round_loser, Some(Two));
    assert_eq!(after.player(Two).active.len(), 1, "a new round rebuilt Two's dice");
}

#[test]
fn two_consecutive_passes_end_round() {
    // Review Focus 5. Stealth dice can neither attack alone nor be captured by one die.
    let stealth_x = DieSpec::new(DieShape::Swing(SwingLetter::X)).with(DieSkill::Stealth);
    let state = new_game(
        [button("A", vec![fixed(6).with(DieSkill::Stealth)]), button("B", vec![stealth_x])],
        2,
        MatchConfig::default(),
    );
    let x4 = Action::SpecifyDice { choice: SpecChoice { swing: vec![(SwingLetter::X, 4)], options: vec![] } };
    let (state, _) = ok(&state, Two, x4.clone());
    let first = state.initiative.expect("initiative decided");
    let (state, _) = ok(&state, first, Action::Pass);
    assert_eq!(state.waiting_for, WaitingFor::Attack { player: first.other() });
    let (state, events) = ok(&state, first.other(), Action::Pass);
    // Kept dice score half their size: 6 → 3 points, 4 → 2 points.
    assert!(events.contains(&GameEvent::RoundEnded { round: 1, scores: [6, 4], winner: Some(One) }));
    // R 2.3: the loser may change their swing dice.
    assert_eq!(state.waiting_for, WaitingFor::SpecifyDice { pending: vec![Two] });
    assert_eq!(state.round, 2);
    let (state, _) = ok(&state, Two, x4);
    assert!(matches!(state.waiting_for, WaitingFor::Attack { .. }));
}

#[test]
fn the_match_ends_at_max_wins() {
    let mut s = attack_state(vec![die(6, 4)], vec![die(6, 3)], One);
    s.config.max_wins = 1;
    let (after, events) = ok(&s, One, attack(&s, AttackKind::Power, &[0], &[0]));
    let result = MatchResult { winner: One, reason: EndReason::Rounds };
    assert_eq!(after.waiting_for, WaitingFor::GameOver { result });
    assert!(events.contains(&GameEvent::MatchEnded { result }));
}
```

- [ ] **Step 2: Lancer les tests, vérifier qu'ils échouent**

Run: `cargo test -p buttonmen-engine --test rounds`
Expected: FAIL — les attaques renvoient `WrongPhase`, `Pass` aussi.

- [ ] **Step 3: Implémenter**

Ajouter à `hooks.rs` :
```rust
use crate::state::CaptureScoring;

/// R 6.6 + R 6.15 + R 7.1: how `target`, captured by these attackers, will score.
pub(crate) fn capture_scoring(attackers: &[&Die], target: &Die) -> CaptureScoring {
    let mut null = false;
    let mut value = false;
    for die in attackers {
        for skill in die.skills.iter() {
            match skill {
                DieSkill::Null => null = true,
                DieSkill::Value => value = true,
                DieSkill::Berserk
                | DieSkill::Chance
                | DieSkill::Fire
                | DieSkill::Focus
                | DieSkill::Konstant
                | DieSkill::Poison
                | DieSkill::Queer
                | DieSkill::Reserve
                | DieSkill::Shadow
                | DieSkill::Speed
                | DieSkill::Stealth
                | DieSkill::Stinger
                | DieSkill::Trip => {}
            }
        }
    }
    if null {
        CaptureScoring::Zero
    } else if value {
        // R 6.15: valued at the face the target shows when captured.
        CaptureScoring::ByValue(target.value)
    } else {
        CaptureScoring::BySize
    }
}
```

`crates/engine/src/scoring.rs` :
```rust
use crate::die::Die;
use crate::ids::PlayerId;
use crate::skill::DieSkill;
use crate::state::{CaptureScoring, GameState};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Holding {
    Kept,
    Captured(CaptureScoring),
}

/// R 5.1 + R 6.6 + R 6.7 + R 6.15 + R 7.1 + R 7.2: points of one die, in half-points.
pub(crate) fn die_points(die: &Die, holding: Holding) -> i32 {
    let mut null = false;
    let mut poison = false;
    let mut value = false;
    for skill in die.skills.iter() {
        match skill {
            DieSkill::Null => null = true,
            DieSkill::Poison => poison = true,
            DieSkill::Value => value = true,
            DieSkill::Berserk
            | DieSkill::Chance
            | DieSkill::Fire
            | DieSkill::Focus
            | DieSkill::Konstant
            | DieSkill::Queer
            | DieSkill::Reserve
            | DieSkill::Shadow
            | DieSkill::Speed
            | DieSkill::Stealth
            | DieSkill::Stinger
            | DieSkill::Trip => {}
        }
    }
    // R 6.6 + R 7.1.
    if null {
        return 0;
    }
    let basis = match holding {
        Holding::Captured(CaptureScoring::Zero) => return 0,
        Holding::Captured(CaptureScoring::ByValue(v)) => i32::from(v),
        Holding::Kept | Holding::Captured(CaptureScoring::BySize) => {
            if value { i32::from(die.value) } else { i32::from(die.max()) }
        }
    };
    let captured = matches!(holding, Holding::Captured(_));
    match (captured, poison) {
        // R 5.1: half the size kept, the full size captured.
        (false, false) => basis,
        (true, false) => 2 * basis,
        // R 6.7: minus the full size kept, minus half the size captured.
        (false, true) => -2 * basis,
        (true, true) => -basis,
    }
}

/// R 5.1: a player's current round score, in half-points.
pub fn round_score(state: &GameState, player: PlayerId) -> i32 {
    let p = state.player(player);
    let kept: i32 = p.active.iter().map(|d| die_points(d, Holding::Kept)).sum();
    let captured: i32 = p
        .captured
        .iter()
        .map(|c| die_points(&c.die, Holding::Captured(c.scoring)))
        .sum();
    kept + captured
}
```
(placer les tests unitaires de l'étape 1 à la fin de ce fichier).

Ajouter à `attack.rs` :
```rust
use crate::ids::DieId;

/// Non-empty subsets of `dice`, in a stable order (capped at 16 dice: no button comes close).
fn subsets(dice: &[&Die]) -> Vec<Vec<DieId>> {
    let n = dice.len().min(16);
    (1u32..(1u32 << n))
        .map(|mask| (0..n).filter(|i| mask & (1 << i) != 0).map(|i| dice[i].id).collect())
        .collect()
}

/// Every legal attack for `player` right now, without Fire assistance.
pub(crate) fn possible_attacks(state: &GameState, player: PlayerId) -> Vec<DeclaredAttack> {
    let mine: Vec<&Die> = state.player(player).active.iter().filter(|d| !d.dizzy).collect();
    let theirs: Vec<&Die> = state.player(player.other()).active.iter().collect();
    let mut candidates = Vec::new();
    for a in &mine {
        let kinds = hooks::attack_kinds(a);
        for t in &theirs {
            for kind in [AttackKind::Power, AttackKind::Shadow, AttackKind::Trip] {
                if kinds.contains(&kind) {
                    candidates.push(DeclaredAttack { kind, attackers: vec![a.id], targets: vec![t.id] });
                }
            }
        }
        for kind in [AttackKind::Speed, AttackKind::Berserk] {
            if kinds.contains(&kind) {
                for targets in subsets(&theirs) {
                    candidates.push(DeclaredAttack { kind, attackers: vec![a.id], targets });
                }
            }
        }
    }
    for t in &theirs {
        for attackers in subsets(&mine) {
            candidates.push(DeclaredAttack { kind: AttackKind::Skill, attackers, targets: vec![t.id] });
        }
    }
    candidates.retain(|attack| check_attack(state, player, attack).is_ok());
    candidates
}
```

`crates/engine/src/resolve.rs` :
```rust
use crate::action::{AttackKind, DeclaredAttack};
use crate::attack::check_attack;
use crate::error::RuleError;
use crate::event::GameEvent;
use crate::hooks;
use crate::ids::{DieId, PlayerId};
use crate::round;
use crate::skill::DieSkill;
use crate::state::{CapturedDie, GameState, WaitingFor};

/// Handles `Action::Attack`.
pub(crate) fn declare_attack(
    state: &mut GameState,
    player: PlayerId,
    attack: DeclaredAttack,
    events: &mut Vec<GameEvent>,
) -> Result<(), RuleError> {
    match state.waiting_for {
        WaitingFor::Attack { player: p } if p == player => {}
        WaitingFor::Attack { .. } => return Err(RuleError::NotYourTurn),
        _ => return Err(RuleError::WrongPhase),
    }
    check_attack(state, player, &attack).map_err(|reason| RuleError::IllegalAttack { reason })?;
    resolve_attack(state, player, attack, events);
    round::end_turn(state, player, events);
    Ok(())
}

/// R 4.4 + R 6.1 + R 6.14: carries out an attack already known to be legal.
pub(crate) fn resolve_attack(
    state: &mut GameState,
    player: PlayerId,
    attack: DeclaredAttack,
    events: &mut Vec<GameEvent>,
) {
    events.push(GameEvent::Attacked { player, attack: attack.clone() });
    state.consecutive_passes = 0;
    match attack.kind {
        AttackKind::Trip => resolve_trip(state, player, attack.attackers[0], attack.targets[0], events),
        AttackKind::Power
        | AttackKind::Skill
        | AttackKind::Shadow
        | AttackKind::Speed
        | AttackKind::Berserk => {
            for &target in &attack.targets {
                capture(state, player, target, &attack.attackers, events);
            }
            for &attacker in &attack.attackers {
                if attack.kind == AttackKind::Berserk {
                    shrink(state, player, attacker, events);
                }
                reroll(state, player, attacker, events);
            }
        }
    }
}

/// R 4.4: the target joins the attacker's score pile.
fn capture(
    state: &mut GameState,
    player: PlayerId,
    target: DieId,
    attackers: &[DieId],
    events: &mut Vec<GameEvent>,
) {
    let defender = player.other();
    let scoring = {
        let attacker_dice: Vec<_> = attackers.iter().filter_map(|&a| state.player(player).die(a)).collect();
        let target_die = state.player(defender).die(target).expect("legal target");
        hooks::capture_scoring(&attacker_dice, target_die)
    };
    let index = state
        .player(defender)
        .active
        .iter()
        .position(|d| d.id == target)
        .expect("legal target");
    let die = state.player_mut(defender).active.remove(index);
    state.player_mut(player).captured.push(CapturedDie { die, scoring });
    events.push(GameEvent::DieCaptured { by: player, die: target });
}

fn reroll(state: &mut GameState, owner: PlayerId, id: DieId, events: &mut Vec<GameEvent>) {
    let GameState { players, rng, .. } = state;
    if let Some(die) = players[owner.index()].die_mut(id) {
        hooks::reroll(die, rng);
        events.push(GameEvent::DieRerolled { die: id, value: die.value });
    }
}

/// R 6.1: half the sides (rounded up), and the die stops being Berserk.
fn shrink(state: &mut GameState, owner: PlayerId, id: DieId, events: &mut Vec<GameEvent>) {
    let die = state.player_mut(owner).die_mut(id).expect("legal attacker");
    die.sides = die.sides.halved();
    die.skills.remove(DieSkill::Berserk);
    events.push(GameEvent::DieShrunk { die: id, sides: die.sides });
}

/// R 6.14: both dice are rerolled; the target is captured if the Trip die shows at least as much.
fn resolve_trip(
    state: &mut GameState,
    player: PlayerId,
    attacker: DieId,
    target: DieId,
    events: &mut Vec<GameEvent>,
) {
    reroll(state, player, attacker, events);
    reroll(state, player.other(), target, events);
    let attacker_value = state.player(player).die(attacker).expect("legal attacker").value;
    let target_value = state.player(player.other()).die(target).expect("legal target").value;
    let captured = attacker_value >= target_value;
    events.push(GameEvent::TripRolled { attacker, attacker_value, target, target_value, captured });
    if captured {
        capture(state, player, target, &[attacker], events);
    }
}
```

Compléter `round.rs` :
```rust
use std::cmp::Ordering;

use crate::attack;
use crate::error::RuleError;
use crate::scoring;
use crate::setup;

/// After an attack or a pass: R 7.4 (dizziness lasts one turn), then R 4.5.
pub(crate) fn end_turn(state: &mut GameState, player: PlayerId, events: &mut Vec<GameEvent>) {
    for p in PlayerId::ALL {
        for die in &mut state.player_mut(p).active {
            die.dizzy = false;
        }
    }
    let out_of_dice = PlayerId::ALL.into_iter().any(|p| state.player(p).active.is_empty());
    if out_of_dice || state.consecutive_passes >= 2 {
        end_round(state, events);
    } else {
        state.waiting_for = WaitingFor::Attack { player: player.other() };
    }
}

/// R 4.1: pass only without any possible attack.
pub(crate) fn pass(
    state: &mut GameState,
    player: PlayerId,
    events: &mut Vec<GameEvent>,
) -> Result<(), RuleError> {
    match state.waiting_for {
        WaitingFor::Attack { player: p } if p == player => {}
        WaitingFor::Attack { .. } => return Err(RuleError::NotYourTurn),
        _ => return Err(RuleError::WrongPhase),
    }
    if !attack::possible_attacks(state, player).is_empty() {
        return Err(RuleError::PassNotAllowed);
    }
    state.consecutive_passes += 1;
    events.push(GameEvent::Passed { player });
    end_turn(state, player, events);
    Ok(())
}

/// R 5.2 + R 2.2 + R 2.3.
fn end_round(state: &mut GameState, events: &mut Vec<GameEvent>) {
    let scores = [
        scoring::round_score(state, PlayerId::One),
        scoring::round_score(state, PlayerId::Two),
    ];
    let winner = match scores[0].cmp(&scores[1]) {
        Ordering::Greater => Some(PlayerId::One),
        Ordering::Less => Some(PlayerId::Two),
        Ordering::Equal => None,
    };
    events.push(GameEvent::RoundEnded { round: state.round, scores, winner });
    match winner {
        Some(w) => {
            state.player_mut(w).record.wins += 1;
            state.player_mut(w.other()).record.losses += 1;
            state.last_round_loser = Some(w.other());
        }
        None => {
            for p in PlayerId::ALL {
                state.player_mut(p).record.draws += 1;
            }
            state.last_round_loser = None;
        }
    }
    let max_wins = state.config.max_wins;
    if let Some(champion) = PlayerId::ALL
        .into_iter()
        .find(|&p| state.player(p).record.wins >= max_wins)
    {
        let result = MatchResult { winner: champion, reason: EndReason::Rounds };
        events.push(GameEvent::MatchEnded { result });
        state.waiting_for = WaitingFor::GameOver { result };
        return;
    }
    state.round = state.round.saturating_add(1);
    setup::begin_round(state, events);
}
```

Dans `apply.rs`, importer `resolve` (`use crate::{initiative, resolve, round, setup};`) et câbler :
```rust
        Action::Attack { attack } => resolve::declare_attack(&mut next, player, attack, &mut events)?,
        Action::Pass => round::pass(&mut next, player, &mut events)?,
```
(le bras `WrongPhase` garde `ChooseReserve`, `FireTurnDown`, `CancelFire`).

Dans `lib.rs` : ajouter `mod resolve;`, `mod scoring;` et `pub use scoring::round_score;`.

- [ ] **Step 4: Lancer les tests, vérifier qu'ils passent**

Run: `cargo test -p buttonmen-engine && cargo clippy -p buttonmen-engine --all-targets --all-features -- -D warnings`
Expected: PASS (unitaires `scoring`, intégration `rounds`, et toutes les tâches précédentes).

- [ ] **Step 5: Commit**

```bash
git add crates/engine
git commit -m "feat(engine): résolution des attaques, passe, score et enchaînement des manches"
```

---
### Task 9: Aide Fire

**Files:**
- Create: `crates/engine/src/fire.rs`, `crates/engine/tests/fire.rs`
- Modify: `crates/engine/src/attack.rs`, `crates/engine/src/resolve.rs`, `crates/engine/src/round.rs`, `crates/engine/src/apply.rs`, `crates/engine/src/lib.rs`

**Interfaces:**
- Consumes: Tâches 7 et 8.
- Produces:
  - `attack::attack_shapes(&GameState, PlayerId) -> Vec<DeclaredAttack>` (toutes les attaques candidates, non filtrées) ; `possible_attacks` devient `attack_shapes` filtré.
  - `fire::{can_help(&GameState, PlayerId, &DeclaredAttack) -> bool, assisted_attacks(&GameState, PlayerId) -> Vec<DeclaredAttack>, fire_dice(&GameState, PlayerId, &DeclaredAttack) -> Vec<(DieId, u8)>, attacker_room(&GameState, PlayerId, &DeclaredAttack) -> Vec<(DieId, u8)>, fire_turn_down(...), cancel_fire(...)}`.
  - Comportement : une attaque Power ou Skill impossible seulement à cause des valeurs, mais rattrapable par Fire, mène à `WaitingFor::AdjustFire`.

- [ ] **Step 1: Écrire les tests**

`crates/engine/tests/fire.rs` :
```rust
mod common;

use buttonmen_engine::testing::{attack_state, die, id};
use buttonmen_engine::{
    Action, AttackError, AttackKind, DeclaredAttack, DieSkill, FireError, GameState, PlayerId,
    RuleError, WaitingFor,
};
use common::{err, ok};
use PlayerId::{One, Two};

fn declared(state: &GameState, kind: AttackKind, attackers: &[usize], targets: &[usize]) -> DeclaredAttack {
    DeclaredAttack {
        kind,
        attackers: attackers.iter().map(|&i| id(state, One, i)).collect(),
        targets: targets.iter().map(|&i| id(state, Two, i)).collect(),
    }
}

/// One: a d6 showing 3 and a Fire d10 showing 8. Two: a d6 showing 5 and a d20 showing 20.
fn power_needing_two() -> (GameState, DeclaredAttack) {
    let s = attack_state(
        vec![die(6, 3), die(10, 8).with(DieSkill::Fire)],
        vec![die(6, 5), die(20, 20)],
        One,
    );
    let attack = declared(&s, AttackKind::Power, &[0], &[0]);
    let (s, _) = ok(&s, One, Action::Attack { attack: attack.clone() });
    (s, attack)
}

#[test]
fn an_attack_fire_can_rescue_waits_for_adjustment() {
    let (s, attack) = power_needing_two();
    assert_eq!(s.waiting_for, WaitingFor::AdjustFire { player: One, attack });
}

#[test]
fn fire_turn_down_completes_the_attack() {
    let (s, _) = power_needing_two();
    let (attacker, fire) = (id(&s, One, 0), id(&s, One, 1));
    let (after, _) = ok(&s, One, Action::FireTurnDown { fire: vec![(fire, 2)], boosts: vec![(attacker, 2)] });
    assert_eq!(after.player(One).captured.len(), 1);
    assert_eq!(after.player(One).die(fire).unwrap().value, 6);
    assert_eq!(after.waiting_for, WaitingFor::Attack { player: Two });
}

#[test]
fn fire_adjustments_are_validated() {
    let (s, _) = power_needing_two();
    let (attacker, fire) = (id(&s, One, 0), id(&s, One, 1));
    let target = id(&s, Two, 0);
    let refuse = |fire_list: Vec<_>, boosts: Vec<_>| {
        err(&s, One, Action::FireTurnDown { fire: fire_list, boosts })
    };
    assert_eq!(
        refuse(vec![(fire, 3)], vec![(attacker, 2)]),
        RuleError::InvalidFire { reason: FireError::AmountsDiffer }
    );
    assert_eq!(
        refuse(vec![(fire, 1)], vec![(attacker, 1)]),
        RuleError::InvalidFire { reason: FireError::AttackStillInvalid }
    );
    assert_eq!(
        refuse(vec![(fire, 4)], vec![(attacker, 4)]),
        RuleError::InvalidFire { reason: FireError::OutOfRange { die: attacker } }
    );
    assert_eq!(
        refuse(vec![(attacker, 2)], vec![(attacker, 2)]),
        RuleError::InvalidFire { reason: FireError::NotFire { die: attacker } }
    );
    assert_eq!(
        refuse(vec![(fire, 2)], vec![(target, 2)]),
        RuleError::InvalidFire { reason: FireError::NotAttacker { die: target } }
    );
}

#[test]
fn cancel_fire_returns_to_the_attack_choice() {
    let (s, _) = power_needing_two();
    let (after, _) = ok(&s, One, Action::CancelFire);
    assert_eq!(after.waiting_for, WaitingFor::Attack { player: One });
    assert_eq!(after.player(One).active[1].value, 8);
}

#[test]
fn fire_helps_skill_attacks_across_several_attackers() {
    let s = attack_state(
        vec![die(6, 2), die(6, 2), die(6, 6).with(DieSkill::Fire)],
        vec![die(10, 7), die(20, 20)],
        One,
    );
    let attack = declared(&s, AttackKind::Skill, &[0, 1], &[0]);
    let (s, _) = ok(&s, One, Action::Attack { attack });
    let (a, b, f) = (id(&s, One, 0), id(&s, One, 1), id(&s, One, 2));
    let (after, _) = ok(&s, One, Action::FireTurnDown { fire: vec![(f, 3)], boosts: vec![(a, 1), (b, 2)] });
    assert_eq!(after.player(One).captured.len(), 1);
}

#[test]
fn fire_cannot_rescue_beyond_its_capacity_or_other_attack_kinds() {
    let s = attack_state(
        vec![die(6, 1), die(4, 2).with(DieSkill::Fire)],
        vec![die(6, 6), die(20, 20)],
        One,
    );
    // Need 5, the Fire die can give only 1.
    let attack = declared(&s, AttackKind::Power, &[0], &[0]);
    assert_eq!(
        err(&s, One, Action::Attack { attack }),
        RuleError::IllegalAttack { reason: AttackError::ValuesDoNotMatch }
    );
    let s = attack_state(
        vec![die(10, 5).with(DieSkill::Shadow), die(10, 10).with(DieSkill::Fire)],
        vec![die(4, 3), die(20, 20)],
        One,
    );
    let attack = declared(&s, AttackKind::Shadow, &[0], &[0]);
    assert_eq!(
        err(&s, One, Action::Attack { attack }),
        RuleError::IllegalAttack { reason: AttackError::ValuesDoNotMatch }
    );
}

#[test]
fn pass_refused_when_only_fire_assisted_attack_exists() {
    // Review Focus 2.
    let s = attack_state(vec![die(6, 3), die(10, 8).with(DieSkill::Fire)], vec![die(6, 5)], One);
    assert_eq!(err(&s, One, Action::Pass), RuleError::PassNotAllowed);
}
```

- [ ] **Step 2: Lancer les tests, vérifier qu'ils échouent**

Run: `cargo test -p buttonmen-engine --test fire`
Expected: FAIL — l'attaque est refusée (`ValuesDoNotMatch`) au lieu de passer en `AdjustFire`.

- [ ] **Step 3: Extraire `attack_shapes`**

Dans `attack.rs`, renommer le corps de `possible_attacks` en `attack_shapes` sans le filtre final, et redéfinir `possible_attacks` :
```rust
/// Every attack worth checking for `player` right now (not filtered for legality).
pub(crate) fn attack_shapes(state: &GameState, player: PlayerId) -> Vec<DeclaredAttack> {
    // … même corps que l'ancien `possible_attacks`, jusqu'à la construction de `candidates` …
    candidates
}

/// Every legal attack for `player` right now, without Fire assistance.
pub(crate) fn possible_attacks(state: &GameState, player: PlayerId) -> Vec<DeclaredAttack> {
    let mut attacks = attack_shapes(state, player);
    attacks.retain(|attack| check_attack(state, player, attack).is_ok());
    attacks
}
```
(Le corps est le code de la Tâche 8 sans la ligne `candidates.retain(...)`.)

- [ ] **Step 4: Implémenter `fire.rs`**

```rust
use std::collections::BTreeSet;

use crate::action::{AttackKind, DeclaredAttack};
use crate::attack::{attack_shapes, check_attack, skill_sums};
use crate::error::{AttackError, FireError, RuleError};
use crate::event::GameEvent;
use crate::ids::{DieId, PlayerId};
use crate::resolve;
use crate::round;
use crate::skill::DieSkill;
use crate::state::{GameState, WaitingFor};

/// R 6.3: Fire dice not taking part in `attack`, with how much each can give.
pub(crate) fn fire_dice(state: &GameState, player: PlayerId, attack: &DeclaredAttack) -> Vec<(DieId, u8)> {
    state
        .player(player)
        .active
        .iter()
        .filter(|d| d.has(DieSkill::Fire) && !attack.attackers.contains(&d.id))
        .map(|d| (d.id, d.value - d.min()))
        .filter(|(_, give)| *give > 0)
        .collect()
}

/// R 6.3: how much each attacker can be raised without leaving its range.
pub(crate) fn attacker_room(state: &GameState, player: PlayerId, attack: &DeclaredAttack) -> Vec<(DieId, u8)> {
    attack
        .attackers
        .iter()
        .filter_map(|&id| state.player(player).die(id))
        .map(|d| (d.id, d.max() - d.value))
        .collect()
}

/// R 6.3: is `attack` a Power or Skill attack that fails only on values, and that
/// Fire can make legal?
pub(crate) fn can_help(state: &GameState, player: PlayerId, attack: &DeclaredAttack) -> bool {
    if !matches!(attack.kind, AttackKind::Power | AttackKind::Skill) {
        return false;
    }
    if check_attack(state, player, attack) != Err(AttackError::ValuesDoNotMatch) {
        return false;
    }
    let capacity: u32 = fire_dice(state, player, attack).iter().map(|(_, g)| u32::from(*g)).sum();
    let room: u32 = attacker_room(state, player, attack).iter().map(|(_, r)| u32::from(*r)).sum();
    let budget = i32::try_from(capacity.min(room)).unwrap_or(i32::MAX);
    let me = state.player(player);
    let target = state
        .player(player.other())
        .die(attack.targets[0])
        .expect("checked by check_attack")
        .value;
    let target = i32::from(target);
    match attack.kind {
        AttackKind::Power => {
            let a = i32::from(me.die(attack.attackers[0]).expect("checked").value);
            target - a <= budget
        }
        AttackKind::Skill => {
            let attackers: Vec<_> = attack.attackers.iter().filter_map(|&id| me.die(id)).collect();
            skill_sums(&attackers).iter().any(|s| *s < target && target - s <= budget)
        }
        AttackKind::Shadow | AttackKind::Speed | AttackKind::Trip | AttackKind::Berserk => false,
    }
}

/// Attacks that are possible only with Fire assistance (they count for R 4.1).
pub(crate) fn assisted_attacks(state: &GameState, player: PlayerId) -> Vec<DeclaredAttack> {
    let mut attacks = attack_shapes(state, player);
    attacks.retain(|attack| can_help(state, player, attack));
    attacks
}

fn fire_error(reason: FireError) -> RuleError {
    RuleError::InvalidFire { reason }
}

/// Handles `Action::FireTurnDown` (R 6.3).
pub(crate) fn fire_turn_down(
    state: &mut GameState,
    player: PlayerId,
    fire: Vec<(DieId, u8)>,
    boosts: Vec<(DieId, u8)>,
    events: &mut Vec<GameEvent>,
) -> Result<(), RuleError> {
    let attack = match &state.waiting_for {
        WaitingFor::AdjustFire { player: p, attack } if *p == player => attack.clone(),
        WaitingFor::AdjustFire { .. } => return Err(RuleError::NotYourTurn),
        _ => return Err(RuleError::WrongPhase),
    };
    let mut seen = BTreeSet::new();
    for &(id, amount) in &fire {
        let die = state.player(player).die(id).ok_or(RuleError::UnknownDie { die: id })?;
        if !die.has(DieSkill::Fire) || attack.attackers.contains(&id) || !seen.insert(id) {
            return Err(fire_error(FireError::NotFire { die: id }));
        }
        if amount == 0 || u16::from(die.value) < u16::from(die.min()) + u16::from(amount) {
            return Err(fire_error(FireError::OutOfRange { die: id }));
        }
    }
    for &(id, amount) in &boosts {
        if !attack.attackers.contains(&id) || !seen.insert(id) {
            return Err(fire_error(FireError::NotAttacker { die: id }));
        }
        let die = state.player(player).die(id).expect("attackers are the player's dice");
        if amount == 0 || u16::from(die.value) + u16::from(amount) > u16::from(die.max()) {
            return Err(fire_error(FireError::OutOfRange { die: id }));
        }
    }
    let total = |list: &[(DieId, u8)]| list.iter().map(|(_, a)| u32::from(*a)).sum::<u32>();
    if total(&fire) != total(&boosts) {
        return Err(fire_error(FireError::AmountsDiffer));
    }
    let me = state.player_mut(player);
    for &(id, amount) in &fire {
        me.die_mut(id).expect("checked").value -= amount;
    }
    for &(id, amount) in &boosts {
        me.die_mut(id).expect("checked").value += amount;
    }
    if check_attack(state, player, &attack).is_err() {
        return Err(fire_error(FireError::AttackStillInvalid));
    }
    events.push(GameEvent::FireUsed { player, fire, boosts });
    resolve::resolve_attack(state, player, attack, events);
    round::end_turn(state, player, events);
    Ok(())
}

/// Handles `Action::CancelFire`: back to choosing an attack.
pub(crate) fn cancel_fire(state: &mut GameState, player: PlayerId) -> Result<(), RuleError> {
    match state.waiting_for {
        WaitingFor::AdjustFire { player: p, .. } if p == player => {
            state.waiting_for = WaitingFor::Attack { player };
            Ok(())
        }
        WaitingFor::AdjustFire { .. } => Err(RuleError::NotYourTurn),
        _ => Err(RuleError::WrongPhase),
    }
}
```

Dans `resolve.rs`, `declare_attack` : remplacer
```rust
    check_attack(state, player, &attack).map_err(|reason| RuleError::IllegalAttack { reason })?;
```
par
```rust
    if let Err(reason) = check_attack(state, player, &attack) {
        // R 6.3: Fire may still make a Power or Skill attack legal.
        if crate::fire::can_help(state, player, &attack) {
            state.waiting_for = WaitingFor::AdjustFire { player, attack };
            return Ok(());
        }
        return Err(RuleError::IllegalAttack { reason });
    }
```

Dans `round.rs`, `pass` : remplacer la condition par
```rust
    // R 4.1 + R 6.3: an attack that Fire makes possible also forbids passing.
    if !attack::possible_attacks(state, player).is_empty()
        || !crate::fire::assisted_attacks(state, player).is_empty()
    {
        return Err(RuleError::PassNotAllowed);
    }
```

Dans `apply.rs`, importer `fire` et câbler :
```rust
        Action::FireTurnDown { fire: amounts, boosts } => {
            fire::fire_turn_down(&mut next, player, amounts, boosts, &mut events)?
        }
        Action::CancelFire => fire::cancel_fire(&mut next, player)?,
```
(le bras `WrongPhase` ne garde que `ChooseReserve`).

Dans `lib.rs` : ajouter `mod fire;`.

- [ ] **Step 5: Lancer les tests, vérifier qu'ils passent**

Run: `cargo test -p buttonmen-engine && cargo clippy -p buttonmen-engine --all-targets --all-features -- -D warnings`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add crates/engine
git commit -m "feat(engine): aide Fire (AdjustFire)"
```

---

### Task 10: Dés Reserve

**Files:**
- Modify: `crates/engine/src/setup.rs`, `crates/engine/src/apply.rs`
- Create: `crates/engine/tests/reserve.rs`

**Interfaces:**
- Consumes: Tâches 5 et 8.
- Produces: `setup::choose_reserve(&mut GameState, PlayerId, Option<DieId>, &mut Vec<GameEvent>) -> Result<(), RuleError>`. Après cette tâche, `apply` n'a plus de bras `WrongPhase` générique.

- [ ] **Step 1: Écrire les tests**

`crates/engine/tests/reserve.rs` :
```rust
mod common;

use buttonmen_engine::testing::{button, fixed};
use buttonmen_engine::{
    Action, AttackKind, DeclaredAttack, DieSkill, DieShape, DieSpec, GameState, MatchConfig,
    PlayerId, RuleError, SpecChoice, SwingLetter, WaitingFor, new_game,
};
use common::{err, ok};
use PlayerId::{One, Two};

/// One has two d1 against Two's single d1 (plus Two's reserve dice, out of play):
/// One always starts (longer list, R 3.3), captures Two's only die, and Two loses round 1.
fn after_first_round(two_dice: Vec<DieSpec>) -> GameState {
    let state = new_game(
        [button("A", vec![fixed(1), fixed(1)]), button("B", two_dice)],
        4,
        MatchConfig::default(),
    );
    assert_eq!(state.waiting_for, WaitingFor::Attack { player: One });
    let attack = DeclaredAttack {
        kind: AttackKind::Power,
        attackers: vec![state.player(One).active[0].id],
        targets: vec![state.player(Two).active[0].id],
    };
    let (state, _) = ok(&state, One, Action::Attack { attack });
    assert_eq!(state.last_round_loser, Some(Two));
    state
}

#[test]
fn the_loser_may_add_a_reserve_die() {
    let state = after_first_round(vec![fixed(1), fixed(20).with(DieSkill::Reserve)]);
    assert_eq!(state.waiting_for, WaitingFor::ChooseReserve { player: Two });
    let reserve = state.player(Two).slots[1].id;
    let (state, _) = ok(&state, Two, Action::ChooseReserve { die: Some(reserve) });
    assert!(state.player(Two).slots[1].in_play);
    assert!(!state.player(Two).slots[1].spec.skills.has(DieSkill::Reserve));
    assert_eq!(state.player(Two).active.len(), 2);
}

#[test]
fn reserve_choice_is_validated_and_may_be_declined() {
    let state = after_first_round(vec![fixed(1), fixed(20).with(DieSkill::Reserve)]);
    let not_reserve = state.player(Two).slots[0].id;
    assert_eq!(
        err(&state, Two, Action::ChooseReserve { die: Some(not_reserve) }),
        RuleError::InvalidReserve { die: not_reserve }
    );
    assert_eq!(err(&state, One, Action::ChooseReserve { die: None }), RuleError::NotYourTurn);
    let (state, _) = ok(&state, Two, Action::ChooseReserve { die: None });
    assert_eq!(state.player(Two).active.len(), 1);
}

#[test]
fn a_reserve_swing_die_must_be_sized() {
    let reserve_x = DieSpec::new(DieShape::Swing(SwingLetter::X)).with(DieSkill::Reserve);
    let state = after_first_round(vec![fixed(1), reserve_x]);
    let reserve = state.player(Two).slots[1].id;
    let (state, _) = ok(&state, Two, Action::ChooseReserve { die: Some(reserve) });
    assert_eq!(state.waiting_for, WaitingFor::SpecifyDice { pending: vec![Two] });
    let x = Action::SpecifyDice { choice: SpecChoice { swing: vec![(SwingLetter::X, 12)], options: vec![] } };
    let (state, _) = ok(&state, Two, x);
    assert_eq!(state.player(Two).active[1].max(), 12);
}
```

- [ ] **Step 2: Lancer les tests, vérifier qu'ils échouent**

Run: `cargo test -p buttonmen-engine --test reserve`
Expected: FAIL — `ChooseReserve` renvoie `WrongPhase`.

- [ ] **Step 3: Implémenter**

Dans `setup.rs`, ajouter :
```rust
/// Handles `Action::ChooseReserve` (R 6.11): the chosen die joins play for the rest of
/// the match and stops being a Reserve die.
pub(crate) fn choose_reserve(
    state: &mut GameState,
    player: PlayerId,
    die: Option<DieId>,
    events: &mut Vec<GameEvent>,
) -> Result<(), RuleError> {
    match state.waiting_for {
        WaitingFor::ChooseReserve { player: p } if p == player => {}
        WaitingFor::ChooseReserve { .. } => return Err(RuleError::NotYourTurn),
        _ => return Err(RuleError::WrongPhase),
    }
    if let Some(id) = die {
        let slot = state
            .player_mut(player)
            .slots
            .iter_mut()
            .find(|s| s.id == id && !s.in_play && s.spec.skills.has(DieSkill::Reserve))
            .ok_or(RuleError::InvalidReserve { die: id })?;
        slot.in_play = true;
        slot.spec.skills.remove(DieSkill::Reserve);
        events.push(GameEvent::ReserveAdded { player, die: id });
    }
    ask_for_choices(state, events);
    Ok(())
}
```

Dans `apply.rs`, remplacer le dernier bras `WrongPhase` par :
```rust
        Action::ChooseReserve { die } => setup::choose_reserve(&mut next, player, die, &mut events)?,
```

- [ ] **Step 4: Lancer les tests, vérifier qu'ils passent**

Run: `cargo test -p buttonmen-engine && cargo clippy -p buttonmen-engine --all-targets --all-features -- -D warnings`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/engine
git commit -m "feat(engine): dés Reserve"
```

---
### Task 11: `legal_actions`, `candidate_actions` et tests de propriétés

**Files:**
- Create: `crates/engine/src/legal.rs`, `crates/engine/tests/legal.rs`, `crates/engine/tests/properties.rs`
- Modify: `crates/engine/src/lib.rs`

**Interfaces:**
- Consumes: Tâches 5 à 10.
- Produces (utilisé par la vue, le WASM, le client et l'IA du jalon 2) :
  - `pub enum LegalActions { Wait, SpecifyDice { swing: Vec<SwingRequest>, options: Vec<OptionRequest> }, ChooseReserve { dice: Vec<DieId> }, ReactToInitiative { focus: Vec<FocusRange>, chance: Vec<DieId> }, Attack { attacks: Vec<AttackOption>, can_pass: bool }, AdjustFire { attack: DeclaredAttack, fire: Vec<FireCapacity>, boosts: Vec<FireCapacity> }, GameOver }`
  - `SwingRequest { letter, min, max, current: Option<u8> }`, `OptionRequest { die, a, b, current: Option<u8> }`, `FocusRange { die, min, current }`, `AttackOption { attack, needs_fire }`, `FireCapacity { die, max_amount }`.
  - `pub fn legal_actions(&GameState, PlayerId) -> LegalActions` ; `pub fn candidate_actions(&GameState, PlayerId) -> Vec<Action>` (jamais vide pour un joueur qui doit agir ; chaque action est acceptée par `apply`).

- [ ] **Step 1: Écrire les tests**

`crates/engine/tests/legal.rs` :
```rust
mod common;

use buttonmen_engine::testing::{attack_state, button, die, fixed, id, opening_state};
use buttonmen_engine::{
    Action, AttackKind, DieShape, DieSkill, DieSpec, LegalActions, MatchConfig, PlayerId,
    SwingLetter, apply, candidate_actions, legal_actions, new_game,
};
use common::ok;
use PlayerId::{One, Two};

#[test]
fn only_actors_get_actions() {
    let s = attack_state(vec![die(6, 4)], vec![die(6, 3)], One);
    assert_eq!(legal_actions(&s, Two), LegalActions::Wait);
    assert!(candidate_actions(&s, Two).is_empty());
}

#[test]
fn specify_dice_lists_ranges() {
    let s = new_game(
        [button("A", vec![DieSpec::new(DieShape::Swing(SwingLetter::X))]), button("B", vec![fixed(6)])],
        1,
        MatchConfig::default(),
    );
    let LegalActions::SpecifyDice { swing, options } = legal_actions(&s, One) else {
        panic!("expected SpecifyDice")
    };
    assert_eq!((swing[0].letter, swing[0].min, swing[0].max, swing[0].current), (SwingLetter::X, 4, 20, None));
    assert!(options.is_empty());
    for action in candidate_actions(&s, One) {
        assert!(apply(&s, One, action).is_ok());
    }
}

#[test]
fn attack_lists_every_legal_attack_and_forbids_pass() {
    let s = attack_state(vec![die(6, 4), die(6, 2)], vec![die(6, 3), die(6, 6)], One);
    let LegalActions::Attack { attacks, can_pass } = legal_actions(&s, One) else {
        panic!("expected Attack")
    };
    assert!(!can_pass);
    // Power 4→3 and Skill 4+2→6 are both possible.
    let kinds: Vec<_> = attacks.iter().map(|a| (a.attack.kind, a.attack.attackers.len())).collect();
    assert!(kinds.contains(&(AttackKind::Power, 1)));
    assert!(kinds.contains(&(AttackKind::Skill, 2)));
    assert!(attacks.iter().all(|a| !a.needs_fire));
    for option in attacks {
        assert!(apply(&s, One, Action::Attack { attack: option.attack }).is_ok());
    }
}

#[test]
fn stuck_players_can_pass() {
    let s = attack_state(vec![die(6, 1)], vec![die(6, 6)], One);
    assert_eq!(legal_actions(&s, One), LegalActions::Attack { attacks: vec![], can_pass: true });
    assert_eq!(candidate_actions(&s, One), vec![Action::Pass]);
}

#[test]
fn fire_assisted_attacks_are_flagged() {
    let s = attack_state(vec![die(6, 3), die(10, 8).with(DieSkill::Fire)], vec![die(6, 5)], One);
    let LegalActions::Attack { attacks, can_pass } = legal_actions(&s, One) else {
        panic!("expected Attack")
    };
    assert!(!can_pass);
    assert!(attacks.iter().any(|a| a.needs_fire && a.attack.kind == AttackKind::Power));
}

#[test]
fn adjust_fire_offers_capacities_and_working_candidates() {
    let s = attack_state(vec![die(6, 3), die(10, 8).with(DieSkill::Fire)], vec![die(6, 5), die(20, 20)], One);
    let attack = buttonmen_engine::DeclaredAttack {
        kind: AttackKind::Power,
        attackers: vec![id(&s, One, 0)],
        targets: vec![id(&s, Two, 0)],
    };
    let (s, _) = ok(&s, One, Action::Attack { attack });
    let LegalActions::AdjustFire { fire, boosts, .. } = legal_actions(&s, One) else {
        panic!("expected AdjustFire")
    };
    assert_eq!(fire[0].max_amount, 7);
    assert_eq!(boosts[0].max_amount, 3);
    let candidates = candidate_actions(&s, One);
    assert!(candidates.contains(&Action::CancelFire));
    assert!(candidates.iter().any(|a| matches!(a, Action::FireTurnDown { .. })));
}

#[test]
fn reactions_offer_pass_chance_and_winning_focus() {
    let s = opening_state(vec![die(10, 3)], vec![die(10, 5).with(DieSkill::Focus)]);
    let LegalActions::ReactToInitiative { focus, chance } = legal_actions(&s, Two) else {
        panic!("expected ReactToInitiative")
    };
    assert_eq!((focus[0].min, focus[0].current), (1, 5));
    assert!(chance.is_empty());
    let candidates = candidate_actions(&s, Two);
    assert!(candidates.contains(&Action::PassInitiative));
    assert!(candidates.iter().any(|a| matches!(a, Action::FocusTurnDown { .. })));
}
```

`crates/engine/tests/properties.rs` :
```rust
use buttonmen_engine::testing::{button, fixed};
use buttonmen_engine::{
    Action, AttackKind, DeclaredAttack, DieId, DieShape, DieSkill, DieSpec, GameState, LegalActions,
    MatchConfig, PlayerId, SeededRng, SwingLetter, TwinPart, WaitingFor, apply, candidate_actions,
    legal_actions, new_game,
};
use proptest::prelude::*;

fn swing(l: SwingLetter) -> DieSpec {
    DieSpec::new(DieShape::Swing(l))
}

/// Buttons covering every v1 skill and die shape.
fn pool() -> Vec<Vec<DieSpec>> {
    use DieSkill::*;
    vec![
        vec![fixed(4), fixed(4), fixed(10), fixed(12), swing(SwingLetter::X)],
        vec![fixed(6).with(Poison), fixed(8).with(Shadow), fixed(10), fixed(12).with(Speed), swing(SwingLetter::Y)],
        vec![fixed(4).with(Trip), fixed(6).with(Berserk), fixed(8).with(Focus), fixed(10).with(Chance), DieSpec::new(DieShape::Option(6, 12))],
        vec![
            fixed(6).with(Konstant),
            fixed(8).with(Stinger),
            fixed(10).with(Queer),
            DieSpec::new(DieShape::Twin(TwinPart::Fixed(4), TwinPart::Fixed(4))),
            fixed(20).with(Null),
        ],
        vec![
            fixed(4).with(Stealth),
            fixed(6).with(Value),
            fixed(10).with(Fire),
            fixed(12),
            fixed(20).with(Reserve),
            swing(SwingLetter::Z).with(Reserve),
        ],
        vec![fixed(4).with(Stealth), fixed(6).with(Stealth), fixed(8), DieSpec::new(DieShape::Twin(TwinPart::Swing(SwingLetter::X), TwinPart::Swing(SwingLetter::X)))],
    ]
}

fn total_dice(state: &GameState) -> usize {
    PlayerId::ALL
        .iter()
        .map(|&p| {
            let ps = state.player(p);
            ps.active.len() + ps.captured.len()
        })
        .sum()
}

fn in_play_slots(state: &GameState) -> usize {
    PlayerId::ALL
        .iter()
        .map(|&p| state.player(p).slots.iter().filter(|s| s.in_play).count())
        .sum()
}

/// A random attack shape: random kind, random subsets of each side's dice (in table order,
/// like the engine's own enumeration).
fn random_attack(state: &GameState, actor: PlayerId, rng: &mut SeededRng) -> DeclaredAttack {
    let kind = AttackKind::ALL[(rng.next_u64() % AttackKind::ALL.len() as u64) as usize];
    let mut pick = |p: PlayerId| -> Vec<DieId> {
        state.player(p).active.iter().map(|d| d.id).filter(|_| rng.coin()).collect()
    };
    let attackers = pick(actor);
    let targets = pick(actor.other());
    DeclaredAttack { kind, attackers, targets }
}

/// Plays a whole match with random candidate actions, checking invariants on the way.
fn play(a: usize, b: usize, seed: u64) -> Result<(), TestCaseError> {
    let pool = pool();
    let mut state = new_game(
        [button("A", pool[a].clone()), button("B", pool[b].clone())],
        seed,
        MatchConfig::default(),
    );
    let mut chooser = SeededRng::new(seed ^ 0xABCD);
    for _ in 0..20_000 {
        if let WaitingFor::GameOver { .. } = state.waiting_for {
            return Ok(());
        }
        let actor = state.actors()[0];
        if let LegalActions::Attack { attacks, .. } = legal_actions(&state, actor) {
            // Conservation: every die in play is either active or captured.
            prop_assert_eq!(total_dice(&state), in_play_slots(&state));
            // Every listed attack is accepted.
            for option in &attacks {
                prop_assert!(apply(&state, actor, Action::Attack { attack: option.attack.clone() }).is_ok());
            }
            // No attack outside the list is accepted.
            for _ in 0..8 {
                let attack = random_attack(&state, actor, &mut chooser);
                if apply(&state, actor, Action::Attack { attack: attack.clone() }).is_ok() {
                    prop_assert!(
                        attacks.iter().any(|o| o.attack == attack),
                        "accepted but not listed: {:?}",
                        attack
                    );
                }
            }
        }
        let candidates = candidate_actions(&state, actor);
        prop_assert!(!candidates.is_empty(), "no action for {:?} in {:?}", actor, state.waiting_for);
        let pick = candidates[(chooser.next_u64() % candidates.len() as u64) as usize].clone();
        let (next, _) = apply(&state, actor, pick).map_err(|e| TestCaseError::fail(e.to_string()))?;
        state = next;
    }
    Err(TestCaseError::fail("match did not end within 20 000 actions"))
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn random_matches_end_and_keep_invariants(a in 0usize..6, b in 0usize..6, seed in any::<u64>()) {
        play(a, b, seed)?;
    }

    #[test]
    fn surrender_is_always_accepted(a in 0usize..6, b in 0usize..6, seed in any::<u64>()) {
        let pool = pool();
        let state = new_game([button("A", pool[a].clone()), button("B", pool[b].clone())], seed, MatchConfig::default());
        prop_assert!(apply(&state, PlayerId::Two, Action::Surrender).is_ok());
    }
}
```

- [ ] **Step 2: Lancer les tests, vérifier qu'ils échouent**

Run: `cargo test -p buttonmen-engine --test legal --test properties`
Expected: échec de compilation (`legal_actions`, `LegalActions`… introuvables).

- [ ] **Step 3: Implémenter `legal.rs`**

```rust
use serde::{Deserialize, Serialize};

use crate::action::{Action, DeclaredAttack, SpecChoice};
use crate::apply::apply;
use crate::attack;
use crate::die::SwingLetter;
use crate::fire;
use crate::ids::{DieId, PlayerId};
use crate::setup::{option_slots, required_letters};
use crate::skill::DieSkill;
use crate::state::{GameState, WaitingFor};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct SwingRequest {
    pub letter: SwingLetter,
    pub min: u8,
    pub max: u8,
    pub current: Option<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct OptionRequest {
    pub die: DieId,
    pub a: u8,
    pub b: u8,
    pub current: Option<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct FocusRange {
    pub die: DieId,
    pub min: u8,
    pub current: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct AttackOption {
    pub attack: DeclaredAttack,
    /// R 6.3: legal only once Fire dice are turned down.
    pub needs_fire: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct FireCapacity {
    pub die: DieId,
    pub max_amount: u8,
}

/// What `player` may do now, described for an interface.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[serde(tag = "type")]
pub enum LegalActions {
    Wait,
    SpecifyDice { swing: Vec<SwingRequest>, options: Vec<OptionRequest> },
    ChooseReserve { dice: Vec<DieId> },
    ReactToInitiative { focus: Vec<FocusRange>, chance: Vec<DieId> },
    Attack { attacks: Vec<AttackOption>, can_pass: bool },
    AdjustFire { attack: DeclaredAttack, fire: Vec<FireCapacity>, boosts: Vec<FireCapacity> },
    GameOver,
}

pub fn legal_actions(state: &GameState, player: PlayerId) -> LegalActions {
    let me = state.player(player);
    match &state.waiting_for {
        WaitingFor::GameOver { .. } => LegalActions::GameOver,
        WaitingFor::SpecifyDice { pending } if pending.contains(&player) => LegalActions::SpecifyDice {
            swing: required_letters(me)
                .into_iter()
                .map(|letter| SwingRequest {
                    letter,
                    min: *letter.range().start(),
                    max: *letter.range().end(),
                    current: me.choice.swing_size(letter),
                })
                .collect(),
            options: option_slots(me)
                .into_iter()
                .map(|(die, a, b)| OptionRequest { die, a, b, current: me.choice.option_size(die) })
                .collect(),
        },
        WaitingFor::ChooseReserve { player: p } if *p == player => LegalActions::ChooseReserve {
            dice: me
                .slots
                .iter()
                .filter(|s| !s.in_play && s.spec.skills.has(DieSkill::Reserve))
                .map(|s| s.id)
                .collect(),
        },
        WaitingFor::ReactToInitiative { player: p } if *p == player => LegalActions::ReactToInitiative {
            focus: me
                .active
                .iter()
                .filter(|d| d.has(DieSkill::Focus) && d.value > d.min())
                .map(|d| FocusRange { die: d.id, min: d.min(), current: d.value })
                .collect(),
            chance: me.active.iter().filter(|d| d.has(DieSkill::Chance)).map(|d| d.id).collect(),
        },
        WaitingFor::Attack { player: p } if *p == player => {
            let mut attacks: Vec<AttackOption> = attack::possible_attacks(state, player)
                .into_iter()
                .map(|attack| AttackOption { attack, needs_fire: false })
                .collect();
            attacks.extend(
                fire::assisted_attacks(state, player)
                    .into_iter()
                    .map(|attack| AttackOption { attack, needs_fire: true }),
            );
            let can_pass = attacks.is_empty();
            LegalActions::Attack { attacks, can_pass }
        }
        WaitingFor::AdjustFire { player: p, attack } if *p == player => LegalActions::AdjustFire {
            attack: attack.clone(),
            fire: fire::fire_dice(state, player, attack)
                .into_iter()
                .map(|(die, max_amount)| FireCapacity { die, max_amount })
                .collect(),
            boosts: fire::attacker_room(state, player, attack)
                .into_iter()
                .filter(|(_, room)| *room > 0)
                .map(|(die, max_amount)| FireCapacity { die, max_amount })
                .collect(),
        },
        WaitingFor::SpecifyDice { .. }
        | WaitingFor::ChooseReserve { .. }
        | WaitingFor::ReactToInitiative { .. }
        | WaitingFor::Attack { .. }
        | WaitingFor::AdjustFire { .. } => LegalActions::Wait,
    }
}

const MAX_SPEC_CANDIDATES: usize = 64;

fn spec_candidates(swing: &[SwingRequest], options: &[OptionRequest]) -> Vec<Action> {
    let mut choices = vec![SpecChoice::default()];
    for req in swing {
        let mut sizes = vec![req.min, req.min + (req.max - req.min) / 2, req.max];
        sizes.dedup();
        let mut next = Vec::new();
        for c in &choices {
            for &size in &sizes {
                let mut c = c.clone();
                c.swing.push((req.letter, size));
                next.push(c);
            }
        }
        next.truncate(MAX_SPEC_CANDIDATES);
        choices = next;
    }
    for req in options {
        let mut sizes = vec![req.a, req.b];
        sizes.dedup();
        let mut next = Vec::new();
        for c in &choices {
            for &size in &sizes {
                let mut c = c.clone();
                c.options.push((req.die, size));
                next.push(c);
            }
        }
        next.truncate(MAX_SPEC_CANDIDATES);
        choices = next;
    }
    choices.into_iter().map(|choice| Action::SpecifyDice { choice }).collect()
}

fn focus_candidates(focus: &[FocusRange]) -> Vec<Action> {
    let mut actions: Vec<Action> = focus
        .iter()
        .flat_map(|f| (f.min..f.current).map(move |v| Action::FocusTurnDown { dice: vec![(f.die, v)] }))
        .collect();
    if focus.len() > 1 {
        actions.push(Action::FocusTurnDown { dice: focus.iter().map(|f| (f.die, f.min)).collect() });
    }
    actions
}

/// Spreads `total` greedily over `capacities`, in order.
fn spread(total: u8, capacities: &[FireCapacity]) -> Vec<(DieId, u8)> {
    let mut left = total;
    let mut out = Vec::new();
    for c in capacities {
        if left == 0 {
            break;
        }
        let take = left.min(c.max_amount);
        if take > 0 {
            out.push((c.die, take));
            left -= take;
        }
    }
    out
}

fn fire_candidates(fire: &[FireCapacity], boosts: &[FireCapacity]) -> Vec<Action> {
    let sum = |list: &[FireCapacity]| list.iter().map(|c| u32::from(c.max_amount)).sum::<u32>();
    let limit = u8::try_from(sum(fire).min(sum(boosts))).unwrap_or(u8::MAX);
    (1..=limit)
        .map(|total| Action::FireTurnDown { fire: spread(total, fire), boosts: spread(total, boosts) })
        .collect()
}

/// Concrete actions for `player`, all accepted by `apply` (for the AI and for tests).
/// Never empty for a player who must act. Surrender is left out.
pub fn candidate_actions(state: &GameState, player: PlayerId) -> Vec<Action> {
    let raw = match legal_actions(state, player) {
        LegalActions::Wait | LegalActions::GameOver => Vec::new(),
        LegalActions::SpecifyDice { swing, options } => spec_candidates(&swing, &options),
        LegalActions::ChooseReserve { dice } => std::iter::once(None)
            .chain(dice.into_iter().map(Some))
            .map(|die| Action::ChooseReserve { die })
            .collect(),
        LegalActions::ReactToInitiative { focus, chance } => {
            let mut actions = vec![Action::PassInitiative];
            actions.extend(chance.into_iter().map(|die| Action::ChanceReroll { die }));
            actions.extend(focus_candidates(&focus));
            actions
        }
        LegalActions::Attack { attacks, can_pass } => {
            let mut actions: Vec<Action> =
                attacks.into_iter().map(|o| Action::Attack { attack: o.attack }).collect();
            if can_pass {
                actions.push(Action::Pass);
            }
            actions
        }
        LegalActions::AdjustFire { fire, boosts, .. } => {
            let mut actions = vec![Action::CancelFire];
            actions.extend(fire_candidates(&fire, &boosts));
            actions
        }
    };
    raw.into_iter().filter(|a| apply(state, player, a.clone()).is_ok()).collect()
}
```

Dans `lib.rs` : ajouter `mod legal;` et
```rust
pub use legal::{
    AttackOption, FireCapacity, FocusRange, LegalActions, OptionRequest, SwingRequest,
    candidate_actions, legal_actions,
};
```

- [ ] **Step 4: Lancer les tests, vérifier qu'ils passent**

Run: `cargo test -p buttonmen-engine && cargo clippy -p buttonmen-engine --all-targets --all-features -- -D warnings`
Expected: PASS. Si `random_matches_end_and_keep_invariants` échoue, proptest affiche le cas réduit (`a`, `b`, `seed`) : c'est un vrai défaut du moteur (blocage, invariant cassé ou action listée mais refusée). Le corriger dans le module concerné (avec un test de régression dans le fichier de test de ce module), jamais en assouplissant la propriété.

- [ ] **Step 5: Commit**

```bash
git add crates/engine
git commit -m "feat(engine): actions légales, actions candidates et tests de propriétés"
```

---
### Task 12: Vue par siège (`GameView`)

**Files:**
- Create: `crates/engine/src/view.rs`, `crates/engine/tests/view.rs`
- Modify: `crates/engine/src/lib.rs`

**Interfaces:**
- Consumes: Tâches 5 à 11.
- Produces : `Viewer { Seat { player }, Spectator }` ; `DieView { id, notation, skills, sides, min, max, value, dizzy }` ; `CapturedView { die, points }` ; `SlotView { id, notation }` ; `SwingView { letter, size }` ; `PlayerView { seat, button, recipe, active, captured, reserve, record, round_score, swing, has_chosen, pending_choice }` ; `GameView { viewer, round, max_wins, initiative, players: Vec<PlayerView>, waiting_for, actors, legal }` ; `pub fn view_for(&GameState, Viewer) -> GameView`. Les points et scores sont en demi-points.

- [ ] **Step 1: Écrire les tests**

`crates/engine/tests/view.rs` :
```rust
mod common;

use buttonmen_engine::testing::{attack_state, button, die, fixed};
use buttonmen_engine::{
    Action, DieShape, DieSkill, DieSpec, LegalActions, MatchConfig, PlayerId, SpecChoice,
    SwingLetter, Viewer, view_for, new_game,
};
use common::ok;
use PlayerId::{One, Two};

#[test]
fn pending_swing_hidden_from_opponent() {
    // Review Focus 4.
    let x = || DieSpec::new(DieShape::Swing(SwingLetter::X));
    let s = new_game(
        [button("A", vec![fixed(6), x()]), button("B", vec![fixed(6), x()])],
        1,
        MatchConfig::default(),
    );
    let choice = SpecChoice { swing: vec![(SwingLetter::X, 17)], options: vec![] };
    let (s, _) = ok(&s, One, Action::SpecifyDice { choice: choice.clone() });
    let theirs = view_for(&s, Viewer::Seat { player: Two });
    assert!(theirs.players[0].has_chosen);
    assert_eq!(theirs.players[0].pending_choice, None);
    assert_eq!(theirs.players[0].swing[0].size, None);
    let mine = view_for(&s, Viewer::Seat { player: One });
    assert_eq!(mine.players[0].pending_choice, Some(choice));
    let json = serde_json::to_string(&theirs).unwrap();
    assert!(!json.contains("17"), "the hidden size leaked: {json}");
}

#[test]
fn dice_carry_their_notation_and_scores_are_in_half_points() {
    let s = attack_state(vec![die(6, 4).with(DieSkill::Poison), die(7, 1)], vec![die(6, 3)], One);
    let v = view_for(&s, Viewer::Seat { player: One });
    assert_eq!(v.players[0].active[0].notation, "p(6)");
    assert_eq!(v.players[0].recipe, "p(6) (7)");
    // Poison 6 kept: -6 points (-12); d7 kept: 3.5 points (7).
    assert_eq!(v.players[0].round_score, -5);
    assert_eq!(v.actors, vec![One]);
    assert!(matches!(v.legal, LegalActions::Attack { .. }));
}

#[test]
fn spectators_and_waiting_seats_get_no_actions() {
    let s = attack_state(vec![die(6, 4)], vec![die(6, 3)], One);
    assert_eq!(view_for(&s, Viewer::Spectator).legal, LegalActions::Wait);
    assert_eq!(view_for(&s, Viewer::Seat { player: Two }).legal, LegalActions::Wait);
}

#[test]
fn captured_dice_show_their_points() {
    let s = attack_state(vec![die(6, 4)], vec![die(6, 3), die(20, 20)], One);
    let attack = buttonmen_engine::DeclaredAttack {
        kind: buttonmen_engine::AttackKind::Power,
        attackers: vec![s.player(One).active[0].id],
        targets: vec![s.player(Two).active[0].id],
    };
    let (s, _) = ok(&s, One, Action::Attack { attack });
    let v = view_for(&s, Viewer::Spectator);
    assert_eq!(v.players[0].captured[0].points, 12);
    assert_eq!(v.players[0].captured[0].die.notation, "(6)");
}
```

Ajouter `serde_json` aux dev-dependencies si absent (il l'est depuis la Tâche 1).

- [ ] **Step 2: Lancer les tests, vérifier qu'ils échouent**

Run: `cargo test -p buttonmen-engine --test view`
Expected: échec de compilation (`view_for`, `Viewer` introuvables).

- [ ] **Step 3: Implémenter `view.rs`**

```rust
use serde::{Deserialize, Serialize};

use crate::action::SpecChoice;
use crate::die::{Die, Sides, SwingLetter};
use crate::ids::{DieId, PlayerId};
use crate::legal::{LegalActions, legal_actions};
use crate::scoring::{Holding, die_points, round_score};
use crate::setup::required_letters;
use crate::skill::DieSkill;
use crate::state::{GameState, Record, WaitingFor};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[serde(tag = "type")]
pub enum Viewer {
    Seat { player: PlayerId },
    Spectator,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct DieView {
    pub id: DieId,
    /// R 1.1: the die as written in its button's recipe.
    pub notation: String,
    pub skills: Vec<DieSkill>,
    pub sides: Sides,
    pub min: u8,
    pub max: u8,
    pub value: u8,
    pub dizzy: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct CapturedView {
    pub die: DieView,
    /// Half-points.
    pub points: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct SlotView {
    pub id: DieId,
    pub notation: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct SwingView {
    pub letter: SwingLetter,
    pub size: Option<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct PlayerView {
    pub seat: PlayerId,
    pub button: String,
    pub recipe: String,
    pub active: Vec<DieView>,
    pub captured: Vec<CapturedView>,
    /// R 6.11: reserve dice not yet in play.
    pub reserve: Vec<SlotView>,
    pub record: Record,
    /// Half-points.
    pub round_score: i32,
    /// Committed swing sizes (public).
    pub swing: Vec<SwingView>,
    /// R 3.1: this player has submitted a choice that is not revealed yet.
    pub has_chosen: bool,
    /// Only shown to the player who made it.
    pub pending_choice: Option<SpecChoice>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct GameView {
    pub viewer: Viewer,
    pub round: u8,
    pub max_wins: u8,
    pub initiative: Option<PlayerId>,
    pub players: Vec<PlayerView>,
    pub waiting_for: WaitingFor,
    pub actors: Vec<PlayerId>,
    pub legal: LegalActions,
}

fn notation_of(state: &GameState, id: DieId) -> String {
    PlayerId::ALL
        .iter()
        .find_map(|&p| state.player(p).slot(id))
        .map(|s| s.spec.notation())
        .unwrap_or_default()
}

fn die_view(state: &GameState, die: &Die) -> DieView {
    DieView {
        id: die.id,
        notation: notation_of(state, die.id),
        skills: die.skills.iter().collect(),
        sides: die.sides,
        min: die.min(),
        max: die.max(),
        value: die.value,
        dizzy: die.dizzy,
    }
}

fn player_view(state: &GameState, seat: PlayerId, viewer: Viewer) -> PlayerView {
    let p = state.player(seat);
    let is_me = viewer == Viewer::Seat { player: seat };
    PlayerView {
        seat,
        button: p.button.clone(),
        recipe: p.slots.iter().map(|s| s.spec.notation()).collect::<Vec<_>>().join(" "),
        active: p.active.iter().map(|d| die_view(state, d)).collect(),
        captured: p
            .captured
            .iter()
            .map(|c| CapturedView {
                die: die_view(state, &c.die),
                points: die_points(&c.die, Holding::Captured(c.scoring)),
            })
            .collect(),
        reserve: p
            .slots
            .iter()
            .filter(|s| !s.in_play)
            .map(|s| SlotView { id: s.id, notation: s.spec.notation() })
            .collect(),
        record: p.record,
        round_score: round_score(state, seat),
        swing: required_letters(p)
            .into_iter()
            .map(|letter| SwingView { letter, size: p.choice.swing_size(letter) })
            .collect(),
        has_chosen: p.pending_choice.is_some(),
        pending_choice: if is_me { p.pending_choice.clone() } else { None },
    }
}

/// What `viewer` may see of the match (R 3.1: unrevealed choices stay hidden).
pub fn view_for(state: &GameState, viewer: Viewer) -> GameView {
    GameView {
        viewer,
        round: state.round,
        max_wins: state.config.max_wins,
        initiative: state.initiative,
        players: PlayerId::ALL.iter().map(|&p| player_view(state, p, viewer)).collect(),
        waiting_for: state.waiting_for.clone(),
        actors: state.actors(),
        legal: match viewer {
            Viewer::Seat { player } => legal_actions(state, player),
            Viewer::Spectator => LegalActions::Wait,
        },
    }
}
```

Dans `lib.rs` : ajouter `mod view;` et
```rust
pub use view::{CapturedView, DieView, GameView, PlayerView, SlotView, SwingView, Viewer, view_for};
```

- [ ] **Step 4: Lancer les tests, vérifier qu'ils passent**

Run: `cargo test -p buttonmen-engine && cargo clippy -p buttonmen-engine --all-targets --all-features -- -D warnings`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/engine
git commit -m "feat(engine): vue de partie par siège"
```

---
### Task 13: Catalogue de boutons, boutons originaux et import buttonweavers

**Files:**
- Create: `data/buttons.toml`, `crates/button-data/src/catalog.rs`, `scripts/import-buttonweavers.sh`, `scripts/import_buttonweavers.py`
- Modify: `crates/button-data/Cargo.toml`, `crates/button-data/src/lib.rs`, `README.md`

**Interfaces:**
- Consumes: `parse_recipe` (Tâche 4), `Button` (Tâche 5).
- Produces:
  - `ButtonSource { Original, Buttonweavers }` ; `ButtonRecipe { name, recipe, set: Option<String>, source, special: bool }` ; `Playability { Playable, Unplayable { reason: String } }` ; `CatalogEntry { name, recipe, set, source, playability }`.
  - `original_buttons() -> Vec<ButtonRecipe>` (embarqués à la compilation) ; `parse_buttonweavers_json(&str) -> Result<Vec<ButtonRecipe>, serde_json::Error>` ; `catalog(&[ButtonRecipe]) -> Vec<CatalogEntry>` ; `to_button(&ButtonRecipe) -> Result<Button, RecipeError>`.
  - Feature `ts` de `buttonmen-data` (types TypeScript du catalogue).

- [ ] **Step 1: Écrire les boutons originaux**

`data/buttons.toml` (personnages originaux ; chacun met en avant une compétence v1 ou une forme de dé) :
```toml
# Boutons originaux de buttonmen-rs (diffusables). Notation : docs/RULES.md, R 1.1.

[[button]]
name = "Avoine"
recipe = "(4) (4) (10) (12) (X)"

[[button]]
name = "Brindille"
recipe = "(6) (8) (12) (20) (Y)"

[[button]]
name = "Carmin"
recipe = "p(4) (8) (10) (12) (X)"

[[button]]
name = "Dune"
recipe = "s(4) s(6) (10) (12) (Z)"

[[button]]
name = "Écho"
recipe = "z(4) (6) z(10) (12) (X)"

[[button]]
name = "Fenouil"
recipe = "t(4) (6) (8) (12) (V)"

[[button]]
name = "Grêle"
recipe = "B(8) (10) (12) B(20) (W)"

[[button]]
name = "Houle"
recipe = "f(4) f(6) (10) (12) (X)"

[[button]]
name = "Iris"
recipe = "c(4) (8) c(10) (12) (Y)"

[[button]]
name = "Jade"
recipe = "n(6) (8) (10) (12) (X)"

[[button]]
name = "Kilim"
recipe = "k(4) (6) (10) (12) (S)"

[[button]]
name = "Lierre"
recipe = "g(4) g(6) (10) (12) (X)"

[[button]]
name = "Mistral"
recipe = "q(6) q(8) (10) (12) (T)"

[[button]]
name = "Nacre"
recipe = "(4) (6) (10) (12) r(20) r(X)"

[[button]]
name = "Opale"
recipe = "v(6) v(10) (12) (20) (U)"

[[button]]
name = "Pavot"
recipe = "d(4) d(6) (10) (12) (X)"

[[button]]
name = "Quartz"
recipe = "F(6) (8) F(10) (12) (X)"

[[button]]
name = "Roseau"
recipe = "(4/8) (6/10) (10) (12) (X)"

[[button]]
name = "Sable"
recipe = "(4,4) (8) (10) (X,X)"

[[button]]
name = "Tonnerre"
recipe = "ps(10) zt(8) (12) Bf(6) (R)"
```

- [ ] **Step 2: Écrire les tests du catalogue**

Dans `crates/button-data/Cargo.toml`, ajouter :
```toml
[features]
ts = ["dep:ts-rs", "buttonmen-engine/ts"]

[dependencies]
# … dépendances existantes …
ts-rs = { workspace = true, optional = true }
```

`crates/button-data/src/catalog.rs`, tests seulement :
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_original_button_is_playable_and_unique() {
        let originals = original_buttons();
        assert_eq!(originals.len(), 20);
        let entries = catalog(&originals);
        for entry in &entries {
            assert_eq!(entry.playability, Playability::Playable, "{} : {}", entry.name, entry.recipe);
            assert_eq!(entry.source, ButtonSource::Original);
        }
        let mut names: Vec<_> = originals.iter().map(|b| b.name.clone()).collect();
        names.sort();
        names.dedup();
        assert_eq!(names.len(), 20);
    }

    #[test]
    fn originals_cover_every_v1_skill() {
        use buttonmen_engine::DieSkill;
        let originals = original_buttons();
        for skill in DieSkill::ALL {
            let covered = originals
                .iter()
                .filter_map(|b| to_button(b).ok())
                .any(|b| b.dice.iter().any(|d| d.skills.has(skill)));
            assert!(covered, "no original button uses {skill:?}");
        }
    }

    #[test]
    fn buttonweavers_json_is_read_and_classified() {
        let json = r#"[
            {"name": "Avis", "recipe": "(4) (4) (10) (12) (X)", "set": "Soldiers", "special": false},
            {"name": "Zeppo", "recipe": "(4) (12) (20) (X)!", "set": "1999 Rare / Promo", "special": false},
            {"name": "Echo", "recipe": "(4) (4) (10) (12) (X)", "set": "Promo", "special": true}
        ]"#;
        let recipes = parse_buttonweavers_json(json).unwrap();
        assert!(recipes.iter().all(|r| r.source == ButtonSource::Buttonweavers));
        let entries = catalog(&recipes);
        assert_eq!(entries[0].playability, Playability::Playable);
        assert!(matches!(&entries[1].playability, Playability::Unplayable { reason } if reason.contains('!')));
        assert!(matches!(&entries[2].playability, Playability::Unplayable { .. }));
    }

    #[test]
    fn local_buttonweavers_import_never_panics() {
        // Runs only when the optional import exists (scripts/import-buttonweavers.sh).
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../data/buttonweavers/buttons.json");
        let Ok(json) = std::fs::read_to_string(path) else { return };
        let recipes = parse_buttonweavers_json(&json).unwrap();
        let entries = catalog(&recipes);
        let playable = entries.iter().filter(|e| e.playability == Playability::Playable).count();
        println!("{playable} / {} boutons buttonweavers jouables", entries.len());
        assert!(playable > 0);
    }
}
```

Dans `lib.rs` : ajouter `pub mod catalog;` et `pub use catalog::{ButtonRecipe, ButtonSource, CatalogEntry, Playability, catalog, original_buttons, parse_buttonweavers_json, to_button};`.

- [ ] **Step 3: Lancer les tests, vérifier qu'ils échouent**

Run: `cargo test -p buttonmen-data`
Expected: échec de compilation (`original_buttons` introuvable).

- [ ] **Step 4: Implémenter `catalog.rs`**

Au-dessus des tests :
```rust
use buttonmen_engine::Button;
use serde::{Deserialize, Serialize};

use crate::recipe::{RecipeError, parse_recipe};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub enum ButtonSource {
    Original,
    Buttonweavers,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ButtonRecipe {
    pub name: String,
    pub recipe: String,
    pub set: Option<String>,
    pub source: ButtonSource,
    /// The button has a button-level skill (buttonweavers `btn_special`), not supported in v1.
    pub special: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[serde(tag = "type")]
pub enum Playability {
    Playable,
    Unplayable { reason: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct CatalogEntry {
    pub name: String,
    pub recipe: String,
    pub set: Option<String>,
    pub source: ButtonSource,
    pub playability: Playability,
}

#[derive(Deserialize)]
struct OriginalFile {
    button: Vec<OriginalButton>,
}

#[derive(Deserialize)]
struct OriginalButton {
    name: String,
    recipe: String,
}

/// The original buttons shipped with the game (`data/buttons.toml`).
pub fn original_buttons() -> Vec<ButtonRecipe> {
    let file: OriginalFile = toml::from_str(include_str!("../../../data/buttons.toml"))
        .expect("data/buttons.toml is valid (checked by tests)");
    file.button
        .into_iter()
        .map(|b| ButtonRecipe {
            name: b.name,
            recipe: b.recipe,
            set: Some("Originaux".to_string()),
            source: ButtonSource::Original,
            special: false,
        })
        .collect()
}

#[derive(Deserialize)]
struct ImportedButton {
    name: String,
    recipe: String,
    set: Option<String>,
    #[serde(default)]
    special: bool,
}

/// Reads `data/buttonweavers/buttons.json`, written by `scripts/import-buttonweavers.sh`.
pub fn parse_buttonweavers_json(json: &str) -> Result<Vec<ButtonRecipe>, serde_json::Error> {
    let imported: Vec<ImportedButton> = serde_json::from_str(json)?;
    Ok(imported
        .into_iter()
        .map(|b| ButtonRecipe {
            name: b.name,
            recipe: b.recipe,
            set: b.set,
            source: ButtonSource::Buttonweavers,
            special: b.special,
        })
        .collect())
}

pub fn to_button(recipe: &ButtonRecipe) -> Result<Button, RecipeError> {
    Ok(Button { name: recipe.name.clone(), dice: parse_recipe(&recipe.recipe)? })
}

/// Every button with whether v1 can play it, and why not.
pub fn catalog(recipes: &[ButtonRecipe]) -> Vec<CatalogEntry> {
    recipes
        .iter()
        .map(|r| CatalogEntry {
            name: r.name.clone(),
            recipe: r.recipe.clone(),
            set: r.set.clone(),
            source: r.source,
            playability: if r.special {
                Playability::Unplayable { reason: "compétence de bouton non prise en charge".to_string() }
            } else {
                match parse_recipe(&r.recipe) {
                    Ok(_) => Playability::Playable,
                    Err(e) => Playability::Unplayable { reason: e.to_string() },
                }
            },
        })
        .collect()
}
```

- [ ] **Step 5: Écrire le script d'import**

`scripts/import_buttonweavers.py` (vérifié sur `data.button.sql` de buttonweavers : 877 boutons) :
```python
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
```

`scripts/import-buttonweavers.sh` :
```bash
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
```

Run: `chmod +x scripts/import-buttonweavers.sh scripts/import_buttonweavers.py`

Ajouter au `README.md` :
```markdown
## Boutons buttonweavers (facultatif, usage privé)

`./scripts/import-buttonweavers.sh` importe les ~880 boutons de Button Men Online dans `data/buttonweavers/buttons.json` (gitignoré). Les noms des personnages appartiennent à Cheapass Games : ce fichier n'est pas versionné ni diffusé. Les boutons qui utilisent des compétences hors v1 apparaissent « non jouables ».
```

- [ ] **Step 6: Lancer les tests, puis l'import une fois**

Run: `cargo test -p buttonmen-data && ./scripts/import-buttonweavers.sh && cargo test -p buttonmen-data local_buttonweavers_import_never_panics -- --nocapture`
Expected: PASS ; le script affiche `877 boutons écrits…` (environ) ; le dernier test affiche le nombre de boutons jouables.

- [ ] **Step 7: Commit**

```bash
git add data/buttons.toml crates/button-data scripts/import-buttonweavers.sh scripts/import_buttonweavers.py README.md
git commit -m "feat(data): catalogue, boutons originaux et import buttonweavers"
```

---
### Task 14: Pont WASM et types TypeScript générés

**Files:**
- Create: `crates/engine-wasm/Cargo.toml`, `crates/engine-wasm/src/lib.rs`, `crates/engine-wasm/src/bridge.rs`, `scripts/build-wasm.sh`, `scripts/gen-types.sh`
- Create (générés, versionnés) : `client/src/engine/generated/*.ts`
- Modify: `Cargo.toml` (membre du workspace)

**Interfaces:**
- Consumes: moteur complet (Tâches 5 à 12), catalogue (Tâche 13).
- Produces (JavaScript, après `wasm-bindgen`) :
  - `catalog_json(imported?: string | null): string` → `CatalogEntry[]` ; `skills_json(): string` → `SkillInfo[]`.
  - `class WasmMatch { constructor(setupJson: string); view(viewerJson: string): string; apply(playerJson: string, actionJson: string): string; state_json(): string; free(): void }`.
  - JSON : `MatchSetup { buttons: [SeatButton, SeatButton], seed: number, max_wins: number }`, `SeatButton { name, recipe }`, `ApplyOutcome = { type: "Applied", events: GameEvent[] } | { type: "Rejected", error: RuleError, message: string }`.
  - Types TS générés dans `client/src/engine/generated/` (un fichier par type : `GameView.ts`, `Action.ts`, `LegalActions.ts`, `CatalogEntry.ts`, `MatchSetup.ts`, `ApplyOutcome.ts`…).

- [ ] **Step 1: Déclarer la crate**

Dans `Cargo.toml` du workspace : `members = ["crates/engine", "crates/button-data", "crates/engine-wasm"]`.

`crates/engine-wasm/Cargo.toml` :
```toml
[package]
name = "buttonmen-wasm"
description = "WASM bridge: a thin JSON layer over the engine (no game logic)"
version.workspace = true
edition.workspace = true
license.workspace = true

[lib]
crate-type = ["cdylib", "rlib"]

[features]
ts = ["dep:ts-rs", "buttonmen-engine/ts", "buttonmen-data/ts"]

[dependencies]
buttonmen-engine = { workspace = true }
buttonmen-data = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
ts-rs = { workspace = true, optional = true }
# Must match the installed wasm-bindgen CLI exactly (`wasm-bindgen --version`).
wasm-bindgen = "=0.2.121"
console_error_panic_hook = "0.1"
```

- [ ] **Step 2: Écrire les tests du pont (natifs)**

La logique de sérialisation vit dans `bridge.rs`, en fonctions pures qui renvoient `Result<_, String>` ; `lib.rs` ne fait que les exposer à JavaScript. On la teste donc en natif.

`crates/engine-wasm/src/bridge.rs`, tests seulement :
```rust
#[cfg(test)]
mod tests {
    use super::*;

    const SETUP: &str = r#"{
        "buttons": [{"name": "A", "recipe": "(4) (6) (X)"}, {"name": "B", "recipe": "(8) (X)"}],
        "seed": 7,
        "max_wins": 3
    }"#;

    #[test]
    fn a_match_starts_with_both_players_choosing_swing() {
        let state = new_match(SETUP).unwrap();
        let view: serde_json::Value = serde_json::from_str(&view(&state, r#"{"type":"Spectator"}"#).unwrap()).unwrap();
        assert_eq!(view["waiting_for"]["type"], "SpecifyDice");
        assert_eq!(view["actors"], serde_json::json!(["One", "Two"]));
    }

    #[test]
    fn apply_reports_events_or_a_rejection() {
        let mut state = new_match(SETUP).unwrap();
        let bad = r#"{"type":"SpecifyDice","choice":{"swing":[["X",99]],"options":[]}}"#;
        let out: serde_json::Value = serde_json::from_str(&apply(&mut state, r#""One""#, bad).unwrap()).unwrap();
        assert_eq!(out["type"], "Rejected");
        assert_eq!(out["error"]["type"], "SwingOutOfRange");
        assert!(out["message"].as_str().unwrap().contains("hors de la plage"));
        let good = r#"{"type":"SpecifyDice","choice":{"swing":[["X",10]],"options":[]}}"#;
        let out: serde_json::Value = serde_json::from_str(&apply(&mut state, r#""One""#, good).unwrap()).unwrap();
        assert_eq!(out["type"], "Applied");
    }

    #[test]
    fn malformed_input_is_an_error_not_a_panic() {
        assert!(new_match("{}").is_err());
        assert!(new_match(&SETUP.replace("(8) (X)", "o(8)")).is_err());
        let mut state = new_match(SETUP).unwrap();
        assert!(apply(&mut state, r#""Three""#, r#"{"type":"Pass"}"#).is_err());
        assert!(view(&state, "nope").is_err());
    }

    #[test]
    fn catalog_and_skills_serialize() {
        let catalog: serde_json::Value = serde_json::from_str(&catalog(None).unwrap()).unwrap();
        assert_eq!(catalog.as_array().unwrap().len(), 20);
        let skills: serde_json::Value = serde_json::from_str(&skills()).unwrap();
        assert_eq!(skills.as_array().unwrap().len(), 15);
    }
}
```

- [ ] **Step 3: Lancer les tests, vérifier qu'ils échouent**

Run: `cargo test -p buttonmen-wasm`
Expected: échec de compilation (`new_match`… introuvables).

- [ ] **Step 4: Implémenter le pont**

Au-dessus des tests dans `bridge.rs` :
```rust
use buttonmen_data::{original_buttons, parse_buttonweavers_json, parse_recipe};
use buttonmen_engine::{
    Action, Button, DieSkill, GameEvent, GameState, MatchConfig, PlayerId, RuleError, Viewer,
    new_game, skill_info, view_for,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct SeatButton {
    pub name: String,
    pub recipe: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct MatchSetup {
    pub buttons: [SeatButton; 2],
    pub seed: u32,
    pub max_wins: u8,
}

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[serde(tag = "type")]
pub enum ApplyOutcome {
    Applied { events: Vec<GameEvent> },
    Rejected { error: RuleError, message: String },
}

fn to_json<T: Serialize>(value: &T) -> String {
    serde_json::to_string(value).expect("engine types always serialize")
}

fn parse<'a, T: Deserialize<'a>>(json: &'a str) -> Result<T, String> {
    serde_json::from_str(json).map_err(|e| e.to_string())
}

pub fn new_match(setup_json: &str) -> Result<GameState, String> {
    let setup: MatchSetup = parse(setup_json)?;
    let button = |b: &SeatButton| -> Result<Button, String> {
        Ok(Button {
            name: b.name.clone(),
            dice: parse_recipe(&b.recipe).map_err(|e| format!("{} : {e}", b.name))?,
        })
    };
    let buttons = [button(&setup.buttons[0])?, button(&setup.buttons[1])?];
    Ok(new_game(buttons, u64::from(setup.seed), MatchConfig { max_wins: setup.max_wins }))
}

pub fn view(state: &GameState, viewer_json: &str) -> Result<String, String> {
    let viewer: Viewer = parse(viewer_json)?;
    Ok(to_json(&view_for(state, viewer)))
}

pub fn apply(state: &mut GameState, player_json: &str, action_json: &str) -> Result<String, String> {
    let player: PlayerId = parse(player_json)?;
    let action: Action = parse(action_json)?;
    let outcome = match buttonmen_engine::apply(state, player, action) {
        Ok((next, events)) => {
            *state = next;
            ApplyOutcome::Applied { events }
        }
        Err(error) => ApplyOutcome::Rejected { message: error.to_string(), error },
    };
    Ok(to_json(&outcome))
}

pub fn catalog(imported_json: Option<&str>) -> Result<String, String> {
    let mut recipes = original_buttons();
    if let Some(json) = imported_json {
        recipes.extend(parse_buttonweavers_json(json).map_err(|e| e.to_string())?);
    }
    Ok(to_json(&buttonmen_data::catalog(&recipes)))
}

pub fn skills() -> String {
    to_json(&DieSkill::ALL.map(skill_info))
}

pub fn state_json(state: &GameState) -> String {
    to_json(state)
}
```

`crates/engine-wasm/src/lib.rs` :
```rust
//! WASM bridge: exposes `bridge` to JavaScript. No game logic here.

pub mod bridge;

use buttonmen_engine::GameState;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
}

fn js(message: String) -> JsError {
    JsError::new(&message)
}

/// The button catalog: the originals, plus the local buttonweavers import when given.
#[wasm_bindgen]
pub fn catalog_json(imported: Option<String>) -> Result<String, JsError> {
    bridge::catalog(imported.as_deref()).map_err(js)
}

#[wasm_bindgen]
pub fn skills_json() -> String {
    bridge::skills()
}

/// A match held on the WASM side; JavaScript only sees views and outcomes.
#[wasm_bindgen]
pub struct WasmMatch {
    state: GameState,
}

#[wasm_bindgen]
impl WasmMatch {
    #[wasm_bindgen(constructor)]
    pub fn new(setup_json: &str) -> Result<WasmMatch, JsError> {
        Ok(WasmMatch { state: bridge::new_match(setup_json).map_err(js)? })
    }

    pub fn view(&self, viewer_json: &str) -> Result<String, JsError> {
        bridge::view(&self.state, viewer_json).map_err(js)
    }

    pub fn apply(&mut self, player_json: &str, action_json: &str) -> Result<String, JsError> {
        bridge::apply(&mut self.state, player_json, action_json).map_err(js)
    }

    /// The full state (saving, debugging); never used for display.
    pub fn state_json(&self) -> String {
        bridge::state_json(&self.state)
    }
}
```

Dans `crates/button-data/src/lib.rs`, vérifier que `parse_recipe` est réexporté (Tâche 4).

- [ ] **Step 5: Lancer les tests**

Run: `cargo test -p buttonmen-wasm && cargo clippy --workspace --all-targets --all-features -- -D warnings`
Expected: PASS.

- [ ] **Step 6: Scripts de build et de génération des types**

`scripts/build-wasm.sh` :
```bash
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
```

`scripts/gen-types.sh` :
```bash
#!/usr/bin/env bash
# Régénère les types TypeScript du client (client/src/engine/generated/) depuis les types Rust.
# ts-rs écrit un fichier par type quand on lance ses tests `export_bindings_*`.
set -euo pipefail
cd "$(dirname "$0")/.."
rm -rf client/src/engine/generated
for crate in buttonmen-engine buttonmen-data buttonmen-wasm; do
  cargo test -p "$crate" --features ts export_bindings --quiet
done
ls client/src/engine/generated
```

Run: `chmod +x scripts/build-wasm.sh scripts/gen-types.sh && ./scripts/gen-types.sh && ./scripts/build-wasm.sh dev`
Expected: `client/src/engine/generated/` contient notamment `GameView.ts`, `Action.ts`, `LegalActions.ts`, `GameEvent.ts`, `RuleError.ts`, `CatalogEntry.ts`, `SkillInfo.ts`, `MatchSetup.ts`, `ApplyOutcome.ts`, `Viewer.ts` ; `client/src/wasm/buttonmen_bg.wasm` existe.

Si un type référencé n'a pas de fichier (erreur d'import dans un `.ts` généré), il manque `#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]` sur ce type Rust : l'ajouter et relancer.

- [ ] **Step 7: Commit**

```bash
git add Cargo.toml crates/engine-wasm scripts/build-wasm.sh scripts/gen-types.sh client/src/engine/generated
git commit -m "feat(wasm): pont WASM et types TypeScript générés"
```

---
### Task 15: Socle du client, adaptateur local et aides d'affichage

**Files:**
- Create: `client/package.json`, `client/pnpm-workspace.yaml`, `client/vite.config.ts`, `client/tsconfig.json`, `client/index.html`, `client/src/main.tsx`, `client/src/styles.css`, `client/src/vite-env.d.ts`
- Create: `client/src/engine/wasm.ts`, `client/src/engine/format.ts`, `client/src/adapters/GameAdapter.ts`, `client/src/adapters/LocalAdapter.ts`, `client/src/adapters/useGame.ts`, `client/src/table/selection.ts`
- Test: `client/src/engine/__tests__/format.test.ts`, `client/src/adapters/__tests__/LocalAdapter.test.ts`, `client/src/table/__tests__/selection.test.ts`

**Interfaces:**
- Consumes: module WASM et types générés (Tâche 14).
- Produces:
  - `loadEngine(): Promise<void>`, `loadCatalog(): Promise<CatalogEntry[]>`, `loadSkills(): Promise<Map<DieSkill, SkillInfo>>`, `createMatch(setup: MatchSetup): EngineMatch` (wasm.ts).
  - `formatPoints(half: number): string` (format.ts).
  - `interface EngineMatch { view(viewerJson: string): string; apply(playerJson: string, actionJson: string): string; free(): void }` ; `interface Snapshot { view: GameView; lastEvents: GameEvent[] }` ; `interface GameAdapter { getSnapshot(): Snapshot; subscribe(listener: () => void): () => void; dispatch(player: PlayerId, action: Action): string | null; dispose(): void }` (`dispatch` renvoie le message d'erreur du moteur, ou `null`).
  - `class LocalAdapter implements GameAdapter` (hotseat : la vue est celle du premier joueur qui doit agir).
  - `useGame(adapter): Snapshot` (hook React).
  - `interface Selection { attackers: DieId[]; targets: DieId[] }`, `matchingAttacks(attacks, selection): AttackOption[]`, `selectableDice(attacks, selection): { attackers: Set<DieId>; targets: Set<DieId> }`, `toggle(list, id): DieId[]` (selection.ts).

- [ ] **Step 1: Créer le projet**

`client/package.json` :
```json
{
  "name": "buttonmen-client",
  "private": true,
  "version": "0.1.0",
  "type": "module",
  "scripts": {
    "wasm": "../scripts/build-wasm.sh",
    "wasm:dev": "../scripts/build-wasm.sh dev",
    "types": "../scripts/gen-types.sh",
    "dev": "vite",
    "build": "tsc --noEmit && vite build",
    "type-check": "tsc --noEmit",
    "test": "vitest run"
  },
  "dependencies": {
    "motion": "^12.23.0",
    "react": "^19.1.0",
    "react-dom": "^19.1.0"
  },
  "devDependencies": {
    "@tailwindcss/vite": "^4.1.0",
    "@types/react": "^19.1.0",
    "@types/react-dom": "^19.1.0",
    "@vitejs/plugin-react": "^5.0.0",
    "tailwindcss": "^4.1.0",
    "typescript": "^5.9.0",
    "vite": "^7.1.0",
    "vitest": "^3.2.0"
  }
}
```

`client/pnpm-workspace.yaml` :
```yaml
# esbuild ships its binary as a platform package: its postinstall script is not needed.
allowBuilds:
  esbuild: false
```

`client/vite.config.ts` :
```ts
/// <reference types="vitest/config" />
import tailwindcss from "@tailwindcss/vite";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";

export default defineConfig({
  // Relative paths: the build works from any folder.
  base: "./",
  plugins: [react(), tailwindcss()],
  test: { environment: "node" },
});
```

`client/tsconfig.json` :
```json
{
  "compilerOptions": {
    "target": "ES2022",
    "lib": ["ES2022", "DOM", "DOM.Iterable"],
    "module": "ESNext",
    "moduleResolution": "bundler",
    "jsx": "react-jsx",
    "strict": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "skipLibCheck": true,
    "isolatedModules": true,
    "allowJs": false,
    "noEmit": true
  },
  "include": ["src", "vite.config.ts"]
}
```

`client/index.html` :
```html
<!doctype html>
<html lang="fr">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>Button Men</title>
  </head>
  <body>
    <div id="root"></div>
    <script type="module" src="/src/main.tsx"></script>
  </body>
</html>
```

`client/src/vite-env.d.ts` :
```ts
/// <reference types="vite/client" />
```

`client/src/styles.css` :
```css
@import "tailwindcss";

@theme {
  --color-felt: #1f5135;
  --color-felt-dark: #163b27;
  --color-ivory: #f6f1e4;
  --color-ink: #1d1b16;
  --color-gold: #d9a441;
  --color-danger: #c4453c;
  --font-display: Georgia, "Times New Roman", serif;
}

body {
  @apply min-h-screen bg-felt text-ivory antialiased;
}
```

`client/src/main.tsx` (l'écran `App` arrive à la Tâche 16 ; en attendant un écran de chargement suffit) :
```tsx
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import "./styles.css";

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <p className="p-8 font-display text-2xl">Button Men</p>
  </StrictMode>,
);
```

Run: `cd client && pnpm install && pnpm type-check`
Expected: installation OK, aucune erreur TypeScript.

- [ ] **Step 2: Écrire les tests**

`client/src/engine/__tests__/format.test.ts` :
```ts
import { describe, expect, it } from "vitest";
import { formatPoints } from "../format";

describe("formatPoints", () => {
  it("shows half-points as French decimals", () => {
    expect(formatPoints(28)).toBe("14");
    expect(formatPoints(7)).toBe("3,5");
    expect(formatPoints(-5)).toBe("−2,5");
    expect(formatPoints(0)).toBe("0");
  });
});
```

`client/src/table/__tests__/selection.test.ts` :
```ts
import { describe, expect, it } from "vitest";
import type { AttackOption } from "../../engine/generated/AttackOption";
import { matchingAttacks, selectableDice, toggle } from "../selection";

const option = (kind: AttackOption["attack"]["kind"], attackers: number[], targets: number[]): AttackOption => ({
  attack: { kind, attackers, targets },
  needs_fire: false,
});

const attacks = [option("Power", [0], [5]), option("Skill", [0, 1], [6]), option("Skill", [0], [5])];

describe("selection", () => {
  it("matches attacks with exactly the selected dice, in any order", () => {
    expect(matchingAttacks(attacks, { attackers: [0], targets: [5] }).map((o) => o.attack.kind)).toEqual([
      "Power",
      "Skill",
    ]);
    expect(matchingAttacks(attacks, { attackers: [1, 0], targets: [6] })).toHaveLength(1);
    expect(matchingAttacks(attacks, { attackers: [1], targets: [6] })).toHaveLength(0);
  });

  it("offers only dice that extend the current selection to some attack", () => {
    const none = selectableDice(attacks, { attackers: [], targets: [] });
    expect([...none.attackers].sort()).toEqual([0, 1]);
    expect([...none.targets].sort()).toEqual([5, 6]);
    const afterOne = selectableDice(attacks, { attackers: [1], targets: [] });
    expect([...afterOne.attackers]).toEqual([0, 1]);
    expect([...afterOne.targets]).toEqual([6]);
  });

  it("toggles ids", () => {
    expect(toggle([1, 2], 2)).toEqual([1]);
    expect(toggle([1], 3)).toEqual([1, 3]);
  });
});
```

`client/src/adapters/__tests__/LocalAdapter.test.ts` :
```ts
import { describe, expect, it, vi } from "vitest";
import type { EngineMatch } from "../GameAdapter";
import { LocalAdapter } from "../LocalAdapter";

/** A fake engine: Two must act; One's actions are rejected. */
function fakeMatch(): EngineMatch & { viewers: string[] } {
  const viewers: string[] = [];
  return {
    viewers,
    view(viewerJson) {
      viewers.push(viewerJson);
      return JSON.stringify({ viewer: JSON.parse(viewerJson), actors: ["Two"], round: 1 });
    },
    apply(playerJson) {
      return JSON.parse(playerJson) === "Two"
        ? JSON.stringify({ type: "Applied", events: [{ type: "Passed", player: "Two" }] })
        : JSON.stringify({ type: "Rejected", error: { type: "NotYourTurn" }, message: "ce n'est pas à ce joueur d'agir" });
    },
    free: vi.fn(),
  };
}

describe("LocalAdapter", () => {
  it("shows the view of the first player who must act", () => {
    const engine = fakeMatch();
    const adapter = new LocalAdapter(engine);
    expect(adapter.getSnapshot().view.viewer).toEqual({ type: "Seat", player: "Two" });
  });

  it("notifies listeners and keeps events on success, returns the message on rejection", () => {
    const adapter = new LocalAdapter(fakeMatch());
    const listener = vi.fn();
    adapter.subscribe(listener);
    expect(adapter.dispatch("One", { type: "Pass" })).toBe("ce n'est pas à ce joueur d'agir");
    expect(listener).not.toHaveBeenCalled();
    expect(adapter.dispatch("Two", { type: "Pass" })).toBeNull();
    expect(listener).toHaveBeenCalledOnce();
    expect(adapter.getSnapshot().lastEvents).toEqual([{ type: "Passed", player: "Two" }]);
  });

  it("frees the engine on dispose", () => {
    const engine = fakeMatch();
    new LocalAdapter(engine).dispose();
    expect(engine.free).toHaveBeenCalled();
  });
});
```

- [ ] **Step 3: Lancer les tests, vérifier qu'ils échouent**

Run: `cd client && pnpm test`
Expected: FAIL — modules `../format`, `../selection`, `../LocalAdapter` introuvables.

- [ ] **Step 4: Implémenter**

`client/src/engine/format.ts` :
```ts
/** Engine scores are in half-points: 7 → "3,5". */
export function formatPoints(half: number): string {
  const sign = half < 0 ? "−" : "";
  const abs = Math.abs(half);
  const whole = Math.floor(abs / 2);
  return abs % 2 === 0 ? `${sign}${whole}` : `${sign}${whole},5`;
}
```

`client/src/adapters/GameAdapter.ts` :
```ts
import type { Action } from "../engine/generated/Action";
import type { GameEvent } from "../engine/generated/GameEvent";
import type { GameView } from "../engine/generated/GameView";
import type { PlayerId } from "../engine/generated/PlayerId";

/** The part of `WasmMatch` the adapters use (lets tests use a fake). */
export interface EngineMatch {
  view(viewerJson: string): string;
  apply(playerJson: string, actionJson: string): string;
  free(): void;
}

export interface Snapshot {
  view: GameView;
  lastEvents: GameEvent[];
}

/** One interface for every way of playing (hotseat now; AI and online in later milestones). */
export interface GameAdapter {
  getSnapshot(): Snapshot;
  subscribe(listener: () => void): () => void;
  /** Returns the engine's message when the action is refused, `null` otherwise. */
  dispatch(player: PlayerId, action: Action): string | null;
  dispose(): void;
}
```

`client/src/adapters/LocalAdapter.ts` :
```ts
import type { Action } from "../engine/generated/Action";
import type { ApplyOutcome } from "../engine/generated/ApplyOutcome";
import type { GameView } from "../engine/generated/GameView";
import type { PlayerId } from "../engine/generated/PlayerId";
import type { Viewer } from "../engine/generated/Viewer";
import type { EngineMatch, GameAdapter, Snapshot } from "./GameAdapter";

/** Hotseat: both players share the screen, which shows the seat that must act. */
export class LocalAdapter implements GameAdapter {
  private snapshot: Snapshot;
  private readonly listeners = new Set<() => void>();

  constructor(private readonly engine: EngineMatch) {
    this.snapshot = { view: this.currentView(), lastEvents: [] };
  }

  private viewAs(viewer: Viewer): GameView {
    return JSON.parse(this.engine.view(JSON.stringify(viewer))) as GameView;
  }

  private currentView(): GameView {
    const spectator = this.viewAs({ type: "Spectator" });
    const actor = spectator.actors[0];
    return actor === undefined ? spectator : this.viewAs({ type: "Seat", player: actor });
  }

  getSnapshot = (): Snapshot => this.snapshot;

  subscribe = (listener: () => void): (() => void) => {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  };

  dispatch(player: PlayerId, action: Action): string | null {
    const outcome = JSON.parse(this.engine.apply(JSON.stringify(player), JSON.stringify(action))) as ApplyOutcome;
    if (outcome.type === "Rejected") return outcome.message;
    this.snapshot = { view: this.currentView(), lastEvents: outcome.events };
    this.listeners.forEach((listener) => listener());
    return null;
  }

  dispose(): void {
    this.engine.free();
  }
}
```

`client/src/adapters/useGame.ts` :
```ts
import { useSyncExternalStore } from "react";
import type { GameAdapter, Snapshot } from "./GameAdapter";

export function useGame(adapter: GameAdapter): Snapshot {
  return useSyncExternalStore(adapter.subscribe, adapter.getSnapshot);
}
```

`client/src/table/selection.ts` :
```ts
import type { AttackOption } from "../engine/generated/AttackOption";
import type { DieId } from "../engine/generated/DieId";

/** The dice the player has clicked so far (UI state only). */
export interface Selection {
  attackers: DieId[];
  targets: DieId[];
}

const sameSet = (a: DieId[], b: DieId[]) => a.length === b.length && a.every((id) => b.includes(id));
const contains = (all: DieId[], some: DieId[]) => some.every((id) => all.includes(id));

/** The engine-listed attacks made of exactly the selected dice. */
export function matchingAttacks(attacks: AttackOption[], selection: Selection): AttackOption[] {
  return attacks.filter(
    (o) => sameSet(o.attack.attackers, selection.attackers) && sameSet(o.attack.targets, selection.targets),
  );
}

/** Dice worth highlighting: those of an engine-listed attack that contains the current selection. */
export function selectableDice(attacks: AttackOption[], selection: Selection) {
  const attackers = new Set<DieId>();
  const targets = new Set<DieId>();
  for (const o of attacks) {
    if (contains(o.attack.attackers, selection.attackers) && contains(o.attack.targets, selection.targets)) {
      o.attack.attackers.forEach((id) => attackers.add(id));
      o.attack.targets.forEach((id) => targets.add(id));
    }
  }
  return { attackers, targets };
}

export function toggle(list: DieId[], id: DieId): DieId[] {
  return list.includes(id) ? list.filter((x) => x !== id) : [...list, id];
}
```

`client/src/engine/wasm.ts` :
```ts
import init, { WasmMatch, catalog_json, skills_json } from "../wasm/buttonmen";
import type { EngineMatch } from "../adapters/GameAdapter";
import type { CatalogEntry } from "./generated/CatalogEntry";
import type { DieSkill } from "./generated/DieSkill";
import type { MatchSetup } from "./generated/MatchSetup";
import type { SkillInfo } from "./generated/SkillInfo";

let ready: Promise<void> | null = null;

export function loadEngine(): Promise<void> {
  ready ??= init().then(() => undefined);
  return ready;
}

/** Original buttons, plus the optional local buttonweavers import (public/buttonweavers/buttons.json). */
export async function loadCatalog(): Promise<CatalogEntry[]> {
  await loadEngine();
  // Without the import, Vite answers with index.html: keep the response only if it is JSON.
  const imported = await fetch("./buttonweavers/buttons.json")
    .then((r) => (r.ok && (r.headers.get("content-type") ?? "").includes("json") ? r.text() : null))
    .catch(() => null);
  return JSON.parse(catalog_json(imported)) as CatalogEntry[];
}

export async function loadSkills(): Promise<Map<DieSkill, SkillInfo>> {
  await loadEngine();
  const skills = JSON.parse(skills_json()) as SkillInfo[];
  return new Map(skills.map((s) => [s.skill, s]));
}

/** Requires `loadEngine()` to have resolved. */
export function createMatch(setup: MatchSetup): EngineMatch {
  return new WasmMatch(JSON.stringify(setup));
}
```

- [ ] **Step 5: Lancer les tests, vérifier qu'ils passent**

Run: `cd client && pnpm test && pnpm type-check`
Expected: PASS ; aucune erreur TypeScript (le module `../wasm/buttonmen` existe depuis `./scripts/build-wasm.sh dev`, Tâche 14).

- [ ] **Step 6: Commit**

```bash
git add client/package.json client/pnpm-lock.yaml client/pnpm-workspace.yaml client/vite.config.ts client/tsconfig.json client/index.html client/src
git commit -m "feat(client): socle React, adaptateur local et aides d'affichage"
```

---
### Task 16: Accueil, choix des boutons et navigation

**Files:**
- Create: `client/src/App.tsx`, `client/src/screens/Home.tsx`, `client/src/screens/ButtonPicker.tsx`, `client/src/screens/catalogFilter.ts`, `client/src/table/SkillsContext.ts`, `client/src/table/seats.ts`
- Create: `client/src/screens/Table.tsx` (version minimale, complétée à la Tâche 17)
- Modify: `client/src/main.tsx`
- Test: `client/src/screens/__tests__/catalogFilter.test.ts`

**Interfaces:**
- Consumes: Tâche 15.
- Produces: `filterCatalog(entries: CatalogEntry[], query: string, source: SourceFilter): CatalogEntry[]` avec `type SourceFilter = "all" | ButtonSource` ; `SkillsContext` (React, `Map<DieSkill, SkillInfo>`) ; `seatLabel(p: PlayerId): string` ; composants `Home({ onHotseat })`, `ButtonPicker({ catalog, onStart, onBack })`, `Table({ adapter, onLeave })`.

- [ ] **Step 1: Écrire le test du filtre**

`client/src/screens/__tests__/catalogFilter.test.ts` :
```ts
import { describe, expect, it } from "vitest";
import type { CatalogEntry } from "../../engine/generated/CatalogEntry";
import { filterCatalog } from "../catalogFilter";

const entry = (name: string, source: CatalogEntry["source"]): CatalogEntry => ({
  name,
  recipe: "(6)",
  set: null,
  source,
  playability: { type: "Playable" },
});

const entries = [entry("Écho", "Original"), entry("Avis", "Buttonweavers"), entry("Avoine", "Original")];

describe("filterCatalog", () => {
  it("filters by source", () => {
    expect(filterCatalog(entries, "", "Original").map((e) => e.name)).toEqual(["Écho", "Avoine"]);
    expect(filterCatalog(entries, "", "all")).toHaveLength(3);
  });

  it("searches names without case or accents", () => {
    expect(filterCatalog(entries, "AV", "all").map((e) => e.name)).toEqual(["Avis", "Avoine"]);
    expect(filterCatalog(entries, "echo", "all").map((e) => e.name)).toEqual(["Écho"]);
  });
});
```

- [ ] **Step 2: Lancer le test, vérifier qu'il échoue**

Run: `cd client && pnpm test catalogFilter`
Expected: FAIL — module `../catalogFilter` introuvable.

- [ ] **Step 3: Implémenter**

`client/src/screens/catalogFilter.ts` :
```ts
import type { ButtonSource } from "../engine/generated/ButtonSource";
import type { CatalogEntry } from "../engine/generated/CatalogEntry";

export type SourceFilter = "all" | ButtonSource;

const fold = (s: string) => s.normalize("NFD").replace(/\p{Diacritic}/gu, "").toLowerCase();

/** Search box and source tabs of the button picker (display filtering only). */
export function filterCatalog(entries: CatalogEntry[], query: string, source: SourceFilter): CatalogEntry[] {
  const q = fold(query.trim());
  return entries.filter((e) => (source === "all" || e.source === source) && fold(e.name).includes(q));
}
```

`client/src/table/SkillsContext.ts` :
```ts
import { createContext } from "react";
import type { DieSkill } from "../engine/generated/DieSkill";
import type { SkillInfo } from "../engine/generated/SkillInfo";

/** Skill names and help texts, as provided by the engine. */
export const SkillsContext = createContext<Map<DieSkill, SkillInfo>>(new Map());
```

`client/src/table/seats.ts` :
```ts
import type { PlayerId } from "../engine/generated/PlayerId";

export function seatLabel(p: PlayerId): string {
  return p === "One" ? "Joueur 1" : "Joueur 2";
}
```

`client/src/screens/Home.tsx` :
```tsx
interface Props {
  onHotseat: () => void;
}

export function Home({ onHotseat }: Props) {
  const modes = [
    { title: "Deux joueurs, même écran", text: "Passez-vous l'appareil à chaque tour.", onClick: onHotseat },
    { title: "Contre l'IA", text: "Bientôt.", onClick: undefined },
    { title: "En ligne", text: "Bientôt.", onClick: undefined },
  ];
  return (
    <main className="mx-auto flex max-w-3xl flex-col gap-8 p-8">
      <header>
        <h1 className="font-display text-5xl">Button Men</h1>
        <p className="mt-2 text-ivory/80">Le jeu de dés de James Ernest, dans le navigateur.</p>
      </header>
      <div className="grid gap-4 sm:grid-cols-3">
        {modes.map((m) => (
          <button
            key={m.title}
            type="button"
            disabled={!m.onClick}
            onClick={m.onClick}
            className="rounded-xl bg-felt-dark p-5 text-left shadow-lg transition enabled:hover:ring-2 enabled:hover:ring-gold disabled:opacity-50"
          >
            <span className="block font-display text-xl">{m.title}</span>
            <span className="mt-1 block text-sm text-ivory/70">{m.text}</span>
          </button>
        ))}
      </div>
    </main>
  );
}
```

`client/src/screens/ButtonPicker.tsx` :
```tsx
import { useMemo, useState } from "react";
import type { CatalogEntry } from "../engine/generated/CatalogEntry";
import { filterCatalog, type SourceFilter } from "./catalogFilter";

interface Props {
  catalog: CatalogEntry[];
  onStart: (one: CatalogEntry, two: CatalogEntry) => void;
  onBack: () => void;
}

const SOURCES: { value: SourceFilter; label: string }[] = [
  { value: "Original", label: "Originaux" },
  { value: "Buttonweavers", label: "buttonweavers" },
  { value: "all", label: "Tous" },
];

export function ButtonPicker({ catalog, onStart, onBack }: Props) {
  const [query, setQuery] = useState("");
  const [source, setSource] = useState<SourceFilter>("Original");
  const [picks, setPicks] = useState<[CatalogEntry | null, CatalogEntry | null]>([null, null]);
  const [seat, setSeat] = useState<0 | 1>(0);
  const hasImport = catalog.some((e) => e.source === "Buttonweavers");
  const shown = useMemo(() => filterCatalog(catalog, query, source), [catalog, query, source]);

  function pick(entry: CatalogEntry) {
    const next: [CatalogEntry | null, CatalogEntry | null] = [...picks];
    next[seat] = entry;
    setPicks(next);
    if (seat === 0 && !next[1]) setSeat(1);
  }

  function pickRandom() {
    const playable = shown.filter((e) => e.playability.type === "Playable");
    if (playable.length > 0) pick(playable[Math.floor(Math.random() * playable.length)]);
  }

  return (
    <main className="mx-auto flex max-w-5xl flex-col gap-4 p-6">
      <div className="flex items-center justify-between">
        <h1 className="font-display text-3xl">Choix des boutons</h1>
        <button type="button" onClick={onBack} className="text-ivory/70 hover:text-ivory">
          ← Accueil
        </button>
      </div>

      <div className="grid gap-3 sm:grid-cols-2">
        {([0, 1] as const).map((i) => (
          <button
            key={i}
            type="button"
            onClick={() => setSeat(i)}
            className={`rounded-xl p-4 text-left ${seat === i ? "bg-gold text-ink" : "bg-felt-dark"}`}
          >
            <span className="text-sm opacity-70">Joueur {i + 1}</span>
            <span className="block font-display text-xl">{picks[i]?.name ?? "—"}</span>
            <span className="block font-mono text-sm">{picks[i]?.recipe ?? ""}</span>
          </button>
        ))}
      </div>

      <div className="flex flex-wrap items-center gap-2">
        <input
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          placeholder="Rechercher un bouton"
          className="min-w-48 flex-1 rounded-lg bg-ivory px-3 py-2 text-ink"
        />
        {hasImport &&
          SOURCES.map((s) => (
            <button
              key={s.value}
              type="button"
              onClick={() => setSource(s.value)}
              className={`rounded-lg px-3 py-2 ${source === s.value ? "bg-gold text-ink" : "bg-felt-dark"}`}
            >
              {s.label}
            </button>
          ))}
        <button type="button" onClick={pickRandom} className="rounded-lg bg-felt-dark px-3 py-2">
          Au hasard
        </button>
        <button
          type="button"
          disabled={!picks[0] || !picks[1]}
          onClick={() => picks[0] && picks[1] && onStart(picks[0], picks[1])}
          className="rounded-lg bg-gold px-4 py-2 font-semibold text-ink disabled:opacity-40"
        >
          Jouer
        </button>
      </div>

      <ul className="grid gap-2 sm:grid-cols-2 lg:grid-cols-3">
        {shown.map((e) => {
          const playable = e.playability.type === "Playable";
          return (
            <li key={`${e.source}-${e.name}`}>
              <button
                type="button"
                disabled={!playable}
                onClick={() => pick(e)}
                title={e.playability.type === "Unplayable" ? e.playability.reason : undefined}
                className="w-full rounded-lg bg-felt-dark p-3 text-left enabled:hover:ring-2 enabled:hover:ring-gold disabled:opacity-40"
              >
                <span className="font-display">{e.name}</span>
                {e.set && <span className="ml-2 text-xs text-ivory/60">{e.set}</span>}
                <span className="block font-mono text-sm text-ivory/80">{e.recipe}</span>
                {e.playability.type === "Unplayable" && (
                  <span className="block text-xs text-danger">{e.playability.reason}</span>
                )}
              </button>
            </li>
          );
        })}
      </ul>
    </main>
  );
}
```

`client/src/screens/Table.tsx` (minimal, remplacé à la Tâche 17) :
```tsx
import type { GameAdapter } from "../adapters/GameAdapter";
import { useGame } from "../adapters/useGame";

interface Props {
  adapter: GameAdapter;
  onLeave: () => void;
}

export function Table({ adapter, onLeave }: Props) {
  const { view } = useGame(adapter);
  return (
    <main className="p-6">
      <p>Manche {view.round} — en attente : {view.waiting_for.type}</p>
      <button type="button" onClick={onLeave}>Quitter</button>
    </main>
  );
}
```

`client/src/App.tsx` :
```tsx
import { useEffect, useState } from "react";
import type { GameAdapter } from "./adapters/GameAdapter";
import { LocalAdapter } from "./adapters/LocalAdapter";
import type { CatalogEntry } from "./engine/generated/CatalogEntry";
import type { DieSkill } from "./engine/generated/DieSkill";
import type { SkillInfo } from "./engine/generated/SkillInfo";
import { createMatch, loadCatalog, loadSkills } from "./engine/wasm";
import { ButtonPicker } from "./screens/ButtonPicker";
import { Home } from "./screens/Home";
import { Table } from "./screens/Table";
import { SkillsContext } from "./table/SkillsContext";

type Screen = { kind: "home" } | { kind: "pick" } | { kind: "table"; adapter: GameAdapter };

export function App() {
  const [screen, setScreen] = useState<Screen>({ kind: "home" });
  const [catalog, setCatalog] = useState<CatalogEntry[] | null>(null);
  const [skills, setSkills] = useState<Map<DieSkill, SkillInfo>>(new Map());
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    Promise.all([loadCatalog(), loadSkills()])
      .then(([c, s]) => {
        setCatalog(c);
        setSkills(s);
      })
      .catch((e: unknown) => setError(String(e)));
  }, []);

  function start(one: CatalogEntry, two: CatalogEntry) {
    const [seed] = crypto.getRandomValues(new Uint32Array(1));
    const engine = createMatch({
      buttons: [
        { name: one.name, recipe: one.recipe },
        { name: two.name, recipe: two.recipe },
      ],
      seed,
      max_wins: 3,
    });
    setScreen({ kind: "table", adapter: new LocalAdapter(engine) });
  }

  function leave() {
    if (screen.kind === "table") screen.adapter.dispose();
    setScreen({ kind: "home" });
  }

  if (error) return <p className="p-8 text-danger">Le moteur n'a pas pu démarrer : {error}</p>;
  if (!catalog) return <p className="p-8">Chargement…</p>;

  return (
    <SkillsContext.Provider value={skills}>
      {screen.kind === "home" && <Home onHotseat={() => setScreen({ kind: "pick" })} />}
      {screen.kind === "pick" && (
        <ButtonPicker catalog={catalog} onStart={start} onBack={() => setScreen({ kind: "home" })} />
      )}
      {screen.kind === "table" && <Table adapter={screen.adapter} onLeave={leave} />}
    </SkillsContext.Provider>
  );
}
```

`client/src/main.tsx` :
```tsx
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { App } from "./App";
import "./styles.css";

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
```

- [ ] **Step 4: Vérifier**

Run: `cd client && pnpm test && pnpm type-check`
Expected: PASS, aucune erreur TypeScript.

Puis lancer `pnpm dev`, ouvrir l'URL affichée : l'accueil s'affiche ; « Deux joueurs, même écran » mène au choix des boutons ; la liste montre les 20 boutons originaux ; choisir deux boutons puis « Jouer » affiche « Manche 1 — en attente : SpecifyDice ».

- [ ] **Step 5: Commit**

```bash
git add client/src
git commit -m "feat(client): accueil, choix des boutons et navigation"
```

---
### Task 17: Table de jeu, dés et attaques

**Files:**
- Create: `client/src/table/DieChip.tsx`, `client/src/table/PlayerArea.tsx`, `client/src/table/AttackPanel.tsx`, `client/src/table/ActionArea.tsx`, `client/src/table/labels.ts`
- Modify: `client/src/screens/Table.tsx`
- Test: `client/src/table/__tests__/labels.test.ts`

**Interfaces:**
- Consumes: Tâches 15 et 16 (`useGame`, `selection`, `SkillsContext`, `seatLabel`, `formatPoints`).
- Produces: `DieChip({ die, state, onClick?, small? })` avec `type ChipState = "idle" | "selectable" | "selected"` ; `PlayerArea({ player, label, acting, selectable, selected, onDieClick })` ; `AttackPanel({ attacks, canPass, selection, act, onClear })` ; `ActionArea({ view, act, selection, onClearSelection })` (aiguillage par `view.legal.type`, complété à la Tâche 18) ; `attackLabel(kind: AttackKind): string`, `skillTitle(die: DieView, skills): string`.

- [ ] **Step 1: Écrire le test des libellés**

`client/src/table/__tests__/labels.test.ts` :
```ts
import { describe, expect, it } from "vitest";
import type { DieView } from "../../engine/generated/DieView";
import type { SkillInfo } from "../../engine/generated/SkillInfo";
import { attackLabel, skillTitle } from "../labels";

describe("labels", () => {
  it("names attack kinds", () => {
    expect(attackLabel("Power")).toBe("Power");
    expect(attackLabel("Skill")).toBe("Skill");
  });

  it("builds a die tooltip from engine-provided skill texts", () => {
    const die: DieView = {
      id: 3,
      notation: "p(6)",
      skills: ["Poison"],
      sides: { Single: 6 },
      min: 1,
      max: 6,
      value: 4,
      dizzy: false,
    };
    const skills = new Map<SkillInfo["skill"], SkillInfo>([
      ["Poison", { skill: "Poison", letter: "p", name: "Poison", rule: "R 6.7", description: "Points négatifs." }],
    ]);
    expect(skillTitle(die, skills)).toBe("p(6) — 4 sur 1–6\nPoison : Points négatifs.");
  });
});
```

Note : le littéral `sides: { Single: 6 }` suit la sérialisation serde par défaut de `Sides` ; si le type généré `Sides.ts` diffère, adapter le littéral au type généré (le compilateur TypeScript le signale).

- [ ] **Step 2: Lancer le test, vérifier qu'il échoue**

Run: `cd client && pnpm test labels`
Expected: FAIL — module `../labels` introuvable.

- [ ] **Step 3: Implémenter les libellés, le dé et la zone de joueur**

`client/src/table/labels.ts` :
```ts
import type { AttackKind } from "../engine/generated/AttackKind";
import type { DieSkill } from "../engine/generated/DieSkill";
import type { DieView } from "../engine/generated/DieView";
import type { SkillInfo } from "../engine/generated/SkillInfo";

/** Attack kinds keep their Button Men names (docs/RULES.md). */
export function attackLabel(kind: AttackKind): string {
  return kind;
}

/** Tooltip of a die: notation, value and range, then each skill's engine-provided text. */
export function skillTitle(die: DieView, skills: Map<DieSkill, SkillInfo>): string {
  const head = `${die.notation} — ${die.value} sur ${die.min}–${die.max}`;
  const lines = die.skills.map((s) => {
    const info = skills.get(s);
    return info ? `${info.name} : ${info.description}` : s;
  });
  return [head, ...lines].join("\n");
}
```

`client/src/table/DieChip.tsx` :
```tsx
import { motion } from "motion/react";
import { useContext } from "react";
import type { DieView } from "../engine/generated/DieView";
import { skillTitle } from "./labels";
import { SkillsContext } from "./SkillsContext";

export type ChipState = "idle" | "selectable" | "selected";

interface Props {
  die: DieView;
  state: ChipState;
  onClick?: () => void;
  small?: boolean;
}

const RING: Record<ChipState, string> = {
  idle: "ring-1 ring-ink/20",
  selectable: "ring-2 ring-gold/70 hover:ring-gold",
  selected: "ring-4 ring-gold -translate-y-1",
};

export function DieChip({ die, state, onClick, small = false }: Props) {
  const skills = useContext(SkillsContext);
  const letters = die.skills.map((s) => skills.get(s)?.letter ?? "").join("");
  const size = small ? "h-12 w-12 text-lg" : "h-20 w-20 text-3xl";
  return (
    <motion.button
      layout
      type="button"
      onClick={onClick}
      disabled={!onClick}
      title={skillTitle(die, skills)}
      className={`relative flex ${size} flex-col items-center justify-center rounded-xl bg-ivory font-display text-ink shadow-md transition ${RING[state]} ${die.dizzy ? "opacity-50" : ""}`}
    >
      <motion.span
        key={`${die.id}-${die.value}`}
        initial={{ rotate: -120, scale: 0.6, opacity: 0 }}
        animate={{ rotate: 0, scale: 1, opacity: 1 }}
        transition={{ type: "spring", stiffness: 300, damping: 18 }}
      >
        {die.value}
      </motion.span>
      <span className="text-[0.65rem] leading-none text-ink/70">
        {letters && <span className="mr-0.5 font-bold text-danger">{letters}</span>}
        {die.max}
      </span>
    </motion.button>
  );
}
```

`client/src/table/PlayerArea.tsx` :
```tsx
import type { DieId } from "../engine/generated/DieId";
import type { PlayerView } from "../engine/generated/PlayerView";
import { formatPoints } from "../engine/format";
import { DieChip, type ChipState } from "./DieChip";

interface Props {
  player: PlayerView;
  label: string;
  acting: boolean;
  selectable: Set<DieId>;
  selected: DieId[];
  onDieClick?: (id: DieId) => void;
}

export function PlayerArea({ player, label, acting, selectable, selected, onDieClick }: Props) {
  const chipState = (id: DieId): ChipState =>
    selected.includes(id) ? "selected" : selectable.has(id) ? "selectable" : "idle";
  return (
    <section className={`rounded-2xl bg-felt-dark/70 p-4 ${acting ? "ring-2 ring-gold" : ""}`}>
      <header className="mb-3 flex flex-wrap items-baseline gap-x-4 gap-y-1">
        <span className="font-display text-xl">{player.button}</span>
        <span className="text-sm text-ivory/70">{label}</span>
        <span className="font-mono text-xs text-ivory/60">{player.recipe}</span>
        <span className="ml-auto text-sm">
          Manches : <strong>{player.record.wins}</strong> · Score : <strong>{formatPoints(player.round_score)}</strong>
        </span>
      </header>
      <div className="flex min-h-20 flex-wrap gap-3">
        {player.active.map((die) => (
          <DieChip
            key={die.id}
            die={die}
            state={chipState(die.id)}
            onClick={onDieClick && (selectable.has(die.id) || selected.includes(die.id)) ? () => onDieClick(die.id) : undefined}
          />
        ))}
      </div>
      {(player.captured.length > 0 || player.reserve.length > 0) && (
        <div className="mt-3 flex flex-wrap items-center gap-2 text-sm text-ivory/70">
          {player.captured.length > 0 && <span>Capturés :</span>}
          {player.captured.map((c) => (
            <DieChip key={c.die.id} die={c.die} state="idle" small />
          ))}
          {player.reserve.length > 0 && (
            <span className="ml-4">Réserve : {player.reserve.map((r) => r.notation).join(" ")}</span>
          )}
        </div>
      )}
    </section>
  );
}
```

- [ ] **Step 4: Implémenter le panneau d'attaque, l'aiguillage et la table**

`client/src/table/AttackPanel.tsx` :
```tsx
import type { Action } from "../engine/generated/Action";
import type { AttackOption } from "../engine/generated/AttackOption";
import { attackLabel } from "./labels";
import { matchingAttacks, type Selection } from "./selection";

interface Props {
  attacks: AttackOption[];
  canPass: boolean;
  selection: Selection;
  act: (action: Action) => void;
  onClear: () => void;
}

export function AttackPanel({ attacks, canPass, selection, act, onClear }: Props) {
  const matching = matchingAttacks(attacks, selection);
  const empty = selection.attackers.length === 0 && selection.targets.length === 0;
  return (
    <div className="flex flex-wrap items-center gap-2">
      {canPass ? (
        <>
          <span>Aucune attaque possible.</span>
          <button type="button" onClick={() => act({ type: "Pass" })} className="rounded-lg bg-gold px-4 py-2 text-ink">
            Passer
          </button>
        </>
      ) : empty ? (
        <span>Choisissez vos dés attaquants puis la ou les cibles (dés surlignés).</span>
      ) : matching.length === 0 ? (
        <span>Complétez la sélection…</span>
      ) : (
        matching.map((o) => (
          <button
            key={o.attack.kind}
            type="button"
            onClick={() => act({ type: "Attack", attack: o.attack })}
            className="rounded-lg bg-gold px-4 py-2 font-semibold text-ink"
          >
            {attackLabel(o.attack.kind)}
            {o.needs_fire ? " (avec Fire)" : ""}
          </button>
        ))
      )}
      {!empty && (
        <button type="button" onClick={onClear} className="rounded-lg bg-felt px-3 py-2">
          Effacer
        </button>
      )}
    </div>
  );
}
```

`client/src/table/ActionArea.tsx` :
```tsx
import type { Action } from "../engine/generated/Action";
import type { GameView } from "../engine/generated/GameView";
import { AttackPanel } from "./AttackPanel";
import { seatLabel } from "./seats";
import type { Selection } from "./selection";

interface Props {
  view: GameView;
  act: (action: Action) => void;
  selection: Selection;
  onClearSelection: () => void;
}

/** The decision the acting player faces, chosen from `view.legal` (engine-provided). */
export function ActionArea({ view, act, selection, onClearSelection }: Props) {
  const legal = view.legal;
  const who = view.viewer.type === "Seat" ? seatLabel(view.viewer.player) : "";
  let body;
  switch (legal.type) {
    case "Attack":
      body = (
        <AttackPanel
          attacks={legal.attacks}
          canPass={legal.can_pass}
          selection={selection}
          act={act}
          onClear={onClearSelection}
        />
      );
      break;
    case "SpecifyDice":
    case "ChooseReserve":
    case "ReactToInitiative":
    case "AdjustFire":
      // Panels added in Task 18.
      body = <span>{legal.type}</span>;
      break;
    case "Wait":
    case "GameOver":
      body = null;
      break;
  }
  return (
    <section className="rounded-2xl bg-ink/30 p-4">
      {who && <p className="mb-2 font-display text-lg">À {who}</p>}
      {body}
    </section>
  );
}
```

`client/src/screens/Table.tsx` (remplace la version minimale) :
```tsx
import { useEffect, useState } from "react";
import type { GameAdapter } from "../adapters/GameAdapter";
import { useGame } from "../adapters/useGame";
import type { Action } from "../engine/generated/Action";
import type { DieId } from "../engine/generated/DieId";
import { ActionArea } from "../table/ActionArea";
import { PlayerArea } from "../table/PlayerArea";
import { seatLabel } from "../table/seats";
import { selectableDice, toggle, type Selection } from "../table/selection";

interface Props {
  adapter: GameAdapter;
  onLeave: () => void;
}

const EMPTY: Selection = { attackers: [], targets: [] };

export function Table({ adapter, onLeave }: Props) {
  const { view } = useGame(adapter);
  const [selection, setSelection] = useState<Selection>(EMPTY);
  const [error, setError] = useState<string | null>(null);
  const seat = view.viewer.type === "Seat" ? view.viewer.player : null;

  // A new view (after any action) starts a fresh selection.
  useEffect(() => setSelection(EMPTY), [view]);

  function act(action: Action) {
    if (!seat) return;
    setError(adapter.dispatch(seat, action));
  }

  const attacks = view.legal.type === "Attack" ? view.legal.attacks : [];
  const selectable = selectableDice(attacks, selection);

  function onDieClick(owner: "mine" | "theirs") {
    return (id: DieId) =>
      setSelection((s) =>
        owner === "mine" ? { ...s, attackers: toggle(s.attackers, id) } : { ...s, targets: toggle(s.targets, id) },
      );
  }

  // Fixed layout: player 2 at the top, player 1 at the bottom.
  const [one, two] = view.players;
  const area = (p: typeof one) => {
    const mine = p.seat === seat;
    return (
      <PlayerArea
        player={p}
        label={seatLabel(p.seat)}
        acting={view.actors.includes(p.seat)}
        selectable={mine ? selectable.attackers : selectable.targets}
        selected={mine ? selection.attackers : selection.targets}
        onDieClick={attacks.length > 0 ? onDieClick(mine ? "mine" : "theirs") : undefined}
      />
    );
  };

  return (
    <main className="mx-auto flex min-h-screen max-w-5xl flex-col gap-4 p-4">
      <header className="flex items-center gap-4">
        <h1 className="font-display text-2xl">Manche {view.round}</h1>
        <span className="text-sm text-ivory/70">Premier à {view.max_wins} manches</span>
        <div className="ml-auto flex gap-2">
          {seat && view.legal.type !== "GameOver" && (
            <button type="button" onClick={() => act({ type: "Surrender" })} className="rounded-lg bg-danger/80 px-3 py-1">
              Abandonner
            </button>
          )}
          <button type="button" onClick={onLeave} className="rounded-lg bg-felt-dark px-3 py-1">
            Quitter
          </button>
        </div>
      </header>
      {area(two)}
      <ActionArea view={view} act={act} selection={selection} onClearSelection={() => setSelection(EMPTY)} />
      {area(one)}
      {error && (
        <p role="alert" className="rounded-lg bg-danger/90 px-4 py-2">
          {error}
        </p>
      )}
    </main>
  );
}
```

- [ ] **Step 5: Vérifier**

Run: `cd client && pnpm test && pnpm type-check`
Expected: PASS.

Puis `pnpm dev` : choisir **Nacre** pour les deux joueurs (son dé Swing est en réserve, donc la partie commence sans choix de Swing) et jouer quelques attaques : les dés attaquables se surlignent, le bouton d'attaque n'apparaît que pour une sélection valide, la capture déplace le dé dans « Capturés » et le score se met à jour. Les panneaux Swing, Reserve, initiative et Fire affichent pour l'instant leur seul nom : ils arrivent à la Tâche 18.

- [ ] **Step 6: Commit**

```bash
git add client/src
git commit -m "feat(client): table de jeu, dés et sélection des attaques"
```

---
### Task 18: Panneaux Swing, Reserve, initiative et Fire ; passage d'écran ; fin de match

**Files:**
- Create: `client/src/table/SpecifyDicePanel.tsx`, `client/src/table/ReservePanel.tsx`, `client/src/table/InitiativePanel.tsx`, `client/src/table/FirePanel.tsx`, `client/src/table/Handoff.tsx`, `client/src/table/MatchEnd.tsx`, `client/src/table/fireTotals.ts`
- Modify: `client/src/table/ActionArea.tsx`, `client/src/screens/Table.tsx`, `README.md`
- Test: `client/src/table/__tests__/fireTotals.test.ts`

**Interfaces:**
- Consumes: Tâches 15 à 17.
- Produces: les quatre panneaux de décision, `Handoff({ seat, onReady })`, `MatchEnd({ view, onLeave })`, `fireAction(fire, boosts): Action | null` (construit `FireTurnDown` à partir des montants saisis ; `null` si les totaux diffèrent ou sont nuls — simple mise en forme de la saisie, la validation reste au moteur).

- [ ] **Step 1: Écrire le test**

`client/src/table/__tests__/fireTotals.test.ts` :
```ts
import { describe, expect, it } from "vitest";
import { fireAction } from "../fireTotals";

describe("fireAction", () => {
  it("builds a FireTurnDown from non-zero amounts when totals match", () => {
    expect(fireAction({ 4: 2, 5: 0 }, { 1: 2 })).toEqual({
      type: "FireTurnDown",
      fire: [[4, 2]],
      boosts: [[1, 2]],
    });
  });

  it("returns null when totals differ or are zero", () => {
    expect(fireAction({ 4: 3 }, { 1: 2 })).toBeNull();
    expect(fireAction({ 4: 0 }, { 1: 0 })).toBeNull();
  });
});
```

- [ ] **Step 2: Lancer le test, vérifier qu'il échoue**

Run: `cd client && pnpm test fireTotals`
Expected: FAIL — module introuvable.

- [ ] **Step 3: Implémenter les panneaux**

`client/src/table/fireTotals.ts` :
```ts
import type { Action } from "../engine/generated/Action";
import type { DieId } from "../engine/generated/DieId";

type Amounts = Record<DieId, number>;

const entries = (amounts: Amounts): [DieId, number][] =>
  Object.entries(amounts)
    .map(([id, n]) => [Number(id), n] as [DieId, number])
    .filter(([, n]) => n > 0);

/** Turns the amounts typed in the Fire panel into an action; the engine validates it. */
export function fireAction(fire: Amounts, boosts: Amounts): Action | null {
  const f = entries(fire);
  const b = entries(boosts);
  const total = (list: [DieId, number][]) => list.reduce((sum, [, n]) => sum + n, 0);
  if (total(f) === 0 || total(f) !== total(b)) return null;
  return { type: "FireTurnDown", fire: f, boosts: b };
}
```

`client/src/table/SpecifyDicePanel.tsx` :
```tsx
import { useState } from "react";
import type { Action } from "../engine/generated/Action";
import type { OptionRequest } from "../engine/generated/OptionRequest";
import type { SwingRequest } from "../engine/generated/SwingRequest";

interface Props {
  swing: SwingRequest[];
  options: OptionRequest[];
  act: (action: Action) => void;
}

export function SpecifyDicePanel({ swing, options, act }: Props) {
  const [sizes, setSizes] = useState(() => swing.map((r) => r.current ?? r.min));
  const [picks, setPicks] = useState(() => options.map((r) => r.current ?? r.a));

  function submit() {
    act({
      type: "SpecifyDice",
      choice: {
        swing: swing.map((r, i) => [r.letter, sizes[i]]),
        options: options.map((r, i) => [r.die, picks[i]]),
      },
    });
  }

  return (
    <div className="flex flex-col gap-3">
      <p>Choisissez la taille de vos dés (en secret).</p>
      {swing.map((r, i) => (
        <label key={r.letter} className="flex items-center gap-3">
          <span className="w-24 font-mono">Swing {r.letter}</span>
          <input
            type="range"
            min={r.min}
            max={r.max}
            value={sizes[i]}
            onChange={(e) => setSizes(sizes.map((s, j) => (j === i ? Number(e.target.value) : s)))}
            className="flex-1 accent-gold"
          />
          <span className="w-10 text-right font-display text-xl">{sizes[i]}</span>
        </label>
      ))}
      {options.map((r, i) => (
        <div key={r.die} className="flex items-center gap-3">
          {/* Option dice have no die view before the first roll: show their two sizes. */}
          <span className="w-24 font-mono">{`(${r.a}/${r.b})`}</span>
          {[r.a, r.b].map((size) => (
            <button
              key={size}
              type="button"
              onClick={() => setPicks(picks.map((p, j) => (j === i ? size : p)))}
              className={`rounded-lg px-3 py-1 ${picks[i] === size ? "bg-gold text-ink" : "bg-felt"}`}
            >
              {size}
            </button>
          ))}
        </div>
      ))}
      <button type="button" onClick={submit} className="self-start rounded-lg bg-gold px-4 py-2 font-semibold text-ink">
        Valider
      </button>
    </div>
  );
}
```

`client/src/table/ReservePanel.tsx` :
```tsx
import type { Action } from "../engine/generated/Action";
import type { DieId } from "../engine/generated/DieId";

interface Props {
  dice: DieId[];
  notation: (id: DieId) => string;
  act: (action: Action) => void;
}

export function ReservePanel({ dice, notation, act }: Props) {
  return (
    <div className="flex flex-wrap items-center gap-2">
      <span>Vous avez perdu la manche : ajouter un dé de réserve ?</span>
      {dice.map((die) => (
        <button
          key={die}
          type="button"
          onClick={() => act({ type: "ChooseReserve", die })}
          className="rounded-lg bg-gold px-3 py-1 font-mono text-ink"
        >
          {notation(die)}
        </button>
      ))}
      <button type="button" onClick={() => act({ type: "ChooseReserve", die: null })} className="rounded-lg bg-felt px-3 py-1">
        Non merci
      </button>
    </div>
  );
}
```

`client/src/table/InitiativePanel.tsx` :
```tsx
import { useState } from "react";
import type { Action } from "../engine/generated/Action";
import type { DieId } from "../engine/generated/DieId";
import type { FocusRange } from "../engine/generated/FocusRange";

interface Props {
  focus: FocusRange[];
  chance: DieId[];
  notation: (id: DieId) => string;
  act: (action: Action) => void;
}

export function InitiativePanel({ focus, chance, notation, act }: Props) {
  const [lowered, setLowered] = useState<Record<DieId, number>>(() =>
    Object.fromEntries(focus.map((f) => [f.die, f.current])),
  );
  const changed = focus.filter((f) => lowered[f.die] < f.current).map((f) => [f.die, lowered[f.die]] as [DieId, number]);

  return (
    <div className="flex flex-col gap-3">
      <p>Votre adversaire a l'initiative. Voulez-vous réagir ?</p>
      {focus.map((f) => (
        <label key={f.die} className="flex items-center gap-3">
          <span className="w-24 font-mono">{notation(f.die)}</span>
          <select
            value={lowered[f.die]}
            onChange={(e) => setLowered({ ...lowered, [f.die]: Number(e.target.value) })}
            className="rounded bg-ivory px-2 py-1 text-ink"
          >
            {Array.from({ length: f.current - f.min + 1 }, (_, k) => f.current - k).map((v) => (
              <option key={v} value={v}>
                {v}
              </option>
            ))}
          </select>
        </label>
      ))}
      <div className="flex flex-wrap gap-2">
        {focus.length > 0 && (
          <button
            type="button"
            disabled={changed.length === 0}
            onClick={() => act({ type: "FocusTurnDown", dice: changed })}
            className="rounded-lg bg-gold px-3 py-1 text-ink disabled:opacity-40"
          >
            Baisser (Focus)
          </button>
        )}
        {chance.map((die) => (
          <button
            key={die}
            type="button"
            onClick={() => act({ type: "ChanceReroll", die })}
            className="rounded-lg bg-gold px-3 py-1 text-ink"
          >
            Relancer {notation(die)} (Chance)
          </button>
        ))}
        <button type="button" onClick={() => act({ type: "PassInitiative" })} className="rounded-lg bg-felt px-3 py-1">
          Laisser commencer l'adversaire
        </button>
      </div>
    </div>
  );
}
```

`client/src/table/FirePanel.tsx` :
```tsx
import { useState } from "react";
import type { Action } from "../engine/generated/Action";
import type { DieId } from "../engine/generated/DieId";
import type { FireCapacity } from "../engine/generated/FireCapacity";
import { fireAction } from "./fireTotals";

interface Props {
  fire: FireCapacity[];
  boosts: FireCapacity[];
  notation: (id: DieId) => string;
  act: (action: Action) => void;
}

function Amounts({ title, list, values, onChange, notation }: {
  title: string;
  list: FireCapacity[];
  values: Record<DieId, number>;
  onChange: (die: DieId, n: number) => void;
  notation: (id: DieId) => string;
}) {
  return (
    <fieldset className="flex flex-col gap-2">
      <legend className="text-sm text-ivory/70">{title}</legend>
      {list.map((c) => (
        <label key={c.die} className="flex items-center gap-3">
          <span className="w-24 font-mono">{notation(c.die)}</span>
          <input
            type="number"
            min={0}
            max={c.max_amount}
            value={values[c.die] ?? 0}
            onChange={(e) => onChange(c.die, Math.max(0, Math.min(c.max_amount, Number(e.target.value))))}
            className="w-20 rounded bg-ivory px-2 py-1 text-ink"
          />
          <span className="text-xs text-ivory/60">max {c.max_amount}</span>
        </label>
      ))}
    </fieldset>
  );
}

export function FirePanel({ fire, boosts, notation, act }: Props) {
  const [taken, setTaken] = useState<Record<DieId, number>>({});
  const [added, setAdded] = useState<Record<DieId, number>>({});
  const action = fireAction(taken, added);
  return (
    <div className="flex flex-col gap-3">
      <p>Cette attaque demande l'aide de vos dés Fire : retirez des points aux dés Fire, ajoutez-les aux attaquants.</p>
      <div className="flex flex-wrap gap-8">
        <Amounts title="Retirer (Fire)" list={fire} values={taken} onChange={(d, n) => setTaken({ ...taken, [d]: n })} notation={notation} />
        <Amounts title="Ajouter (attaquants)" list={boosts} values={added} onChange={(d, n) => setAdded({ ...added, [d]: n })} notation={notation} />
      </div>
      <div className="flex gap-2">
        <button
          type="button"
          disabled={!action}
          onClick={() => action && act(action)}
          className="rounded-lg bg-gold px-4 py-2 font-semibold text-ink disabled:opacity-40"
        >
          Attaquer
        </button>
        <button type="button" onClick={() => act({ type: "CancelFire" })} className="rounded-lg bg-felt px-3 py-2">
          Annuler l'attaque
        </button>
      </div>
    </div>
  );
}
```

`client/src/table/Handoff.tsx` :
```tsx
import type { PlayerId } from "../engine/generated/PlayerId";
import { seatLabel } from "./seats";

/** Hotseat: hides the screen between two secret choices. */
export function Handoff({ seat, onReady }: { seat: PlayerId; onReady: () => void }) {
  return (
    <div className="fixed inset-0 z-10 flex flex-col items-center justify-center gap-4 bg-felt-dark">
      <p className="font-display text-3xl">Passez l'écran à {seatLabel(seat)}</p>
      <button type="button" onClick={onReady} className="rounded-lg bg-gold px-6 py-3 font-semibold text-ink">
        C'est moi
      </button>
    </div>
  );
}
```

`client/src/table/MatchEnd.tsx` :
```tsx
import type { GameView } from "../engine/generated/GameView";
import { seatLabel } from "./seats";

export function MatchEnd({ view, onLeave }: { view: GameView; onLeave: () => void }) {
  if (view.waiting_for.type !== "GameOver") return null;
  const { winner, reason } = view.waiting_for.result;
  const champion = view.players.find((p) => p.seat === winner);
  const [one, two] = view.players;
  return (
    <div className="fixed inset-0 z-10 flex flex-col items-center justify-center gap-4 bg-felt-dark/95">
      <p className="font-display text-4xl">
        {seatLabel(winner)} gagne avec {champion?.button}
      </p>
      <p className="text-ivory/80">
        {reason === "Surrender" ? "Par abandon." : `${one.record.wins} manche(s) à ${two.record.wins}.`}
      </p>
      <button type="button" onClick={onLeave} className="rounded-lg bg-gold px-6 py-3 font-semibold text-ink">
        Retour à l'accueil
      </button>
    </div>
  );
}
```

Dans `ActionArea.tsx`, ajouter les imports des quatre panneaux et une fonction de notation, puis remplacer le bras temporaire :
```tsx
  // Recipe notation of any die id, from the engine-provided views.
  const notation = (id: number) => {
    for (const p of view.players) {
      const die = [...p.active, ...p.captured.map((c) => c.die)].find((d) => d.id === id);
      if (die) return die.notation;
      const slot = p.reserve.find((r) => r.id === id);
      if (slot) return slot.notation;
    }
    return `#${id}`;
  };
```
```tsx
    case "SpecifyDice":
      body = <SpecifyDicePanel swing={legal.swing} options={legal.options} act={act} />;
      break;
    case "ChooseReserve":
      body = <ReservePanel dice={legal.dice} notation={notation} act={act} />;
      break;
    case "ReactToInitiative":
      body = <InitiativePanel focus={legal.focus} chance={legal.chance} notation={notation} act={act} />;
      break;
    case "AdjustFire":
      body = <FirePanel fire={legal.fire} boosts={legal.boosts} notation={notation} act={act} />;
      break;
```
Dans `Table.tsx`, ajouter le passage d'écran et la fin de match :
```tsx
import { Handoff } from "../table/Handoff";
import { MatchEnd } from "../table/MatchEnd";
// …
  // Hotseat: before a secret choice, the screen waits for the right player.
  const [readySeat, setReadySeat] = useState<string | null>(null);
  const secret = view.legal.type === "SpecifyDice";
  const needsHandoff = secret && seat !== null && readySeat !== `${view.round}-${seat}`;
```
et, dans le JSX, juste avant `</main>` :
```tsx
      {needsHandoff && seat && <Handoff seat={seat} onReady={() => setReadySeat(`${view.round}-${seat}`)} />}
      <MatchEnd view={view} onLeave={onLeave} />
```

- [ ] **Step 4: Vérifier**

Run: `cd client && pnpm test && pnpm type-check`
Expected: PASS.

- [ ] **Step 5: Partie complète à la main**

Run: `cd client && pnpm dev` puis, dans le navigateur :
1. **Avoine** contre **Brindille** : écran de passage, Swing X puis Y choisis en secret, manche jouée jusqu'au bout, score affiché, manche 2 : seul le perdant rechoisit sa Swing.
2. **Houle** (Focus) contre **Iris** (Chance) : le panneau d'initiative apparaît pour le joueur sans initiative ; baisser un Focus qui donne l'initiative fonctionne, un Focus insuffisant affiche le message du moteur.
3. **Quartz** (Fire) contre n'importe qui : une attaque Power trop faible propose « (avec Fire) », le panneau Fire valide l'attaque quand les totaux sont égaux, « Annuler l'attaque » revient au choix.
4. **Nacre** (Reserve) : après une manche perdue, le panneau Reserve propose `(20)` et `(X)` ; choisir `(X)` demande ensuite sa taille.
5. **Roseau** (Option) et **Sable** (Twin `(X,X)`) : les choix d'Option s'affichent `(4/8)`, le Twin montre une taille double.
6. « Abandonner » termine le match ; « Retour à l'accueil » ramène à l'accueil.

Noter tout écart et le corriger avant le commit (un défaut de règle se corrige dans le moteur, avec un test).

- [ ] **Step 6: README et commit**

Ajouter au `README.md` :
````markdown
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
````

```bash
git add client/src README.md
git commit -m "feat(client): panneaux Swing, Reserve, initiative et Fire ; fin de match"
```

---

## Fin du jalon 1

- [ ] **Vérification finale**

Run:
```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
./scripts/gen-types.sh && git diff --exit-code client/src/engine/generated
./scripts/build-wasm.sh
cd client && pnpm test && pnpm type-check && pnpm build
```
Expected: tout passe ; `git diff --exit-code` confirme que les types générés versionnés sont à jour ; `pnpm build` produit `client/dist/`.
