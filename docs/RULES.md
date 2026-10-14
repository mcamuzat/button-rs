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
- **R 3.3 Initiative.** On compare les dés de chaque joueur, triés par valeur croissante, en ignorant ceux que leurs compétences excluent (R 6.14 Trip, R 6.13 Stinger, R 6.16 Slow). Le joueur dont le plus petit dé est le plus bas commence ; à égalité on compare le suivant, et ainsi de suite. Si les dés d'un joueur sont épuisés alors que l'autre en a encore, celui qui en a encore commence.
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
- **R 6.2 Chance (c).** Si le joueur n'a pas l'initiative en début de manche, il peut relancer un de ses dés Chance. S'il prend ainsi l'initiative, l'adversaire peut relancer un de ses dés Chance, et ainsi de suite, jusqu'à ce qu'un joueur échoue à prendre l'initiative (une égalité est un échec) ou laisse l'autre commencer. Un dé Chance relancé peut attaquer au premier tour.
- **R 6.3 Fire (F).** Ne fait pas d'attaque Power. Peut aider une attaque Power ou Skill : avant l'attaque, on augmente la valeur de dés attaquants et on diminue d'autant la valeur de dés Fire qui ne participent pas à l'attaque. Aucun dé ne sort de sa plage de valeurs ; Fire ne fait qu'augmenter les attaquants (un dé Konstant augmenté compte pour ± sa nouvelle valeur). Fire n'aide aucun autre type d'attaque.
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
- **R 6.14 Trip (t).** Peut faire des attaques Trip : on choisit un dé adverse, on relance les deux dés ; si le dé Trip montre alors une valeur supérieure ou égale, la cible est capturée, sinon les deux dés gardent leurs nouvelles valeurs. Une attaque Trip sans aucune chance de capturer est illégale : la meilleure valeur possible du Trip (sa taille, ou sa valeur s'il est Konstant) doit atteindre la plus basse valeur possible de la cible (sa valeur minimale, ou sa valeur si elle est Konstant). Ne compte pas pour l'initiative.
- **R 6.15 Value (v).** Un dé Value compte comme si sa taille était égale à sa valeur actuelle. Quand un dé Value participe à une attaque, les dés capturés comptent pour la valeur qu'ils montraient au moment de la capture.
- **R 6.16 Slow (w).** Ne compte pas pour l'initiative. (Source : `BMSkillSlow.php`.)
- **R 6.17 Mighty (H).** Chaque fois qu'il se relance, pour quelque raison que ce soit, le dé grandit d'abord jusqu'à la taille suivante de la liste 1, 2, 4, 6, 8, 10, 12, 16, 20, 30 (une taille hors liste passe à la suivante ; 30 et plus ne bougent pas ; un Twin fait grandir chaque moitié). Il ne grandit pas au jet de début de manche, ni quand il se relance après avoir fait une attaque Trip, ni s'il ne se relance pas (Konstant). Il compte à sa taille du moment. (Source : `BMSkillMighty.php`, `BMDie::grow`.)

## R 7 — Interactions retenues

- **R 7.1** Null l'emporte sur Value et Poison : un dé capturé par une attaque avec un dé Null, ou un dé Null lui-même, vaut zéro.
- **R 7.2** Poison et Value se combinent : la base du calcul Poison est la valeur affichée.
- **R 7.3** Konstant et Stinger se combinent : le dé compte pour toute valeur de ±1 à ±sa valeur.
- **R 7.4** Un dé Focus baissé par le premier joueur reste inutilisable jusqu'à la fin de son premier tour.
- **R 7.5** Chance et Mighty : une relance Chance fait grandir le dé. Berserk et Mighty : le dé est divisé, puis grandit en se relançant.
