mod checkers;
mod chess;
mod minesweeper;
mod tic_tac_toe;

use rand::Rng;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::time::Instant;

// ── ANSI Color Helpers ────────────────────────────────────────────────────────
const RESET:   &str = "\x1b[0m";
const BOLD:    &str = "\x1b[1m";
const RED:     &str = "\x1b[31m";
const GREEN:   &str = "\x1b[32m";
const YELLOW:  &str = "\x1b[33m";
const BLUE:    &str = "\x1b[34m";
const MAGENTA: &str = "\x1b[35m";
const CYAN:    &str = "\x1b[36m";

fn col(color: &str, text: impl std::fmt::Display) -> String {
    format!("{}{}{}", color, text, RESET)
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum Roaster {
    Ramsay,
    UncleRoger,
    RickAstley,
    SimonCowell,
    NikkiGlaser,
    JoanRivers,
    CaseOh,
    GenX,
    Millennial,
    GenZ,
}

impl Roaster {
    fn name(&self) -> &'static str {
        match self {
            Roaster::Ramsay => "Gordon Ramsay",
            Roaster::UncleRoger => "Uncle Roger",
            Roaster::RickAstley => "Rick Astley",
            Roaster::SimonCowell => "Simon Cowell",
            Roaster::NikkiGlaser => "Nikki Glaser",
            Roaster::JoanRivers => "Joan Rivers",
            Roaster::CaseOh => "CaseOh",
            Roaster::GenX => "Gen X Teen",
            Roaster::Millennial => "Millennial Teen",
            Roaster::GenZ => "Gen Z Teen",
        }
    }

    fn description(&self) -> &'static str {
        match self {
            Roaster::Ramsay => "Brutal British chef burns 🔪",
            Roaster::UncleRoger => "Haiyaa! Asian uncle cooking roasts 🍚",
            Roaster::RickAstley => "Never gonna give you up... on the puns 🎵",
            Roaster::SimonCowell => "Blunt, \"It's a no from me\" ❌",
            Roaster::NikkiGlaser => "Sharp, modern comedy roast 💅",
            Roaster::JoanRivers => "Legendary savage fashion burns 👗",
            Roaster::CaseOh => "Chaotic YouTube energy & food trauma 🎮",
            Roaster::GenX => "Whatever, this is lame anyway 🙄",
            Roaster::Millennial => "Yas queen, but also anxious & broke 📱",
            Roaster::GenZ => "No cap, this slaps fr fr 💀",
        }
    }
}

// ── Game Mode ─────────────────────────────────────────────────────────────────
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum GameMode {
    GuessingGame,
    Hangman,
    Wordle,
    Minesweeper,
    Checkers,
    Chess,
    TicTacToe,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum Difficulty {
    Easy,
    Medium,
    Hard,
    Insane,
    /// Player-defined range – not tracked on the leaderboard.
    Custom(u32, u32),
}

impl std::hash::Hash for Difficulty {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Difficulty::Easy         => 0u8.hash(state),
            Difficulty::Medium       => 1u8.hash(state),
            Difficulty::Hard         => 2u8.hash(state),
            Difficulty::Insane       => 3u8.hash(state),
            Difficulty::Custom(a, b) => { 4u8.hash(state); a.hash(state); b.hash(state); }
        }
    }
}

impl Difficulty {
    fn range(&self) -> (u32, u32) {
        match self {
            Difficulty::Easy          => (1, 100),
            Difficulty::Medium        => (1, 500),
            Difficulty::Hard          => (1, 1000),
            Difficulty::Insane        => (1, 10000),
            Difficulty::Custom(lo, hi) => (*lo, *hi),
        }
    }

    fn name(&self) -> &'static str {
        match self {
            Difficulty::Easy       => "Easy",
            Difficulty::Medium     => "Medium",
            Difficulty::Hard       => "Hard",
            Difficulty::Insane     => "Insane",
            Difficulty::Custom(..) => "Custom",
        }
    }

    fn emoji(&self) -> &'static str {
        match self {
            Difficulty::Easy       => "😊",
            Difficulty::Medium     => "😤",
            Difficulty::Hard       => "💀",
            Difficulty::Insane     => "👹",
            Difficulty::Custom(..) => "🎨",
        }
    }

    fn is_custom(&self) -> bool {
        matches!(self, Difficulty::Custom(..))
    }
}

const BAD_WORDS: &[&str] = &[
    "fuck", "shit", "cunt", "bastard", "bellend", "wanker", "piss", "asshole", "dick",
];

const HANGMAN_WORDS: &[&str] = &[
    // culinary / kitchen
    "spatula", "saffron", "risotto", "sourdough", "baguette", "fondue",
    "blanched", "flambe", "julienne", "consomme", "marinade", "sauteed",
    // music
    "rhythm", "melody", "symphony", "acoustic", "harmony", "ballad",
    "crescendo", "soprano", "treble",
    // gaming
    "respawn", "dungeon", "inventory", "joystick", "checkpoint", "polygon",
    // fashion
    "couture", "runway", "glamour", "wardrobe", "boutique", "sequin",
    "cashmere", "cravat",
    // general fun
    "avalanche", "blizzard", "jukebox", "kazoo", "fjord", "sphinx",
    "waltz", "cryptic", "phantom", "zephyr", "trophy", "zombie",
    "whirlpool", "vortex", "quartz", "oxygen", "eclipse", "labyrinth",
    "swagger", "bamboozle", "flabbergast", "gobsmacked", "kerfuffle",
    "shenanigan", "brouhaha", "hullabaloo", "rambunctious", "flummox",
];

// All Wordle words must be exactly 5 letters.
const WORDLE_WORDS: &[&str] = &[
    // culinary / kitchen
    "sauce", "broth", "spice", "gravy", "toast", "bagel", "crepe", "glaze", "brine", "basil",
    "cumin", "dough", "flour", "honey", "lemon", "maple", "olive", "sugar", "thyme", "yeast",
    "cauli", "chive", "clove", "mochi", "panko", "ramen", "salsa", "tacos", "pesto", "umami",
    // music
    "chord", "lyric", "notes", "piano", "tempo", "tenor", "vocal", "drums", "flute", "viola",
    "blues", "cello", "verse", "waltz", "pitch", "album", "beats", "synth",
    // gaming
    "quest", "sword", "level", "score", "spawn", "gamer", "pixel", "cheat", "arena", "joust",
    "boost", "bonus", "rally",
    // fashion
    "shirt", "cloak", "scarf", "boots", "skirt", "plaid", "tweed", "vogue", "rouge", "tiara",
    "satin", "linen", "gowns",
    // general fun
    "flame", "storm", "light", "brave", "candy", "cloud", "dream", "frost", "magic", "night",
    "river", "solar", "tiger", "ultra", "witch", "blaze", "crave", "disco", "fancy", "heart",
    "karma", "lunar", "mango", "ninja", "ocean", "queen", "relay", "vapor", "wafer", "zebra",
    "adore", "brisk", "crisp", "jazzy", "eagle", "flick", "gleam", "haste", "irony", "joker",
    "kneel", "lunge", "mirth", "nudge", "onset", "pouch", "quirk", "remit", "sheen", "tread",
    "umber", "vivid", "wrath", "expel", "yacht", "zingy",
];

// ── Achievement System ─────────────────────────────────────────────────────────
#[derive(Debug, PartialEq, Clone)]
enum Achievement {
    /// Nailed it on the very first attempt.
    FirstTry,
    /// Won in ≤ 3 attempts.
    SpeedDemon,
    /// Won on Insane difficulty.
    Insaniac,
    /// Won without using any hints.
    NoHints,
    /// Burned through all 3 hints in one round.
    HintAddict,
    /// Winning guess ended in the digit 7.
    LuckyNumber,
    /// Completed 5 or more rounds in a single session.
    Persistent,
    /// First-try win on Hard or Insane.
    Perfectionist,
    /// Solved a hangman word with no wrong guesses.
    HangmanFlawless,
    /// Won hangman with exactly 5 wrong guesses (one life remaining).
    NarrowEscape,
    /// Solved Wordle on the very first guess.
    WordleFlawless,
    /// Solved Wordle in ≤ 3 guesses.
    WordleGenius,
    /// Solved Wordle on the last possible guess (6th).
    WordleNarrowEscape,
    /// Won any minesweeper game.
    MinesweeperVictor,
    /// Won minesweeper on Champion difficulty.
    MinesweeperChampion,
    /// Won minesweeper on Peasant difficulty in under 60 seconds.
    MinesweeperSpeedrunner,
    /// Beat the AI at checkers.
    CheckersVictor,
    /// Beat the AI at checkers after both sides had a king on the board.
    CheckersKing,
    /// Beat the AI at chess.
    ChessVictor,
    /// Beat the AI at chess in under 30 moves.
    ChessBlitz,
    /// Beat the AI at chess after it had a queen on the board.
    ChessQueenSlayer,
    /// Beat the AI at Tic Tac Toe.
    TicTacToeVictor,
    /// Force a draw against the Tic Tac Toe AI.
    TicTacToeDraw,
    /// Beat the AI at Tic Tac Toe in the minimum 5 moves.
    TicTacToeFlawless,
}

impl Achievement {
    fn emoji(&self) -> &'static str {
        match self {
            Achievement::FirstTry    => "🎯",
            Achievement::SpeedDemon  => "⚡",
            Achievement::Insaniac    => "👹",
            Achievement::NoHints     => "🧠",
            Achievement::HintAddict  => "💡",
            Achievement::LuckyNumber => "🍀",
            Achievement::Persistent  => "🔄",
            Achievement::Perfectionist => "💎",
            Achievement::HangmanFlawless => "📖",
            Achievement::NarrowEscape    => "😰",
            Achievement::WordleFlawless  => "🟩",
            Achievement::WordleGenius    => "🧩",
            Achievement::WordleNarrowEscape => "😅",
            Achievement::MinesweeperVictor      => "💣",
            Achievement::MinesweeperChampion    => "⚔️",
            Achievement::MinesweeperSpeedrunner => "🏃",
            Achievement::CheckersVictor         => "♟",
            Achievement::CheckersKing           => "♛",
            Achievement::ChessVictor            => "♔",
            Achievement::ChessBlitz             => "⚡",
            Achievement::ChessQueenSlayer       => "♕",
            Achievement::TicTacToeVictor        => "✕",
            Achievement::TicTacToeDraw          => "🤝",
            Achievement::TicTacToeFlawless      => "⚡",
        }
    }

    fn title(&self) -> &'static str {
        match self {
            Achievement::FirstTry    => "First Try!",
            Achievement::SpeedDemon  => "Speed Demon",
            Achievement::Insaniac    => "Insaniac",
            Achievement::NoHints     => "No Hints Needed",
            Achievement::HintAddict  => "Hint Addict",
            Achievement::LuckyNumber => "Lucky Number 7",
            Achievement::Persistent  => "Persistent",
            Achievement::Perfectionist => "Perfectionist",
            Achievement::HangmanFlawless => "Flawless Vocabulary",
            Achievement::NarrowEscape    => "Narrow Escape",
            Achievement::WordleFlawless  => "Wordle Psychic",
            Achievement::WordleGenius    => "Wordle Genius",
            Achievement::WordleNarrowEscape => "Last Word Standing",
            Achievement::MinesweeperVictor      => "Mine Victor",
            Achievement::MinesweeperChampion    => "Champion Cleared",
            Achievement::MinesweeperSpeedrunner => "Speedrunner",
            Achievement::CheckersVictor         => "Checkers Champion",
            Achievement::CheckersKing           => "King of the Board",
            Achievement::ChessVictor            => "Chess Champion",
            Achievement::ChessBlitz             => "Blitz Master",
            Achievement::ChessQueenSlayer       => "Queen Slayer",
            Achievement::TicTacToeVictor        => "Tic Tac Toe Victor",
            Achievement::TicTacToeDraw          => "Stalemate Strategist",
            Achievement::TicTacToeFlawless      => "Five-Move Finisher",
        }
    }

    fn description(&self) -> &'static str {
        match self {
            Achievement::FirstTry    => "Guessed the number on the very first attempt",
            Achievement::SpeedDemon  => "Won a round in 3 attempts or fewer",
            Achievement::Insaniac    => "Won a round on Insane difficulty",
            Achievement::NoHints     => "Completed a round without requesting any hints",
            Achievement::HintAddict  => "Used all 3 hints in a single round",
            Achievement::LuckyNumber => "Won with a guess ending in the digit 7",
            Achievement::Persistent  => "Played 5 or more rounds in one session",
            Achievement::Perfectionist => "First-try win on Hard or Insane difficulty",
            Achievement::HangmanFlawless => "Solved a hangman word without any wrong guesses",
            Achievement::NarrowEscape    => "Won hangman with only one guess remaining",
            Achievement::WordleFlawless  => "Solved the Wordle word on the very first guess",
            Achievement::WordleGenius    => "Solved Wordle in 3 guesses or fewer",
            Achievement::WordleNarrowEscape => "Solved Wordle on the 6th and final guess",
            Achievement::MinesweeperVictor      => "Cleared a minesweeper board without hitting a trap",
            Achievement::MinesweeperChampion    => "Won minesweeper on Champion difficulty",
            Achievement::MinesweeperSpeedrunner => "Won Peasant minesweeper in under 60 seconds",
            Achievement::CheckersVictor         => "Defeated the AI opponent at checkers",
            Achievement::CheckersKing           => "Won a checkers game where both sides had kings",
            Achievement::ChessVictor            => "Defeated the AI opponent at chess",
            Achievement::ChessBlitz             => "Checkmated the AI in under 30 moves",
            Achievement::ChessQueenSlayer       => "Won a chess game after the AI had a queen on the board",
            Achievement::TicTacToeVictor        => "Defeated the AI opponent at Tic Tac Toe",
            Achievement::TicTacToeDraw          => "Forced a draw against the Tic Tac Toe AI",
            Achievement::TicTacToeFlawless      => "Won a Tic Tac Toe game in the minimum 5 moves",
        }
    }
}

fn main() {
    print_banner();

    let game_mode = ask_game_mode();
    let roaster = ask_roaster();
    print_roaster_intro(roaster);

    let profane = ask_profane();
    if profane {
        println!("🔞 Profanity mode: {} – Brace yourself for spicy roasts.\n", col(RED, "ON"));
    } else {
        println!("😇 Profanity mode: {} – Keeping it family-friendly.\n", col(GREEN, "OFF"));
    }

    let mut leaderboards = load_leaderboards();
    let mut session_achievements: Vec<Achievement> = Vec::new();

    // Shared counters
    let mut total_games = 0u32;
    let mut total_secs  = 0u64;

    // Guessing-game stats
    let mut total_attempts = 0u32;

    // Hangman stats
    let mut total_wins  = 0u32;
    let mut total_wrong = 0u32;

    loop {
        match game_mode {
            GameMode::GuessingGame => {
                let difficulty = ask_difficulty();
                let (attempts, guesses, elapsed_secs, hints_used) =
                    play_round(difficulty, roaster, profane);

                total_games    += 1;
                total_attempts += attempts;
                total_secs     += elapsed_secs;

                // ── Collect achievements ───────────────────────────────────────
                let mut new_achievements: Vec<Achievement> = Vec::new();

                if attempts == 1 {
                    new_achievements.push(Achievement::FirstTry);
                }
                if attempts <= 3 {
                    new_achievements.push(Achievement::SpeedDemon);
                }
                if difficulty == Difficulty::Insane {
                    new_achievements.push(Achievement::Insaniac);
                }
                if hints_used == 0 {
                    new_achievements.push(Achievement::NoHints);
                }
                if hints_used >= 3 {
                    new_achievements.push(Achievement::HintAddict);
                }
                if let Some(&last_guess) = guesses.last() {
                    if last_guess % 10 == 7 {
                        new_achievements.push(Achievement::LuckyNumber);
                    }
                }
                if total_games >= 5 && !session_achievements.contains(&Achievement::Persistent) {
                    new_achievements.push(Achievement::Persistent);
                }
                if attempts == 1 && matches!(difficulty, Difficulty::Hard | Difficulty::Insane) {
                    new_achievements.push(Achievement::Perfectionist);
                }

                // Only surface each achievement once per session.
                for ach in new_achievements {
                    if !session_achievements.contains(&ach) {
                        println!("\n{} {} {}: {}",
                            col(YELLOW, "🏅 ACHIEVEMENT UNLOCKED:"),
                            ach.emoji(),
                            col(BOLD, ach.title()),
                            ach.description()
                        );
                        session_achievements.push(ach);
                    }
                }

                print_win_stats(attempts, &guesses, elapsed_secs, hints_used);

                if !difficulty.is_custom() {
                    update_leaderboard(&mut leaderboards, difficulty, attempts, hints_used, elapsed_secs);
                    display_leaderboards(&leaderboards);
                } else {
                    println!("\n{}", col(CYAN, "ℹ️  Custom difficulty rounds are not tracked on the leaderboard."));
                }

                let avg_secs = if total_games > 0 { total_secs / total_games as u64 } else { 0 };
                println!("\n{} {} game{} played  │  {:.1} avg attempts  │  {} avg time/round",
                    col(CYAN, "📊 Session:"),
                    total_games,
                    if total_games == 1 { "" } else { "s" },
                    total_attempts as f32 / total_games as f32,
                    format_duration(avg_secs),
                );
            }

            GameMode::Hangman => {
                let (won, wrong_guesses, elapsed_secs) = play_hangman(roaster, profane);

                total_games += 1;
                total_secs  += elapsed_secs;
                total_wrong += wrong_guesses;
                if won { total_wins += 1; }

                // ── Hangman achievements ───────────────────────────────────────
                let mut new_achievements: Vec<Achievement> = Vec::new();

                if won && wrong_guesses == 0 {
                    new_achievements.push(Achievement::HangmanFlawless);
                }
                if won && wrong_guesses == 5 {
                    new_achievements.push(Achievement::NarrowEscape);
                }
                if total_games >= 5 && !session_achievements.contains(&Achievement::Persistent) {
                    new_achievements.push(Achievement::Persistent);
                }

                for ach in new_achievements {
                    if !session_achievements.contains(&ach) {
                        println!("\n{} {} {}: {}",
                            col(YELLOW, "🏅 ACHIEVEMENT UNLOCKED:"),
                            ach.emoji(),
                            col(BOLD, ach.title()),
                            ach.description()
                        );
                        session_achievements.push(ach);
                    }
                }

                let avg_secs  = if total_games > 0 { total_secs / total_games as u64 } else { 0 };
                let avg_wrong = if total_games > 0 { total_wrong as f32 / total_games as f32 } else { 0.0 };
                println!("\n{} {} game{} played  │  {} win{}  │  {:.1} avg wrong guesses  │  {} avg time/round",
                    col(CYAN, "📊 Session:"),
                    total_games,
                    if total_games == 1 { "" } else { "s" },
                    total_wins,
                    if total_wins == 1 { "" } else { "s" },
                    avg_wrong,
                    format_duration(avg_secs),
                );
            }

            GameMode::Wordle => {
                let (won, guess_count, elapsed_secs) = play_wordle(roaster, profane);

                total_games += 1;
                total_secs  += elapsed_secs;
                if won { total_wins += 1; }

                // ── Wordle achievements ────────────────────────────────────────
                let mut new_achievements: Vec<Achievement> = Vec::new();

                if won && guess_count == 1 {
                    new_achievements.push(Achievement::WordleFlawless);
                }
                if won && guess_count <= 3 {
                    new_achievements.push(Achievement::WordleGenius);
                }
                if won && guess_count == 6 {
                    new_achievements.push(Achievement::WordleNarrowEscape);
                }
                if total_games >= 5 && !session_achievements.contains(&Achievement::Persistent) {
                    new_achievements.push(Achievement::Persistent);
                }

                for ach in new_achievements {
                    if !session_achievements.contains(&ach) {
                        println!("\n{} {} {}: {}",
                            col(YELLOW, "🏅 ACHIEVEMENT UNLOCKED:"),
                            ach.emoji(),
                            col(BOLD, ach.title()),
                            ach.description()
                        );
                        session_achievements.push(ach);
                    }
                }

                let avg_secs = if total_games > 0 { total_secs / total_games as u64 } else { 0 };
                println!("\n{} {} game{} played  │  {} win{}  │  {} avg time/round",
                    col(CYAN, "📊 Session:"),
                    total_games,
                    if total_games == 1 { "" } else { "s" },
                    total_wins,
                    if total_wins == 1 { "" } else { "s" },
                    format_duration(avg_secs),
                );
            }

            GameMode::Minesweeper => {
                println!("\n{}", col(YELLOW, "💣 Launching Iron Age Minesweeper…"));
                println!("{}", col(CYAN, "  (Use arrow keys to navigate, Space/Enter to reveal, F to flag, R to restart, Q to quit)"));
                println!("{}", col(CYAN, "─".repeat(62)));

                let (won, level, elapsed_secs) = play_minesweeper();

                total_games += 1;
                total_secs  += elapsed_secs;
                if won { total_wins += 1; }

                // ── Minesweeper achievements ───────────────────────────────────
                let mut new_achievements: Vec<Achievement> = Vec::new();

                if won {
                    new_achievements.push(Achievement::MinesweeperVictor);
                }
                if won && level == minesweeper::board::Level::Champion {
                    new_achievements.push(Achievement::MinesweeperChampion);
                }
                if won && level == minesweeper::board::Level::Peasant && elapsed_secs < 60 {
                    new_achievements.push(Achievement::MinesweeperSpeedrunner);
                }
                if total_games >= 5 && !session_achievements.contains(&Achievement::Persistent) {
                    new_achievements.push(Achievement::Persistent);
                }

                for ach in new_achievements {
                    if !session_achievements.contains(&ach) {
                        println!("\n{} {} {}: {}",
                            col(YELLOW, "🏅 ACHIEVEMENT UNLOCKED:"),
                            ach.emoji(),
                            col(BOLD, ach.title()),
                            ach.description()
                        );
                        session_achievements.push(ach);
                    }
                }

                let result_str = if won {
                    col(GREEN, format!("✅ Victory on {}!", level.name()))
                } else {
                    col(RED, "💥 Better luck next time!".to_string())
                };
                let avg_secs = if total_games > 0 { total_secs / total_games as u64 } else { 0 };
                println!("\n{}  ⏱ {}  │  {} game{} played  │  {} win{}  │  {} avg time/round",
                    result_str,
                    format_duration(elapsed_secs),
                    col(CYAN, total_games.to_string()),
                    if total_games == 1 { "" } else { "s" },
                    total_wins,
                    if total_wins == 1 { "" } else { "s" },
                    format_duration(avg_secs),
                );
            }

            GameMode::Checkers => {
                println!("\n{}", col(YELLOW, "♟  Launching Iron Age Checkers…"));
                println!("{}", col(CYAN, "  (Arrow keys: move cursor | Enter/Space: select/move | Esc: deselect | Q: quit)"));
                println!("{}", col(CYAN, "─".repeat(62)));

                let (won, kings_appeared, elapsed_secs) = play_checkers();

                total_games += 1;
                total_secs  += elapsed_secs;
                if won { total_wins += 1; }

                // ── Checkers achievements ──────────────────────────────────────
                let mut new_achievements: Vec<Achievement> = Vec::new();

                if won {
                    new_achievements.push(Achievement::CheckersVictor);
                }
                if won && kings_appeared {
                    new_achievements.push(Achievement::CheckersKing);
                }
                if total_games >= 5 && !session_achievements.contains(&Achievement::Persistent) {
                    new_achievements.push(Achievement::Persistent);
                }

                for ach in new_achievements {
                    if !session_achievements.contains(&ach) {
                        println!("\n{} {} {}: {}",
                            col(YELLOW, "🏅 ACHIEVEMENT UNLOCKED:"),
                            ach.emoji(),
                            col(BOLD, ach.title()),
                            ach.description()
                        );
                        session_achievements.push(ach);
                    }
                }

                let result_str = if won {
                    col(GREEN, "✅ You defeated the AI!".to_string())
                } else {
                    col(RED, "💥 The AI outwitted you this time!".to_string())
                };
                let avg_secs = if total_games > 0 { total_secs / total_games as u64 } else { 0 };
                println!("\n{}  ⏱ {}  │  {} game{} played  │  {} win{}  │  {} avg time/round",
                    result_str,
                    format_duration(elapsed_secs),
                    col(CYAN, total_games.to_string()),
                    if total_games == 1 { "" } else { "s" },
                    total_wins,
                    if total_wins == 1 { "" } else { "s" },
                    format_duration(avg_secs),
                );
            }

            GameMode::Chess => {
                println!("\n{}", col(YELLOW, "♔ Launching Iron Age Chess…"));
                println!("{}", col(CYAN, "  (Arrow keys / hjkl: move cursor | Enter/Space: select/move | Esc: deselect | R: restart | Q: quit)"));
                println!("{}", col(CYAN, "─".repeat(62)));

                let (won, move_count, ai_had_queen, elapsed_secs) = play_chess();

                total_games += 1;
                total_secs  += elapsed_secs;
                if won { total_wins += 1; }

                // ── Chess achievements ────────────────────────────────────────
                let mut new_achievements: Vec<Achievement> = Vec::new();

                if won {
                    new_achievements.push(Achievement::ChessVictor);
                }
                if won && move_count < 30 {
                    new_achievements.push(Achievement::ChessBlitz);
                }
                if won && ai_had_queen {
                    new_achievements.push(Achievement::ChessQueenSlayer);
                }
                if total_games >= 5 && !session_achievements.contains(&Achievement::Persistent) {
                    new_achievements.push(Achievement::Persistent);
                }

                for ach in new_achievements {
                    if !session_achievements.contains(&ach) {
                        println!("\n{} {} {}: {}",
                            col(YELLOW, "🏅 ACHIEVEMENT UNLOCKED:"),
                            ach.emoji(),
                            col(BOLD, ach.title()),
                            ach.description()
                        );
                        session_achievements.push(ach);
                    }
                }

                let result_str = if won {
                    col(GREEN, "✅ Checkmate – you won!".to_string())
                } else {
                    col(RED, "💥 Better luck next time!".to_string())
                };
                let avg_secs = if total_games > 0 { total_secs / total_games as u64 } else { 0 };
                println!("\n{}  ⏱ {}  │  {} game{} played  │  {} win{}  │  {} avg time/round",
                    result_str,
                    format_duration(elapsed_secs),
                    col(CYAN, total_games.to_string()),
                    if total_games == 1 { "" } else { "s" },
                    total_wins,
                    if total_wins == 1 { "" } else { "s" },
                    format_duration(avg_secs),
                );
            }

            GameMode::TicTacToe => {
                println!("\n{}", col(YELLOW, "✕ Launching Iron Age Tic Tac Toe…"));
                println!("{}", col(CYAN, "  (Arrow keys / WASD / hjkl: move cursor | Enter / Space: place | R: restart | Q: quit)"));
                println!("{}", col(CYAN, "─".repeat(62)));

                let (won, drawn, player_moves) = play_tic_tac_toe();

                total_games += 1;
                if won { total_wins += 1; }

                // ── Tic Tac Toe achievements ────────────────────────────────
                let mut new_achievements: Vec<Achievement> = Vec::new();

                if won {
                    new_achievements.push(Achievement::TicTacToeVictor);
                }
                // 5 is the minimum number of player moves to win (player's turns 1, 3, 5).
                if won && player_moves <= 5 {
                    new_achievements.push(Achievement::TicTacToeFlawless);
                }
                if drawn {
                    new_achievements.push(Achievement::TicTacToeDraw);
                }
                if total_games >= 5 && !session_achievements.contains(&Achievement::Persistent) {
                    new_achievements.push(Achievement::Persistent);
                }

                for ach in new_achievements {
                    if !session_achievements.contains(&ach) {
                        println!("\n{} {} {}: {}",
                            col(YELLOW, "🏅 ACHIEVEMENT UNLOCKED:"),
                            ach.emoji(),
                            col(BOLD, ach.title()),
                            ach.description()
                        );
                        session_achievements.push(ach);
                    }
                }

                let result_str = if won {
                    col(GREEN, "✅ You defeated the AI!".to_string())
                } else if drawn {
                    col(MAGENTA, "🤝 It's a draw!".to_string())
                } else {
                    col(RED, "💥 The AI wins this time!".to_string())
                };
                let avg_secs = if total_games > 0 { total_secs / total_games as u64 } else { 0 };
                println!("\n{}  │  {} game{} played  │  {} win{}  │  {} avg time/round",
                    result_str,
                    col(CYAN, total_games.to_string()),
                    if total_games == 1 { "" } else { "s" },
                    total_wins,
                    if total_wins == 1 { "" } else { "s" },
                    format_duration(avg_secs),
                );
            }
        }

        if !session_achievements.is_empty() {
            println!("{} {}", col(YELLOW, "🏅 Achievements this session:"),
                session_achievements.iter()
                    .map(|a| format!("{} {}", a.emoji(), a.title()))
                    .collect::<Vec<_>>()
                    .join("  │  ")
            );
        }

        if !ask_play_again() {
            save_leaderboards(&leaderboards);
            print_goodbye(roaster);
            break;
        }
        println!("\n{}\n", "=".repeat(60));
    }
}

fn print_banner() {
    println!("{}", col(CYAN, "╔════════════════════════════════════════════════════════════╗"));
    println!("{}", col(CYAN, "║") + &col(BOLD, "         🎮  ULTRA GAME SUITE v7.0 – HOME PAGE  🎮        ") + &col(CYAN, "║"));
    println!("{}", col(CYAN, "╚════════════════════════════════════════════════════════════╝"));
    println!();
    println!("{}", col(BOLD, "  🕹️  Choose from 6 exciting games:"));
    println!();
    println!("  {} {}",
        col(YELLOW, "1."),
        col(BOLD, "🎲 Number Guessing Game")
    );
    println!("     Guess the secret number with roaster commentary!");
    println!("     – 4 difficulty levels (Easy → Insane) + Custom range");
    println!("     – Warmth hints, in-round clues, persistent leaderboards");
    println!();
    println!("  {} {}",
        col(YELLOW, "2."),
        col(BOLD, "💀 Hangman")
    );
    println!("     Guess the hidden word letter by letter before you're hanged!");
    println!("     – Themed word pool: culinary, music, gaming & more");
    println!("     – 6 lives, roaster commentary on every wrong guess");
    println!();
    println!("  {} {}",
        col(YELLOW, "3."),
        col(BOLD, "🟩 Wordle")
    );
    println!("     Guess the secret 5-letter word in 6 tries!");
    println!("     – 🟩 correct position  🟨 wrong position  ⬛ not in word");
    println!("     – Roaster commentary after every guess");
    println!();
    println!("  {} {}",
        col(YELLOW, "4."),
        col(BOLD, "💣 Iron Age Minesweeper")
    );
    println!("     Navigate a cursed ruin and flag every goblin trap!");
    println!("     – 3 difficulty levels: Peasant, Knight, Champion");
    println!("     – Full TUI keyboard interface with Roman-numeral clues");
    println!();
    println!("  {} {}",
        col(YELLOW, "5."),
        col(BOLD, "♟  Iron Age Checkers")
    );
    println!("     Outmanoeuvre the AI on the ancient 8×8 board!");
    println!("     – Full American checkers rules: mandatory captures, kings, multi-jumps");
    println!("     – Minimax AI opponent with alpha-beta pruning (depth 5)");
    println!();
    println!("  {} {}",
        col(YELLOW, "6."),
        col(BOLD, "♔  Iron Age Chess")
    );
    println!("     Face the AI across the 64-square battlefield – single player!");
    println!("     – Full chess rules: castling, en passant, promotion, check & checkmate");
    println!("     – Minimax AI opponent with alpha-beta pruning + piece-square tables");
    println!();
    println!("  {} {}",
        col(YELLOW, "7."),
        col(BOLD, "✕  Iron Age Tic Tac Toe")
    );
    println!("     Face the AI on the classic 3×3 grid – can you outwit it?");
    println!("     – Minimax AI opponent (80 % optimal + 20 % random to keep it fun)");
    println!("     – Full TUI keyboard interface");
    println!();
    println!("{}", col(BOLD, "  ✨ Features across all games:"));
    println!("     • 10 unique roasters with personality");
    println!("     • Optional profanity mode 🔞");
    println!("     • 24 achievements to unlock 🏅");
    println!("     • Per-round timer ⏱️");
    println!("     • Session statistics 📊");
    println!("{}", col(CYAN, "─".repeat(62)));
    println!();
}

fn print_roaster_intro(roaster: Roaster) {
    println!("\n{}", "─".repeat(60));
    match roaster {
        Roaster::Ramsay => println!("🔪 Gordon Ramsay: \"Right, you donut. Let's see if you can count!\""),
        Roaster::UncleRoger => println!("🍚 Uncle Roger: \"Haiyaa! You better not disappoint Uncle Roger!\""),
        Roaster::RickAstley => println!("🎵 Rick Astley: \"Never gonna give you up on this game!\""),
        Roaster::SimonCowell => println!("❌ Simon Cowell: \"Let's see if you're any good at this.\""),
        Roaster::NikkiGlaser => println!("💅 Nikki Glaser: \"Oh honey, this should be interesting...\""),
        Roaster::JoanRivers => println!("👗 Joan Rivers: \"Can we talk? Let's see those guessing skills!\""),
        Roaster::CaseOh => println!("🎮 CaseOh: \"CHAT! CHAT! Watch me destroy this person at guessing!\""),
        Roaster::GenX => println!("🙄 Gen X: \"Whatever, this is probably rigged anyway.\""),
        Roaster::Millennial => println!("📱 Millennial: \"OMG this is giving early 2000s vibes! Let's do this!\""),
        Roaster::GenZ => println!("💀 Gen Z: \"Bestie, this about to be a whole vibe, no cap.\""),
    }
    println!("{}\n", "─".repeat(60));
}

fn print_win_stats(attempts: u32, guesses: &[u32], elapsed_secs: u64, hints_used: u32) {
    println!("\n{}", col(YELLOW, "🌟".repeat(30)));
    println!("{}", col(BOLD, format!(
        "🏆 VICTORY! You nailed it in {} attempt{}!",
        attempts,
        if attempts == 1 { "" } else { "s" }
    )));
    
    if attempts == 1 {
        println!("{}", col(GREEN, "💯 PERFECT! First try! Are you psychic?!"));
    } else if attempts <= 3 {
        println!("{}", col(GREEN, "🔥 INCREDIBLE! You're a natural!"));
    } else if attempts <= 5 {
        println!("👏 Well done! Solid performance!");
    } else if attempts <= 10 {
        println!("👍 Not bad! Room for improvement!");
    } else {
        println!("😅 Finally! That was... a journey!");
    }

    println!("⏱️  Time taken: {}", col(CYAN, format_duration(elapsed_secs)));

    if hints_used == 0 {
        println!("{}", col(GREEN, "🧠 No hints used – pure skill!"));
    } else {
        println!("💡 Hints used: {} (each hint adds +5 to your effective score)", col(YELLOW, hints_used));
    }
    
    println!("Your guessing journey: {}", 
        col(MAGENTA, guesses.iter()
            .map(|g| g.to_string())
            .collect::<Vec<_>>()
            .join(" → "))
    );
    println!("{}\n", col(YELLOW, "🌟".repeat(30)));
}

/// Format a duration in seconds as a human-readable string.
fn format_duration(secs: u64) -> String {
    if secs < 60 {
        format!("{}s", secs)
    } else {
        format!("{}m {}s", secs / 60, secs % 60)
    }
}

fn print_goodbye(roaster: Roaster) {
    println!("\n{}", "═".repeat(60));
    match roaster {
        Roaster::Ramsay => println!("🔪 Ramsay: \"Get out! ...But well done, honestly.\""),
        Roaster::UncleRoger => println!("🍚 Uncle Roger: \"Okay lah, you did good. Uncle Roger approve!\""),
        Roaster::RickAstley => println!("🎵 Rick: \"Never gonna say goodbye! ...Wait, actually, goodbye!\""),
        Roaster::SimonCowell => println!("❌ Simon: \"You know what? That wasn't terrible. See you.\""),
        Roaster::NikkiGlaser => println!("💅 Nikki: \"Thanks babe, that was fun! Don't be a stranger!\""),
        Roaster::JoanRivers => println!("👗 Joan: \"Darling, you were fabulous! Mwah!\""),
        Roaster::CaseOh => println!("🎮 CaseOh: \"GG CHAT! That was actually fire! Peace out!\""),
        Roaster::GenX => println!("🙄 Gen X: \"Cool, whatever. Later.\""),
        Roaster::Millennial => println!("📱 Millennial: \"This was honestly iconic! Ttyl bestie!\""),
        Roaster::GenZ => println!("💀 Gen Z: \"No cap you ate that up! Purr! Bye bestie!\""),
    }
    println!("💾 Leaderboard saved. Thanks for playing ULTRA GUESSING GAME!");
    println!("{}\n", "═".repeat(60));
}

fn play_round(difficulty: Difficulty, roaster: Roaster, profane: bool) -> (u32, Vec<u32>, u64, u32) {
    let (lower, upper) = difficulty.range();
    let secret_number = rand::thread_rng().gen_range(lower..=upper);

    println!("\n{} {} Mode: Guess between {} and {}", 
        difficulty.emoji(), 
        col(BOLD, difficulty.name()),
        col(CYAN, lower),
        col(CYAN, upper),
    );
    println!("💡 Hint: I've picked a number. Time to prove yourself!");
    println!("   (Type {} at any prompt for a coded clue — up to 3 per round, +5 effective attempts each)\n",
        col(YELLOW, "'h'"));

    let mut attempts   = 0u32;
    let mut hints_used = 0u32;
    let mut guesses    = Vec::new();
    let mut previous_diff: Option<u32> = None;
    let round_start = Instant::now();

    let (mut low_jibes, mut high_jibes, win_message): (Vec<String>, Vec<String>, &'static str) = match roaster {
        Roaster::Ramsay => (
            vec![
                "Too small! You absolute donkey!",
                "What are you—an idiot sandwich guessing low?",
                "Too small! My gran could do better, and she's dead!",
                "Hey, panini head, wake up and guess higher!",
                "Too low! You stupid donut!",
                "That's so low it's raw—still mooing!",
                "Too small! You fucking idiot—aim up!",
                "Blimey, that's colder than my freezer. Try harder!",
                "Too low! You muppet, you're embarrassing yourself!",
                "Too small! You're having a laugh, aren't you?",
                "A bit low there, mate – guess better.",
                "Higher, you plonker!",
                "Cold as a London winter. Aim up!",
                "Too low! Absolute pants.",
                "You wanker—guess higher!",
                "Piss off with that low rubbish!",
            ].into_iter().map(String::from).collect(),
            vec![
                "Too high! You overcooked donkey!",
                "Lower! It's so high it's burnt to a crisp!",
                "Too big! Greedy panini head!",
                "Reel it in, you absolute plank!",
                "That's so high it's fucking charred!",
                "Too high! You twit—come down to earth!",
                "Way too big! Piss off with that guess!",
                "Lower! You donut, you're scorching everything!",
                "Too high! My dead gran wouldn't overshoot like that!",
                "Greedy sod—dial it back before I lose it!",
                "Too big! Greedy, aren't ya?",
                "Way too high – reel it in, guv'nor!",
                "Too big! You're taking the mickey.",
                "Lower, you wally!",
                "That's overshot by a country mile.",
                "You absolute bastard—lower!",
            ].into_iter().map(String::from).collect(),
            "🎯 Bang on! Finally, you got it right – about bloody time!",
        ),
        Roaster::UncleRoger => (
            vec![
                "Haiyaa! Too low lah! So weak!",
                "Why you guess so low? No strength at all!",
                "Haiyah! Too small – you fry rice like this ah?",
                "Too low! Emotionally damage my wok!",
                "Haiyaa! Guess higher lah, don't be so sad!",
                "So low... you put no MSG in your guess?",
                "Aiyo! Too low – children guess better!",
                "Why so weak? Lift your guess higher!",
                "Haiyaa! This guess no flavor – too low!",
                "Too small lah! Uncle Roger disappointed!",
                "Aiyah! Guess low like no confidence!",
                "Too low! You make my ancestors cry!",
                "Haiyaa! Higher please, don't torture Uncle!",
                "So low... like putting colander on rice cooker!",
                "Aiyo! You guess like Jamie Oliver cook rice!",
            ].into_iter().map(String::from).collect(),
            vec![
                "Fuiyoh! Too high lah! Overcook already!",
                "Haiyah! Too big – you put too much MSG!",
                "Haiyaa! Way too high – wok on fire!",
                "Too high! You deep fry until burnt ah?",
                "Fuiyoh! Reel it in – too much oil!",
                "So high... you make Uncle Roger scream!",
                "Aiyo! Too big – lower lah, don't be crazy!",
                "Haiyaa! This guess over-seasoned!",
                "Too high! You boil soup until dry?",
                "Fuiyoh! Calm down – guess lower!",
                "Aiyah! Too much – Uncle Roger cannot take!",
                "Way too high! You add chili until die!",
                "Haiyaa! Lower please, save the rice!",
                "Too high! Like putting ketchup in fried rice!",
            ].into_iter().map(String::from).collect(),
            "🎯 Fuiyoh! Correct lah! Uncle Roger proud of you! MSG approved!",
        ),
        Roaster::RickAstley => (
            vec![
                "Too low! But I'm never gonna let you down... so guess higher!",
                "Never gonna give you up... but that guess is too small!",
                "We're no strangers to bad guesses – aim up!",
                "Too low! Never gonna run around and desert the right number!",
                "Never gonna make you cry... unless you keep guessing low!",
                "That guess is too small – never gonna say goodbye to roasting!",
                "Never gonna tell a lie... your guess is low!",
                "Too low! You've known the rules, and so do I – higher!",
                "Never gonna give this up... try a bigger number!",
                "A full commitment's what I'm thinking of – guess higher!",
            ].into_iter().map(String::from).collect(),
            vec![
                "Too high! Never gonna run around with big numbers!",
                "That guess is too big – never gonna give you up!",
                "Never gonna let you down... by guessing lower!",
                "Too high! You've got to make me understand – reel it in!",
                "Never gonna desert you... with overshoots like that!",
                "Way too high! Never gonna say goodbye to banter!",
                "Never gonna tell a lie and hurt you... but that guess hurts!",
                "Too big! A full commitment to lower numbers now!",
                "Never gonna make you cry... unless you keep going high!",
                "Guess lower – never gonna give this roast up!",
            ].into_iter().map(String::from).collect(),
            "🎯 Never gonna give you up... you finally got it! Well played!",
        ),
        Roaster::SimonCowell => (
            vec![
                "Too low. That was absolutely dreadful.",
                "It's a no from me – guess higher.",
                "Honestly, that guess was terrible.",
                "Far too low. I didn't like it at all.",
                "That was one of the worst guesses I've seen. Higher.",
                "Dreadful. Absolutely dreadful.",
                "Too low! Not good enough, I'm afraid.",
                "I don't mean to be rude, but that's pants.",
                "That guess was forgettable – too small.",
                "No. Just no. Try higher.",
                "If I'm being honest, that's not it.",
                "Too low – you've got no chance with that.",
                "I'm sorry, but that's a disaster.",
            ].into_iter().map(String::from).collect(),
            vec![
                "Too high. Over the top.",
                "It's a no from me – reel it in.",
                "That was far too much.",
                "Way too high. Honestly, terrible.",
                "Too big! I didn't like it.",
                "That's just not right – lower.",
                "Absolutely dreadful. Lower please.",
                "Too high – one of the worst I've heard.",
                "No from me. Guess lower.",
                "That guess was completely off.",
                "Ghastly. Simply ghastly.",
            ].into_iter().map(String::from).collect(),
            "🎯 Well done. That was actually very good. I'm impressed.",
        ),
        Roaster::NikkiGlaser => (
            vec![
                "Too low – that's disappointing.",
                "Too small! Come on, aim higher.",
                "That's like my standards – way too low.",
                "Too low! You're undershooting, babe.",
                "Ouch, too low – that's sad.",
                "Too small! Step your game up.",
                "Too low – fucking embarrassing.",
                "Guess higher, you idiot.",
                "That's so low it's pathetic.",
                "Too low! What the fuck?",
                "Lower than my expectations – higher please.",
                "Babe, no. Higher.",
                "That's giving desperate energy – aim up.",
            ].into_iter().map(String::from).collect(),
            vec![
                "Too high – greedy much?",
                "Way too big! Reel it in.",
                "That's overcompensating – lower.",
                "Too high! Calm down.",
                "Overshot it – classic overreach.",
                "Too big! Fucking relax.",
                "That's way too high, babe.",
                "Too high – you're trying too hard.",
                "Lower! Jesus Christ.",
                "Too big – dial it back.",
                "Honey, that's too much.",
            ].into_iter().map(String::from).collect(),
            "🎯 Yes! Finally – you got there. Proud of you, babe!",
        ),
        Roaster::JoanRivers => (
            vec![
                "Too low! Can we talk? That guess is hideous.",
                "Oh honey, too low – that's tragic.",
                "That number looks like it needs work – higher!",
                "Too small, darling – it fell off the ugly tree.",
                "Guess higher! That was atrocious.",
                "Too low – who let you dress like that?",
                "That's so low it's disgusting.",
                "Higher! That guess is a disaster.",
                "Too low! You look ridiculous.",
                "Can we talk? Too fucking low.",
                "That guess is ugly – higher please.",
                "Darling, no. That's awful.",
            ].into_iter().map(String::from).collect(),
            vec![
                "Too high! That's overdone, darling.",
                "Way too big – calm down.",
                "Too high! It looks ridiculous.",
                "Reel it in – that's hideous.",
                "Too big! Oh honey, no.",
                "That's over the top – tragic.",
                "Lower! Fucking terrible.",
                "Too high – who dressed you?",
                "That guess is a mess – lower.",
                "Too big! Disaster.",
                "Honey, that's a crime against numbers.",
            ].into_iter().map(String::from).collect(),
            "🎯 Oh honey, you got it! Fabulous! Simply divine!",
        ),
        Roaster::CaseOh => (
            vec![
                "CHAT! Too low! This person is TROLLING!",
                "Bro, that's so low! CHAT is laughing at you!",
                "Too small! You're getting timed out for that guess!",
                "CHAT CHAT CHAT! Too low! This is embarrassing!",
                "Nah bro, higher! You're making me look bad!",
                "Too low! That's it, I'm eating another burger out of stress!",
                "WHAT?! Too low! CHAT, spam L's!",
                "Bro, that's lower than my K/D ratio! Higher!",
                "Too small! I'm literally malding right now!",
                "CHAT! This person needs help! Too low!",
                "Nah nah nah, too low! You're cooked!",
                "Higher bro! CHAT is NOT impressed!",
                "Too low! This is giving bot behavior!",
                "Bro really guessed that low! L + ratio + higher!",
                "CHAT! Too fucking low! This is content!",
            ].into_iter().map(String::from).collect(),
            vec![
                "TOO HIGH! CHAT, they're trolling!",
                "Bro went way too high! Lower!",
                "CHAT CHAT! Too big! This is crazy!",
                "Nah bro, reel it in! Way too high!",
                "Too high! I'm stress eating Takis over this!",
                "WHAT?! Lower! CHAT, clip that!",
                "Too big! You're as wrong as my diet!",
                "Bro, lower! This is painful to watch!",
                "CHAT! Too high! Someone help this person!",
                "Way too high! You're griefing me right now!",
                "Lower bro! This is NOT it!",
                "Too high! CHAT is cringing!",
                "Bro really overshot! That's an L! Lower!",
                "Too fucking high! I'm dying! CHAT, help!",
            ].into_iter().map(String::from).collect(),
            "🎯 YOOOOO! CHAT! THEY GOT IT! GG! That was actually fire!",
        ),
        Roaster::GenX => (
            vec![
                "Too low. Whatever.",
                "Like, too small. Not that I care.",
                "Too low. This is lame anyway.",
                "That guess sucks. Go higher.",
                "Too low. As if.",
                "Ugh, too small. Try harder, I guess.",
                "Too low. Talk to the hand.",
                "That's low. Whatever, guess higher.",
                "Too small. This is so bogus.",
                "Too low. Gag me with a spoon.",
                "Higher. Not that it matters.",
                "Too low. Psych! Go up.",
                "That's weak sauce. Higher.",
                "Too low. Don't have a cow, just guess higher.",
            ].into_iter().map(String::from).collect(),
            vec![
                "Too high. Whatever.",
                "Way too big. Lower, I guess.",
                "Too high. This is so lame.",
                "That's high. Lower. Not that I care.",
                "Too big. As if I care. Lower.",
                "Ugh, too high. Try lower.",
                "Too high. Whatevs.",
                "That's too much. Lower.",
                "Way too high. Bogus guess.",
                "Too high. Lower or whatever.",
                "Too big. This is dumb anyway.",
                "Lower. Not like it matters.",
            ].into_iter().map(String::from).collect(),
            "🎯 Cool, you got it. Whatever. I guess that's good or something.",
        ),
        Roaster::Millennial => (
            vec![
                "Too low bestie! That's not giving what it needs to give!",
                "OMG too small! Guess higher, I'm literally dying!",
                "Too low! This is NOT the vibe! Higher please!",
                "Bestie... too low. I can't even. Go higher!",
                "Too small! That's so cringe! Higher!",
                "Oof, too low! That hit different (badly). Higher!",
                "Too low! Periodt! Guess higher!",
                "No cap that's too low! Higher bestie!",
                "Too small! That's giving broke millennial energy! Up!",
                "Too low! I'm having an existential crisis! Higher!",
                "Bestie that's too low! Slay somewhere higher!",
                "Too small! My anxiety can't take this! Higher!",
                "Too low! That's not it, sis! Aim up!",
                "OMG too low! I'm too emotionally invested! Higher!",
                "Too fucking low! Higher or I'm cancelling you!",
            ].into_iter().map(String::from).collect(),
            vec![
                "Too high bestie! Lower! I'm literally shaking!",
                "Way too big! That's giving try-hard energy! Lower!",
                "Too high! Sis, no! Bring it down!",
                "Bestie... too high. I can't. Lower please!",
                "Too big! That's so extra! Lower!",
                "Oof, too high! That's not the tea! Lower!",
                "Too high! This ain't it, chief! Down!",
                "Way too big bestie! Lower or I'm unfollowing!",
                "Too high! My therapist will hear about this! Lower!",
                "Bestie that's too high! Reel it in!",
                "Too big! I'm having a moment! Lower!",
                "Too high! That's not the vibe! Down!",
                "Way too big sis! I'm too anxious for this! Lower!",
                "Too fucking high! I'm literally crying! Lower!",
            ].into_iter().map(String::from).collect(),
            "🎯 YASSS QUEEN! You did that! I'm so proud! That's so slay! 💅",
        ),
        Roaster::GenZ => (
            vec![
                "Too low bestie! That's giving L energy fr! Higher!",
                "Nah that's too small! No cap, aim up!",
                "Too low! Bestie you're cooked! Higher fr fr!",
                "Bro that's mid and too low! Up!",
                "Too small! That's not bussin! Higher!",
                "Low key too low! High key need higher!",
                "Too low! Deadass guess higher!",
                "That ain't it bestie! Too low! Up!",
                "Too small! This ain't giving! Higher fr!",
                "Nah bro, too low! Periodt! Guess up!",
                "Too low! Bro fell off! Higher!",
                "That's cap! Too low! Go higher bestie!",
                "Too small! You're tweaking! Up!",
                "Low key too low fr fr! Higher!",
                "Too fucking low! You're cooked! Higher!",
                "Nah that's too low! Ratio + L + higher!",
                "Too small bestie! This ain't giving main character! Up!",
            ].into_iter().map(String::from).collect(),
            vec![
                "Too high bestie! That's doing too much! Lower!",
                "Nah that's too big! No cap, down!",
                "Too high! Bro you're cooked! Lower fr!",
                "That's too much! Not bussin! Lower!",
                "Way too high! That's sus! Down!",
                "High key too high! Low key need lower!",
                "Too high! Deadass lower bestie!",
                "That ain't it! Too high! Down fr!",
                "Too big! This ain't the vibe! Lower!",
                "Nah bro, too high! Periodt! Lower!",
                "Too high! You fell off! Down!",
                "That's cap! Too high! Lower bestie!",
                "Too big! You're tweaking! Down fr!",
                "High key too high! Lower!",
                "Too fucking high! You're cooked! Lower!",
                "Nah that's too high! L + ratio + lower!",
            ].into_iter().map(String::from).collect(),
            "🎯 YOOO YOU ATE THAT UP! No cap that was bussin! Purr bestie! 💅💀",
        ),
    };

    // Profanity filter
    if !profane {
        low_jibes = low_jibes
            .into_iter()
            .filter(|j| !BAD_WORDS.iter().any(|&w| j.to_lowercase().contains(w)))
            .collect();
        high_jibes = high_jibes
            .into_iter()
            .filter(|j| !BAD_WORDS.iter().any(|&w| j.to_lowercase().contains(w)))
            .collect();

        if low_jibes.is_empty() {
            low_jibes.push(String::from("Too low!"));
        }
        if high_jibes.is_empty() {
            high_jibes.push(String::from("Too high!"));
        }
    }

    loop {
        print!("💭 Your guess ({}-{}) or {} for a hint: ", lower, upper, col(YELLOW, "'h'"));
        io::stdout().flush().expect("Failed to flush stdout");

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");

        let trimmed = input.trim();

        // ── Hint system ──────────────────────────────────────────────────
        if trimmed.eq_ignore_ascii_case("h") || trimmed.eq_ignore_ascii_case("hint") {
            if hints_used >= 3 {
                println!("{}", col(RED, "💡 No hints left! You've used all 3 for this round."));
            } else {
                hints_used += 1;
                give_hint(secret_number, lower, upper, hints_used);
            }
            continue;
        }

        let guess: u32 = match trimmed.parse() {
            Ok(num) => num,
            Err(_) => {
                println!("{}", col(RED, "❌ That's not even a proper number. Try again."));
                continue;
            }
        };

        if guess < lower || guess > upper {
            println!("{}", col(RED, format!("⚠️  Out of range – stick to {}-{}!", lower, upper)));
            continue;
        }

        attempts += 1;
        guesses.push(guess);

        let current_diff = guess.abs_diff(secret_number);

        println!("\n📍 Attempt #{}: You guessed {}", attempts, col(BOLD, guess));
        
        match guess.cmp(&secret_number) {
            Ordering::Less => {
                let jibe = low_jibes[rand::thread_rng().gen_range(0..low_jibes.len())].as_str();
                println!("{}", col(RED, format!("🔥 {jibe}")));
            }
            Ordering::Greater => {
                let jibe = high_jibes[rand::thread_rng().gen_range(0..high_jibes.len())].as_str();
                println!("{}", col(BLUE, format!("🔥 {jibe}")));
            }
            Ordering::Equal => {
                println!("\n{}", col(GREEN, win_message));
                let elapsed = round_start.elapsed().as_secs();
                return (attempts, guesses, elapsed, hints_used);
            }
        }

        // Warmth system
        if let Some(prev_diff) = previous_diff {
            if current_diff < prev_diff {
                println!("{}", col(RED, "🌡️  Getting WARMER! 🔥"));
            } else if current_diff > prev_diff {
                println!("{}", col(CYAN, "❄️  Getting COLDER! 🧊"));
            } else {
                println!("😐 Same distance – you're circling it!");
            }
        }

        // Extra hint for Insane mode
        if difficulty == Difficulty::Insane && attempts >= 5 {
            if current_diff <= 100 {
                println!("{}", col(RED, "🎯 SUPER HOT! You're within 100!"));
            } else if current_diff <= 500 {
                println!("{}", col(YELLOW, "🔥 Getting warm! Within 500!"));
            }
        }

        previous_diff = Some(current_diff);
        
        println!("📜 History: {}\n", 
            col(MAGENTA, guesses.iter()
                .map(|g| g.to_string())
                .collect::<Vec<_>>()
                .join(", "))
        );
    }
}

/// Give the player a contextual hint based on how many they've already used.
fn give_hint(secret: u32, lower: u32, upper: u32, hint_num: u32) {
    let hint_text = match hint_num {
        1 => {
            // Parity hint
            let parity = if secret % 2 == 0 { "even" } else { "odd" };
            format!("The secret number is {}.", parity)
        }
        2 => {
            // Mid-range hint: indicate which half the number falls in.
            let mid = lower + (upper - lower) / 2;
            let position = if secret <= mid { "in the lower half" } else { "in the upper half" };
            format!("The secret number is {} of the range ({}-{}).", position, lower, upper)
        }
        3 => {
            // Divisibility hint
            let div5 = if secret % 5 == 0 { "divisible" } else { "not divisible" };
            format!("The secret number is {} by 5.", div5)
        }
        _ => String::from("No more hints available."),
    };
    println!("{} {} (+5 to your effective score)",
        col(YELLOW, "💡 Hint:"),
        col(CYAN, &hint_text)
    );
}

fn ask_roaster() -> Roaster {
    loop {
        println!("\n🎭 Choose your roaster (they'll roast your guesses):\n");
        println!("  1. {:<20} – {}", Roaster::Ramsay.name(), Roaster::Ramsay.description());
        println!("  2. {:<20} – {}", Roaster::UncleRoger.name(), Roaster::UncleRoger.description());
        println!("  3. {:<20} – {}", Roaster::RickAstley.name(), Roaster::RickAstley.description());
        println!("  4. {:<20} – {}", Roaster::SimonCowell.name(), Roaster::SimonCowell.description());
        println!("  5. {:<20} – {}", Roaster::NikkiGlaser.name(), Roaster::NikkiGlaser.description());
        println!("  6. {:<20} – {}", Roaster::JoanRivers.name(), Roaster::JoanRivers.description());
        println!("  7. {:<20} – {}", Roaster::CaseOh.name(), Roaster::CaseOh.description());
        println!("  8. {:<20} – {}", Roaster::GenX.name(), Roaster::GenX.description());
        println!("  9. {:<20} – {}", Roaster::Millennial.name(), Roaster::Millennial.description());
        println!(" 10. {:<20} – {}", Roaster::GenZ.name(), Roaster::GenZ.description());
        
        print!("\n🎯 Your choice (1-10): ");
        io::stdout().flush().expect("Failed to flush stdout");

        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read line");

        match input.trim() {
            "1" => return Roaster::Ramsay,
            "2" => return Roaster::UncleRoger,
            "3" => return Roaster::RickAstley,
            "4" => return Roaster::SimonCowell,
            "5" => return Roaster::NikkiGlaser,
            "6" => return Roaster::JoanRivers,
            "7" => return Roaster::CaseOh,
            "8" => return Roaster::GenX,
            "9" => return Roaster::Millennial,
            "10" => return Roaster::GenZ,
            _ => println!("❌ Please enter a number between 1-10.\n"),
        }
    }
}

fn ask_profane() -> bool {
    loop {
        print!("🔞 Enable profanity in roasts? (y/n): ");
        io::stdout().flush().expect("Failed to flush stdout");

        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read line");

        match input.trim().to_lowercase().as_str() {
            "y" | "yes" => return true,
            "n" | "no" => return false,
            _ => println!("❌ Just y or n, please!"),
        }
    }
}

fn ask_difficulty() -> Difficulty {
    loop {
        println!("\n🎮 Choose your difficulty:\n");
        println!("  1. {} Easy   (1–100)         – Perfect for beginners", Difficulty::Easy.emoji());
        println!("  2. {} Medium (1–500)         – A fair challenge", Difficulty::Medium.emoji());
        println!("  3. {} Hard   (1–1000)        – For the brave", Difficulty::Hard.emoji());
        println!("  4. {} Insane (1–10000)       – Are you psychic?", Difficulty::Insane.emoji());
        println!("  5. 🎨 Custom (your range)   – Define your own boundaries");
        
        print!("\n🎯 Your choice (1-5): ");
        io::stdout().flush().expect("Failed to flush stdout");

        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read line");

        match input.trim() {
            "1" => return Difficulty::Easy,
            "2" => return Difficulty::Medium,
            "3" => return Difficulty::Hard,
            "4" => return Difficulty::Insane,
            "5" => {
                if let Some(range) = ask_custom_range() {
                    return Difficulty::Custom(range.0, range.1);
                }
            }
            _ => println!("{}", col(RED, "❌ Please enter 1, 2, 3, 4, or 5.\n")),
        }
    }
}

/// Prompt the player to enter a custom min/max range. Returns None if input is invalid.
fn ask_custom_range() -> Option<(u32, u32)> {
    print!("  Enter minimum (≥ 1): ");
    io::stdout().flush().expect("Failed to flush stdout");
    let mut buf = String::new();
    io::stdin().read_line(&mut buf).expect("Failed to read line");
    let min: u32 = match buf.trim().parse() {
        Ok(n) if n >= 1 => n,
        _ => {
            println!("{}", col(RED, "❌ Invalid minimum – must be a whole number ≥ 1."));
            return None;
        }
    };

    print!("  Enter maximum (> minimum): ");
    io::stdout().flush().expect("Failed to flush stdout");
    buf.clear();
    io::stdin().read_line(&mut buf).expect("Failed to read line");
    let max: u32 = match buf.trim().parse() {
        Ok(n) if n > min => n,
        _ => {
            println!("{}", col(RED, "❌ Invalid maximum – must be a whole number greater than the minimum."));
            return None;
        }
    };

    println!("🎨 Custom range set: {} – {}", col(CYAN, min), col(CYAN, max));
    Some((min, max))
}

fn load_leaderboards() -> HashMap<Difficulty, Vec<(String, u32, u32, u64)>> {
    let mut map: HashMap<Difficulty, Vec<(String, u32, u32, u64)>> = HashMap::new();
    for diff in [Difficulty::Easy, Difficulty::Medium, Difficulty::Hard, Difficulty::Insane] {
        map.insert(diff, Vec::new());
    }

    if let Ok(content) = fs::read_to_string("leaderboard.txt") {
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() { continue; }
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() >= 3 {
                let diff = match parts[0] {
                    "Easy"   => Difficulty::Easy,
                    "Medium" => Difficulty::Medium,
                    "Hard"   => Difficulty::Hard,
                    "Insane" => Difficulty::Insane,
                    _ => continue,
                };
                let name = parts[1].to_string();
                if let Ok(attempts) = parts[2].parse::<u32>() {
                    // Backward-compatible: hints and time default to 0 if absent.
                    let hints = parts.get(3).and_then(|s| s.parse().ok()).unwrap_or(0u32);
                    let secs  = parts.get(4).and_then(|s| s.parse().ok()).unwrap_or(0u64);
                    map.entry(diff).or_default().push((name, attempts, hints, secs));
                }
            }
        }
    }

    for vec in map.values_mut() {
        vec.sort_by_key(|e| e.1); // sort by attempts (ascending)
        vec.truncate(5);
    }

    map
}

fn save_leaderboards(leaderboards: &HashMap<Difficulty, Vec<(String, u32, u32, u64)>>) {
    let mut content = String::new();
    for (&diff, board) in leaderboards {
        if diff.is_custom() { continue; }
        let diff_name = diff.name();
        for (name, attempts, hints, secs) in board {
            content.push_str(&format!("{}|{}|{}|{}|{}\n", diff_name, name, attempts, hints, secs));
        }
    }
    let _ = fs::write("leaderboard.txt", content);
}

fn update_leaderboard(
    leaderboards: &mut HashMap<Difficulty, Vec<(String, u32, u32, u64)>>,
    difficulty: Difficulty,
    attempts: u32,
    hints_used: u32,
    elapsed_secs: u64,
) {
    let board = leaderboards.entry(difficulty).or_default();
    let max_entries = 5;

    let threshold = if board.len() < max_entries {
        u32::MAX
    } else {
        board.last().unwrap().1
    };

    if board.len() < max_entries || attempts <= threshold {
        print!("\n{} Enter your name: ",
            col(YELLOW, format!("🌟 NEW TOP-5 SCORE on {}!", difficulty.name())));
        io::stdout().flush().expect("Failed to flush stdout");

        let mut name = String::new();
        io::stdin().read_line(&mut name).expect("Failed to read name");
        let name = name.trim();
        let name: String = if name.is_empty() {
            "Anonymous".to_string()
        } else {
            name.chars().take(20).collect()
        };

        board.push((name.clone(), attempts, hints_used, elapsed_secs));
        board.sort_by_key(|e| e.1);
        board.truncate(max_entries);

        save_leaderboards(leaderboards);
        
        println!("{}", col(GREEN, format!("✅ {} has been added to the {} leaderboard!", name, difficulty.name())));
    } else {
        println!("\n👍 Solid effort! You needed {} attempts to beat the top-5 on {}.", 
            threshold, 
            difficulty.name()
        );
    }
}

fn display_leaderboards(leaderboards: &HashMap<Difficulty, Vec<(String, u32, u32, u64)>>) {
    println!("\n{}", col(BOLD, "═".repeat(60)));
    println!("{}", col(BOLD, "🏅 LEADERBOARDS – Top 5 Lowest Attempts Per Difficulty 🏅"));
    println!("{}", col(BOLD, "═".repeat(60)));
    
    for &diff in &[Difficulty::Easy, Difficulty::Medium, Difficulty::Hard, Difficulty::Insane] {
        let (_, upper) = diff.range();
        println!("\n{} {} (1–{}):", diff.emoji(), col(BOLD, diff.name()), upper);
        let board = leaderboards.get(&diff).unwrap();
        
        if board.is_empty() {
            println!("   💤 No entries yet – be the first legend!");
        } else {
            for (rank, (name, attempts, hints, secs)) in board.iter().enumerate() {
                let medal = match rank {
                    0 => "🥇",
                    1 => "🥈",
                    2 => "🥉",
                    _ => "  ",
                };
                let time_str = if *secs > 0 {
                    format!("  ⏱ {}", format_duration(*secs))
                } else {
                    String::new()
                };
                let hints_str = if *hints > 0 {
                    format!("  💡 {}h", hints)
                } else {
                    String::new()
                };
                println!(
                    "   {} {}. {:<20} – {}{}{}",
                    medal,
                    rank + 1,
                    name,
                    col(CYAN, format!("{} attempt{}", attempts, if *attempts == 1 { "" } else { "s" })),
                    time_str,
                    hints_str,
                );
            }
        }
    }
    println!("\n{}", col(BOLD, "═".repeat(60)));
}

fn ask_play_again() -> bool {
    loop {
        print!("\n🔄 Play another round? (y/n): ");
        io::stdout().flush().expect("Failed to flush stdout");

        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read line");

        match input.trim().to_lowercase().as_str() {
            "y" | "yes" => return true,
            "n" | "no" => return false,
            _ => println!("❌ Just y or n, please!"),
        }
    }
}

// ── Game Mode Selection ───────────────────────────────────────────────────────
fn ask_game_mode() -> GameMode {
    println!("{}", col(BOLD, "🎮 Select a game:"));
    println!();
    println!("  1. 🎲 Number Guessing Game – Guess the secret number with roaster commentary!");
    println!("  2. 💀 Hangman              – Guess the hidden word letter by letter!");
    println!("  3. 🟩 Wordle               – Guess the 5-letter word in 6 tries!");
    println!("  4. 💣 Iron Age Minesweeper – Clear the cursed ruins without hitting a trap!");
    println!("  5. ♟  Iron Age Checkers    – Outmanoeuvre the AI on the ancient board!");
    println!("  6. ♔  Iron Age Chess       – Face the AI on the 64-square battlefield!");
    println!("  7. ✕  Iron Age Tic Tac Toe – Outwit the AI on the classic 3×3 grid!");
    loop {
        print!("\n🎯 Your choice (1-7): ");
        io::stdout().flush().expect("Failed to flush stdout");

        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read line");

        match input.trim() {
            "1" => return GameMode::GuessingGame,
            "2" => return GameMode::Hangman,
            "3" => return GameMode::Wordle,
            "4" => return GameMode::Minesweeper,
            "5" => return GameMode::Checkers,
            "6" => return GameMode::Chess,
            "7" => return GameMode::TicTacToe,
            _   => println!("{}", col(RED, "❌ Please enter 1, 2, 3, 4, 5, 6, or 7.\n")),
        }
    }
}

// ── Hangman ───────────────────────────────────────────────────────────────────

fn hangman_art(wrong: usize) -> &'static str {
    match wrong {
        0 => "  +---+\n  |   |\n      |\n      |\n      |\n      |\n=========",
        1 => "  +---+\n  |   |\n  O   |\n      |\n      |\n      |\n=========",
        2 => "  +---+\n  |   |\n  O   |\n  |   |\n      |\n      |\n=========",
        3 => "  +---+\n  |   |\n  O   |\n /|   |\n      |\n      |\n=========",
        4 => "  +---+\n  |   |\n  O   |\n /|\\  |\n      |\n      |\n=========",
        5 => "  +---+\n  |   |\n  O   |\n /|\\  |\n /    |\n      |\n=========",
        _ => "  +---+\n  |   |\n  O   |\n /|\\  |\n / \\  |\n      |\n=========",
    }
}

/// Returns (wrong_jibes, win_message, loss_message) for the chosen roaster.
fn hangman_roaster_jibes(roaster: Roaster) -> (Vec<String>, &'static str, &'static str) {
    match roaster {
        Roaster::Ramsay => (
            vec![
                "Wrong letter, you absolute donut!",
                "Not in there! You idiot sandwich!",
                "Wrong! My dead gran could do better!",
                "That letter? Really, you muppet?!",
                "Bloody hell, that's wrong! Try harder!",
                "Not it! You useless panini head!",
                "Wrong letter – you're embarrassing yourself!",
                "That's not in the word, you plonker!",
                "Pathetic! Absolutely pathetic!",
                "Wrong! You absolute bellend!",
                "Not in there, you stupid donut!",
                "Blimey, wrong again! Are you even trying?!",
            ].into_iter().map(String::from).collect(),
            "🎯 You got the word! Finally, you donut – I'm almost impressed!",
            "💀 HANGED! That word slaughtered you! Absolutely disgraceful!",
        ),
        Roaster::UncleRoger => (
            vec![
                "Haiyaa! Wrong letter! So weak!",
                "Aiyo! Not that letter lah!",
                "Haiyaa! You guess like Jamie Oliver spell!",
                "Wrong! Emotionally damage Uncle Roger!",
                "Aiyo! That letter not even close!",
                "Haiyaa! So wrong, cannot take!",
                "Wrong letter! You make Uncle Roger cry!",
                "Aiyo! Even children spell better!",
                "Haiyaa! No MSG in your brain ah?",
                "Wrong! Uncle Roger very disappointed!",
                "Aiyah! That letter useless lah!",
            ].into_iter().map(String::from).collect(),
            "🎯 Fuiyoh! You got the word! Uncle Roger so proud! MSG approved!",
            "💀 Haiyaa! You got hanged! So embarrassing! Uncle Roger faint already!",
        ),
        Roaster::RickAstley => (
            vec![
                "Wrong! Never gonna give you the right letter!",
                "Not it! You've known the rules – try again!",
                "Never gonna let you guess wrong... oh wait, you just did!",
                "Wrong letter! A full commitment you're failing!",
                "Never gonna make you cry... but wrong!",
                "That letter's never gonna be in there!",
                "Wrong! Never gonna run around with correct guesses!",
                "You've known the rules – that letter's not it!",
                "Never gonna give that letter up? Well I am – it's wrong!",
                "Never gonna tell a lie: that's wrong!",
            ].into_iter().map(String::from).collect(),
            "🎯 Never gonna give you up – you got the word! Well played!",
            "💀 Never gonna let you win... and I didn't! You've been hanged!",
        ),
        Roaster::SimonCowell => (
            vec![
                "Wrong. Absolutely dreadful.",
                "That letter isn't there. It's a no from me.",
                "Wrong. I'm not even surprised anymore.",
                "Dreadful. That was truly dreadful.",
                "Not in the word. Terrible effort.",
                "Wrong. One of the worst guesses I've ever seen.",
                "No. Just no.",
                "That letter? Really? Awful.",
                "Wrong. I didn't like it at all.",
                "Not there. Honestly, it's embarrassing.",
                "Wrong letter. Ghastly.",
            ].into_iter().map(String::from).collect(),
            "🎯 Well done. That was actually... not bad. I'm mildly impressed.",
            "💀 Hanged. I knew you'd fail. Utterly predictable.",
        ),
        Roaster::NikkiGlaser => (
            vec![
                "Wrong letter! That's embarrassing, babe.",
                "Not in the word! Come on, seriously?",
                "Wrong! That's giving desperate energy.",
                "Babe, no. That letter isn't there.",
                "Wrong! That's so cringe.",
                "Not it! You're struggling and it's painful to watch.",
                "Wrong letter. I'm secondhand embarrassed.",
                "That letter? Really? You okay?",
                "Wrong! This is awkward for all of us.",
                "Not in there! Try harder, babe.",
                "Wrong! Fucking embarrassing.",
            ].into_iter().map(String::from).collect(),
            "🎯 Yes! You got it! See, you can do things right! Proud of you, babe!",
            "💀 Hanged! I can't even. That was rough to watch. Yikes.",
        ),
        Roaster::JoanRivers => (
            vec![
                "Wrong letter! Can we talk? That was hideous.",
                "Not in the word! Oh honey, no.",
                "Wrong! That guess looks like my last marriage – a disaster.",
                "Darling, that letter isn't there. Tragic.",
                "Wrong! Who taught you the alphabet?",
                "Not it! That was atrocious, darling.",
                "Wrong letter! I've seen better spelling from my Chihuahua.",
                "Can we talk? That's just awful.",
                "Wrong! You look ridiculous guessing that.",
                "Not in the word! Darling, it's giving disaster.",
                "Wrong! Honey, that's a crime against letters.",
            ].into_iter().map(String::from).collect(),
            "🎯 Oh darling, you got it! Fabulous! Simply divine! I'm speechless!",
            "💀 Hanged! Oh honey, what a tragedy. Tragic, just tragic.",
        ),
        Roaster::CaseOh => (
            vec![
                "CHAT! Wrong letter! This person is TROLLING!",
                "Bro, that's wrong! CHAT, spam L's!",
                "Not in the word! You're cooked bro!",
                "CHAT CHAT! Wrong again! This is embarrassing!",
                "Wrong letter! I'm stress eating over this!",
                "Bro, that letter? CHAT is cringing!",
                "WHAT?! Wrong! You're making me malding!",
                "Not it! Bro needs help! CHAT, pray for them!",
                "Wrong letter! That's an L! Ratio!",
                "CHAT! They're griefing me! Wrong again!",
                "Bro really guessed that! Wrong! L + ratio + get rekt!",
                "Not in the word! This is content I guess!",
            ].into_iter().map(String::from).collect(),
            "🎯 YOOOOO! CHAT! THEY GOT THE WORD! GG! That was actually fire bro!",
            "💀 CHAT! THEY GOT HANGED! OMFG! L + ratio + get good! Better luck next time bro!",
        ),
        Roaster::GenX => (
            vec![
                "Wrong. Whatever.",
                "Not in there. Not that I care.",
                "Wrong letter. This is lame anyway.",
                "That's not it. As if.",
                "Wrong. Gag me with a spoon.",
                "Not in the word. Whatever, try again I guess.",
                "Wrong. This is so bogus.",
                "That letter's not there. Talk to the hand.",
                "Wrong. Not that it matters.",
                "Not it. Psych!",
                "Wrong. Don't have a cow, just try again.",
            ].into_iter().map(String::from).collect(),
            "🎯 You got it. Cool, I guess. Whatever, it's fine.",
            "💀 Hanged. As if. That's what happens when you don't pay attention.",
        ),
        Roaster::Millennial => (
            vec![
                "Wrong letter bestie! That's not giving what it needs to give!",
                "OMG wrong! I'm literally dying! Not that letter!",
                "Not in the word! This is NOT the vibe!",
                "Bestie... wrong letter. I can't even.",
                "Wrong! That's so cringe! Try again!",
                "Oof, wrong! That hit different (badly).",
                "Wrong letter! Periodt! Keep trying bestie!",
                "No cap that's wrong! Come on!",
                "Not it! That's giving broke millennial energy!",
                "Wrong! I'm having an existential crisis over this!",
                "Bestie that's wrong! Slay somewhere else!",
                "Wrong letter! My anxiety cannot handle this!",
            ].into_iter().map(String::from).collect(),
            "🎯 YASSS QUEEN! You got the word! I'm SO proud! That's literally iconic! 💅",
            "💀 You got HANGED bestie! I'm literally shaking! We don't talk about this. 😭",
        ),
        Roaster::GenZ => (
            vec![
                "Wrong letter bestie! That's giving L energy fr!",
                "Nah that's wrong! No cap, try again!",
                "Not in the word! You're cooked fr fr!",
                "Bro that's mid and wrong! Come on!",
                "Wrong! That's not bussin!",
                "Low key wrong! High key embarrassing!",
                "Wrong letter! Deadass, try again!",
                "That ain't it bestie! Wrong!",
                "Not it! This ain't giving! Try harder fr!",
                "Nah bro, wrong! Periodt!",
                "Wrong! Bro fell off! Try again!",
                "That's cap! Wrong letter! Come on bestie!",
                "Wrong! You're tweaking fr!",
                "Not in the word! Ratio + L + wrong!",
            ].into_iter().map(String::from).collect(),
            "🎯 YOOO YOU ATE THAT WORD UP! No cap that was bussin! Purr bestie! 💅💀",
            "💀 YOU GOT HANGED! You are so cooked fr fr! Massive L bestie! 💀💀",
        ),
    }
}

/// Play a round of Hangman.  Returns (won, wrong_guesses, elapsed_secs).
fn play_hangman(roaster: Roaster, profane: bool) -> (bool, u32, u64) {
    let word = HANGMAN_WORDS[rand::thread_rng().gen_range(0..HANGMAN_WORDS.len())];
    let word_upper: Vec<char> = word.to_uppercase().chars().collect();
    let word_len = word_upper.len();

    let mut guessed: Vec<char> = Vec::new();
    let mut wrong_guesses = 0u32;
    let max_wrong = 6u32;
    let round_start = Instant::now();

    let (mut wrong_jibes, win_message, loss_message) = hangman_roaster_jibes(roaster);

    // Apply profanity filter to jibes
    if !profane {
        wrong_jibes = wrong_jibes
            .into_iter()
            .filter(|j| !BAD_WORDS.iter().any(|&w| j.to_lowercase().contains(w)))
            .collect();
        if wrong_jibes.is_empty() {
            wrong_jibes.push(String::from("Wrong letter! Try again."));
        }
    }

    println!("\n{} {} – Guess the {}-letter word!",
        col(BOLD, "💀 HANGMAN"),
        col(CYAN, roaster.name()),
        col(YELLOW, word_len.to_string()),
    );
    println!("You have {} wrong guesses before you're hanged!\n",
        col(RED, max_wrong.to_string()));

    loop {
        // Display current hangman art
        println!("{}", col(CYAN, hangman_art(wrong_guesses as usize)));

        // Word display
        let display: String = word_upper
            .iter()
            .map(|c| if guessed.contains(c) { c.to_string() } else { "_".to_string() })
            .collect::<Vec<_>>()
            .join(" ");
        println!("\n📝 Word: {}", col(BOLD, &display));

        // Guessed letters (sorted)
        if !guessed.is_empty() {
            let mut sorted = guessed.clone();
            sorted.sort_unstable();
            let guessed_str: String = sorted.iter()
                .map(|c| c.to_string())
                .collect::<Vec<_>>()
                .join(", ");
            println!("🔤 Letters guessed: {}", col(MAGENTA, &guessed_str));
        }

        println!("❌ Wrong guesses: {}/{}\n", wrong_guesses, max_wrong);

        // Check win condition
        if word_upper.iter().all(|c| guessed.contains(c)) {
            let elapsed = round_start.elapsed().as_secs();
            println!("{}", col(GREEN, win_message));
            println!("✅ The word was: {}\n", col(YELLOW, &word.to_uppercase()));
            return (true, wrong_guesses, elapsed);
        }

        // Check loss condition
        if wrong_guesses >= max_wrong {
            let elapsed = round_start.elapsed().as_secs();
            println!("{}", col(RED, loss_message));
            println!("💀 The word was: {}\n", col(YELLOW, &word.to_uppercase()));
            return (false, wrong_guesses, elapsed);
        }

        // Get player input
        print!("🔤 Guess a letter: ");
        io::stdout().flush().expect("Failed to flush stdout");

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");
        let trimmed = input.trim().to_uppercase();

        let valid_letter = trimmed.len() == 1
            && trimmed.chars().next().map(|c| c.is_ascii_alphabetic()).unwrap_or(false);
        if !valid_letter {
            println!("{}", col(RED, "❌ Please enter a single letter (A-Z)."));
            continue;
        }

        let letter = trimmed.chars().next().unwrap();

        if guessed.contains(&letter) {
            println!("{}", col(YELLOW, format!("⚠️  You already guessed '{}' – try a different letter.", letter)));
            continue;
        }

        guessed.push(letter);

        if word_upper.contains(&letter) {
            let count = word_upper.iter().filter(|&&c| c == letter).count();
            println!("{}", col(GREEN, format!("✅ Nice! '{}' is in the word! ({} time{})",
                letter, count, if count == 1 { "" } else { "s" })));
        } else {
            wrong_guesses += 1;
            let jibe = &wrong_jibes[rand::thread_rng().gen_range(0..wrong_jibes.len())];
            println!("{}", col(RED, format!("❌ '{}' is not in the word! {}", letter, jibe)));
            if wrong_guesses < max_wrong {
                println!("{} wrong guess{} used ({} remaining)",
                    col(YELLOW, wrong_guesses.to_string()),
                    if wrong_guesses == 1 { "" } else { "es" },
                    max_wrong - wrong_guesses,
                );
            }
        }
        println!();
    }
}

// ── Wordle ────────────────────────────────────────────────────────────────────

/// Result for a single letter position in a Wordle guess.
#[derive(PartialEq, Clone, Copy)]
enum LetterResult {
    /// Correct letter in the correct position (🟩).
    Correct,
    /// Correct letter but in the wrong position (🟨).
    Present,
    /// Letter is not in the word at all (⬛).
    Absent,
}

/// Score a 5-letter Wordle guess against the secret word.
/// Handles duplicate letters correctly (same rules as the official Wordle).
/// Score a 5-letter Wordle guess against the secret word.
/// Handles duplicate letters correctly (same rules as the official Wordle).
///
/// # Panics
/// Both `secret` and `guess` must contain uppercase ASCII alphabetic characters ('A'–'Z').
fn score_wordle_guess(secret: &[char], guess: &[char]) -> Vec<LetterResult> {
    debug_assert!(secret.iter().all(|c| c.is_ascii_uppercase()), "secret must be uppercase ASCII");
    debug_assert!(guess.iter().all(|c| c.is_ascii_uppercase()), "guess must be uppercase ASCII");

    let mut result = vec![LetterResult::Absent; 5];
    // Track how many of each letter in the secret remain unaccounted for.
    let mut remaining: [u8; 26] = [0; 26];

    // First pass: mark correct positions.
    for i in 0..5 {
        if guess[i] == secret[i] {
            result[i] = LetterResult::Correct;
        } else {
            let idx = (secret[i] as u8 - b'A') as usize;
            remaining[idx] += 1;
        }
    }

    // Second pass: mark present (wrong position) letters.
    for i in 0..5 {
        if result[i] == LetterResult::Correct {
            continue;
        }
        let idx = (guess[i] as u8 - b'A') as usize;
        if remaining[idx] > 0 {
            result[i] = LetterResult::Present;
            remaining[idx] -= 1;
        }
    }

    result
}

/// Render a single Wordle row: colored letters + emoji squares side by side.
fn render_wordle_row(guess: &[char], result: &[LetterResult]) -> String {
    let mut letters = String::new();
    let mut squares = String::new();

    for (i, (&ch, res)) in guess.iter().zip(result.iter()).enumerate() {
        if i > 0 {
            letters.push(' ');
            squares.push(' ');
        }
        let (color, square) = match res {
            LetterResult::Correct => (GREEN, "🟩"),
            LetterResult::Present => (YELLOW, "🟨"),
            LetterResult::Absent  => ("\x1b[90m", "⬛"), // dark gray
        };
        letters.push_str(&format!("{}{}{}",  color, BOLD, ch));
        letters.push_str(RESET);
        squares.push_str(square);
    }

    format!("  {}   {}", letters, squares)
}

/// Returns (guess_jibes, close_jibes, win_message, loss_message) for each roaster.
/// `guess_jibes` are delivered after a poor guess, `close_jibes` when 3+ letters match.
fn wordle_roaster_jibes(roaster: Roaster) -> (Vec<String>, Vec<String>, &'static str, &'static str) {
    match roaster {
        Roaster::Ramsay => (
            vec![
                "That's not even a real word, you donut!",
                "Wrong! Are you guessing with your eyes closed, idiot sandwich?",
                "Pathetic guess! My gran could spell better and she's been dead for years!",
                "What is THAT?! You useless plonker!",
                "WRONG! You're embarrassing yourself, you absolute muppet!",
            ].into_iter().map(String::from).collect(),
            vec![
                "Getting warmer, you donkey – keep going!",
                "So close! Don't screw it up now, you plank!",
                "You're nearly there – don't be an idiot and overthink it!",
            ].into_iter().map(String::from).collect(),
            "🟩 FINALLY! You got it right – about bloody time, you donut!",
            "🔪 Pathetic. You couldn't spell your way out of a kitchen. The word was",
        ),
        Roaster::UncleRoger => (
            vec![
                "Haiyaa! That word is so wrong lah!",
                "Why you guess like that? No brain one ah?",
                "Aiyo! Uncle Roger cover his face. So embarrassing!",
                "Haiyaa! You guess like Jamie Oliver cook rice. All wrong!",
                "Fuiyoh! That's the worst guess Uncle Roger ever see!",
            ].into_iter().map(String::from).collect(),
            vec![
                "Fuiyoh! Getting closer lah! Uncle Roger believe in you!",
                "Aiyah! So near already! Don't mess up now!",
                "Haiyaa! Almost! Think harder lah!",
            ].into_iter().map(String::from).collect(),
            "🟩 Fuiyoh! Correct lah! Uncle Roger so proud! MSG approved!",
            "🍚 Haiyaa! You fail! Uncle Roger emotionally damage. The word was",
        ),
        Roaster::RickAstley => (
            vec![
                "Never gonna give you the right word at this rate!",
                "We've known the rules and so have I – that's wrong!",
                "Never gonna run around and desert the answer, but you sure deserted logic!",
                "Inside hurts to see that guess – never gonna make you cry though!",
            ].into_iter().map(String::from).collect(),
            vec![
                "We're no strangers to progress – you're getting warmer!",
                "Never gonna give up on you – keep going!",
                "That's a commitment I can respect – so close!",
            ].into_iter().map(String::from).collect(),
            "🟩 Never gonna let you down – you got it! Knew you could do it!",
            "🎵 Never gonna say goodbye... but that was terrible. The word was",
        ),
        Roaster::SimonCowell => (
            vec![
                "That was absolutely dreadful. Genuinely terrible.",
                "I've seen better guesses from people who don't speak English.",
                "It's a no from me. Completely wrong.",
                "That guess had no thought, no structure, no logic.",
                "Honestly? That was one of the worst guesses I've ever witnessed.",
            ].into_iter().map(String::from).collect(),
            vec![
                "You're improving. Marginally. Don't get too excited.",
                "That's... actually not terrible. Keep going.",
                "Getting there. Don't ruin it now.",
            ].into_iter().map(String::from).collect(),
            "🟩 I didn't think you had it in you. But there it is. Well done.",
            "❌ You failed. Completely and utterly. The word was",
        ),
        Roaster::NikkiGlaser => (
            vec![
                "Okay that guess was giving absolutely nothing, babe.",
                "Honey, that word isn't even in the dictionary of your brain.",
                "That guess is giving 'tried my best and still flopped.'",
                "Oh sweetie, no. Just... no.",
                "That was a choice. A wrong one, but a choice.",
            ].into_iter().map(String::from).collect(),
            vec![
                "Okay you're getting closer! Don't overthink it, babe!",
                "Ooh getting warm! You've got this!",
                "Almost! The universe is rooting for you!",
            ].into_iter().map(String::from).collect(),
            "🟩 YES! You got it! That was actually impressive, not gonna lie!",
            "💅 Oof, honey. The word was",
        ),
        Roaster::JoanRivers => (
            vec![
                "Can we talk? Because that guess was a DISASTER, darling.",
                "Oh my God, who taught you to spell? A goldfish?",
                "Darling, that guess was uglier than my first facelift.",
                "That word? Really? In this economy of intelligence?",
                "I've seen better letter choices in alphabet soup, sweetheart.",
            ].into_iter().map(String::from).collect(),
            vec![
                "Getting closer, darling – don't stop now!",
                "Ooh, you're on to something! Keep going, sweetheart!",
                "Almost! I can feel it! Don't let me down!",
            ].into_iter().map(String::from).collect(),
            "🟩 You got it! Darling, I'm actually impressed – and I'm NEVER impressed!",
            "👗 Tragic. Absolutely tragic, darling. The word was",
        ),
        Roaster::CaseOh => (
            vec![
                "CHAT! CHAT! They just guessed THAT?! We are SO cooked!",
                "BRO WHAT IS THAT WORD?! My chat is going crazy rn!",
                "Oh no no no no! That's not it! Chat is losing it!",
                "KEKW that guess was TERRIBLE! Chat I can't do this!",
                "Oh my goodness gracious! That guess is AWFUL! Chat help!",
            ].into_iter().map(String::from).collect(),
            vec![
                "CHAT! CHAT! They're getting close! LETS GOOO!",
                "Oh we are SO close! Chat is going feral rn!",
                "YOOO almost! Don't mess it up! Chat believes in you!",
            ].into_iter().map(String::from).collect(),
            "🟩 YOOO THEY GOT IT! CHAT! CHAT! LETS GOOO! That was INSANE!",
            "🎮 Oh my goodness we are SO cooked. The word was",
        ),
        Roaster::GenX => (
            vec![
                "Whatever. That guess was wrong. Not surprising.",
                "Cool guess. Super wrong. Classic.",
                "Yeah no. That's not it. Obviously.",
                "Sure, guess that. See how far it gets you. Spoiler: not far.",
            ].into_iter().map(String::from).collect(),
            vec![
                "Fine, you're getting warmer. Whatever.",
                "Okay not bad I guess. Don't let it go to your head.",
                "Almost. Sure. Yeah.",
            ].into_iter().map(String::from).collect(),
            "🟩 You got it. Cool. Good for you I guess.",
            "🙄 Whatever. You ran out of guesses. The word was",
        ),
        Roaster::Millennial => (
            vec![
                "Oh no bestie, that is NOT the word! We don't do that here!",
                "That guess is giving 'I haven't slept in 3 days and ate cereal for dinner.'",
                "Oof, that's giving big 'participation trophy energy', hon.",
                "Bestie I love you but WHAT was that guess?!",
                "That is NOT the vibe we're going for right now!",
            ].into_iter().map(String::from).collect(),
            vec![
                "Okay we're getting somewhere! Very 'glow-up' energy!",
                "Yas! Getting warmer! You're literally thriving!",
                "So close bestie! You've got this, I literally believe in you!",
            ].into_iter().map(String::from).collect(),
            "🟩 YASSS BESTIE! You got it! This is literally iconic! So proud!",
            "📱 Bestie no... that was a whole journey for nothing. The word was",
        ),
        Roaster::GenZ => (
            vec![
                "No cap that guess was lowkey terrible fr fr.",
                "Bestie that is NOT it. That's giving dictionary crimes.",
                "That guess said 'I do not know how to spell' and honestly same but still.",
                "That was an absolute ratio. You got ratio'd by the dictionary.",
                "Bro that guess ate nothing. Left nothing on the plate.",
            ].into_iter().map(String::from).collect(),
            vec![
                "Okay lowkey you're kind of eating rn! Keep going!",
                "Bestie you're literally glowing up with these guesses fr!",
                "No cap that was bussin! So close tho!",
            ].into_iter().map(String::from).collect(),
            "🟩 YOOO YOU ATE THAT WORD UP! No cap bestie you SLAY! 💀🟩",
            "💀 You are so cooked fr fr. The word was",
        ),
    }
}

/// Play a round of Wordle. Returns (won, guess_count, elapsed_secs).
fn play_wordle(roaster: Roaster, profane: bool) -> (bool, u32, u64) {
    // Pick a random 5-letter word (validated at selection time).
    let valid_words: Vec<&str> = WORDLE_WORDS.iter()
        .copied()
        .filter(|w| w.len() == 5)
        .collect();
    let word = valid_words[rand::thread_rng().gen_range(0..valid_words.len())];
    let secret: Vec<char> = word.to_uppercase().chars().collect();

    let max_guesses = 6u32;
    let round_start = Instant::now();

    let (mut guess_jibes, mut close_jibes, win_message, loss_prefix) =
        wordle_roaster_jibes(roaster);

    // Apply profanity filter.
    if !profane {
        guess_jibes = guess_jibes
            .into_iter()
            .filter(|j| !BAD_WORDS.iter().any(|&w| j.to_lowercase().contains(w)))
            .collect();
        if guess_jibes.is_empty() {
            guess_jibes.push(String::from("Wrong guess! Try again."));
        }
        close_jibes = close_jibes
            .into_iter()
            .filter(|j| !BAD_WORDS.iter().any(|&w| j.to_lowercase().contains(w)))
            .collect();
        if close_jibes.is_empty() {
            close_jibes.push(String::from("Getting closer! Keep going!"));
        }
    }

    println!("\n{} {} – Guess the 5-letter word!",
        col(BOLD, "🟩 WORDLE"),
        col(CYAN, roaster.name()),
    );
    println!("You have {} attempts. Each guess must be exactly 5 letters.",
        col(YELLOW, max_guesses.to_string()));
    println!("  🟩 = correct position   🟨 = wrong position   ⬛ = not in word\n");

    // Board: stores (guess_chars, result) for each completed guess.
    let mut board: Vec<(Vec<char>, Vec<LetterResult>)> = Vec::new();

    // Track which letters have been tried and what their status is.
    let mut known: std::collections::HashMap<char, LetterResult> = std::collections::HashMap::new();

    loop {
        // Reprint the board on each turn.
        println!("{}", col(CYAN, "─".repeat(40)));
        for (i, (g, r)) in board.iter().enumerate() {
            println!("  Guess {}: {}", i + 1, render_wordle_row(g, r));
        }
        // Show empty remaining rows.
        let remaining_rows = max_guesses as usize - board.len();
        for _ in 0..remaining_rows {
            println!("  {}", col("\x1b[90m", "_ _ _ _ _   ⬛ ⬛ ⬛ ⬛ ⬛"));
        }
        println!("{}", col(CYAN, "─".repeat(40)));

        // Show keyboard hints (letters tried so far).
        if !known.is_empty() {
            let mut sorted_keys: Vec<char> = known.keys().copied().collect();
            sorted_keys.sort_unstable();
            let kb: String = sorted_keys.iter().map(|&c| {
                let color = match known[&c] {
                    LetterResult::Correct => GREEN,
                    LetterResult::Present => YELLOW,
                    LetterResult::Absent  => "\x1b[90m",
                };
                format!("{}{}{} ", color, c, RESET)
            }).collect();
            println!("🔤 Letters tried: {}", kb);
        }

        // Check win / loss before prompting next guess.
        if let Some((_, last_r)) = board.last() {
            if last_r.iter().all(|r| *r == LetterResult::Correct) {
                let elapsed = round_start.elapsed().as_secs();
                let count = board.len() as u32;
                println!("\n{}", col(GREEN, win_message));
                println!("🎉 Solved in {} guess{}!\n",
                    col(BOLD, count.to_string()),
                    if count == 1 { "" } else { "es" },
                );
                // Print share-able emoji grid.
                println!("{}", col(BOLD, "📋 Your Wordle:"));
                for (_, r) in &board {
                    let row: String = r.iter().map(|res| match res {
                        LetterResult::Correct => "🟩",
                        LetterResult::Present => "🟨",
                        LetterResult::Absent  => "⬛",
                    }).collect::<Vec<_>>().join(" ");
                    println!("  {}", row);
                }
                println!();
                return (true, count, elapsed);
            }
        }

        if board.len() as u32 >= max_guesses {
            let elapsed = round_start.elapsed().as_secs();
            println!("\n{} {}", col(RED, loss_prefix),
                col(YELLOW, &word.to_uppercase()));
            println!();
            return (false, max_guesses, elapsed);
        }

        // Prompt for the next guess.
        let guess_num = board.len() as u32 + 1;
        print!("\n🔤 Guess {}/{}: ", guess_num, max_guesses);
        io::stdout().flush().expect("Failed to flush stdout");

        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read line");
        let trimmed = input.trim().to_uppercase();

        // Validate: must be exactly 5 alphabetic characters.
        if trimmed.len() != 5 || !trimmed.chars().all(|c| c.is_ascii_alphabetic()) {
            println!("{}", col(RED, "❌ Please enter exactly 5 letters (A–Z)."));
            continue;
        }

        let guess_chars: Vec<char> = trimmed.chars().collect();
        let result = score_wordle_guess(&secret, &guess_chars);

        // Update keyboard hints – upgrade status monotonically: Absent → Present → Correct.
        for (&gc, res) in guess_chars.iter().zip(result.iter()) {
            let entry = known.entry(gc).or_insert(LetterResult::Absent);
            if *res == LetterResult::Correct {
                // Correct always wins regardless of previous state (handles Present → Correct).
                *entry = LetterResult::Correct;
            } else if *res == LetterResult::Present && *entry == LetterResult::Absent {
                *entry = LetterResult::Present;
            }
        }

        // Count how many letters are Correct or Present (a proxy for "closeness").
        let hits = result.iter().filter(|&&r| r != LetterResult::Absent).count();

        board.push((guess_chars, result));

        // Check if just won (will be handled at top of next loop iteration).
        let just_won = board.last().map(|(_, r)| r.iter().all(|x| *x == LetterResult::Correct)).unwrap_or(false);
        if just_won {
            continue;
        }

        // Roaster commentary based on closeness.
        println!();
        if hits >= 3 {
            let jibe = &close_jibes[rand::thread_rng().gen_range(0..close_jibes.len())];
            println!("{}", col(YELLOW, format!("🔥 {jibe}")));
        } else {
            let jibe = &guess_jibes[rand::thread_rng().gen_range(0..guess_jibes.len())];
            println!("{}", col(RED, format!("💬 {jibe}")));
        }

        // Extra encouragement when one guess away from losing.
        if board.len() as u32 == max_guesses - 1 {
            println!("{}", col(MAGENTA, "⚠️  Last guess! Think carefully!"));
        }
        println!();
    }
}

// ── Iron Age Tic Tac Toe ──────────────────────────────────────────────────────

/// Run a full Tic Tac Toe session (play → optional restart → return).
/// Returns `(player_won, was_drawn, player_moves_in_last_game)`.
fn play_tic_tac_toe() -> (bool, bool, u32) {
    use crossterm::{cursor, execute, terminal::{self, EnterAlternateScreen, LeaveAlternateScreen}};
    use tic_tac_toe::board::{Board, GameStatus};
    use tic_tac_toe::display::{draw, read_action, InputAction};

    let mut stdout = io::stdout();
    let _ = execute!(stdout, EnterAlternateScreen, cursor::Hide);
    let _ = terminal::enable_raw_mode();

    let mut player_won  = false;
    let mut was_drawn   = false;
    let mut player_moves = 0u32;

    'outer: loop {
        let mut board = Board::new();
        let mut cursor_row = 1usize;
        let mut cursor_col = 1usize;

        loop {
            let _ = draw(&board, cursor_row, cursor_col);

            if board.is_over() {
                player_moves = board.player_moves;
                match board.status {
                    GameStatus::PlayerWon => { player_won = true; was_drawn = false; }
                    GameStatus::Draw      => { was_drawn  = true; }
                    _                    => { was_drawn  = false; }
                }
                // Wait for restart or quit.
                loop {
                    match read_action().unwrap_or(InputAction::Quit) {
                        InputAction::Restart => continue 'outer,
                        InputAction::Quit    => break 'outer,
                        _ => {}
                    }
                }
            }

            match read_action().unwrap_or(InputAction::Quit) {
                InputAction::Move(dr, dc) => {
                    cursor_row = (cursor_row as i32 + dr).clamp(0, 2) as usize;
                    cursor_col = (cursor_col as i32 + dc).clamp(0, 2) as usize;
                }
                InputAction::Confirm => {
                    if board.player_move(cursor_row, cursor_col) && !board.is_over() {
                        // AI responds immediately.
                        board.ai_move();
                    }
                }
                InputAction::Restart => continue 'outer,
                InputAction::Quit    => break 'outer,
                InputAction::None    => {}
            }
        }
    }

    let _ = terminal::disable_raw_mode();
    let _ = execute!(io::stdout(), cursor::Show, LeaveAlternateScreen);

    (player_won, was_drawn, player_moves)
}

// ── Minesweeper ───────────────────────────────────────────────────────────────

/// Run a full minesweeper session (level-select → play → repeat until quit).
/// Returns (won_at_least_once, last_level_played, total_elapsed_secs).
fn play_minesweeper() -> (bool, minesweeper::board::Level, u64) {
    use crossterm::{cursor, execute, terminal::{self, EnterAlternateScreen, LeaveAlternateScreen}};
    use minesweeper::board::{Board, GameStatus, Level};
    use minesweeper::display::{draw, draw_level_select, read_action, read_menu_action, InputAction, MenuAction};
    use std::time::Instant;

    let mut stdout = io::stdout();

    // Enter alternate screen + raw mode for the TUI game.
    let _ = execute!(stdout, EnterAlternateScreen, cursor::Hide);
    let _ = terminal::enable_raw_mode();

    let mut last_level = Level::Peasant;
    let mut won_any = false;
    let mut total_secs = 0u64;

    'outer: loop {
        // ── Level select ────────────────────────────────────────────────────
        let levels = [Level::Peasant, Level::Knight, Level::Champion];
        let mut selected = 0usize;

        let level = loop {
            let _ = draw_level_select(selected);
            match read_menu_action().unwrap_or(MenuAction::Quit) {
                MenuAction::Up => { if selected > 0 { selected -= 1; } }
                MenuAction::Down => { if selected < levels.len() - 1 { selected += 1; } }
                MenuAction::Select(idx) => {
                    let actual = if idx == usize::MAX { selected } else { idx };
                    if actual < levels.len() {
                        break levels[actual];
                    }
                }
                MenuAction::Quit => break 'outer,
                MenuAction::None => {}
            }
        };

        last_level = level;

        // ── Play loop ───────────────────────────────────────────────────────
        let mut board = Board::new(level);
        let mut cursor_row = level.rows() / 2;
        let mut cursor_col = level.cols() / 2;
        let start = Instant::now();

        let restart = loop {
            let elapsed = start.elapsed().as_secs();
            let _ = draw(&board, cursor_row, cursor_col, elapsed);

            match read_action().unwrap_or(InputAction::Quit) {
                InputAction::Move(dr, dc) => {
                    cursor_row = (cursor_row as i32 + dr)
                        .clamp(0, (board.rows - 1) as i32) as usize;
                    cursor_col = (cursor_col as i32 + dc)
                        .clamp(0, (board.cols - 1) as i32) as usize;
                }
                InputAction::Reveal => { board.reveal(cursor_row, cursor_col); }
                InputAction::Flag   => { board.toggle_flag(cursor_row, cursor_col); }
                InputAction::Restart => break true,
                InputAction::Quit    => break false,
                InputAction::None    => {}
            }

            if board.status != GameStatus::Playing {
                let elapsed = start.elapsed().as_secs();
                total_secs += elapsed;
                if board.status == GameStatus::Won {
                    won_any = true;
                }
                // Show final state until R or Q.
                let _ = draw(&board, cursor_row, cursor_col, elapsed);
                loop {
                    match read_action().unwrap_or(InputAction::Quit) {
                        InputAction::Restart => { break; }
                        InputAction::Quit    => {
                            // Restore terminal before returning.
                            let _ = terminal::disable_raw_mode();
                            let _ = execute!(io::stdout(), cursor::Show, LeaveAlternateScreen);
                            return (won_any, last_level, total_secs);
                        }
                        _ => {}
                    }
                }
                // User pressed R – go back to level select.
                continue 'outer;
            }
        };

        if !restart {
            break 'outer;
        }
    }

    // Restore the terminal before returning to the ultra game suite prompt.
    let _ = terminal::disable_raw_mode();
    let _ = execute!(io::stdout(), cursor::Show, LeaveAlternateScreen);

    (won_any, last_level, total_secs)
}

// ── Iron Age Checkers ─────────────────────────────────────────────────────────

/// Run a full checkers session (play → optional restart → return).
/// Returns `(player_won, kings_appeared_during_game, total_elapsed_secs)`.
fn play_checkers() -> (bool, bool, u64) {
    use crossterm::{cursor, execute, terminal::{self, EnterAlternateScreen, LeaveAlternateScreen}};
    use checkers::board::{Board, GameStatus, Piece};
    use checkers::display::{draw, read_action, InputAction};
    use std::time::Instant;

    let mut stdout = io::stdout();

    let _ = execute!(stdout, EnterAlternateScreen, cursor::Hide);
    let _ = terminal::enable_raw_mode();

    let mut player_won  = false;
    let mut kings_ever  = false;
    let mut total_secs  = 0u64;

    'outer: loop {
        let mut board = Board::new();
        let mut cursor_row = 5usize;
        let mut cursor_col = 1usize;
        let start = Instant::now();

        loop {
            // Track whether any king ever appeared.
            if !kings_ever {
                'king_check: for row in &board.cells {
                    for &cell in row {
                        if matches!(cell, Piece::PlayerKing | Piece::AiKing) {
                            kings_ever = true;
                            break 'king_check;
                        }
                    }
                }
            }

            let _ = draw(&board, cursor_row, cursor_col);

            // AI's turn – apply automatically, then re-render.
            if board.status == GameStatus::Playing
                && board.turn == checkers::board::Turn::Ai
            {
                board.ai_move();
                let _ = draw(&board, cursor_row, cursor_col);
                if board.status != GameStatus::Playing {
                    total_secs += start.elapsed().as_secs();
                    if board.status == GameStatus::PlayerWon { player_won = true; }
                    // Show result until R or Q.
                    loop {
                        match read_action().unwrap_or(InputAction::Quit) {
                            InputAction::Restart => {
                                continue 'outer;
                            }
                            InputAction::Quit => break 'outer,
                            _ => {}
                        }
                    }
                }
                continue;
            }

            if board.status != GameStatus::Playing {
                total_secs += start.elapsed().as_secs();
                if board.status == GameStatus::PlayerWon { player_won = true; }
                loop {
                    match read_action().unwrap_or(InputAction::Quit) {
                        InputAction::Restart => continue 'outer,
                        InputAction::Quit    => break 'outer,
                        _ => {}
                    }
                }
            }

            // Player input.
            match read_action().unwrap_or(InputAction::Quit) {
                InputAction::Move(dr, dc) => {
                    cursor_row = (cursor_row as i32 + dr).clamp(0, 7) as usize;
                    cursor_col = (cursor_col as i32 + dc).clamp(0, 7) as usize;
                }
                InputAction::Confirm => {
                    if board.selected.is_some() {
                        // Try to move to cursor; if not a valid dest, re-select.
                        if !board.move_selected_to(cursor_row, cursor_col) {
                            board.deselect();
                            board.select(cursor_row, cursor_col);
                        }
                    } else {
                        board.select(cursor_row, cursor_col);
                    }
                }
                InputAction::Deselect => {
                    board.deselect();
                }
                InputAction::Restart => continue 'outer,
                InputAction::Quit    => break 'outer,
                InputAction::None    => {}
            }
        }
    }

    let _ = terminal::disable_raw_mode();
    let _ = execute!(io::stdout(), cursor::Show, LeaveAlternateScreen);

    (player_won, kings_ever, total_secs)
}

// ── Iron Age Chess ────────────────────────────────────────────────────────────

/// Run a full chess session (play → optional restart → return).
/// Returns `(player_won, total_move_count, ai_ever_had_queen, total_elapsed_secs)`.
fn play_chess() -> (bool, u32, bool, u64) {
    use crossterm::{cursor, execute, terminal::{self, EnterAlternateScreen, LeaveAlternateScreen}};
    use chess::board::{Board, GameStatus, PieceKind};
    use chess::board::Color as PColor;
    use chess::board::Turn;
    use chess::display::{draw, read_action, InputAction};
    use std::time::Instant;

    let mut stdout = io::stdout();

    let _ = execute!(stdout, EnterAlternateScreen, cursor::Hide);
    let _ = terminal::enable_raw_mode();

    let mut player_won    = false;
    let mut ai_had_queen  = false;
    let mut total_secs    = 0u64;
    let mut total_moves   = 0u32;

    'outer: loop {
        let mut board = Board::new();
        // Start cursor near the white pieces.
        let mut cursor_row = 6usize;
        let mut cursor_col = 4usize;
        let start = Instant::now();

        loop {
            // Track whether the AI ever had a queen.
            if !ai_had_queen {
                'qcheck: for row in &board.cells {
                    for sq in row {
                        if let Some(p) = sq {
                            if p.color == PColor::Black && p.kind == PieceKind::Queen {
                                ai_had_queen = true;
                                break 'qcheck;
                            }
                        }
                    }
                }
            }

            let _ = draw(&board, cursor_row, cursor_col);

            let is_over = matches!(
                board.status,
                GameStatus::PlayerWon | GameStatus::AiWon | GameStatus::Stalemate | GameStatus::Draw
            );

            // AI's turn – apply automatically.
            if !is_over && board.turn == Turn::Black {
                board.ai_move();
                let _ = draw(&board, cursor_row, cursor_col);

                let is_over_after = matches!(
                    board.status,
                    GameStatus::PlayerWon | GameStatus::AiWon | GameStatus::Stalemate | GameStatus::Draw
                );
                if is_over_after {
                    total_secs  += start.elapsed().as_secs();
                    total_moves += board.move_count;
                    if board.status == GameStatus::PlayerWon { player_won = true; }
                    // Wait for restart or quit.
                    'end1: loop {
                        match read_action().unwrap_or(InputAction::Quit) {
                            InputAction::Restart => continue 'outer,
                            InputAction::Quit    => break 'end1,
                            _ => {}
                        }
                    }
                    break 'outer;
                }
                continue;
            }

            if is_over {
                total_secs  += start.elapsed().as_secs();
                total_moves += board.move_count;
                if board.status == GameStatus::PlayerWon { player_won = true; }
                // Wait for restart or quit.
                'end2: loop {
                    match read_action().unwrap_or(InputAction::Quit) {
                        InputAction::Restart => continue 'outer,
                        InputAction::Quit    => break 'end2,
                        _ => {}
                    }
                }
                break 'outer;
            }

            // Player input.
            match read_action().unwrap_or(InputAction::Quit) {
                InputAction::Move(dr, dc) => {
                    cursor_row = (cursor_row as i32 + dr).clamp(0, 7) as usize;
                    cursor_col = (cursor_col as i32 + dc).clamp(0, 7) as usize;
                }
                InputAction::Confirm => {
                    if board.selected.is_some() {
                        if !board.move_selected_to(cursor_row, cursor_col) {
                            board.deselect();
                            board.select(cursor_row, cursor_col);
                        }
                    } else {
                        board.select(cursor_row, cursor_col);
                    }
                }
                InputAction::Deselect => {
                    board.deselect();
                }
                InputAction::Restart => continue 'outer,
                InputAction::Quit    => break 'outer,
                InputAction::None    => {}
            }
        }
    }

    let _ = terminal::disable_raw_mode();
    let _ = execute!(io::stdout(), cursor::Show, LeaveAlternateScreen);

    (player_won, total_moves, ai_had_queen, total_secs)
}
