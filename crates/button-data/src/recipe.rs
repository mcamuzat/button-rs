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
        map(separated_pair(number, char('/'), number), |(a, b)| {
            RawCore::Option(a, b)
        }),
        map(separated_pair(part, char(','), part), |(a, b)| {
            RawCore::Twin(a, b)
        }),
        map(part, RawCore::Single),
    ))
    .parse(i)
}

fn skill_char(i: &str) -> IResult<&str, char> {
    satisfy(|c| !c.is_whitespace() && c != '(' && c != ')').parse(i)
}

fn die(i: &str) -> IResult<&str, RawDie> {
    map(
        (
            many0(skill_char),
            delimited(char('('), core, char(')')),
            many0(skill_char),
        ),
        |(prefix, core, suffix)| RawDie {
            prefix,
            core,
            suffix,
        },
    )
    .parse(i)
}

fn recipe(i: &str) -> IResult<&str, Vec<RawDie>> {
    all_consuming(delimited(
        multispace0,
        separated_list1(multispace1, die),
        multispace0,
    ))
    .parse(i)
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
    if (1..=99).contains(&n) {
        Ok(n)
    } else {
        Err(RecipeError::InvalidSize { size: n })
    }
}

fn swing(c: char) -> Result<SwingLetter, RecipeError> {
    SwingLetter::from_letter(c).ok_or(RecipeError::UnsupportedDie {
        core: c.to_string(),
    })
}

fn twin_part(p: RawPart) -> Result<TwinPart, RecipeError> {
    match p {
        RawPart::Number(n) => Ok(TwinPart::Fixed(size(n)?)),
        RawPart::Letter(c) => Ok(TwinPart::Swing(swing(c)?)),
    }
}

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
        assert_eq!(
            dice[1].shape,
            DieShape::Twin(TwinPart::Fixed(4), TwinPart::Fixed(4))
        );
        assert_eq!(
            dice[2].shape,
            DieShape::Twin(
                TwinPart::Swing(SwingLetter::X),
                TwinPart::Swing(SwingLetter::X)
            )
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
        assert_eq!(
            parse_recipe("o(6)"),
            Err(RecipeError::UnsupportedSkill { letter: 'o' })
        );
        assert_eq!(
            parse_recipe("+(6)"),
            Err(RecipeError::UnsupportedSkill { letter: '+' })
        );
        assert_eq!(
            parse_recipe("(C)"),
            Err(RecipeError::UnsupportedDie { core: "C".into() })
        );
        assert_eq!(
            parse_recipe("(0)"),
            Err(RecipeError::InvalidSize { size: 0 })
        );
        assert_eq!(
            parse_recipe("(200,200)"),
            Err(RecipeError::InvalidSize { size: 200 })
        );
        assert_eq!(parse_recipe("   "), Err(RecipeError::Empty));
        assert!(matches!(
            parse_recipe("(6"),
            Err(RecipeError::Syntax { .. })
        ));
        assert!(matches!(
            parse_recipe("(1/2/3)"),
            Err(RecipeError::Syntax { .. })
        ));
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
