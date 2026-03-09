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
        }
    }
}

fn main() {
    print_banner();
    
    let roaster = ask_roaster();
    print_roaster_intro(roaster);

    let profane = ask_profane();
    if profane {
        println!("🔞 Profanity mode: {} – Brace yourself for spicy roasts.\n", col(RED, "ON"));
    } else {
        println!("😇 Profanity mode: {} – Keeping it family-friendly.\n", col(GREEN, "OFF"));
    }

    let mut leaderboards = load_leaderboards();
    let mut total_games   = 0u32;
    let mut total_attempts = 0u32;
    let mut total_secs    = 0u64;
    let mut session_achievements: Vec<Achievement> = Vec::new();

    loop {
        let difficulty = ask_difficulty();
        let (attempts, guesses, elapsed_secs, hints_used) =
            play_round(difficulty, roaster, profane);

        total_games   += 1;
        total_attempts += attempts;
        total_secs    += elapsed_secs;

        // ── Collect achievements ───────────────────────────────────────────
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
    println!("{}", col(CYAN, "=".repeat(60)));
    println!("{}", col(BOLD, "🎲  ULTRA GUESSING GAME v3.0 – Now with Achievements & Hints  🎲"));
    println!("{}", col(CYAN, "=".repeat(60)));
    println!("Features:");
    println!("  ✨ 10 unique roasters with personality");
    println!("  🏆 Persistent leaderboards across 4 difficulties");
    println!("  🌡️  Warmth hints (getting warmer/colder)");
    println!("  🔥 Optional profanity mode");
    println!("  📊 Session statistics tracking");
    println!("  💡 In-round hint system (type {} for a clue!)", col(YELLOW, "'h'"));
    println!("  🏅 Achievement system – 8 badges to unlock");
    println!("  ⏱️  Per-round timer");
    println!("  🎨 Custom difficulty – define your own number range\n");
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
