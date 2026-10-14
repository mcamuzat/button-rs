# Jalon 4A — banc avec stats humaines et compétences simples : plan d'implémentation

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Le banc d'équilibrage compare ses taux aux parties humaines, puis le moteur, l'IA et le client prennent en charge sept compétences (Slow, Mighty, Ornery, TimeAndSpace, Rage, Rush, Boom), ce qui rend jouables environ 25 boutons officiels de plus.

**Architecture:** Chaque compétence est une variante de `DieSkill` interrogée aux accroches de `hooks.rs` (matchs exhaustifs). Rush et Boom ajoutent deux `AttackKind`. La résolution d'une attaque (`resolve.rs`) suit l'ordre de `BMAttack::commit_attack` de buttonweavers et rend un booléen « l'attaquant rejoue » que `round::end_turn` consomme. Le moteur expose `attack_swing` (score immédiat d'une attaque) pour que l'IA n'ait aucune règle de score à elle.

**Tech Stack:** Rust 2024 (crates `buttonmen-engine`, `buttonmen-data`, `buttonmen-ai`, `buttonmen-wasm`), nom 8, proptest, serde_json ; React 19 + Vitest ; ts-rs pour les types générés.

**Spec:** `docs/superpowers/specs/2026-10-14-jalon-4-competences-design.md` (§1, §3 famille A, §4, §6, §7, §8, §9, §10 plan 4A)

## Global Constraints

- Règles reprises de buttonweavers (`src/engine/BMSkill*.php`, `BMAttackRush.php`, `BMAttackBoom.php`, `BMAttack::commit_attack`) ; chaque règle implémentée est annotée `// R x.y: <résumé>` et la section existe dans `docs/RULES.md`.
- Matchs exhaustifs sur `DieSkill` et `AttackKind`, sans joker `_` (sauf ceux qui existent déjà dans `fire.rs`).
- Le moteur porte toute la logique ; l'IA n'a aucune règle de score propre (elle appelle `capture_swing` / `attack_swing`) ; le client n'affiche que ce que le moteur fournit.
- Lettres de recette : `w` Slow, `H` Mighty, `o` Ornery, `^` TimeAndSpace, `G` Rage, `#` Rush, `b` Boom.
- Tout tirage passe par `state.rng` (déterministe pour une graine).
- Interface et textes en français.
- Pas de Tilt dans ce dépôt : `cargo` et `pnpm` se lancent directement. `cargo fmt --all` avant chaque commit ; `cargo clippy --workspace --all-targets -- -D warnings` doit passer.
- Commits conventionnels terminés par `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
- Les noms de personnages buttonweavers et les stats humaines restent dans `data/buttonweavers/` (gitignoré) ; rien de ces fichiers n'est commité.

## Review Focus

1. **Un dé Mighty relancé par une attaque Trip** : l'attaquant Trip ne grandit pas, mais la cible Mighty d'un Trip grandit (BMSkillMighty::pre_roll ne saute que `JustPerformedTripAttack`). → test `mighty_grows_on_a_trip_target_but_not_on_the_trip_attacker` (Tâche 2).
2. **Tour supplémentaire quand l'adversaire n'a plus de dés** : la manche se termine quand même. → test `extra_turn_never_prevents_the_end_of_the_round` (Tâche 3).
3. **Clone Rage dans les manches suivantes et dans la vue** : le clone disparaît à la manche suivante et la vue lui donne une notation. → tests `rage_clone_does_not_outlive_the_round` et `rage_clone_has_a_notation_in_the_view` (Tâche 4).
4. **Joueur qui n'a qu'une attaque Boom possible** : il ne peut pas passer, et l'IA joue le Boom sans boucler. → tests `a_possible_boom_forbids_passing` (Tâche 6) et partie aléatoire avec Boom (Tâche 7).
5. **Rush contre deux cibles dont une Stealth** : refusé, même si l'autre cible est Rush. → test `rush_cannot_capture_stealth` (Tâche 5).

---

## Structure des fichiers

```
scripts/import-buttonweavers.sh            # + stats humaines (data/buttonweavers/stats.json)
crates/ai/Cargo.toml                       # + serde_json en dev-dependency
crates/ai/examples/balance.rs              # + colonne humains, corrélation, candidats competence:<nom>
crates/engine/src/skill.rs                 # 7 variantes, lettres, textes
crates/engine/src/die.rs                   # Sides::grown (Mighty)
crates/engine/src/hooks.rs                 # accroches : initiative, relance (RerollCause), types d'attaque…
crates/engine/src/action.rs                # AttackKind::Rush, AttackKind::Boom
crates/engine/src/attack.rs                # nombres, compétences, valeurs, énumération Rush/Boom
crates/engine/src/resolve.rs               # ordre des effets ; Rage, Boom, Ornery, TimeAndSpace
crates/engine/src/round.rs                 # end_turn(…, plays_again, …)
crates/engine/src/fire.rs                  # appel de end_turn
crates/engine/src/initiative.rs            # relance Chance avec RerollCause
crates/engine/src/state.rs                 # PlayerState::out_of_play, GameState::fresh_die_id
crates/engine/src/setup.rs                 # out_of_play vidé en début de manche
crates/engine/src/event.rs                 # DieResized, ExtraTurn, RageReplaced, BoomRerolled
crates/engine/src/scoring.rs               # capture_swing (Rage), attack_swing
crates/engine/src/view.rs                  # notation d'un dé sans emplacement (clone Rage)
crates/engine/src/testing.rs               # out_of_play dans attack_state
crates/engine/tests/family_a.rs            # nouveau : scénarios des 7 compétences
crates/engine/tests/properties.rs          # pool étendu, conservation réécrite
crates/ai/src/policy.rs                    # attack_gain = attack_swing
crates/engine-wasm/src/bridge.rs           # test du nombre de compétences
crates/button-data/src/catalog.rs          # originaux, test « aucun bouton officiel bloqué par la famille A »
data/buttons.toml                          # 7 nouveaux originaux
docs/RULES.md                              # R 6.16–6.22, R 7.5–7.8
client/src/table/labels.ts                 # extraTurnText
client/src/screens/Table.tsx               # bandeau « Tour supplémentaire »
client/src/engine/generated/*              # régénérés (scripts/gen-types.sh)
```

---

### Task 1: Banc d'équilibrage avec stats humaines

**Files:**
- Modify: `scripts/import-buttonweavers.sh`
- Modify: `crates/ai/Cargo.toml` (dev-dependencies)
- Modify: `crates/ai/examples/balance.rs`

**Interfaces:**
- Consumes: `buttonmen_data::{ButtonRecipe, SetCategory, to_button}`, `buttonmen_engine::{DieSkill, skill_info}`.
- Produces: option `--candidates competence:<Nom>` (boutons officiels jouables qui portent la compétence `<Nom>`, nom de `skill_info`), colonnes « humains » et « écart », ligne « Corrélation banc / humains ». Fichier local `data/buttonweavers/stats.json`.

- [ ] **Step 1: Récupérer les stats dans le script d'import**

Dans `scripts/import-buttonweavers.sh`, après la ligne `python3 -I scripts/import_buttonweavers.py …`, ajouter :

```bash
# Taux de victoire des parties humaines (Button Men Online), pour le banc d'équilibrage.
# Facultatif : le jeu et le banc fonctionnent sans.
curl -fsSL http://buttonmen.s3-website-us-east-1.amazonaws.com/buttonstats.json \
  -o data/buttonweavers/stats.json \
  && echo "stats humaines écrites dans data/buttonweavers/stats.json" \
  || echo "stats humaines indisponibles (le banc s'en passe)"
```

Lancer : `./scripts/import-buttonweavers.sh` (sans argument, il clone buttonweavers). Attendu : `877 boutons écrits…` puis `stats humaines écrites…`, et `python3 -I -c "import json;print(len(json.load(open('data/buttonweavers/stats.json'))['data']))"` affiche environ 760.

- [ ] **Step 2: Ajouter serde_json aux dev-dependencies de l'IA**

Dans `crates/ai/Cargo.toml`, section `[dev-dependencies]`, ajouter :

```toml
serde_json = { workspace = true }
```

- [ ] **Step 3: Écrire les tests (qui échouent)**

Dans `crates/ai/examples/balance.rs`, module `tests`, ajouter :

```rust
    #[test]
    fn human_stats_are_read_from_the_published_json() {
        let json = r#"{"message": "Button stats generated on 2026-09-19",
            "data": {"Avis": {"bset": "Soldiers", "count": 420, "name": "Avis", "rate": 42.6, "tl": true}}}"#;
        let stats = parse_stats(json).unwrap();
        assert_eq!(stats["Avis"], Human { rate: 42.6, count: 420 });
        assert!(parse_stats("[]").is_err());
    }

    #[test]
    fn pearson_correlation() {
        let perfect = [(1.0, 2.0), (2.0, 4.0), (3.0, 6.0)];
        assert!((pearson(&perfect).unwrap() - 1.0).abs() < 1e-9);
        let inverse = [(1.0, 3.0), (2.0, 2.0), (3.0, 1.0)];
        assert!((pearson(&inverse).unwrap() + 1.0).abs() < 1e-9);
        assert_eq!(pearson(&[(1.0, 1.0), (2.0, 2.0)]), None);
        assert_eq!(pearson(&[(1.0, 5.0), (2.0, 5.0), (3.0, 5.0)]), None);
    }

    #[test]
    fn skill_selection_takes_official_playable_buttons_with_that_skill() {
        let json = r#"[
            {"name": "Coil", "recipe": "p(4) (12) p(20) (20) (V)", "set": "BROM", "category": "Official", "set_order": 8},
            {"name": "Fan", "recipe": "p(4) (8) (12) (X)", "set": "Fan", "category": "Unofficial", "set_order": 1},
            {"name": "Avis", "recipe": "(4) (4) (10) (12) (X)", "set": "Soldiers", "category": "Official", "set_order": 0}
        ]"#;
        let recipes = parse_buttonweavers_json(json).unwrap();
        let buttons =
            select_candidates(&recipes, &[Selection::Skill("Poison".to_string())]).unwrap();
        let names: Vec<&str> = buttons.iter().map(|b| b.name.as_str()).collect();
        assert_eq!(names, ["Coil"]);
        assert!(select_candidates(&recipes, &[Selection::Skill("Nope".to_string())]).is_err());
        assert_eq!(
            parse_selection("competence:Poison"),
            Ok(Selection::Skill("Poison".to_string()))
        );
    }

    #[test]
    fn the_report_shows_human_rates_and_the_gap() {
        let summary = Summary {
            name: "Avis".to_string(),
            wins: 30,
            games: 60,
            unfinished: 0,
            worst: None,
            best: None,
        };
        let stats = BTreeMap::from([("Avis".to_string(), Human { rate: 42.6, count: 420 })]);
        let text = report(&[summary], (0.4, 0.6), &stats);
        assert!(text.contains("43 %"), "{text}");
        assert!(text.contains("+7"), "{text}");
    }
```

Run: `cargo test -p buttonmen-ai --example balance`
Expected: échec de compilation (`parse_stats`, `Human`, `pearson`, `Selection::Skill` inconnus ; `report` attend 2 arguments).

- [ ] **Step 4: Implémenter**

En tête de `balance.rs`, compléter la doc des options :

```rust
//!   --candidates originaux | edition:<nom> | bouton:<nom> | recette:<recette> | competence:<Nom>
//!                répétable, les sélections s'ajoutent           (défaut : originaux)
```

et la phrase finale de la doc :

```rust
//! The reference editions and the human win rates (Button Men Online) come from the local
//! buttonweavers import (`./scripts/import-buttonweavers.sh`). Both seats are played by
//! the AI at the same level: a skill the AI plays badly makes its button look weaker than
//! it is.
```

Imports : ajouter `use buttonmen_data::SetCategory;` et `use buttonmen_engine::{DieSkill, skill_info};` (garder les imports existants), et `use serde::Deserialize;`.

Constantes et types, sous `const MAX_STEPS` :

```rust
const STATS: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../data/buttonweavers/stats.json"
);
/// Human rates measured on fewer games than this are too noisy to compare.
const MIN_HUMAN_GAMES: u32 = 30;

/// Win rate of a button in human games on Button Men Online.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
struct Human {
    /// Percent.
    rate: f64,
    count: u32,
}

#[derive(Deserialize)]
struct StatsFile {
    data: BTreeMap<String, Human>,
}

fn parse_stats(json: &str) -> Result<BTreeMap<String, Human>, serde_json::Error> {
    Ok(serde_json::from_str::<StatsFile>(json)?.data)
}

/// The local human stats, or none when the import has not fetched them.
fn load_stats() -> BTreeMap<String, Human> {
    std::fs::read_to_string(STATS)
        .ok()
        .and_then(|json| parse_stats(&json).ok())
        .unwrap_or_default()
}

/// Pearson correlation of `(x, y)` pairs; `None` below three pairs or without spread.
fn pearson(pairs: &[(f64, f64)]) -> Option<f64> {
    if pairs.len() < 3 {
        return None;
    }
    let n = pairs.len() as f64;
    let (mx, my) = pairs
        .iter()
        .fold((0.0, 0.0), |(sx, sy), (x, y)| (sx + x / n, sy + y / n));
    let (mut cov, mut vx, mut vy) = (0.0, 0.0, 0.0);
    for (x, y) in pairs {
        cov += (x - mx) * (y - my);
        vx += (x - mx).powi(2);
        vy += (y - my).powi(2);
    }
    if vx == 0.0 || vy == 0.0 {
        None
    } else {
        Some(cov / (vx.sqrt() * vy.sqrt()))
    }
}
```

`Selection` gagne une variante, et `parse_selection` sa branche (avant le `else`) :

```rust
    /// Official playable buttons with a die of this skill (name as in `skill_info`).
    Skill(String),
```

```rust
    } else if let Some(name) = text.strip_prefix("competence:") {
        Ok(Selection::Skill(name.to_string()))
```

Mettre à jour le message d'erreur de `parse_selection` : `(originaux, edition:<nom>, bouton:<nom>, recette:<recette>, competence:<Nom>)`.

Dans `select_candidates`, ajouter la branche :

```rust
            Selection::Skill(name) => {
                let skill = DieSkill::ALL
                    .into_iter()
                    .find(|s| skill_info(*s).name.eq_ignore_ascii_case(name))
                    .ok_or(format!("compétence inconnue : {name}"))?;
                select(recipes, |r| {
                    r.category == SetCategory::Official
                        && to_button(r).is_ok_and(|b| b.dice.iter().any(|d| d.skills.has(skill)))
                })
            }
```

`report` prend les stats et affiche deux colonnes de plus (remplacer la fonction) :

```rust
fn report(summaries: &[Summary], range: (f64, f64), stats: &BTreeMap<String, Human>) -> String {
    let mut sorted: Vec<&Summary> = summaries.iter().collect();
    sorted.sort_by(|a, b| b.rate().total_cmp(&a.rate()));
    let width = sorted.iter().map(|s| s.name.chars().count()).max().unwrap_or(6).max(6);
    let mut out = String::new();
    let _ = writeln!(
        out,
        "{:width$}  {:>9}  {:>13}  {:>8}  {:>5}  {:11}  pire adversaire / meilleur",
        "Bouton", "victoires", "IC 95 %", "humains", "écart", "verdict"
    );
    for s in sorted {
        let interval = wilson(s.wins, s.games);
        let flag = match verdict(interval, range) {
            Verdict::TooStrong => "TROP FORT",
            Verdict::TooWeak => "TROP FAIBLE",
            Verdict::Normal => "",
        };
        let opponent = |o: &Option<(String, f64)>| {
            o.as_ref()
                .map_or("—".to_string(), |(name, rate)| format!("{name} {}", percent(*rate)))
        };
        let (human, gap) = match stats.get(&s.name) {
            Some(h) if h.count >= MIN_HUMAN_GAMES => (
                percent(h.rate / 100.0),
                format!("{:+.0}", s.rate() * 100.0 - h.rate),
            ),
            _ => ("—".to_string(), "—".to_string()),
        };
        let _ = write!(
            out,
            "{:width$}  {:>9}  {:>13}  {:>8}  {:>5}  {:11}  {} / {}",
            s.name,
            percent(s.rate()),
            format!("{}–{}", percent(interval.0), percent(interval.1)),
            human,
            gap,
            flag,
            opponent(&s.worst),
            opponent(&s.best),
        );
        if s.unfinished > 0 {
            let _ = write!(out, "  ({} parties sans fin)", s.unfinished);
        }
        out.push('\n');
    }
    out
}

/// "Corrélation banc / humains : …" over every measured button with enough human games.
fn correlation_line(summaries: &[&Summary], stats: &BTreeMap<String, Human>) -> String {
    let pairs: Vec<(f64, f64)> = summaries
        .iter()
        .filter_map(|s| {
            let h = stats.get(&s.name).filter(|h| h.count >= MIN_HUMAN_GAMES)?;
            Some((s.rate() * 100.0, h.rate))
        })
        .collect();
    match pearson(&pairs) {
        Some(r) => {
            let gap = pairs.iter().map(|(b, h)| (b - h).abs()).sum::<f64>() / pairs.len() as f64;
            format!(
                "Corrélation banc / humains : {r:.2} sur {} boutons (écart moyen {gap:.1} points)",
                pairs.len()
            )
        }
        None => "Corrélation banc / humains : pas assez de boutons avec des stats humaines".to_string(),
    }
}
```

Dans `main`, charger les stats et les passer partout (remplacer les deux `println!("…{}", report(…))`) :

```rust
    let stats = load_stats();
    println!("Candidats :\n{}", report(&summaries, range, &stats));
    println!("Référence :\n{}", report(&reference_summaries, range, &stats));
    let measured: Vec<&Summary> = summaries.iter().chain(&reference_summaries).collect();
    println!("{}", correlation_line(&measured, &stats));
```

- [ ] **Step 5: Lancer les tests**

Run: `cargo test -p buttonmen-ai --example balance`
Expected: PASS (12 tests).

- [ ] **Step 6: Vérifier sur un vrai run**

Run: `cargo run -p buttonmen-ai --release --example balance -- --games 10 --candidates edition:BRAWL 2>/dev/null | tail -45`
Expected : les colonnes « humains » et « écart » sont remplies pour les boutons de BRAWL et de la référence, et la dernière ligne donne une corrélation (autour de 0,7–0,85).

- [ ] **Step 7: Commit**

```bash
cargo fmt --all && cargo clippy -p buttonmen-ai --all-targets -- -D warnings
git add scripts/import-buttonweavers.sh crates/ai/Cargo.toml crates/ai/examples/balance.rs
git commit -m "feat(ai): le banc compare ses taux aux parties humaines

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 2: Slow et Mighty (initiative, relances qui font grandir)

**Files:**
- Modify: `crates/engine/src/skill.rs`, `crates/engine/src/hooks.rs`, `crates/engine/src/die.rs`, `crates/engine/src/scoring.rs`, `crates/engine/src/resolve.rs`, `crates/engine/src/initiative.rs`, `crates/engine/src/event.rs`, `crates/engine-wasm/src/bridge.rs`, `docs/RULES.md`
- Create: `crates/engine/tests/family_a.rs`

**Interfaces:**
- Consumes: rien des tâches précédentes.
- Produces:
  - `DieSkill::Slow` (`'w'`), `DieSkill::Mighty` (`'H'`).
  - `Sides::grown(self) -> Sides` (die.rs).
  - `pub(crate) enum RerollCause { Normal, TripAttacker }` et `pub(crate) fn reroll(die: &mut Die, rng: &mut SeededRng, cause: RerollCause) -> Option<Sides>` (hooks.rs) : rend les faces d'avant si le dé a grandi.
  - `GameEvent::DieResized { die: DieId, from: Sides, to: Sides, cause: ResizeCause }` et `pub enum ResizeCause { Mighty }` (event.rs, exporté par lib.rs).

- [ ] **Step 1: Écrire les tests (qui échouent)**

Créer `crates/engine/tests/family_a.rs` :

```rust
mod common;

use PlayerId::{One, Two};
use buttonmen_engine::testing::{attack_state, die, id, opening_state};
use buttonmen_engine::{
    Action, AttackKind, DeclaredAttack, DieSkill, GameEvent, GameState, PlayerId, ResizeCause,
    SeededRng, Sides, WaitingFor,
};
use common::ok;

fn attack(state: &GameState, kind: AttackKind, attackers: &[usize], targets: &[usize]) -> Action {
    Action::Attack {
        attack: DeclaredAttack {
            kind,
            attackers: attackers.iter().map(|&i| id(state, One, i)).collect(),
            targets: targets.iter().map(|&i| id(state, Two, i)).collect(),
        },
    }
}

/// The die `index` of `player`'s button, wherever it is still active.
fn active(state: &GameState, player: PlayerId, index: usize) -> &buttonmen_engine::Die {
    state
        .player(player)
        .die(id(state, player, index))
        .expect("still active")
}

#[test]
fn slow_dice_do_not_count_for_initiative() {
    // R 6.16: without Slow, One's 1 would win the initiative.
    let s = opening_state(
        vec![die(6, 1).with(DieSkill::Slow), die(10, 9)],
        vec![die(6, 3), die(10, 8)],
    );
    assert_eq!(s.initiative, Some(Two));
}

#[test]
fn sides_grow_through_the_mighty_sizes() {
    // R 6.17: 1, 2, 4, 6, 8, 10, 12, 16, 20, 30; an off-list size grows to the next one.
    assert_eq!(Sides::Single(4).grown(), Sides::Single(6));
    assert_eq!(Sides::Single(12).grown(), Sides::Single(16));
    assert_eq!(Sides::Single(5).grown(), Sides::Single(6));
    assert_eq!(Sides::Single(30).grown(), Sides::Single(30));
    assert_eq!(Sides::Single(40).grown(), Sides::Single(40));
    assert_eq!(Sides::Twin(4, 6).grown(), Sides::Twin(6, 8));
}

#[test]
fn a_mighty_attacker_grows_before_rerolling() {
    let s = attack_state(vec![die(6, 5).with(DieSkill::Mighty)], vec![die(6, 3), die(20, 20)], One);
    let (after, events) = ok(&s, One, attack(&s, AttackKind::Power, &[0], &[0]));
    let grown = active(&after, One, 0);
    assert_eq!(grown.sides, Sides::Single(8));
    assert!((1..=8).contains(&grown.value));
    assert!(events.iter().any(|e| matches!(
        e,
        GameEvent::DieResized { from: Sides::Single(6), to: Sides::Single(8), cause: ResizeCause::Mighty, .. }
    )));
}

#[test]
fn mighty_grows_on_a_trip_target_but_not_on_the_trip_attacker() {
    // BMSkillMighty::pre_roll skips only the die that just made the Trip attack.
    let s = attack_state(
        vec![die(6, 2).with(DieSkill::Trip).with(DieSkill::Mighty)],
        vec![die(4, 4).with(DieSkill::Mighty), die(20, 20)],
        One,
    );
    let (after, _) = ok(&s, One, attack(&s, AttackKind::Trip, &[0], &[0]));
    assert_eq!(active(&after, One, 0).sides, Sides::Single(6));
    let target = after
        .player(Two)
        .die(id(&s, Two, 0))
        .map(|d| d.sides)
        .or_else(|| after.player(One).captured.first().map(|c| c.die.sides));
    assert_eq!(target, Some(Sides::Single(6)));
}

#[test]
fn a_konstant_mighty_die_neither_rerolls_nor_grows() {
    // R 6.5 + R 6.17: the die does not reroll, so Mighty does not trigger.
    let s = attack_state(
        vec![die(6, 3).with(DieSkill::Konstant).with(DieSkill::Mighty), die(4, 2)],
        vec![die(10, 5), die(20, 20)],
        One,
    );
    let (after, _) = ok(&s, One, attack(&s, AttackKind::Skill, &[0, 1], &[0]));
    let k = active(&after, One, 0);
    assert_eq!((k.sides, k.value), (Sides::Single(6), 3));
}

#[test]
fn a_chance_reroll_makes_a_mighty_die_grow() {
    // Interaction documented by buttonweavers: Chance + Mighty grows.
    let mut s = opening_state(
        vec![die(6, 6).with(DieSkill::Chance).with(DieSkill::Mighty)],
        vec![die(6, 1)],
    );
    s.rng = SeededRng::new(3);
    assert_eq!(s.waiting_for, WaitingFor::ReactToInitiative { player: One });
    let (after, _) = ok(&s, One, Action::ChanceReroll { die: id(&s, One, 0) });
    assert_eq!(active(&after, One, 0).sides, Sides::Single(8));
}
```

Run: `cargo test -p buttonmen-engine --test family_a`
Expected: échec de compilation (`DieSkill::Slow`, `Sides::grown`, `ResizeCause`… inconnus).

- [ ] **Step 2: Ajouter les compétences, leurs lettres et leurs textes**

Dans `skill.rs`, insérer `Mighty` (entre `Konstant` et `Null`) et `Slow` (entre `Shadow` et `Speed`) dans l'enum **et** dans `ALL` (qui passe à `[DieSkill; 17]`). Dans `letter()` :

```rust
            DieSkill::Mighty => 'H',
            DieSkill::Slow => 'w',
```

Dans `skill_info` :

```rust
        DieSkill::Mighty => (
            "Mighty",
            "R 6.17",
            "Grandit d'un cran à chaque relance : 1, 2, 4, 6, 8, 10, 12, 16, 20, 30.",
        ),
        DieSkill::Slow => (
            "Slow",
            "R 6.16",
            "Ne compte pas pour l'initiative.",
        ),
```

- [ ] **Step 3: `Sides::grown`**

Dans `die.rs`, dans `impl Sides`, après `halved` :

```rust
    /// R 6.17: the next size in buttonweavers' grow list (BMDie::grow_shrink_die_sizes);
    /// a size at or above the last one stays. A twin grows each of its halves.
    pub fn grown(self) -> Sides {
        const SIZES: [u8; 10] = [1, 2, 4, 6, 8, 10, 12, 16, 20, 30];
        let grow = |n: u8| SIZES.into_iter().find(|&s| s > n).unwrap_or(n);
        match self {
            Sides::Single(n) => Sides::Single(grow(n)),
            Sides::Twin(a, b) => Sides::Twin(grow(a), grow(b)),
        }
    }
```

- [ ] **Step 4: Accroches**

Dans `hooks.rs` :

1. `counts_for_initiative` : la branche `false` devient `DieSkill::Slow | DieSkill::Stinger | DieSkill::Trip => false,` avec le commentaire `// R 6.13 / R 6.14 / R 6.16: Stinger, Trip and Slow dice are ignored for initiative.` ; ajouter `DieSkill::Mighty` à la branche `true`.
2. Ajouter `DieSkill::Mighty | DieSkill::Slow` à la branche sans effet de **chacune** des autres accroches : `rerolls` (branche `true`), `attack_kinds` (les deux boucles), `needs_partner` (branche `false`), `can_be_captured` (branche `true`), `skill_values`, `capture_scoring`. Dans `scoring.rs`, `die_points` : ajouter les deux à la branche `{}`.
3. Remplacer `reroll` par :

```rust
/// Why a die is rerolled. Mighty (and Mood, in milestone 4B) skip the die that has just
/// made a Trip attack (BMSkillMighty::pre_roll, flag `JustPerformedTripAttack`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RerollCause {
    Normal,
    TripAttacker,
}

/// R 6.17: does this die grow before each reroll?
fn grows_on_reroll(die: &Die) -> bool {
    die.skills.iter().any(|skill| match skill {
        DieSkill::Mighty => true,
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
        | DieSkill::Slow
        | DieSkill::Speed
        | DieSkill::Stealth
        | DieSkill::Stinger
        | DieSkill::Trip
        | DieSkill::Value => false,
    })
}

/// Rerolls a die, honouring `rerolls` (R 6.5) and Mighty (R 6.17). Returns the sides
/// before a Mighty growth when the die grew.
pub(crate) fn reroll(die: &mut Die, rng: &mut SeededRng, cause: RerollCause) -> Option<Sides> {
    if !rerolls(die) {
        return None;
    }
    let before = die.sides;
    if grows_on_reroll(die) && cause != RerollCause::TripAttacker {
        die.sides = die.sides.grown();
    }
    die.value = die.sides.roll(rng);
    (die.sides != before).then_some(before)
}
```

(ajouter `use crate::die::Sides;`).

- [ ] **Step 5: Événement et appelants**

Dans `event.rs`, ajouter le type et la variante :

```rust
/// Why a die changed size during a round.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub enum ResizeCause {
    Mighty,
}
```

```rust
    /// R 6.17 (and 4B): a die took a new size before rolling.
    DieResized {
        die: DieId,
        from: Sides,
        to: Sides,
        cause: ResizeCause,
    },
```

Exporter `ResizeCause` dans `lib.rs` (`pub use event::{GameEvent, ResizeCause};`).

Dans `resolve.rs`, `reroll` prend la cause et émet l'événement :

```rust
fn reroll(
    state: &mut GameState,
    owner: PlayerId,
    id: DieId,
    cause: hooks::RerollCause,
    events: &mut Vec<GameEvent>,
) {
    let GameState { players, rng, .. } = state;
    if let Some(die) = players[owner.index()].die_mut(id) {
        if let Some(from) = hooks::reroll(die, rng, cause) {
            events.push(GameEvent::DieResized {
                die: id,
                from,
                to: die.sides,
                cause: ResizeCause::Mighty,
            });
        }
        events.push(GameEvent::DieRerolled {
            die: id,
            value: die.value,
        });
    }
}
```

Appels : dans la branche des attaques qui capturent, `reroll(state, player, attacker, hooks::RerollCause::Normal, events)` ; dans `resolve_trip`, l'attaquant avec `RerollCause::TripAttacker` et la cible avec `RerollCause::Normal`.

Dans `initiative.rs`, `chance_reroll` : `hooks::reroll(die, rng, hooks::RerollCause::Normal);` (la valeur retournée est ignorée ici : l'événement `ChanceRerolled` porte déjà la nouvelle valeur ; ajouter juste avant un `let from = …` et pousser `DieResized` de la même façon si `Some`).

- [ ] **Step 6: Test du pont WASM et règles**

Dans `crates/engine-wasm/src/bridge.rs`, test `catalog_and_skills_serialize` : remplacer `assert_eq!(skills.as_array().unwrap().len(), 15);` par `assert_eq!(skills.as_array().unwrap().len(), buttonmen_engine::DieSkill::ALL.len());`.

Dans `skill.rs`, test `letters_round_trip_and_are_unique` : remplacer `assert_eq!(DieSkill::from_letter('o'), None); // Ornery : hors v1` par `assert_eq!(DieSkill::from_letter('D'), None); // Doppelganger : hors jalon 4`.

Dans `docs/RULES.md`, en fin de R 6 :

```markdown
- **R 6.16 Slow (w).** Ne compte pas pour l'initiative. (Source : `BMSkillSlow.php`.)
- **R 6.17 Mighty (H).** Chaque fois qu'il se relance, pour quelque raison que ce soit, le dé grandit d'abord jusqu'à la taille suivante de la liste 1, 2, 4, 6, 8, 10, 12, 16, 20, 30 (une taille hors liste passe à la suivante ; 30 et plus ne bougent pas ; un Twin fait grandir chaque moitié). Il ne grandit pas au jet de début de manche, ni quand il se relance après avoir fait une attaque Trip, ni s'il ne se relance pas (Konstant). Il compte à sa taille du moment. (Source : `BMSkillMighty.php`, `BMDie::grow`.)
```

Et en fin de R 7 :

```markdown
- **R 7.5** Chance et Mighty : une relance Chance fait grandir le dé. Berserk et Mighty : le dé est divisé, puis grandit en se relançant.
```

Dans `counts_for_initiative`, mettre à jour la doc : `/// R 3.3 + R 6.13 + R 6.14 + R 6.16: does this die count when comparing initiative?` ; et dans RULES R 3.3, compléter la parenthèse : `(R 6.14 Trip, R 6.13 Stinger, R 6.16 Slow)`.

- [ ] **Step 7: Lancer les tests**

Run: `cargo test -p buttonmen-engine && cargo test -p buttonmen-wasm`
Expected: PASS, dont les 6 tests de `family_a`.

- [ ] **Step 8: Commit**

```bash
cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings
git add crates/engine crates/engine-wasm/src/bridge.rs docs/RULES.md
git commit -m "feat(engine): compétences Slow et Mighty (R 6.16, R 6.17)

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 3: Ornery et TimeAndSpace (relances de fin de tour, tour supplémentaire)

**Files:**
- Modify: `crates/engine/src/skill.rs`, `hooks.rs`, `scoring.rs`, `resolve.rs`, `round.rs`, `fire.rs`, `event.rs`, `docs/RULES.md`
- Test: `crates/engine/tests/family_a.rs`

**Interfaces:**
- Consumes: `hooks::RerollCause`, `resolve::reroll(state, owner, id, cause, events)` (Tâche 2).
- Produces:
  - `DieSkill::Ornery` (`'o'`), `DieSkill::TimeAndSpace` (`'^'`).
  - `resolve::resolve_attack(…) -> bool` : `true` quand l'attaquant rejoue.
  - `round::end_turn(state, player, plays_again: bool, events)`.
  - `GameEvent::ExtraTurn { player: PlayerId }`.

- [ ] **Step 1: Écrire les tests (qui échouent)**

Ajouter à `family_a.rs` :

```rust
#[test]
fn ornery_dice_reroll_after_any_attack_of_their_owner() {
    // R 6.18: the Ornery d20 did not attack, yet it rerolls; it stays put on a pass.
    let mut rerolled = 0;
    for seed in 0..20 {
        let mut s = attack_state(
            vec![die(6, 5), die(20, 20).with(DieSkill::Ornery)],
            vec![die(6, 3), die(30, 30)],
            One,
        );
        s.rng = SeededRng::new(seed);
        let (after, events) = ok(&s, One, attack(&s, AttackKind::Power, &[0], &[0]));
        let ornery = id(&s, One, 1);
        assert!(events.iter().any(|e| matches!(e, GameEvent::DieRerolled { die, .. } if *die == ornery)));
        if active(&after, One, 1).value != 20 {
            rerolled += 1;
        }
    }
    assert!(rerolled > 15, "the Ornery die kept 20 too often: {rerolled}");
}

#[test]
fn ornery_dice_do_not_reroll_when_their_owner_passes() {
    let s = attack_state(vec![die(6, 1), die(20, 2).with(DieSkill::Ornery)], vec![die(6, 6), die(20, 20)], One);
    let (after, _) = ok(&s, One, Action::Pass);
    assert_eq!(active(&after, One, 1).value, 2);
}

#[test]
fn an_attacking_ornery_die_rerolls_once() {
    let s = attack_state(vec![die(6, 5).with(DieSkill::Ornery)], vec![die(6, 3), die(20, 20)], One);
    let (_, events) = ok(&s, One, attack(&s, AttackKind::Power, &[0], &[0]));
    let me = id(&s, One, 0);
    let count = events.iter().filter(|e| matches!(e, GameEvent::DieRerolled { die, .. } if *die == me)).count();
    assert_eq!(count, 1);
}

#[test]
fn time_and_space_gives_an_extra_turn_on_an_odd_reroll() {
    // R 6.19: the attacker plays again exactly when its new value is odd.
    let (mut odd, mut even) = (0, 0);
    for seed in 0..40 {
        let mut s = attack_state(
            vec![die(6, 5).with(DieSkill::TimeAndSpace), die(10, 1)],
            vec![die(6, 3), die(20, 20)],
            One,
        );
        s.rng = SeededRng::new(seed);
        let (after, events) = ok(&s, One, attack(&s, AttackKind::Power, &[0], &[0]));
        let value = active(&after, One, 0).value;
        let again = value % 2 == 1;
        let next = if again { One } else { Two };
        assert_eq!(after.waiting_for, WaitingFor::Attack { player: next }, "value {value}");
        assert_eq!(events.iter().any(|e| matches!(e, GameEvent::ExtraTurn { player: One })), again);
        if again { odd += 1 } else { even += 1 }
    }
    assert!(odd > 0 && even > 0);
}

#[test]
fn a_konstant_time_and_space_die_gives_no_extra_turn() {
    // Interaction: Konstant does not reroll, so TimeAndSpace never triggers.
    let s = attack_state(
        vec![die(6, 3).with(DieSkill::Konstant).with(DieSkill::TimeAndSpace), die(4, 2)],
        vec![die(10, 5), die(20, 20)],
        One,
    );
    let (after, _) = ok(&s, One, attack(&s, AttackKind::Skill, &[0, 1], &[0]));
    assert_eq!(after.waiting_for, WaitingFor::Attack { player: Two });
}

#[test]
fn extra_turn_never_prevents_the_end_of_the_round() {
    for seed in 0..20 {
        let mut s = attack_state(vec![die(6, 5).with(DieSkill::TimeAndSpace)], vec![die(6, 3)], One);
        s.rng = SeededRng::new(seed);
        let (_, events) = ok(&s, One, attack(&s, AttackKind::Power, &[0], &[0]));
        assert!(events.iter().any(|e| matches!(e, GameEvent::RoundEnded { .. })));
        assert!(!events.iter().any(|e| matches!(e, GameEvent::ExtraTurn { .. })));
    }
}
```

Run: `cargo test -p buttonmen-engine --test family_a`
Expected: échec de compilation (`DieSkill::Ornery`, `GameEvent::ExtraTurn` inconnus).

- [ ] **Step 2: Compétences**

`skill.rs` : insérer `Ornery` (entre `Null` et `Poison`) et `TimeAndSpace` (entre `Stinger` et `Trip`) dans l'enum et `ALL` (`[DieSkill; 19]`). Lettres : `DieSkill::Ornery => 'o'`, `DieSkill::TimeAndSpace => '^'`. Textes :

```rust
        DieSkill::Ornery => (
            "Ornery",
            "R 6.18",
            "Se relance après chaque attaque de son propriétaire, même s'il n'y participe pas ; pas quand il passe.",
        ),
        DieSkill::TimeAndSpace => (
            "TimeAndSpace",
            "R 6.19",
            "S'il attaque et se relance sur une valeur impaire, son propriétaire rejoue.",
        ),
```

Accroches de `hooks.rs` (toutes, y compris `grows_on_reroll`) et `die_points` : ajouter `DieSkill::Ornery | DieSkill::TimeAndSpace` à la branche sans effet. Ajouter deux accroches :

```rust
/// R 6.18: does this die reroll after every attack of its owner?
pub(crate) fn rerolls_after_any_attack(die: &Die) -> bool {
    die.skills.iter().any(|skill| match skill {
        DieSkill::Ornery => true,
        // …toutes les autres variantes => false (liste exhaustive, comme `grows_on_reroll`)
    })
}

/// R 6.19: does this attacker, rerolled to an odd value, give an extra turn?
pub(crate) fn grants_extra_turn(die: &Die) -> bool {
    die.value % 2 == 1
        && rerolls(die)
        && die.skills.iter().any(|skill| match skill {
            DieSkill::TimeAndSpace => true,
            // …toutes les autres variantes => false (liste exhaustive)
        })
}
```

(écrire la liste complète des 18 autres variantes dans chaque `match`, sans joker).

- [ ] **Step 3: Résolution et fin de tour**

`resolve.rs` : `resolve_attack` rend `bool`. Après le `match attack.kind { … }` existant, ajouter :

```rust
    // R 6.19: one extra turn at most, when a rerolled TimeAndSpace attacker shows odd.
    let plays_again = attack.attackers.iter().any(|&a| {
        state
            .player(player)
            .die(a)
            .is_some_and(hooks::grants_extra_turn)
    });
    // R 6.18: the owner's other Ornery dice reroll after the attack.
    let ornery: Vec<DieId> = state
        .player(player)
        .active
        .iter()
        .filter(|d| !attack.attackers.contains(&d.id) && hooks::rerolls_after_any_attack(d))
        .map(|d| d.id)
        .collect();
    for id in ornery {
        reroll(state, player, id, hooks::RerollCause::Normal, events);
    }
    plays_again
```

`declare_attack` : `let again = resolve_attack(state, player, attack, events); round::end_turn(state, player, again, events);`. Même changement dans `fire.rs` (`fire_turn_down`).

`round.rs` : `end_turn` reçoit `plays_again: bool` ; la branche `else` devient :

```rust
    } else if plays_again {
        // R 6.19.
        events.push(GameEvent::ExtraTurn { player });
        state.waiting_for = WaitingFor::Attack { player };
    } else {
        state.waiting_for = WaitingFor::Attack {
            player: player.other(),
        };
    }
```

`pass` appelle `end_turn(state, player, false, events)`. Mettre à jour la doc de `end_turn` : `/// After an attack or a pass: R 7.4 (dizziness lasts one turn), R 6.19 (extra turn), then R 4.5.`

`event.rs` : ajouter `/// R 6.19.\n    ExtraTurn { player: PlayerId },`.

- [ ] **Step 4: Règles**

`docs/RULES.md`, fin de R 6 :

```markdown
- **R 6.18 Ornery (o).** Se relance après chaque attaque de son propriétaire, qu'il y ait participé ou non (un attaquant Ornery ne se relance qu'une fois). Ne se relance pas quand le joueur passe. (Source : `BMSkillOrnery.php`.)
- **R 6.19 TimeAndSpace (^).** Si un dé TimeAndSpace participe à une attaque et se relance sur une valeur impaire, son propriétaire joue un tour de plus ; un seul tour de plus par attaque. La manche s'arrête quand même si un joueur n'a plus de dés. (Source : `BMSkillTimeAndSpace.php`.)
```

Fin de R 7 :

```markdown
- **R 7.6** Konstant et Ornery : le dé garde sa valeur lors de la relance Ornery. Konstant et TimeAndSpace : un attaquant Konstant ne se relance pas et ne donne pas de tour supplémentaire.
```

- [ ] **Step 5: Lancer les tests**

Run: `cargo test -p buttonmen-engine`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings
git add crates/engine docs/RULES.md
git commit -m "feat(engine): compétences Ornery et TimeAndSpace (R 6.18, R 6.19)

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 4: Rage (clone du dé capturé)

**Files:**
- Modify: `crates/engine/src/skill.rs`, `hooks.rs`, `scoring.rs`, `state.rs`, `resolve.rs`, `event.rs`, `view.rs`, `docs/RULES.md`
- Test: `crates/engine/tests/family_a.rs`

**Interfaces:**
- Consumes: `resolve::resolve_attack` (Tâche 3).
- Produces:
  - `DieSkill::Rage` (`'G'`).
  - `GameState::fresh_die_id(&self) -> DieId` (state.rs).
  - `GameEvent::RageReplaced { player: PlayerId, captured: DieId, replacement: DieId }`.
  - `capture_swing` tient compte de Rage.

- [ ] **Step 1: Écrire les tests (qui échouent)**

Ajouter à `family_a.rs` (et `use buttonmen_engine::{capture_swing, view_for, Viewer};` en tête) :

```rust
#[test]
fn a_captured_rage_die_is_replaced_by_a_clone_without_rage() {
    // R 6.20.
    let s = attack_state(vec![die(10, 9)], vec![die(8, 4).with(DieSkill::Rage).with(DieSkill::Poison), die(20, 20)], One);
    let (after, events) = ok(&s, One, attack(&s, AttackKind::Power, &[0], &[0]));
    let two = after.player(Two);
    assert_eq!(two.active.len(), 2);
    let clone = &two.active[0];
    assert_ne!(clone.id, id(&s, Two, 0));
    assert_eq!(clone.sides, Sides::Single(8));
    assert!(!clone.has(DieSkill::Rage) && clone.has(DieSkill::Poison));
    assert!((1..=8).contains(&clone.value));
    assert_eq!(after.player(One).captured.len(), 1);
    assert!(events.iter().any(|e| matches!(e, GameEvent::RageReplaced { player: Two, .. })));
}

#[test]
fn a_rage_die_that_attacks_loses_rage() {
    let s = attack_state(vec![die(10, 9).with(DieSkill::Rage)], vec![die(8, 4), die(20, 20)], One);
    let (after, _) = ok(&s, One, attack(&s, AttackKind::Power, &[0], &[0]));
    assert!(!active(&after, One, 0).has(DieSkill::Rage));
}

#[test]
fn rage_dice_do_not_count_for_initiative() {
    let s = opening_state(vec![die(6, 1).with(DieSkill::Rage), die(10, 9)], vec![die(6, 3), die(10, 8)]);
    assert_eq!(s.initiative, Some(Two));
}

#[test]
fn a_failed_trip_on_a_rage_die_makes_no_clone() {
    for seed in 0..30 {
        let mut s = attack_state(vec![die(4, 1).with(DieSkill::Trip)], vec![die(20, 20).with(DieSkill::Rage), die(30, 30)], One);
        s.rng = SeededRng::new(seed);
        let (after, events) = ok(&s, One, attack(&s, AttackKind::Trip, &[0], &[0]));
        let captured = !after.player(One).captured.is_empty();
        assert_eq!(events.iter().any(|e| matches!(e, GameEvent::RageReplaced { .. })), captured);
        assert_eq!(after.player(Two).active.len(), 2);
    }
}

#[test]
fn capturing_rage_only_gains_the_captured_points() {
    let s = attack_state(vec![die(10, 9)], vec![die(8, 4).with(DieSkill::Rage)], One);
    let attacker = &s.player(One).active[0];
    let rage = &s.player(Two).active[0];
    // The owner keeps a clone worth the same 4 kept points: only the 16 captured half-points swing.
    assert_eq!(capture_swing(&[attacker], rage), 16);
}

#[test]
fn rage_clone_does_not_outlive_the_round() {
    let s = attack_state(vec![die(10, 9)], vec![die(8, 4).with(DieSkill::Rage)], One);
    // Two now holds only the clone. Force values: Two cannot attack and passes, then One
    // captures the clone, which ends the round.
    let (mut s2, _) = ok(&s, One, attack(&s, AttackKind::Power, &[0], &[0]));
    s2.player_mut(Two).active[0].value = 1;
    s2.player_mut(One).active[0].value = 10;
    let (s3, _) = ok(&s2, Two, Action::Pass);
    let clone = s3.player(Two).active[0].id;
    let capture_clone = Action::Attack {
        attack: DeclaredAttack {
            kind: AttackKind::Power,
            attackers: vec![id(&s, One, 0)],
            targets: vec![clone],
        },
    };
    let (next, events) = ok(&s3, One, capture_clone);
    assert!(events.iter().any(|e| matches!(e, GameEvent::RoundEnded { .. })));
    // The next round starts from the recipe: only the slot die, with Rage again.
    let two = &next.player(Two).active;
    assert_eq!(two.len(), 1);
    assert_eq!(two[0].id, id(&s, Two, 0));
    assert!(two[0].has(DieSkill::Rage));
}

#[test]
fn rage_clone_has_a_notation_in_the_view() {
    let s = attack_state(vec![die(10, 9)], vec![die(8, 4).with(DieSkill::Rage).with(DieSkill::Poison), die(20, 20)], One);
    let (after, _) = ok(&s, One, attack(&s, AttackKind::Power, &[0], &[0]));
    let view = view_for(&after, Viewer::Spectator);
    assert_eq!(view.players[1].active[0].notation, "p(8)");
}
```

Run: `cargo test -p buttonmen-engine --test family_a`
Expected: échec de compilation (`DieSkill::Rage`, `GameEvent::RageReplaced` inconnus).

- [ ] **Step 2: Compétence et accroches**

`skill.rs` : `Rage` entre `Queer` et `Reserve` (enum et `ALL`, `[DieSkill; 20]`), lettre `'G'`, texte :

```rust
        DieSkill::Rage => (
            "Rage",
            "R 6.20",
            "Ne compte pas pour l'initiative. Capturé, il est remplacé chez son propriétaire par un clone sans Rage ; il perd Rage s'il attaque.",
        ),
```

`hooks.rs` : `counts_for_initiative`, branche `false` : `DieSkill::Rage | DieSkill::Slow | DieSkill::Stinger | DieSkill::Trip => false,` (commentaire `// R 6.13 / R 6.14 / R 6.16 / R 6.20`). Toutes les autres accroches et `die_points` : `DieSkill::Rage` dans la branche sans effet. Nouvelle accroche :

```rust
/// R 6.20: is this die replaced by a clone when it is captured?
pub(crate) fn leaves_a_clone(die: &Die) -> bool {
    die.skills.iter().any(|skill| match skill {
        DieSkill::Rage => true,
        // …les 19 autres variantes => false (liste exhaustive)
    })
}
```

- [ ] **Step 3: Identifiant neuf, capture et vue**

`state.rs`, `impl GameState` :

```rust
    /// An id no die of this match uses yet (R 6.20: Rage clones). Ids of slots are never
    /// reused, so a clone never collides with a die of the recipe.
    pub fn fresh_die_id(&self) -> DieId {
        let used = self.players.iter().flat_map(|p| {
            p.slots
                .iter()
                .map(|s| s.id)
                .chain(p.active.iter().map(|d| d.id))
                .chain(p.captured.iter().map(|c| c.die.id))
        });
        DieId(used.map(|d| d.0 + 1).max().unwrap_or(0))
    }
```

(vérifier que le champ de `DieId` est public, `pub struct DieId(pub u16)` ; sinon le rendre `pub`).

`resolve.rs`, `capture` : remplacer les lignes qui retirent le dé par :

```rust
    let die = state.player_mut(defender).active.remove(index);
    // R 6.20: a captured Rage die leaves a clone without Rage, rolled, in its place.
    if hooks::leaves_a_clone(&die) {
        let mut clone = die.clone();
        clone.id = state.fresh_die_id();
        clone.skills.remove(DieSkill::Rage);
        clone.dizzy = false;
        clone.value = clone.sides.roll(&mut state.rng);
        events.push(GameEvent::RageReplaced {
            player: defender,
            captured: target,
            replacement: clone.id,
        });
        state.player_mut(defender).active.insert(index, clone);
    }
```

`resolve_attack`, juste avant le calcul de `plays_again` :

```rust
    // R 6.20: a Rage die that took part in the attack loses Rage.
    for &a in &attack.attackers {
        if let Some(die) = state.player_mut(player).die_mut(a) {
            die.skills.remove(DieSkill::Rage);
        }
    }
```

`event.rs` : `/// R 6.20.\n    RageReplaced { player: PlayerId, captured: DieId, replacement: DieId },`.

`view.rs` : la notation d'un dé sans emplacement est construite depuis le dé.

```rust
/// Notation of a die that has no slot of its own (R 6.20: Rage clone), from its skills and sides.
fn die_notation(die: &Die) -> String {
    let skills: String = die.skills.iter().map(|s| s.letter()).collect();
    let core = match die.sides {
        Sides::Single(n) => n.to_string(),
        Sides::Twin(a, b) => format!("{a},{b}"),
    };
    format!("{skills}({core})")
}

fn notation_of(state: &GameState, die: &Die) -> String {
    PlayerId::ALL
        .iter()
        .find_map(|&p| state.player(p).slot(die.id))
        .map_or_else(|| die_notation(die), |s| s.spec.notation())
}
```

et dans `die_view`, `notation: notation_of(state, die),` (ajouter `use crate::die::Sides;`).

`scoring.rs`, `capture_swing` :

```rust
pub fn capture_swing(attackers: &[&Die], target: &Die) -> i32 {
    let scoring = capture_scoring(attackers, target);
    // R 6.20: a Rage target leaves its owner a clone worth the same kept points.
    let kept_lost = if crate::hooks::leaves_a_clone(target) {
        0
    } else {
        die_points(target, Holding::Kept)
    };
    die_points(target, Holding::Captured(scoring)) + kept_lost
}
```

et compléter sa doc : `/// R 5.1 + R 6.6 + R 6.7 + R 6.15 + R 6.20: …`.

- [ ] **Step 4: Règles**

`docs/RULES.md`, R 6 :

```markdown
- **R 6.20 Rage (G).** Ne compte pas pour l'initiative. Quand un dé Rage est capturé, son propriétaire reçoit à sa place un clone (même taille, mêmes compétences sauf Rage), qui est lancé ; le clone disparaît à la fin de la manche comme tout dé hors recette. Un dé Rage qui participe à une attaque perd Rage. Une attaque Trip ratée ne capture pas et ne crée pas de clone. (Source : `BMSkillRage.php`, `BMAttack::commit_attack`.)
```

et R 3.3 : `(R 6.14 Trip, R 6.13 Stinger, R 6.16 Slow, R 6.20 Rage)`.

- [ ] **Step 5: Lancer les tests**

Run: `cargo test -p buttonmen-engine`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings
git add crates/engine docs/RULES.md
git commit -m "feat(engine): compétence Rage (R 6.20)

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 5: Rush (attaque contre deux dés)

**Files:**
- Modify: `crates/engine/src/skill.rs`, `hooks.rs`, `scoring.rs`, `action.rs`, `attack.rs`, `resolve.rs`, `docs/RULES.md`
- Test: `crates/engine/tests/family_a.rs`

**Interfaces:**
- Consumes: `resolve_attack` (Tâche 3).
- Produces: `DieSkill::Rush` (`'#'`), `AttackKind::Rush` (dans `AttackKind::ALL`).

- [ ] **Step 1: Écrire les tests (qui échouent)**

Ajouter à `family_a.rs` (et `use buttonmen_engine::{AttackError, check_attack};`) :

```rust
fn check(state: &GameState, kind: AttackKind, attackers: &[usize], targets: &[usize]) -> Result<(), AttackError> {
    let Action::Attack { attack } = attack(state, kind, attackers, targets) else { unreachable!() };
    check_attack(state, One, &attack)
}

#[test]
fn a_rush_die_captures_exactly_two_dice_summing_to_its_value() {
    // R 6.21.
    let s = attack_state(vec![die(10, 7).with(DieSkill::Rush)], vec![die(6, 3), die(6, 4), die(6, 2)], One);
    assert_eq!(check(&s, AttackKind::Rush, &[0], &[0, 1]), Ok(()));
    assert_eq!(check(&s, AttackKind::Rush, &[0], &[0, 2]), Err(AttackError::ValuesDoNotMatch));
    assert_eq!(check(&s, AttackKind::Rush, &[0], &[0]), Err(AttackError::WrongDiceCount));
    assert_eq!(check(&s, AttackKind::Rush, &[0], &[0, 1, 2]), Err(AttackError::WrongDiceCount));
}

#[test]
fn any_die_can_rush_two_targets_when_one_of_them_is_rush() {
    let s = attack_state(vec![die(10, 7)], vec![die(6, 3).with(DieSkill::Rush), die(6, 4), die(6, 3)], One);
    assert_eq!(check(&s, AttackKind::Rush, &[0], &[0, 1]), Ok(()));
    let plain = attack_state(vec![die(10, 7)], vec![die(6, 3), die(6, 4)], One);
    assert!(matches!(check(&plain, AttackKind::Rush, &[0], &[0, 1]), Err(AttackError::KindUnavailable { .. })));
}

#[test]
fn rush_cannot_capture_stealth() {
    let s = attack_state(vec![die(10, 7).with(DieSkill::Rush)], vec![die(6, 3).with(DieSkill::Stealth), die(6, 4).with(DieSkill::Rush)], One);
    assert!(matches!(check(&s, AttackKind::Rush, &[0], &[0, 1]), Err(AttackError::TargetProtected { .. })));
}

#[test]
fn stealth_dice_cannot_rush() {
    let s = attack_state(vec![die(10, 7).with(DieSkill::Stealth).with(DieSkill::Rush)], vec![die(6, 3), die(6, 4)], One);
    assert!(matches!(check(&s, AttackKind::Rush, &[0], &[0, 1]), Err(AttackError::KindUnavailable { .. })));
}

#[test]
fn a_rush_attack_captures_both_dice_and_is_listed() {
    let s = attack_state(vec![die(10, 7).with(DieSkill::Rush)], vec![die(6, 3), die(6, 4), die(20, 20)], One);
    let listed = buttonmen_engine::candidate_actions(&s, One);
    assert!(listed.contains(&attack(&s, AttackKind::Rush, &[0], &[0, 1])));
    let (after, _) = ok(&s, One, attack(&s, AttackKind::Rush, &[0], &[0, 1]));
    assert_eq!(after.player(One).captured.len(), 2);
    assert_eq!(after.player(Two).active.len(), 1);
}
```

Run: `cargo test -p buttonmen-engine --test family_a`
Expected: échec de compilation (`AttackKind::Rush`, `DieSkill::Rush` inconnus).

- [ ] **Step 2: Compétence et type d'attaque**

`skill.rs` : `Rush` entre `Reserve` et `Shadow` (enum, `ALL` `[DieSkill; 21]`), lettre `'#'`, texte :

```rust
        DieSkill::Rush => (
            "Rush",
            "R 6.21",
            "Attaque Rush : capture exactement deux dés dont la somme égale sa valeur. Tout dé peut faire une attaque Rush contre deux dés dont l'un est Rush.",
        ),
```

`action.rs` : `Rush` ajouté à `AttackKind` (après `Berserk`) et à `ALL` (`[AttackKind; 7]`) ; doc de l'enum : `/// R 4.2, R 4.3, R 6.1, R 6.9, R 6.10, R 6.14, R 6.21.`

`hooks.rs`, `attack_kinds` : le point de départ devient

```rust
    // R 6.21: any die may make a Rush attack (it still needs a Rush die among the dice
    // involved, checked with the targets in `attack::check_skills`).
    let mut kinds = BTreeSet::from([AttackKind::Power, AttackKind::Skill, AttackKind::Rush]);
```

(Stealth garde déjà seulement Skill, ce qui retire Rush.) Ajouter `DieSkill::Rush` à la branche sans effet de chaque accroche (y compris celles des tâches 2–4) et de `die_points`. Nouvelle accroche :

```rust
/// R 6.21: does this die make Rush attacks possible (as attacker or as target)?
pub(crate) fn enables_rush(die: &Die) -> bool {
    die.skills.iter().any(|skill| match skill {
        DieSkill::Rush => true,
        // …les 20 autres variantes => false (liste exhaustive)
    })
}
```

- [ ] **Step 3: Vérification et énumération**

`attack.rs` :

`check_counts` : `AttackKind::Rush => a == 1 && t == 2,` (commentaire `// R 6.21.`).

`check_skills`, après la boucle des attaquants :

```rust
    // R 6.21: a Rush attack needs a Rush die, attacker or target.
    if kind == AttackKind::Rush
        && !p.attackers.iter().chain(&p.targets).any(|d| hooks::enables_rush(d))
    {
        return Err(AttackError::KindUnavailable {
            die: p.attackers[0].id,
            kind,
        });
    }
```

`values_allow` : `AttackKind::Speed | AttackKind::Berserk | AttackKind::Rush => …` (même somme exacte ; commentaire `// R 6.9 / R 6.1 / R 6.21.`).

`attack_shapes`, dans la boucle `for a in &mine` :

```rust
        // R 6.21: one attacker against each pair of dice.
        if kinds.contains(&AttackKind::Rush) {
            for (i, first) in theirs.iter().enumerate() {
                for second in &theirs[i + 1..] {
                    candidates.push(DeclaredAttack {
                        kind: AttackKind::Rush,
                        attackers: vec![a.id],
                        targets: vec![first.id, second.id],
                    });
                }
            }
        }
```

`resolve.rs`, `resolve_attack` : ajouter `| AttackKind::Rush` à la branche des attaques qui capturent.

- [ ] **Step 4: Règles**

`docs/RULES.md`, R 6 :

```markdown
- **R 6.21 Rush (#).** Attaque Rush : un seul dé capture exactement deux dés adverses dont la somme des valeurs égale la sienne. L'attaque est permise si l'attaquant ou au moins une des deux cibles est Rush : n'importe quel dé peut donc attaquer en Rush deux dés dont un est Rush. Un dé Stealth ne fait pas d'attaque Rush et ne peut pas en être la cible. (Source : `BMAttackRush.php`, `BMAttackSpeed::are_skills_compatible`, `BMAttack::possible_attack_types`.)
```

- [ ] **Step 5: Lancer les tests**

Run: `cargo test -p buttonmen-engine`
Expected: PASS (la propriété « aucune attaque hors liste acceptée » tire maintenant aussi des attaques Rush au hasard).

- [ ] **Step 6: Commit**

```bash
cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings
git add crates/engine docs/RULES.md
git commit -m "feat(engine): compétence et attaque Rush (R 6.21)

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 6: Boom (dé qui sort du jeu, cible relancée) et score immédiat d'une attaque

**Files:**
- Modify: `crates/engine/src/skill.rs`, `hooks.rs`, `scoring.rs`, `action.rs`, `attack.rs`, `resolve.rs`, `state.rs`, `setup.rs`, `testing.rs`, `event.rs`, `lib.rs`, `docs/RULES.md`
- Test: `crates/engine/tests/family_a.rs`

**Interfaces:**
- Consumes: `resolve_attack` (Tâche 3), `RerollCause` (Tâche 2).
- Produces:
  - `DieSkill::Boom` (`'b'`), `AttackKind::Boom`.
  - `PlayerState::out_of_play: Vec<Die>` (dés Boom sortis, vidés en début de manche).
  - `GameEvent::BoomRerolled { boom: DieId, target: DieId, value: u8 }`.
  - `pub fn attack_swing(state: &GameState, player: PlayerId, attack: &DeclaredAttack) -> i32` (scoring.rs, exporté) : demi-points gagnés par `player` si l'attaque réussit, avant relances.

- [ ] **Step 1: Écrire les tests (qui échouent)**

Ajouter à `family_a.rs` (et `use buttonmen_engine::{attack_swing, RuleError};`) :

```rust
#[test]
fn a_boom_die_leaves_play_and_rerolls_its_target() {
    // R 6.22.
    let s = attack_state(vec![die(6, 2).with(DieSkill::Boom), die(10, 1)], vec![die(20, 20), die(4, 4)], One);
    let (after, events) = ok(&s, One, attack(&s, AttackKind::Boom, &[0], &[0]));
    let one = after.player(One);
    assert_eq!(one.active.len(), 1);
    assert_eq!(one.out_of_play.len(), 1);
    assert!(one.captured.is_empty());
    assert_eq!(after.player(Two).active.len(), 2);
    assert!(events.iter().any(|e| matches!(e, GameEvent::BoomRerolled { .. })));
    assert_eq!(after.waiting_for, WaitingFor::Attack { player: Two });
}

#[test]
fn a_boomed_die_scores_for_nobody() {
    let s = attack_state(vec![die(20, 2).with(DieSkill::Boom), die(10, 1)], vec![die(20, 20), die(4, 4)], One);
    let before = buttonmen_engine::round_score(&s, One);
    let (after, _) = ok(&s, One, attack(&s, AttackKind::Boom, &[0], &[0]));
    assert_eq!(buttonmen_engine::round_score(&after, One), before - 20);
}

#[test]
fn boom_may_target_stealth_but_stealth_cannot_boom() {
    let s = attack_state(vec![die(6, 2).with(DieSkill::Boom)], vec![die(6, 3).with(DieSkill::Stealth)], One);
    assert_eq!(check(&s, AttackKind::Boom, &[0], &[0]), Ok(()));
    let stealthy = attack_state(vec![die(6, 2).with(DieSkill::Boom).with(DieSkill::Stealth)], vec![die(6, 3)], One);
    assert!(matches!(check(&stealthy, AttackKind::Boom, &[0], &[0]), Err(AttackError::KindUnavailable { .. })));
}

#[test]
fn a_possible_boom_forbids_passing() {
    // Buttonweavers treats Boom as an obligatory attack: it is not optional like Warrior.
    let s = attack_state(vec![die(6, 1).with(DieSkill::Boom)], vec![die(6, 6)], One);
    assert_eq!(buttonmen_engine::apply(&s, One, Action::Pass).unwrap_err(), RuleError::PassNotAllowed);
}

#[test]
fn boomed_dice_come_back_next_round() {
    let s = attack_state(vec![die(6, 1).with(DieSkill::Boom)], vec![die(6, 6)], One);
    let (after, events) = ok(&s, One, attack(&s, AttackKind::Boom, &[0], &[0]));
    // One has no die left: the round ends and the next one starts from the recipe.
    assert!(events.iter().any(|e| matches!(e, GameEvent::RoundEnded { .. })));
    assert!(after.player(One).out_of_play.is_empty());
}

#[test]
fn attack_swing_counts_captures_rage_and_boom() {
    let s = attack_state(
        vec![die(10, 9), die(8, 2).with(DieSkill::Boom)],
        vec![die(6, 4), die(8, 4).with(DieSkill::Rage)],
        One,
    );
    let swing = |kind, a: &[usize], t: &[usize]| {
        let Action::Attack { attack } = attack(&s, kind, a, t) else { unreachable!() };
        attack_swing(&s, One, &attack)
    };
    assert_eq!(swing(AttackKind::Power, &[0], &[0]), 18);
    assert_eq!(swing(AttackKind::Power, &[0], &[1]), 16);
    // The Boom die leaves One's kept dice (4 points) and captures nothing.
    assert_eq!(swing(AttackKind::Boom, &[1], &[0]), -8);
}
```

Run: `cargo test -p buttonmen-engine --test family_a`
Expected: échec de compilation (`DieSkill::Boom`, `AttackKind::Boom`, `out_of_play`, `attack_swing` inconnus).

- [ ] **Step 2: Compétence, type d'attaque, état**

`skill.rs` : `Boom` entre `Berserk` et `Chance` (enum, `ALL` `[DieSkill; 22]`), lettre `'b'`, texte :

```rust
        DieSkill::Boom => (
            "Boom",
            "R 6.22",
            "Attaque Boom : le dé sort du jeu (il ne compte pour personne) et fait relancer un dé adverse, sans le capturer.",
        ),
```

`action.rs` : `Boom` ajouté à `AttackKind` et à `ALL` (`[AttackKind; 8]`).

`state.rs`, `PlayerState` :

```rust
    /// R 6.22: Boom dice that left play this round; they score for nobody.
    pub out_of_play: Vec<Die>,
```

Initialiser `out_of_play: Vec::new()` dans `setup::new_game`. Dans `setup::start_round`, à côté de `player.captured.clear();`, ajouter `player.out_of_play.clear();`. Dans `testing::attack_state`, à côté de `ps.captured.clear();`, ajouter `ps.out_of_play.clear();`.

- [ ] **Step 3: Accroches**

`hooks.rs`, `attack_kinds`, première boucle :

```rust
            // R 6.22.
            DieSkill::Boom => {
                kinds.insert(AttackKind::Boom);
            }
```

(et `DieSkill::Boom` dans la branche sans effet de la seconde boucle ; Stealth, qui ne garde que Skill, retire Boom.)

`can_be_captured` : `DieSkill::Stealth => (kind == AttackKind::Skill && attackers >= 2) || kind == AttackKind::Boom,` avec le commentaire `// R 6.12 + R 6.22: Boom only rerolls its target, and may aim at a Stealth die.` ; doc de la fonction : `/// R 6.12 + R 6.22: may `target` be the target of a `kind` attack made by `attackers` dice?`.

Toutes les autres accroches et `die_points` : `DieSkill::Boom` dans la branche sans effet.

- [ ] **Step 4: Vérification, énumération, résolution**

`attack.rs` : `check_counts` : `AttackKind::Power | AttackKind::Shadow | AttackKind::Trip | AttackKind::Boom => a == 1 && t == 1,` ; `values_allow` : `// R 6.22: any target.\n        AttackKind::Boom => true,` ; `attack_shapes`, dans la boucle sur les cibles d'un attaquant, ajouter `AttackKind::Boom` au tableau `[AttackKind::Power, AttackKind::Shadow, AttackKind::Trip]`.

`resolve.rs`, `resolve_attack` : nouvelle branche du `match` :

```rust
        AttackKind::Boom => resolve_boom(state, player, attack.attackers[0], attack.targets[0], events),
```

et la fonction :

```rust
/// R 6.22: the Boom die leaves play (scoring for nobody) and the target is rerolled.
fn resolve_boom(
    state: &mut GameState,
    player: PlayerId,
    boom: DieId,
    target: DieId,
    events: &mut Vec<GameEvent>,
) {
    let index = state
        .player(player)
        .active
        .iter()
        .position(|d| d.id == boom)
        .expect("legal attacker");
    let die = state.player_mut(player).active.remove(index);
    state.player_mut(player).out_of_play.push(die);
    reroll(state, player.other(), target, hooks::RerollCause::Normal, events);
    let value = state
        .player(player.other())
        .die(target)
        .expect("legal target")
        .value;
    events.push(GameEvent::BoomRerolled { boom, target, value });
}
```

`event.rs` : `/// R 6.22.\n    BoomRerolled { boom: DieId, target: DieId, value: u8 },`.

- [ ] **Step 5: `attack_swing`**

`scoring.rs` :

```rust
/// Half-points `attack` swings towards `player` if it succeeds, before any reroll: the
/// captured dice (R 5.1, R 6.20), or the Boom die leaving `player`'s kept dice (R 6.22).
/// A Trip counts as if it captured; callers weigh its chance.
pub fn attack_swing(state: &GameState, player: PlayerId, attack: &DeclaredAttack) -> i32 {
    let attackers: Vec<&Die> = attack
        .attackers
        .iter()
        .filter_map(|&id| state.player(player).die(id))
        .collect();
    match attack.kind {
        AttackKind::Boom => -attackers
            .iter()
            .map(|d| die_points(d, Holding::Kept))
            .sum::<i32>(),
        AttackKind::Power
        | AttackKind::Skill
        | AttackKind::Shadow
        | AttackKind::Speed
        | AttackKind::Trip
        | AttackKind::Berserk
        | AttackKind::Rush => attack
            .targets
            .iter()
            .filter_map(|&id| state.player(player.other()).die(id))
            .map(|target| capture_swing(&attackers, target))
            .sum(),
    }
}
```

(imports : `use crate::action::{AttackKind, DeclaredAttack};`). `lib.rs` : `pub use scoring::{attack_swing, capture_swing, round_score};`.

- [ ] **Step 6: Règles**

`docs/RULES.md`, R 6 :

```markdown
- **R 6.22 Boom (b).** Attaque Boom : un dé Boom vise un dé adverse ; le dé Boom sort du jeu jusqu'à la fin de la manche et ne compte pour aucun des deux joueurs ; la cible est relancée, jamais capturée. Une attaque Boom possible interdit de passer (R 4.1). Un dé Stealth ne fait pas d'attaque Boom mais peut en être la cible. (Source : `BMSkillBoom.php`, `BMAttackBoom.php`.)
```

et R 4.1 : `… (y compris avec l'aide de Fire, R 6.3, ou une attaque Boom, R 6.22).`

- [ ] **Step 7: Lancer les tests**

Run: `cargo test -p buttonmen-engine && cargo test --workspace`
Expected: PASS. (L'IA compile encore : `attack_swing` n'est pas encore utilisée.)

- [ ] **Step 8: Commit**

```bash
cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings
git add crates/engine docs/RULES.md
git commit -m "feat(engine): compétence et attaque Boom (R 6.22), score immédiat d'une attaque

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 7: Propriétés du moteur et IA avec la famille A

**Files:**
- Modify: `crates/engine/tests/properties.rs`, `crates/ai/src/policy.rs`, `crates/ai/tests/ai.rs`

**Interfaces:**
- Consumes: `attack_swing` (Tâche 6), toutes les compétences des tâches 2–6.
- Produces: `attack_gain` fondée sur `attack_swing` ; propriétés vraies avec la famille A.

- [ ] **Step 1: Étendre le pool et réécrire la conservation**

Dans `properties.rs`, ajouter au `pool()` (avant le `]` final) :

```rust
        // Milestone 4A: every family A skill, and its documented interactions.
        vec![
            fixed(4).with(Slow),
            fixed(6).with(Rage),
            fixed(8).with(Ornery),
            fixed(10).with(Mighty),
            swing(SwingLetter::X).with(TimeAndSpace),
        ],
        vec![
            fixed(6).with(Rush),
            fixed(8).with(Rush),
            fixed(10).with(Boom),
            fixed(12),
            fixed(4).with(Boom),
        ],
        vec![
            fixed(6).with(Konstant).with(Ornery),
            fixed(6).with(Berserk).with(Mighty),
            fixed(4).with(Trip).with(Mighty),
            fixed(8).with(Rage).with(Speed),
            fixed(10).with(Konstant).with(TimeAndSpace),
        ],
```

Les deux `proptest!` passent de `0usize..7` à `0usize..10`.

Remplacer `total_dice` par :

```rust
/// Dice of the recipes that are in play this round: active, captured or boomed (R 6.22).
/// Rage clones (R 6.20) carry ids that no slot has, and are left out.
fn total_dice(state: &GameState) -> usize {
    let is_slot = |d: &buttonmen_engine::Die| {
        PlayerId::ALL
            .iter()
            .any(|&p| state.player(p).slot(d.id).is_some())
    };
    PlayerId::ALL
        .iter()
        .map(|&p| {
            let ps = state.player(p);
            ps.active
                .iter()
                .chain(ps.captured.iter().map(|c| &c.die))
                .chain(&ps.out_of_play)
                .filter(|d| is_slot(d))
                .count()
        })
        .sum()
}
```

- [ ] **Step 2: Lancer les propriétés**

Run: `cargo test -p buttonmen-engine --test properties --release`
Expected: PASS (toute partie se termine ; toute attaque listée est acceptée ; aucune autre ; conservation). Un échec ici désigne un bug d'une tâche précédente : le corriger dans le code de la compétence concernée, pas dans la propriété.

- [ ] **Step 3: Test IA (qui échoue)**

Dans `crates/ai/tests/ai.rs`, ajouter :

```rust
#[test]
fn ai_matches_with_family_a_skills_always_end() {
    use buttonmen_engine::DieSkill::*;
    let a = button("A", vec![fixed(4).with(Boom), fixed(6).with(Rush), fixed(8).with(Rage), fixed(10).with(Ornery), fixed(12).with(TimeAndSpace)]);
    let b = button("B", vec![fixed(4).with(Slow), fixed(6).with(Mighty), fixed(8).with(Rush), fixed(10), fixed(20).with(Boom)]);
    for seed in 0..6u64 {
        let mut state = new_game([a.clone(), b.clone()], seed, MatchConfig::default());
        let mut steps = 0;
        while !matches!(state.waiting_for, WaitingFor::GameOver { .. }) {
            let actor = state.actors()[0];
            let action = choose_action(&state, actor, Level::Normal, seed + steps).expect("an action");
            state = apply(&state, actor, action).expect("legal").0;
            steps += 1;
            assert!(steps < 3000, "no end");
        }
    }
}

#[test]
fn attack_gain_never_values_a_boom_as_a_capture() {
    use buttonmen_engine::DieSkill::Boom;
    let s = attack_state(vec![die(8, 2).with(Boom), die(4, 1)], vec![die(20, 20)], PlayerId::One);
    let boom = DeclaredAttack {
        kind: AttackKind::Boom,
        attackers: vec![id(&s, PlayerId::One, 0)],
        targets: vec![id(&s, PlayerId::Two, 0)],
    };
    assert!(attack_gain(&s, PlayerId::One, &boom) < 0.0);
}
```

(adapter les `use` en tête de `ai.rs` : `buttonmen_ai::{Level, attack_gain, choose_action}` — exporter `attack_gain` dans `crates/ai/src/lib.rs` s'il ne l'est pas — et `buttonmen_engine::testing::{attack_state, button, die, fixed, id}`, `buttonmen_engine::{AttackKind, DeclaredAttack, MatchConfig, PlayerId, WaitingFor, apply, new_game}`).

Run: `cargo test -p buttonmen-ai --test ai`
Expected: FAIL sur `attack_gain_never_values_a_boom_as_a_capture` (la cible est comptée comme capturée).

- [ ] **Step 4: `attack_gain` depuis le moteur**

`crates/ai/src/policy.rs`, remplacer `attack_gain` :

```rust
/// What `attack` is expected to swing towards `player` (spec §7, greedy move ordering), as
/// the engine scores it (`attack_swing`: captures, Rage, Boom). A Trip only captures
/// sometimes: half its gain.
pub fn attack_gain(state: &GameState, player: PlayerId, attack: &DeclaredAttack) -> f64 {
    let swing = f64::from(attack_swing(state, player, attack));
    if attack.kind == AttackKind::Trip {
        swing / 2.0
    } else {
        swing
    }
}
```

(imports : ajouter `attack_swing` à `use buttonmen_engine::{…}` ; retirer `capture_gain` de `use crate::eval::{…}` s'il n'est plus utilisé dans ce fichier.)

- [ ] **Step 5: Lancer les tests de l'IA, puis l'arène**

Run: `cargo test -p buttonmen-ai && cargo test -p buttonmen-ai --release -- --ignored`
Expected: PASS, dont « Difficile bat Facile » et les tests de rapidité.

- [ ] **Step 6: Commit**

```bash
cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings
git add crates/engine/tests/properties.rs crates/ai
git commit -m "test(engine,ai): propriétés et IA avec les compétences de la famille A

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 8: Client — types régénérés et bandeau « Tour supplémentaire »

**Files:**
- Modify: `client/src/engine/generated/*` (régénérés), `client/src/table/labels.ts`, `client/src/table/__tests__/labels.test.ts`, `client/src/screens/Table.tsx`

**Interfaces:**
- Consumes: `GameEvent::ExtraTurn { player }` (Tâche 3), `AttackKind::{Rush, Boom}` (Tâches 5–6).
- Produces: `extraTurnText(events: GameEvent[], names: Record<PlayerId, string>, mySeat: PlayerId | null): string | null`.

- [ ] **Step 1: Régénérer les types et le WASM**

Run: `./scripts/gen-types.sh && ./scripts/build-wasm.sh dev`
Expected : `AttackKind.ts` contient `"Rush" | "Boom"`, `DieSkill.ts` les 7 nouvelles compétences, `GameEvent.ts` les variantes `DieResized`, `ExtraTurn`, `RageReplaced`, `BoomRerolled`, et `ResizeCause.ts` existe.

- [ ] **Step 2: Test (qui échoue)**

Dans `labels.test.ts`, importer `extraTurnText` et ajouter :

```ts
  it("announces an extra turn from the engine's events", () => {
    const names = { One: "Vous", Two: "IA (Normal)" } as const;
    expect(extraTurnText([{ type: "ExtraTurn", player: "One" }], names, "One")).toBe("Vous rejouez (TimeAndSpace)");
    expect(extraTurnText([{ type: "ExtraTurn", player: "Two" }], names, "One")).toBe("IA (Normal) rejoue (TimeAndSpace)");
    expect(extraTurnText([{ type: "Passed", player: "Two" }], names, "One")).toBeNull();
  });

  it("keeps the names of the new attack kinds", () => {
    expect(attackLabel("Rush")).toBe("Rush");
    expect(attackLabel("Boom")).toBe("Boom");
  });
```

Run: `cd client && pnpm test`
Expected: FAIL (`extraTurnText` n'existe pas).

- [ ] **Step 3: Implémenter**

`labels.ts` :

```ts
import type { GameEvent } from "../engine/generated/GameEvent";

/** "Vous rejouez" / "<name> rejoue" when the last action gave an extra turn (R 6.19). */
export function extraTurnText(
  events: GameEvent[],
  names: Record<PlayerId, string>,
  mySeat: PlayerId | null,
): string | null {
  const extra = events.find((e) => e.type === "ExtraTurn");
  if (!extra || extra.type !== "ExtraTurn") return null;
  return extra.player === mySeat ? "Vous rejouez (TimeAndSpace)" : `${names[extra.player]} rejoue (TimeAndSpace)`;
}
```

`Table.tsx` : récupérer `lastEvents` (`const { view, version, thinking, failure, lastEvents } = useGame(adapter);`), importer `extraTurnText` depuis `../table/labels`, et afficher, juste avant `<ActionArea`, :

```tsx
      {extraTurnText(lastEvents, adapter.names, adapter.mySeat) && (
        <p role="status" className="rounded-lg bg-gold/90 px-4 py-2 text-ink">
          {extraTurnText(lastEvents, adapter.names, adapter.mySeat)}
        </p>
      )}
```

- [ ] **Step 4: Vérifier**

Run: `cd client && pnpm test && pnpm type-check`
Expected: PASS, aucune erreur TypeScript.

- [ ] **Step 5: Commit**

```bash
git add client/src
git commit -m "feat(client): attaques Rush et Boom, bandeau de tour supplémentaire

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 9: Boutons originaux et contrôle de clôture de la famille A

**Files:**
- Modify: `data/buttons.toml`, `crates/button-data/src/catalog.rs`, `crates/engine-wasm/src/bridge.rs`

**Interfaces:**
- Consumes: tout ce qui précède ; l'exemple `balance` (Tâche 1).
- Produces: 7 nouveaux originaux ; test `local_import_has_no_family_a_blocker`.

- [ ] **Step 1: Tests (qui échouent)**

Dans `catalog.rs`, tests : `every_original_button_is_playable_and_unique` compte maintenant `27` boutons (remplacer les deux `20` par `27`) ; `originals_cover_every_v1_skill` est renommé `originals_cover_every_skill` (son corps, qui parcourt `DieSkill::ALL`, ne change pas). Ajouter :

```rust
    #[test]
    fn local_import_has_no_family_a_blocker() {
        // Milestone 4A: no official button is unplayable because of a family A letter.
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../data/buttonweavers/buttons.json");
        let Ok(json) = std::fs::read_to_string(path) else {
            return;
        };
        let entries = catalog(&parse_buttonweavers_json(&json).unwrap());
        for e in entries.iter().filter(|e| e.category == SetCategory::Official) {
            if let Playability::Unplayable { reason } = &e.playability {
                for letter in ['w', 'H', 'o', '^', 'G', '#', 'b'] {
                    assert!(!reason.contains(&format!("« {letter} »")), "{} : {reason}", e.name);
                }
            }
        }
    }
```

Dans `crates/engine-wasm/src/bridge.rs`, `catalog_and_skills_serialize` : `20` devient `27`.

Run: `cargo test -p buttonmen-data -p buttonmen-wasm`
Expected: FAIL (20 boutons au lieu de 27, compétences non couvertes).

- [ ] **Step 2: Mesurer les recettes candidates au banc**

Point de départ (5 dés ; compétences qui ignorent l'initiative placées sur le deuxième dé, cf. rééquilibrage du 2026-10-14) :

| Nom | Recette de départ | Compétence |
|---|---|---|
| Ubac | `(4) w(6) (10) (12) (X)` | Slow |
| Vétiver | `(4) G(8) (10) G(12) (X)` | Rage |
| Wapiti | `o(4) o(6) (10) (12) (X)` | Ornery |
| Xylème | `H(4) H(6) (10) (12) (X)` | Mighty |
| Yeuse | `^(4) (6) ^(10) (12) (X)` | TimeAndSpace |
| Zinnia | `#(6) (8) #(10) (12) (X)` | Rush |
| Ambre | `b(4) (6) (10) (12) (X)` | Boom |

Run :

```bash
cargo build -p buttonmen-ai --release --example balance
target/release/examples/balance --games 30 \
  --candidates "recette:(4) w(6) (10) (12) (X)" --candidates "recette:(4) G(8) (10) G(12) (X)" \
  --candidates "recette:o(4) o(6) (10) (12) (X)" --candidates "recette:H(4) H(6) (10) (12) (X)" \
  --candidates "recette:^(4) (6) ^(10) (12) (X)" --candidates "recette:#(6) (8) #(10) (12) (X)" \
  --candidates "recette:b(4) (6) (10) (12) (X)" 2>/dev/null | head -14
```

Règle d'acceptation : aucune recette marquée TROP FORT / TROP FAIBLE. Pour une recette signalée, essayer jusqu'à trois variantes par série, une variable à la fois, en gardant la compétence signature : (a) déplacer la compétence sur un autre dé, (b) changer la taille du plus petit ou du plus gros dé d'un cran, (c) changer la lettre Swing (X ↔ Z pour renforcer, X ↔ V pour affaiblir). Ne jamais passer à 4 ou 6 dés (le nombre de dés domine tout le reste). Garder la variante la plus proche de 50 %.

- [ ] **Step 3: Ajouter les originaux retenus**

À la fin de `data/buttons.toml`, ajouter les 7 boutons avec les recettes **retenues à l'étape 2** (même format que les existants) :

```toml
[[button]]
name = "Ubac"
recipe = "(4) w(6) (10) (12) (X)"
```

(une entrée `[[button]]` par bouton, dans l'ordre du tableau ; remplacer chaque recette par celle retenue).

- [ ] **Step 4: Lancer les tests**

Run: `cargo test -p buttonmen-data -p buttonmen-wasm`
Expected: PASS (27 originaux, chaque compétence couverte ; aucun bouton officiel bloqué par une lettre de la famille A si l'import est présent).

- [ ] **Step 5: Contrôle de clôture au banc (spec §1)**

Run :

```bash
target/release/examples/balance --games 30 \
  --candidates competence:Slow --candidates competence:Mighty --candidates competence:Ornery \
  --candidates competence:TimeAndSpace --candidates competence:Rage --candidates competence:Rush \
  --candidates competence:Boom 2>/dev/null | tee /tmp/claude-balance-4a.txt | tail -60
```

Critères :
- la dernière ligne donne une corrélation banc / humains **≥ 0,7** ;
- pour chaque compétence, calculer l'écart moyen banc − humains de ses boutons (colonne « écart ») ; tout écart moyen de **plus de 15 points** doit être expliqué avant de clore : relire la règle dans le code buttonweavers et la comparer au moteur (erreur de règle → corriger dans la tâche de la compétence, avec un test), sinon tracer une partie (voir l'enquête Null du 2026-10-14) et conclure à une faiblesse de l'IA ou à l'amplification connue des extrêmes ;
- relancer le banc sur les originaux (`target/release/examples/balance 2>/dev/null | head -32`) : aucun original signalé.

Consigner le résultat (corrélation, écarts par compétence, explications) dans le message du commit.

- [ ] **Step 6: Commit**

```bash
cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
./scripts/build-wasm.sh dev
git add data/buttons.toml crates/button-data/src/catalog.rs crates/engine-wasm/src/bridge.rs
git commit -m "feat(data): originaux de la famille A, contrôle de clôture du jalon 4A

<corrélation banc / humains, écart moyen par compétence, explications>

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

(remplacer la ligne entre chevrons par les mesures de l'étape 5.)
