//! Balance bench: how strong is a button, played by the AI, against official buttons?
//!
//! The reference buttons (official editions) play a round robin among themselves; the
//! spread of their win rates is the normal range for an official button. Each candidate
//! then plays every reference button, and is flagged when its whole 95 % confidence
//! interval falls outside that range.
//!
//! ```text
//! cargo run -p buttonmen-ai --release --example balance -- [options]
//!   --candidates originaux | edition:<nom> | bouton:<nom> | recette:<recette> | competence:<Nom>
//!                répétable, les sélections s'ajoutent           (défaut : originaux)
//!   --reference <édition>[,<édition>…]                       (défaut : Soldiers,The Core)
//!   --games <N>        parties par paire (défaut : 20)
//!   --level easy|normal|hard                                 (défaut : normal)
//!   --seed <N>   --threads <N>   --csv <fichier>
//! ```
//!
//! The reference editions and the human win rates (Button Men Online) come from the local
//! buttonweavers import (`./scripts/import-buttonweavers.sh`). Both seats are played by
//! the AI at the same level: a skill the AI plays badly makes its button look weaker than
//! it is.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

use buttonmen_ai::{Level, choose_action};
use buttonmen_data::SetCategory;
use buttonmen_data::{
    ButtonRecipe, ButtonSource, Playability, catalog, original_buttons, parse_buttonweavers_json,
    parse_recipe, to_button,
};
use buttonmen_engine::{Button, GameState, MatchConfig, PlayerId, WaitingFor, apply, new_game};
use buttonmen_engine::{DieSkill, skill_info};
use serde::Deserialize;

const IMPORT: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../data/buttonweavers/buttons.json"
);
/// A match longer than this is abandoned and reported (it should never happen).
const MAX_STEPS: u64 = 3000;
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

#[derive(Debug, Clone, PartialEq, Eq)]
enum Selection {
    Originals,
    Edition(String),
    Button(String),
    /// An ad hoc recipe, named after itself: to try a variant before editing the catalog.
    Recipe(String),
    /// Official playable buttons with a die of this skill (name as in `skill_info`).
    Skill(String),
}

#[derive(Debug, Clone)]
struct Options {
    candidates: Vec<Selection>,
    reference: Vec<String>,
    games: u32,
    level: Level,
    seed: u64,
    threads: usize,
    csv: Option<String>,
}

fn parse_options(args: impl IntoIterator<Item = String>) -> Result<Options, String> {
    let mut options = Options {
        candidates: Vec::new(),
        reference: vec!["Soldiers".to_string(), "The Core".to_string()],
        games: 20,
        level: Level::Normal,
        seed: 1,
        threads: std::thread::available_parallelism().map_or(1, |n| n.get()),
        csv: None,
    };
    let mut args = args.into_iter();
    while let Some(flag) = args.next() {
        let mut value = || args.next().ok_or(format!("{flag} attend une valeur"));
        match flag.as_str() {
            "--candidates" => options.candidates.push(parse_selection(&value()?)?),
            "--reference" => {
                options.reference = value()?.split(',').map(|s| s.trim().to_string()).collect()
            }
            "--games" => options.games = number(&flag, &value()?)?,
            "--level" => options.level = parse_level(&value()?)?,
            "--seed" => options.seed = number(&flag, &value()?)?,
            "--threads" => options.threads = number::<usize>(&flag, &value()?)?.max(1),
            "--csv" => options.csv = Some(value()?),
            _ => return Err(format!("option inconnue : {flag}")),
        }
    }
    if options.candidates.is_empty() {
        options.candidates.push(Selection::Originals);
    }
    if options.games == 0 {
        return Err("--games doit être au moins 1".to_string());
    }
    Ok(options)
}

fn parse_selection(text: &str) -> Result<Selection, String> {
    if text == "originaux" {
        Ok(Selection::Originals)
    } else if let Some(name) = text.strip_prefix("edition:") {
        Ok(Selection::Edition(name.to_string()))
    } else if let Some(name) = text.strip_prefix("bouton:") {
        Ok(Selection::Button(name.to_string()))
    } else if let Some(recipe) = text.strip_prefix("recette:") {
        Ok(Selection::Recipe(recipe.to_string()))
    } else if let Some(name) = text.strip_prefix("competence:") {
        Ok(Selection::Skill(name.to_string()))
    } else {
        Err(format!(
            "candidats inconnus : {text} (originaux, edition:<nom>, bouton:<nom>, recette:<recette>, competence:<Nom>)"
        ))
    }
}

fn parse_level(text: &str) -> Result<Level, String> {
    match text {
        "easy" => Ok(Level::Easy),
        "normal" => Ok(Level::Normal),
        "hard" => Ok(Level::Hard),
        _ => Err(format!("niveau inconnu : {text} (easy, normal, hard)")),
    }
}

fn number<T: std::str::FromStr>(flag: &str, text: &str) -> Result<T, String> {
    text.parse()
        .map_err(|_| format!("{flag} attend un nombre, pas « {text} »"))
}

/// Originals plus the local import, when present.
fn load_recipes() -> Vec<ButtonRecipe> {
    let mut recipes = original_buttons();
    if let Ok(json) = std::fs::read_to_string(IMPORT) {
        recipes.extend(parse_buttonweavers_json(&json).expect("buttons.json lisible"));
    }
    recipes
}

/// The playable buttons matching `wanted`, in catalog order.
fn select(recipes: &[ButtonRecipe], wanted: impl Fn(&ButtonRecipe) -> bool) -> Vec<Button> {
    let wanted: Vec<ButtonRecipe> = recipes.iter().filter(|r| wanted(r)).cloned().collect();
    catalog(&wanted)
        .into_iter()
        .filter(|e| e.playability == Playability::Playable)
        .map(|e| {
            let recipe = wanted
                .iter()
                .find(|r| r.name == e.name && r.source == e.source)
                .expect("entry comes from these recipes");
            to_button(recipe).expect("playable recipe")
        })
        .collect()
}

fn select_candidates(
    recipes: &[ButtonRecipe],
    selections: &[Selection],
) -> Result<Vec<Button>, String> {
    let mut buttons = Vec::new();
    for selection in selections {
        let found = match selection {
            Selection::Originals => select(recipes, |r| r.source == ButtonSource::Original),
            Selection::Edition(name) => select(recipes, |r| r.set.as_deref() == Some(name)),
            Selection::Button(name) => select(recipes, |r| &r.name == name),
            Selection::Recipe(recipe) => vec![Button {
                name: recipe.clone(),
                dice: parse_recipe(recipe).map_err(|e| format!("{recipe} : {e}"))?,
            }],
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
        };
        if found.is_empty() {
            return Err(format!("Aucun candidat jouable pour {selection:?}"));
        }
        buttons.extend(found);
    }
    Ok(buttons)
}

/// splitmix64 step: independent seeds for each (stream, opponent, game).
fn mix(mut z: u64) -> u64 {
    z = z.wrapping_add(0x9e37_79b9_7f4a_7c15);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

fn match_seed(base: u64, opponent: usize, game: u32) -> u64 {
    mix(mix(base ^ opponent as u64) ^ u64::from(game))
}

/// Plays a full match between two AIs of the same level; `None` if it never ends.
fn play(one: &Button, two: &Button, level: Level, seed: u64) -> Option<PlayerId> {
    let mut state: GameState = new_game([one.clone(), two.clone()], seed, MatchConfig::default());
    for step in 0..MAX_STEPS {
        if let WaitingFor::GameOver { result } = &state.waiting_for {
            return Some(result.winner);
        }
        let actor = state.actors()[0];
        let action = choose_action(&state, actor, level, mix(seed ^ step))?;
        state = apply(&state, actor, action).ok()?.0;
    }
    None
}

/// One match to play: `player` against `opponent`, `player` in seat one on even games.
#[derive(Debug, Clone, Copy)]
struct Job {
    player: usize,
    opponent: usize,
    game: u32,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct Tally {
    wins: u32,
    games: u32,
    unfinished: u32,
}

/// Plays every job on `threads` threads; tallies by (player, opponent). With `mirror`
/// (players and opponents are the same pool), each match also counts for the opponent.
fn run(
    players: &[Button],
    opponents: &[Button],
    jobs: &[Job],
    options: &Options,
    stream: u64,
    mirror: bool,
    label: &str,
) -> BTreeMap<(usize, usize), Tally> {
    let next = AtomicUsize::new(0);
    let tallies = Mutex::new(BTreeMap::<(usize, usize), Tally>::new());
    let start = Instant::now();
    std::thread::scope(|scope| {
        for _ in 0..options.threads {
            scope.spawn(|| {
                loop {
                    let i = next.fetch_add(1, Ordering::Relaxed);
                    let Some(job) = jobs.get(i) else { break };
                    let (me, them) = (&players[job.player], &opponents[job.opponent]);
                    // Same seed for every player facing this opponent in this game: paired.
                    let seed = match_seed(options.seed ^ stream, job.opponent, job.game);
                    let player_seat = if job.game % 2 == 0 {
                        PlayerId::One
                    } else {
                        PlayerId::Two
                    };
                    let winner = if player_seat == PlayerId::One {
                        play(me, them, options.level, seed)
                    } else {
                        play(them, me, options.level, seed)
                    };
                    let mut tallies = tallies.lock().expect("no panic while holding the lock");
                    let player_won = winner.map(|w| w == player_seat);
                    record(&mut tallies, (job.player, job.opponent), player_won);
                    if mirror {
                        record(
                            &mut tallies,
                            (job.opponent, job.player),
                            player_won.map(|won| !won),
                        );
                    }
                    let done = i + 1;
                    if done.is_multiple_of((jobs.len() / 20).max(1)) || done == jobs.len() {
                        eprint!(
                            "\r{label} : {done} / {} parties ({:.0} s)   ",
                            jobs.len(),
                            start.elapsed().as_secs_f64()
                        );
                    }
                }
            });
        }
    });
    eprintln!();
    tallies.into_inner().expect("threads joined")
}

fn record(tallies: &mut BTreeMap<(usize, usize), Tally>, key: (usize, usize), won: Option<bool>) {
    let tally = tallies.entry(key).or_default();
    match won {
        Some(won) => {
            tally.games += 1;
            tally.wins += u32::from(won);
        }
        None => tally.unfinished += 1,
    }
}

/// Wilson score interval at 95 %.
fn wilson(wins: u32, games: u32) -> (f64, f64) {
    if games == 0 {
        return (0.0, 1.0);
    }
    let n = f64::from(games);
    let p = f64::from(wins) / n;
    let z = 1.96_f64;
    let denominator = 1.0 + z * z / n;
    let centre = p + z * z / (2.0 * n);
    let spread = z * (p * (1.0 - p) / n + z * z / (4.0 * n * n)).sqrt();
    (
        ((centre - spread) / denominator).max(0.0),
        ((centre + spread) / denominator).min(1.0),
    )
}

/// The normal range of official buttons: 10th to 90th percentile of their win rates.
fn normal_range(rates: &[f64]) -> (f64, f64) {
    let mut sorted = rates.to_vec();
    sorted.sort_by(f64::total_cmp);
    let at = |q: f64| sorted[((sorted.len() - 1) as f64 * q).round() as usize];
    (at(0.1), at(0.9))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Verdict {
    TooStrong,
    TooWeak,
    Normal,
}

fn verdict(interval: (f64, f64), range: (f64, f64)) -> Verdict {
    if interval.0 > range.1 {
        Verdict::TooStrong
    } else if interval.1 < range.0 {
        Verdict::TooWeak
    } else {
        Verdict::Normal
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Summary {
    name: String,
    wins: u32,
    games: u32,
    unfinished: u32,
    /// (opponent, win rate), worst then best.
    worst: Option<(String, f64)>,
    best: Option<(String, f64)>,
}

impl Summary {
    fn rate(&self) -> f64 {
        if self.games == 0 {
            0.0
        } else {
            f64::from(self.wins) / f64::from(self.games)
        }
    }
}

fn summarize(
    players: &[Button],
    opponents: &[Button],
    tallies: &BTreeMap<(usize, usize), Tally>,
) -> Vec<Summary> {
    players
        .iter()
        .enumerate()
        .map(|(p, button)| {
            let mut summary = Summary {
                name: button.name.clone(),
                wins: 0,
                games: 0,
                unfinished: 0,
                worst: None,
                best: None,
            };
            let mut by_opponent = Vec::new();
            for ((player, opponent), tally) in tallies.range((p, 0)..(p + 1, 0)) {
                debug_assert_eq!(*player, p);
                summary.wins += tally.wins;
                summary.games += tally.games;
                summary.unfinished += tally.unfinished;
                if tally.games > 0 {
                    let rate = f64::from(tally.wins) / f64::from(tally.games);
                    by_opponent.push((opponents[*opponent].name.clone(), rate));
                }
            }
            by_opponent.sort_by(|a, b| a.1.total_cmp(&b.1));
            summary.worst = by_opponent.first().cloned();
            summary.best = by_opponent.last().cloned();
            summary
        })
        .collect()
}

fn percent(x: f64) -> String {
    format!("{:.0} %", x * 100.0)
}

fn report(summaries: &[Summary], range: (f64, f64), stats: &BTreeMap<String, Human>) -> String {
    let mut sorted: Vec<&Summary> = summaries.iter().collect();
    sorted.sort_by(|a, b| b.rate().total_cmp(&a.rate()));
    let width = sorted
        .iter()
        .map(|s| s.name.chars().count())
        .max()
        .unwrap_or(6)
        .max(6);
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
            o.as_ref().map_or("—".to_string(), |(name, rate)| {
                format!("{name} {}", percent(*rate))
            })
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
        None => {
            "Corrélation banc / humains : pas assez de boutons avec des stats humaines".to_string()
        }
    }
}

fn write_csv(
    path: &str,
    players: &[Button],
    opponents: &[Button],
    tallies: &BTreeMap<(usize, usize), Tally>,
) -> std::io::Result<()> {
    let mut out = String::from("bouton,adversaire,victoires,parties,sans_fin\n");
    let quote = |s: &str| format!("\"{}\"", s.replace('"', "\"\""));
    for ((p, o), t) in tallies {
        let _ = writeln!(
            out,
            "{},{},{},{},{}",
            quote(&players[*p].name),
            quote(&opponents[*o].name),
            t.wins,
            t.games,
            t.unfinished
        );
    }
    std::fs::write(path, out)
}

fn main() {
    let options = match parse_options(std::env::args().skip(1)) {
        Ok(options) => options,
        Err(message) => {
            eprintln!("{message}");
            std::process::exit(2);
        }
    };
    let recipes = load_recipes();
    let reference = select(&recipes, |r| {
        r.set
            .as_ref()
            .is_some_and(|set| options.reference.contains(set))
    });
    if reference.len() < 2 {
        eprintln!(
            "Pas assez de boutons de référence jouables dans {:?} : lancez ./scripts/import-buttonweavers.sh",
            options.reference
        );
        std::process::exit(1);
    }
    let candidates = match select_candidates(&recipes, &options.candidates) {
        Ok(candidates) => candidates,
        Err(message) => {
            eprintln!("{message}");
            std::process::exit(1);
        }
    };
    eprintln!(
        "{} candidats contre {} boutons de référence ({}), {} parties par paire, niveau {:?}, {} threads",
        candidates.len(),
        reference.len(),
        options.reference.join(", "),
        options.games,
        options.level,
        options.threads
    );

    // Reference round robin: each unordered pair once, seats alternating, counted for both.
    let round_robin: Vec<Job> = (0..reference.len())
        .flat_map(|player| (player + 1..reference.len()).map(move |opponent| (player, opponent)))
        .flat_map(|(player, opponent)| {
            (0..options.games).map(move |game| Job {
                player,
                opponent,
                game,
            })
        })
        .collect();
    let reference_tallies = run(
        &reference,
        &reference,
        &round_robin,
        &options,
        0x5245_4645,
        true,
        "référence",
    );
    let reference_summaries = summarize(&reference, &reference, &reference_tallies);
    let rates: Vec<f64> = reference_summaries.iter().map(Summary::rate).collect();
    let range = normal_range(&rates);

    let jobs: Vec<Job> = (0..candidates.len())
        .flat_map(|player| (0..reference.len()).map(move |opponent| (player, opponent)))
        .flat_map(|(player, opponent)| {
            (0..options.games).map(move |game| Job {
                player,
                opponent,
                game,
            })
        })
        .collect();
    let tallies = run(
        &candidates,
        &reference,
        &jobs,
        &options,
        0,
        false,
        "candidats",
    );
    let summaries = summarize(&candidates, &reference, &tallies);

    println!(
        "\nPlage normale des boutons officiels (10e–90e centile) : {} – {}\n",
        percent(range.0),
        percent(range.1)
    );
    let stats = load_stats();
    println!("Candidats :\n{}", report(&summaries, range, &stats));
    println!(
        "Référence :\n{}",
        report(&reference_summaries, range, &stats)
    );
    let measured: Vec<&Summary> = summaries.iter().chain(&reference_summaries).collect();
    println!("{}", correlation_line(&measured, &stats));
    if let Some(path) = &options.csv {
        match write_csv(path, &candidates, &reference, &tallies) {
            Ok(()) => eprintln!("Détail écrit dans {path}"),
            Err(e) => eprintln!("Impossible d'écrire {path} : {e}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn button(name: &str, recipe: &str) -> Button {
        Button {
            name: name.to_string(),
            dice: parse_recipe(recipe).expect("valid recipe"),
        }
    }

    #[test]
    fn human_stats_are_read_from_the_published_json() {
        let json = r#"{"message": "Button stats generated on 2026-09-19",
            "data": {"Avis": {"bset": "Soldiers", "count": 420, "name": "Avis", "rate": 42.6, "tl": true}}}"#;
        let stats = parse_stats(json).unwrap();
        assert_eq!(
            stats["Avis"],
            Human {
                rate: 42.6,
                count: 420
            }
        );
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
        let stats = BTreeMap::from([(
            "Avis".to_string(),
            Human {
                rate: 42.6,
                count: 420,
            },
        )]);
        let text = report(&[summary], (0.4, 0.6), &stats);
        assert!(text.contains("43 %"), "{text}");
        assert!(text.contains("+7"), "{text}");
    }

    #[test]
    fn options_have_defaults_and_reject_unknown_flags() {
        let options = parse_options(Vec::new()).unwrap();
        assert_eq!(options.candidates, [Selection::Originals]);
        assert_eq!(options.reference, ["Soldiers", "The Core"]);
        let options = parse_options(
            [
                "--candidates",
                "bouton:Avoine",
                "--reference",
                "BROM, Soldiers",
                "--games",
                "4",
            ]
            .map(String::from),
        )
        .unwrap();
        assert_eq!(
            options.candidates,
            [Selection::Button("Avoine".to_string())]
        );
        assert_eq!(options.reference, ["BROM", "Soldiers"]);
        assert_eq!(options.games, 4);
        assert!(parse_options(["--nope".to_string()]).is_err());
        assert!(parse_options(["--games".to_string(), "0".to_string()]).is_err());
    }

    #[test]
    fn candidate_selections_add_up_and_recipes_are_named_after_themselves() {
        let options = parse_options(
            [
                "--candidates",
                "bouton:Avoine",
                "--candidates",
                "recette:(4) (6) (X)",
            ]
            .map(String::from),
        )
        .unwrap();
        let buttons = select_candidates(&original_buttons(), &options.candidates).unwrap();
        let names: Vec<&str> = buttons.iter().map(|b| b.name.as_str()).collect();
        assert_eq!(names, ["Avoine", "(4) (6) (X)"]);
        assert!(select_candidates(&[], &[Selection::Recipe("o(4)".to_string())]).is_err());
        assert!(select_candidates(&[], &[Selection::Button("Nobody".to_string())]).is_err());
    }

    #[test]
    fn wilson_interval_contains_the_rate_and_narrows_with_games() {
        let (low, high) = wilson(15, 20);
        assert!(low < 0.75 && 0.75 < high);
        let (low_more, high_more) = wilson(150, 200);
        assert!(high_more - low_more < high - low);
        assert_eq!(wilson(0, 0), (0.0, 1.0));
        assert!(wilson(0, 10).0 == 0.0 && wilson(10, 10).1 == 1.0);
    }

    #[test]
    fn normal_range_uses_the_10th_and_90th_percentiles() {
        let rates: Vec<f64> = (0..=10).map(|i| f64::from(i) / 10.0).collect();
        assert_eq!(normal_range(&rates), (0.1, 0.9));
    }

    #[test]
    fn a_button_is_flagged_only_when_its_whole_interval_is_outside() {
        let range = (0.4, 0.6);
        assert_eq!(verdict((0.62, 0.8), range), Verdict::TooStrong);
        assert_eq!(verdict((0.2, 0.38), range), Verdict::TooWeak);
        assert_eq!(verdict((0.55, 0.75), range), Verdict::Normal);
    }

    #[test]
    fn summaries_add_up_per_player_and_name_extreme_opponents() {
        let players = [button("A", "(6)"), button("B", "(8)")];
        let opponents = [button("X", "(4)"), button("Y", "(10)")];
        let tally = |wins, games| Tally {
            wins,
            games,
            unfinished: 0,
        };
        let tallies = BTreeMap::from([
            ((0, 0), tally(3, 4)),
            ((0, 1), tally(1, 4)),
            ((1, 0), tally(2, 4)),
            (
                (1, 1),
                Tally {
                    wins: 2,
                    games: 3,
                    unfinished: 1,
                },
            ),
        ]);
        let summaries = summarize(&players, &opponents, &tallies);
        assert_eq!((summaries[0].wins, summaries[0].games), (4, 8));
        assert_eq!(summaries[0].worst, Some(("Y".to_string(), 0.25)));
        assert_eq!(summaries[0].best, Some(("X".to_string(), 0.75)));
        assert_eq!(summaries[1].unfinished, 1);
    }

    #[test]
    fn a_mirrored_run_counts_each_match_for_both_sides() {
        let pool = [button("X", "(6) (8)"), button("Y", "(4) (4) (10)")];
        let options = Options {
            games: 2,
            level: Level::Easy,
            threads: 1,
            ..parse_options(Vec::new()).unwrap()
        };
        let jobs = [0, 1].map(|game| Job {
            player: 0,
            opponent: 1,
            game,
        });
        let tallies = run(&pool, &pool, &jobs, &options, 0, true, "test");
        let (x, y) = (tallies[&(0, 1)], tallies[&(1, 0)]);
        assert_eq!((x.games, y.games), (2, 2));
        assert_eq!(x.wins + y.wins, 2);
    }

    #[test]
    fn a_tiny_run_plays_every_job() {
        let players = [button("A", "(4) (6) (X)")];
        let opponents = [button("X", "(6) (8)"), button("Y", "(4) (4) (10)")];
        let options = Options {
            games: 2,
            level: Level::Easy,
            threads: 2,
            ..parse_options(Vec::new()).unwrap()
        };
        let jobs: Vec<Job> = (0..2)
            .flat_map(|opponent| {
                (0..2).map(move |game| Job {
                    player: 0,
                    opponent,
                    game,
                })
            })
            .collect();
        let tallies = run(&players, &opponents, &jobs, &options, 0, false, "test");
        let played: u32 = tallies.values().map(|t| t.games + t.unfinished).sum();
        assert_eq!(played, 4);
        assert_eq!(tallies.values().map(|t| t.unfinished).sum::<u32>(), 0);
    }
}
