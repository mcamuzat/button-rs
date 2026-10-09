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
