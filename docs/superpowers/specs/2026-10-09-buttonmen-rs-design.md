# buttonmen-rs — conception

Date : 2026-10-09
Statut : validée en discussion, en attente de relecture de la spec écrite.

## 1. Objectif

Refaire **Button Men** (jeu de dés de James Ernest / Cheapass Games) dans le navigateur, avec l'architecture de phase.rs : un moteur de règles en Rust (natif + WASM), un client React qui n'affiche que ce que le moteur fournit, et un serveur qui fait autorité pour le jeu en ligne.

### Ce que la v1 doit permettre

- Jouer **contre l'IA** (solo, sans serveur, IA en WASM).
- Jouer en **hotseat** (deux joueurs, même écran).
- Jouer **en ligne en temps réel** (salle à code, serveur WebSocket qui fait autorité).
- Règles de base complètes, dés Swing / Option / Twin, et 15 compétences de dés (§5.4).
- Choisir son bouton parmi un jeu de **boutons originaux** livrés dans le dépôt, ou parmi les boutons **importés localement** depuis buttonweavers.

### Hors périmètre v1

- Jeu asynchrone, comptes, classements, tournois, persistance en base.
- Les compétences au-delà des 15 de la v1, les compétences de bouton (`BMBtnSkill*`).
- Handicaps, Auxiliary, Turbo, Mood/Mad, Morphing, etc.
- Illustrations de personnages.

### Critères de réussite

- Une partie complète (3 manches gagnantes) se joue dans les trois modes avec le même composant de table.
- Toute action proposée par `legal_actions` est acceptée par `apply`, et toute partie aléatoire se termine (tests de propriétés).
- Chaque règle implémentée est annotée par sa référence dans `docs/RULES.md`.

## 2. Sources et droits

- **Référence de règles** : Button Men Online / buttonweavers (`github.com/buttonmen-dev/buttonmen`, licence BSD-3). Les règles de base viennent de `src/ui/js/HowToPlay.js`, les compétences de `src/engine/BMSkill*.php`, les bornes Swing de `BMDieSwing.php`, les états de partie de `BMGameState.php`.
- `docs/RULES.md` reformule ces règles en sections numérotées (`R 1.1`, `R 5.3`…), cite la source de chaque section, et sert d'équivalent à la CR de Magic dans phase.rs.
- **Boutons originaux** (`data/buttons.toml`) : personnages et recettes créés pour ce projet, versionnés, diffusables.
- **Boutons buttonweavers** : importés par `scripts/import-buttonweavers.sh` depuis `deploy/database/data.button.sql` vers `data/buttonweavers/buttons.json`, **gitignoré**. Les noms de personnages appartiennent à Cheapass Games ; l'import est réservé à un usage local et privé. Le jeu fonctionne sans lui.

## 3. Principes (repris de phase.rs)

- **Le moteur porte toute la logique.** WASM et serveur sont de simples couches de sérialisation.
- **Le client affiche, il ne calcule pas.** Attaques valides, scores, textes d'aide des compétences : tout vient du moteur.
- **Fidélité aux règles.** Toute règle implémentée est annotée `// R x.y: <résumé>` et la référence existe dans `docs/RULES.md`.
- **Blocs composables.** Une compétence est une variante d'enum interrogée à des points d'accroche ; aucun code spécifique à un bouton.
- **Rust idiomatique.** Enums plutôt que chaînes, `match` exhaustifs sans joker, erreurs typées.
- **Hasard déterministe.** Tout tirage passe par le RNG seedé de l'état.

## 4. Architecture du dépôt

```
buttonmen-rs/
├── Cargo.toml                 # workspace
├── crates/
│   ├── engine/                # règles pures : types, réducteur, compétences, RNG
│   ├── button-data/           # chargement des boutons (TOML originaux + JSON importé), parseur de recettes
│   ├── ai/                    # expectimax sur le moteur
│   ├── engine-wasm/           # pont wasm-bindgen (engine + ai + button-data)
│   └── server/                # axum + WebSocket, salles en mémoire
├── data/
│   ├── buttons.toml           # boutons originaux
│   └── buttonweavers/         # import local (gitignoré)
├── scripts/
│   ├── import-buttonweavers.sh
│   └── build-wasm.sh
├── client/                    # React + Vite + Tailwind 4 + Framer Motion
└── docs/
    ├── RULES.md
    └── superpowers/{specs,plans}/
```

Dépendances : `engine` ne dépend de rien d'autre du workspace. `button-data` → `engine`. `ai` → `engine`. `engine-wasm` → `engine`, `ai`, `button-data`. `server` → `engine`, `button-data`.

### Jalons (un plan d'implémentation chacun)

1. **Hotseat jouable** : `engine`, `button-data`, `engine-wasm`, `client` (accueil, choix des boutons, table, fin de match), `RULES.md`.
2. **Contre l'IA** : `ai`, exposée en WASM, exécutée dans un Web Worker.
3. **En ligne temps réel** : `server`, lobby et adaptateur WebSocket côté client.

## 5. Moteur (`crates/engine`)

### 5.1 État

```rust
pub struct GameState {
    pub players: [PlayerState; 2],
    pub round: u8,
    pub max_wins: u8,                 // 3 par défaut (R : HowToPlay, « The next round »)
    pub waiting_for: WaitingFor,
    pub rng: SeededRng,               // seul générateur de hasard
}

pub struct PlayerState {
    pub button: ButtonRecipe,         // nom + recette d'origine
    pub active: Vec<Die>,             // dés en jeu
    pub captured: Vec<CapturedDie>,   // dés capturés à l'adversaire (+ contexte : Null, Value)
    pub reserve: Vec<Die>,            // dés Reserve pas encore ajoutés
    pub swing_choices: SwingChoices,  // tailles Swing/Option retenues
    pub wins: u8, pub losses: u8, pub draws: u8,
    pub passed_last_turn: bool,
}
```

`GameState` est sérialisable (serde) et sans pointeurs. Les dés ont un identifiant stable (`DieId`) pour que le client, l'IA et le réseau les désignent sans ambiguïté.

### 5.2 Dés

Deux axes indépendants :

```rust
pub enum DieShape {
    Fixed(u8),
    Swing(SwingLetter),               // P R S T U V W X Y Z
    Option(u8, u8),
    Twin(Box<DieShape>, Box<DieShape>),
}

pub struct Die {
    pub id: DieId,
    pub shape: DieShape,
    pub skills: SkillSet,             // ensemble de DieSkill
    pub size: Option<Sides>,          // résolu après SpecifyDice
    pub value: Option<u8>,            // résolu après le jet
    pub flags: DieFlags,              // ex. Focus abaissé pendant l'initiative
}
```

Bornes Swing (source `BMDieSwing.php`) : P 1–30, R 2–16, S 6–20, T 2–12, U 8–30, V 6–12, W 4–12, X 4–20, Y 1–20, Z 4–30.

Un Twin a deux sous-dés : sa taille est la somme des tailles, sa valeur la somme des deux jets.

Le parseur de recettes (`button-data`) lit la notation buttonweavers (`p(4) s(10) (X) (6,10) z(4,4)`…) et la convertit en `Vec<Die>`. Il refuse toute compétence inconnue avec une erreur typée ; ces boutons apparaissent comme « non jouables » dans le client au lieu d'être joués avec des règles fausses.

### 5.3 Actions et attente

États inspirés de `BMGameState.php` :

```rust
pub enum WaitingFor {
    SpecifyDice { pending: PlayerSet },   // choix Swing/Option, simultané, caché jusqu'à révélation
    ChooseReserve { player: PlayerId },   // le perdant de la manche précédente
    ReactToInitiative { player: PlayerId },// Focus / Chance
    Attack { player: PlayerId },
    AdjustFire { player: PlayerId, pending: PendingAttack },
    GameOver { result: MatchResult },
}

pub enum Action {
    SpecifyDice { choices: Vec<(DieId, Sides)> },
    ChooseReserve { die: Option<DieId> },
    FocusTurnDown { dice: Vec<(DieId, u8)> },
    ChanceReroll { die: DieId },
    PassInitiative,
    Attack { kind: AttackKind, attackers: Vec<DieId>, targets: Vec<DieId> },
    FireAdjust { fire_dice: Vec<(DieId, u8)> },
    Pass,
    Surrender,
}

pub enum AttackKind { Power, Skill, Shadow, Speed, Trip, Berserk }
```

API publique :

```rust
pub fn new_game(buttons: [ButtonRecipe; 2], seed: u64, config: MatchConfig) -> GameState;
pub fn apply(state: &GameState, player: PlayerId, action: Action)
    -> Result<(GameState, Vec<GameEvent>), RuleError>;
pub fn legal_actions(state: &GameState, player: PlayerId) -> LegalActions;
pub fn view_for(state: &GameState, viewer: Viewer) -> GameView; // masque les Swing adverses avant révélation
pub fn skill_info(skill: DieSkill) -> SkillInfo;                // nom + texte d'aide pour le client
```

`apply` est pur : il retourne un nouvel état et la liste d'événements (`DiceRolled`, `DieCaptured`, `AttackMade`, `InitiativeDecided`, `RoundEnded`…) qui servent aux animations et au replay. `RuleError` est un enum typé (`NotYourTurn`, `IllegalAttack(AttackError)`, `SwingOutOfRange`, …).

`LegalActions` énumère, pour l'état `Attack`, les attaques valides par type sous une forme exploitable par le client (pour chaque ensemble d'attaquants, les cibles possibles) et par l'IA (liste concrète).

### 5.4 Compétences

```rust
pub enum DieSkill {
    Poison, Shadow, Speed, Trip, Berserk, Focus, Chance,
    Null, Konstant, Stinger, Queer, Reserve, Value, Stealth, Fire,
}
```

Le moteur les interroge à des points d'accroche fixes. Chaque accroche est une fonction avec un `match` exhaustif sur `DieSkill`, de sorte qu'ajouter une compétence oblige le compilateur à demander une décision à chaque accroche.

| Accroche | Rôle | Compétences v1 concernées |
|---|---|---|
| `attack_kinds(die)` | types d'attaque que le dé peut faire | Shadow (Shadow au lieu de Power), Queer (Shadow si impair), Speed, Trip, Berserk (Berserk au lieu de Skill), Konstant (pas de Power, pas de Skill seul), Stealth (Skill multi-dés seulement), Fire (pas de Power) |
| `can_be_captured_by(die, attack)` | protections | Stealth (Skill multi-dés seulement) |
| `skill_contribution(die)` | valeurs possibles dans une Skill attack | Konstant (±valeur), Stinger (de 1 à la valeur affichée) |
| `assist(die, attack)` | aide à une attaque | Fire (Power et Skill) |
| `after_attack(...)` | effets après l'attaque | Berserk (taille ÷ 2 arrondie au-dessus, perd Berserk), Konstant (ne relance pas), Null et Value (marquent les dés capturés) |
| `score(die, ctx)` | points de fin de manche | Poison (gardé : −taille ; capturé : −taille/2), Null (0), Value (valeur affichée au lieu de la taille) |
| `initiative(die)` | participation à l'initiative | Trip et Stinger exclus |
| `initiative_reaction(die)` | réaction à l'initiative | Focus (abaisser pour prendre l'initiative), Chance (relancer un dé) |
| `round_setup(die)` | entrée en jeu | Reserve (hors jeu jusqu'à choix après une manche perdue) |

Les textes de référence de chaque compétence sont recopiés et reformulés dans `docs/RULES.md` §5.

### 5.5 Déroulement (R 1–4, d'après HowToPlay)

1. **SpecifyDice** : chaque joueur choisit ses tailles Swing et Option (simultané, caché). Après la première manche, seul le perdant peut changer ; en cas d'égalité, personne.
2. **ChooseReserve** (si applicable) : le perdant de la manche précédente peut ajouter un dé Reserve.
3. **Jet** de tous les dés.
4. **Initiative** : le joueur qui a le plus petit dé (hors Trip et Stinger) commence ; en cas d'égalité, on compare le suivant, et ainsi de suite. Égalité totale : tirage au sort par le RNG (comme `BMGame::do_next_step_determine_initiative`). Puis **ReactToInitiative** (Focus / Chance) en alternance jusqu'à ce qu'un joueur laisse l'autre commencer. Un dé Focus abaissé par le premier joueur ne peut pas servir à sa première attaque.
5. **Tours** : on doit attaquer si on le peut ; sinon on passe. Après une attaque, les dés capturés vont dans la pile de score de l'attaquant et ses dés attaquants sont relancés (sauf Konstant).
6. **Fin de manche** : un joueur n'a plus de dés, ou les deux passent successivement. Score : dés gardés = taille/2, dés capturés = taille (modulé par Poison, Null, Value). Égalité possible, sans départage.
7. **Fin de match** : le premier à `max_wins` victoires de manche gagne. `Surrender` met fin au match.

## 6. Données de boutons (`crates/button-data`)

- `ButtonRecipe { name, recipe: String, set: Option<String>, source: ButtonSource }`, `ButtonSource = Original | Buttonweavers`.
- `data/buttons.toml` : environ 20 boutons originaux en v1, couvrant toutes les compétences v1 et Swing / Option / Twin.
- `scripts/import-buttonweavers.sh` : lit `data.button.sql` d'un clone local de buttonweavers et produit `data/buttonweavers/buttons.json` (nom, recette, set). À l'exécution, les boutons dont la recette contient une compétence hors v1 sont marqués non jouables avec la raison.
- Le client reçoit la liste via le WASM (originaux embarqués à la compilation, import chargé en `fetch` s'il est présent) ; le serveur lit les mêmes fichiers.

## 7. IA (`crates/ai`)

- Hors choix Swing / Option, l'information est publique ; seul le hasard des relances est inconnu. Algorithme : **expectimax à profondeur limitée**, nœuds de chance = relances des dés attaquants (et jets Trip), échantillonnées au-delà d'un seuil de combinaisons.
- Évaluation : différence de score de manche courante + menace (valeur capturable par chaque camp au coup suivant) + bonus de fin de manche.
- Choix Swing / Option : évaluation de quelques tailles candidates contre des tailles adverses tirées au hasard, par simulation de manches rapides (politique gloutonne).
- Réactions d'initiative, Fire et Reserve : passent par la même recherche, puisque ce sont des `Action` comme les autres.
- Niveaux : `Easy` (glouton, une seule couche), `Normal` (profondeur 1 + chance), `Hard` (profondeur 2, plus d'échantillons).
- Interface : `fn choose_action(state, player, level, seed) -> Action`. Exposée en WASM et appelée depuis un Web Worker.

## 8. Serveur (`crates/server`)

- axum + WebSocket, un processus, salles en mémoire.
- Flux : créer une salle → code à partager → l'invité rejoint → chacun choisit son bouton → le serveur crée la partie avec une seed générée côté serveur.
- Messages (JSON, types partagés avec le moteur via serde) :
  - client → serveur : `CreateRoom`, `JoinRoom { code }`, `ChooseButton`, `Action { action }`, `Leave` ;
  - serveur → client : `RoomState`, `GameView { view }` (via `view_for`), `Events`, `Error { RuleError | ProtocolError }`.
- Le serveur seul exécute `apply`. Reconnexion par jeton de siège remis à la connexion. Une salle sans joueur connecté expire après un délai.

## 9. Client (`client/`)

- React + Vite + Tailwind 4 + Framer Motion, TypeScript strict, types générés depuis les types Rust (tsify ou équivalent).
- Écrans : **Accueil** (IA, hotseat, en ligne), **Choix des boutons** (filtre originaux / importés, recette lisible, compétences en infobulle), **Table**, **Fin de match**.
- **Table** : deux rangées de dés (forme, taille, valeur, icônes de compétences). Le joueur sélectionne des attaquants puis des cibles ; seules les combinaisons présentes dans `legal_actions` sont surlignées et validables. Panneaux de score et de manche alimentés par le moteur.
- **`GameAdapter`** : interface unique (`state$`, `dispatch(action)`) avec trois implémentations : `LocalAdapter` (WASM, hotseat), `AiAdapter` (WASM + Worker IA), `OnlineAdapter` (WebSocket). Le composant Table ignore le mode.
- Langue : français en v1, chaînes regroupées pour une traduction ultérieure.

## 10. Tests

- **Moteur** : tests par accroche et par compétence sur des scénarios seedés (constructeur de scénario : boutons, valeurs forcées, joueur actif).
- **Propriétés** (proptest) : toute partie aléatoire se termine ; toute action de `legal_actions` est acceptée par `apply` ; aucune action hors `legal_actions` n'est acceptée ; le nombre total de dés est conservé (actifs + capturés + réserve).
- **Parseur de recettes** : tous les boutons originaux se chargent ; sur l'import buttonweavers (s'il est présent localement), aucune recette ne fait paniquer le parseur.
- **IA** : matchs IA contre IA en CI (pas de blocage, durée bornée) ; `Hard` bat `Easy` sur un nombre fixe de seeds.
- **Serveur** : test d'intégration de deux clients WebSocket jouant une partie.
- **Client** : Vitest sur les adaptateurs et la sélection d'attaque.

## 11. Risques et questions ouvertes

- **Combinatoire des Skill attacks** (Stinger, Konstant, Fire) : l'énumération de `legal_actions` peut exploser. On énumère les sous-ensembles d'attaquants (au plus ~10 dés par joueur) et on calcule l'ensemble des sommes atteignables ; Fire est traité comme une étape séparée (`AdjustFire`) plutôt que dans l'énumération.
- **Interactions entre compétences** : buttonweavers documente des interactions (`get_interaction_descriptions`). Celles qui touchent les 15 compétences v1 sont reprises dans `RULES.md` et testées.
