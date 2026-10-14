# Jalon 4 — les compétences des boutons officiels

Date : 2026-10-14
Statut : validée en discussion, en attente de relecture de la spec écrite.

## 1. Objectif

Rendre jouables les boutons officiels de Cheapass Games encore bloqués par une compétence de dé que buttonweavers implémente. Le jalon ajoute 13 compétences, regroupées en trois familles selon ce qu'elles touchent dans le moteur :

| Famille | Compétences | Ce qu'elles touchent |
|---|---|---|
| **A. Accroches simples** | Rush, Slow, Ornery, Mighty, Rage, TimeAndSpace, Boom | Accroches existantes (initiative, types d'attaque, relances, captures) ; Rush et Boom ajoutent un type d'attaque. |
| **B. Taille variable** | Mood, Mad, Turbo, Morphing | La taille d'un dé change après le choix initial ; Turbo demande un choix du joueur à chaque attaque. |
| **C. Structure** | Auxiliary, Warrior | Une décision avant le match (Auxiliary) ; des dés hors jeu qui entrent par une attaque Skill (Warrior). |

### Critères de réussite

- Les boutons officiels jouables passent d'environ 258 à environ 316 sur 413 (mesure sur l'import buttonweavers local).
- Les boutons officiels nouvellement jouables, mesurés au banc d'équilibrage, suivent les parties humaines : corrélation banc / humains ≥ 0,7, et toute compétence dont les boutons s'écartent en moyenne de plus de 15 points des humains est expliquée (erreur de règle, faiblesse de l'IA, ou amplification connue des extrêmes) avant de clore sa famille.
- Chaque nouvelle compétence est portée par au moins un bouton original, réglé dans la plage normale du banc.
- Les propriétés du moteur restent vraies avec les nouvelles compétences (§9).

### Hors périmètre

- Les compétences de bouton (`btn_special`, 66 boutons officiels) : exclues depuis la v1, candidates pour un jalon suivant.
- Les notations sans règle de référence dans buttonweavers : `S`, `=`, `A`, `:`, `$`, `R` en préfixe, dé Wildcard `(C)`.
- Les autres compétences buttonweavers (Doppelganger, Insult, Jolt, Maximum, Radioactive, Weak…), surtout présentes dans les éditions de fans.
- Le jeu en ligne (jalon 3, repoussé).

## 2. Sources

- Règles : `src/engine/BMSkill*.php`, `BMAttackRush.php`, `BMAttackBoom.php`, `BMAttack::commit_attack` (ordre des effets d'une attaque) dans `github.com/buttonmen-dev/buttonmen` (BSD-3). Chaque compétence reçoit une section `R 6.x` dans `docs/RULES.md`, avec sa source ; les interactions retenues vont en `R 7.x`.
- Taux de victoire humains : `buttonstats.json` publié avec la page Button Filter de Button Men Online (`buttonmen.s3-website-us-east-1.amazonaws.com`), importé localement et gitignoré comme `buttons.json`.

## 3. Règles retenues

Chaque règle est implémentée telle que buttonweavers l'applique, y compris ses cas particuliers. La rédaction définitive va dans `RULES.md`.

### Famille A

- **Slow (`w`).** Ne compte pas pour l'initiative.
- **Rage (`G`).** Ne compte pas pour l'initiative. Quand un dé Rage est capturé, son propriétaire reçoit un clone (même taille, mêmes compétences sauf Rage), qui est lancé. Un dé Rage qui participe à une attaque perd Rage.
- **Rush (`#`).** Nouvelle attaque `Rush` : un seul attaquant capture exactement deux dés dont la somme des valeurs égale la sienne. L'attaque est permise si l'attaquant ou au moins une des cibles a Rush : n'importe quel dé peut faire une attaque Rush contre deux cibles dont une est Rush.
- **Boom (`b`).** Nouvelle attaque `Boom` : un dé Boom vise un dé adverse. Le dé Boom quitte le jeu et ne compte pour aucun des deux joueurs ; la cible est relancée, jamais capturée. Une attaque Boom possible interdit de passer (comme dans buttonweavers, où elle n'est pas « optionnelle »). Les dés Stealth peuvent être visés.
- **Ornery (`o`).** Relancé après chaque attaque de son propriétaire, qu'il y ait participé ou non ; pas relancé quand le joueur passe.
- **Mighty (`H`).** À chaque relance, il grandit d'un cran dans la liste 1, 2, 4, 6, 8, 10, 12, 16, 20, 30 (un Twin fait grandir chaque moitié). Pas au jet de début de manche, pas pour un clone Rage, pas pour un dé qui ne se relance pas (Konstant). Un attaquant Trip Mighty grandit une fois, avant son jet (`BMSkillTrip::pre_capture` relance avant de poser `JustPerformedTripAttack`).
- **TimeAndSpace (`^`).** Si un attaquant TimeAndSpace se relance sur une valeur impaire, son propriétaire rejoue. Un seul tour supplémentaire par attaque. Un attaquant Konstant ne se relance pas et ne déclenche donc rien.

### Famille B

- **Mood (`?` en suffixe).** Uniquement sur un dé Swing ou un Twin à moitiés Swing. À chaque relance (hors jet de début de manche, hors dé qui ne se relance pas ; un attaquant Trip change de taille une fois, avant son jet), sa taille est tirée au hasard parmi les tailles « réelles » 1, 2, 4, 6, 8, 10, 12, 20, 30 comprises dans la plage de sa lettre.
- **Mad (`&` en suffixe).** Comme Mood, mais parmi toutes les tailles paires de la plage. La taille initiale choisie peut être impaire.
- **Turbo (`!` en suffixe).** Uniquement sur un dé Swing ou Option. Quand un dé Turbo se relance après avoir attaqué, son propriétaire choisit sa nouvelle taille dans la plage Swing (ou entre les deux faces de l'Option) ; le choix est déclaré avec l'attaque. Avec Berserk, le dé est d'abord divisé, puis prend la taille Turbo.
- **Morphing (`m`).** Après une attaque contre une seule cible, l'attaquant prend la forme et la taille de la cible (Twin compris) en gardant ses propres compétences, puis se relance. Un dé Morphing compte à sa taille du moment, qu'il soit gardé ou capturé.

Les dés Swing de même lettre partagent leur taille au choix initial ; ensuite Mood, Mad, Turbo et Morphing font varier chaque dé séparément.

### Famille C

- **Auxiliary (`+`).** Avant la première manche, chaque joueur choisit en secret s'il joue avec ses dés Auxiliary. Ils ne sont en jeu que si les deux joueurs acceptent ; sinon ils sont retirés pour tout le match. L'étape est sautée si aucun des deux boutons n'a de dé Auxiliary. Un dé Auxiliary retiré ne demande pas de choix Swing.
- **Warrior (`` ` ``).** Un dé Warrior est hors jeu : il montre sa valeur maximale, ne peut être ni attaqué ni utilisé autrement, et ne compte ni au score ni pour l'initiative. Au plus un dé Warrior peut s'ajouter à une attaque Skill à plusieurs dés ; il entre alors en jeu, est lancé normalement et perd Warrior jusqu'à la fin de la manche, puis le retrouve. Ce n'est jamais obligatoire : une attaque possible seulement avec un Warrior n'interdit pas de passer. Un Konstant Warrior ne peut ajouter qu'une valeur positive ; un Stinger Warrior ajoute sa valeur entière.

### Interactions

Celles que buttonweavers documente entre ces compétences et les 15 de la v1 sont reprises en `R 7.x` et testées, notamment : Konstant et Ornery (garde sa valeur), Berserk et Mighty (divise puis grandit), Berserk et Turbo (divise puis prend la taille Turbo), Berserk et Speed (au choix), Ornery et Mood / Mad (taille tirée lors de la relance Ornery), Konstant et TimeAndSpace (pas de tour supplémentaire).

## 4. Moteur (`crates/engine`)

### Types

```rust
pub enum DieSkill {
    // 15 compétences v1…
    Rush, Slow, Ornery, Mighty, Rage, TimeAndSpace, Boom,   // famille A
    Mood, Mad, Turbo, Morphing,                             // famille B
    Auxiliary, Warrior,                                     // famille C
}

pub enum AttackKind { Power, Skill, Shadow, Speed, Trip, Berserk, Rush, Boom }

pub struct Die {
    // champs existants…
    /// Règle de changement de taille, remplie au début de chaque manche depuis l'emplacement.
    pub resize: Option<Resize>,
}

pub enum Resize {
    Mood { range: RangeInclusive<u8> },
    Mad { range: RangeInclusive<u8> },
    Turbo { choices: TurboChoices },        // plage Swing ou deux faces d'Option
}

pub enum Action {
    // actions existantes, avec pour Attack :
    Attack { attack: DeclaredAttack, resize: Vec<(DieId, Sides)> },   // tailles Turbo
    ChooseAuxiliary { play: bool },
}

pub enum WaitingFor {
    // états existants…
    ChooseAuxiliary { pending: PlayerSet },   // simultané, caché, avant le premier SpecifyDice
}
```

Les noms exacts des champs sont fixés dans les plans ; la forme ci-dessus est l'engagement. Ajouter une variante à `DieSkill` force une décision à chaque accroche (`match` exhaustifs sans joker).

### Accroches

| Accroche | Ajouts |
|---|---|
| `counts_for_initiative` | Slow, Rage, Warrior : exclus |
| `attack_kinds` | Rush (attaque Rush), Boom (attaque Boom) ; tout dé peut faire Rush si une cible est Rush |
| `can_be_captured` | Warrior : jamais ; cibles d'une attaque Boom : jamais capturées |
| `rerolls` / `reroll` | Ornery (après toute attaque du propriétaire), Mighty (grandit avant), Mood / Mad (taille tirée avant), Turbo (taille choisie avant), Morphing (forme copiée avant) |
| `after_attack` | Rage (perd Rage ; clone si capturé), Boom (sortie du jeu, relance de la cible), TimeAndSpace (tour supplémentaire), Morphing |
| `round_setup` | Auxiliary (retirés si refusés), Warrior (hors jeu, à leur maximum, Warrior rétabli), `resize` rempli |
| `score` | Boom sorti du jeu : 0 pour les deux joueurs ; Warrior hors jeu : 0 |

L'ordre des effets d'une attaque suit `BMAttack::commit_attack` : effets avant capture (Trip, Boom), Rage des cibles, effets de capture des attaquants (Berserk, Morphing, Rage), relance des attaquants (Mighty, Mood, Mad, Turbo, TimeAndSpace), puis Ornery en fin de tour.

### Score des captures

`capture_swing` (déjà exposé pour l'IA) intègre Rage : capturer un dé Rage rend un clone à son propriétaire, donc la capture lui retire moins de points. Le moteur reste la seule source des règles de score.

### Vue et événements

- `GameView` expose les dés Warrior hors jeu de chaque joueur, et pour chaque dé Turbo les tailles permises. Pendant `ChooseAuxiliary`, chaque joueur ne voit que son propre choix.
- Nouveaux événements : `DieResized { die, from, to, cause }` (Mood, Mad, Turbo, Mighty, Morphing), `RageReplaced`, `BoomRerolled`, `ExtraTurn`.
- `skill_info` fournit nom et texte d'aide des 13 compétences.

## 5. Données et parseur (`crates/button-data`)

- Préfixes ajoutés à la table des lettres : `#` Rush, `w` Slow, `o` Ornery, `H` Mighty, `G` Rage, `^` TimeAndSpace, `b` Boom, `m` Morphing, `+` Auxiliary, `` ` `` Warrior.
- Suffixes, nouveaux dans le parseur (combinateurs nom) : `?` Mood, `&` Mad, `!` Turbo. Mood et Mad n'acceptent qu'un dé Swing ou un Twin à moitiés Swing ; Turbo un dé Swing ou Option. Sinon `RecipeError::SuffixNotAllowed { suffix, core }`, et le bouton est affiché non jouable avec la raison.
- Environ 10 nouveaux boutons originaux (un par compétence, avec des regroupements naturels comme Mood et Mad, ou Turbo sur un Option), réglés au banc dans la plage normale avant d'entrer dans `data/buttons.toml`. Le test « les originaux couvrent chaque compétence » passe à 28 compétences ; chaque famille livre ses originaux.
- Nouveau test (seulement si l'import local est présent) : nombre de boutons officiels jouables par famille.

## 6. IA (`crates/ai`)

- Aucune compétence n'est codée en dur : les nouveaux coups arrivent par `candidate_actions` et la recherche les évalue avec `apply`. Tour supplémentaire, clone Rage, croissance Mighty sont vus sans code spécifique.
- Turbo : `candidate_actions` ne propose, par dé Turbo attaquant, que les tailles minimale, maximale et actuelle (les deux faces d'un Option) ; `legal_actions` décrit toute la plage au client.
- Estimation rapide d'une capture : `capture_swing` du moteur, Rage compris. Gain estimé d'une attaque Boom : les points du dé Boom qui sort du jeu (négatif).
- Politique gloutonne des simulations : taille Turbo au hasard, oui à Auxiliary, pas de Warrior sauf si c'est la seule attaque.
- Avant la première manche, le choix Auxiliary est décidé par les mêmes simulations que les choix Swing et Reserve.
- Les tests de rapidité de Difficile couvrent une position chargée en Turbo et Warrior, avec le budget actuel (400 ms en natif).

## 7. Client (`client/`)

- Rush et Boom rejoignent le sélecteur d'attaque ; seules les combinaisons de `legal_actions` sont proposées.
- Turbo : quand un dé Turbo fait partie des attaquants, un sélecteur de taille apparaît près du bouton d'attaque, prérempli avec la taille actuelle, limité aux tailles fournies par le moteur.
- Warrior : une rangée de dés hors jeu sous ceux du joueur ; un dé Warrior se sélectionne comme attaquant d'une attaque Skill (un seul).
- Auxiliary : un panneau avant la première manche qui liste les dés concernés et rappelle qu'ils ne jouent que si les deux joueurs acceptent ; en hotseat, le choix est caché à l'autre joueur comme le choix Swing.
- Changement de taille : la taille affichée change avec une courte animation et une ligne de journal (« d12 → d20, Mighty ») ; tour supplémentaire : bandeau « Tour supplémentaire (TimeAndSpace) ».
- Les adaptateurs ne changent pas.

## 8. Banc d'équilibrage et stats humaines

- `scripts/import-buttonweavers.sh` récupère aussi `buttonstats.json` dans `data/buttonweavers/stats.json` (gitignoré).
- L'exemple `balance` affiche, pour chaque bouton qui a des stats humaines, le taux humain et l'écart banc − humains, et la corrélation banc / humains sur les boutons mesurés.
- Une option sélectionne comme candidats les boutons officiels nouvellement jouables d'une famille, pour le contrôle de clôture (§1).

## 9. Tests

- Moteur : un scénario à graine fixe par compétence (constructeur de scénario existant), un test par interaction `R 7.x`.
- Propriétés (proptest), étendues aux nouvelles compétences : toute partie se termine ; toute action de `legal_actions` est acceptée et aucune autre ; conservation des dés réécrite pour les dés qui apparaissent (clone Rage), sortent du jeu (Boom) ou attendent hors jeu (Warrior, Auxiliary refusé).
- Parseur : préfixes, suffixes, erreurs typées, aucune panique sur l'import.
- IA : « Difficile bat Facile » reste vrai ; tests de rapidité étendus.
- Client (Vitest) : sélection d'attaque Rush et Boom, sélecteur Turbo, attaque avec Warrior, panneau Auxiliary en hotseat.

## 10. Livraison

Trois plans, livrés et fusionnés l'un après l'autre ; chacun se clôt par le contrôle du §1.

| Plan | Contenu | Officiels jouables (≈) |
|---|---|---|
| **4A** | Banc avec stats humaines (§8), puis famille A : parseur, moteur, IA, client, originaux. | 258 → 283 |
| **4B** | Suffixes, `Resize`, famille B, sélecteur Turbo, animations de taille. | 283 → 308 |
| **4C** | `ChooseAuxiliary`, Warrior, panneau Auxiliary, rangée Warrior. | 308 → 316 |

## 11. Risques et questions ouvertes

- **Combinatoire Turbo.** Plusieurs dés Turbo dans une même attaque multiplient les tailles ; la restriction à trois tailles par dé dans `candidate_actions` borne l'IA, le client reçoit les plages.
- **Warrior dans les attaques Skill.** L'énumération des sommes atteignables gagne un dé optionnel par attaque ; la mesure de temps de Difficile le surveille.
- **Comptes approximatifs.** Les nombres de boutons jouables par famille viennent d'une estimation sur la notation ; les plans les remplacent par la mesure du catalogue réel.
