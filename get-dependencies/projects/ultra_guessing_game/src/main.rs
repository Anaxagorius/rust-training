use rand::Rng;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum Roaster {
    Ramsay,
    UncleRoger,
    RickAstley,
    SimonCowell,
    NikkiGlaser,
    JoanRivers,
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
        }
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
enum Difficulty {
    Easy,
    Medium,
    Hard,
}

impl Difficulty {
    fn range(&self) -> (u32, u32) {
        match self {
            Difficulty::Easy => (1, 100),
            Difficulty::Medium => (1, 500),
            Difficulty::Hard => (1, 1000),
        }
    }

    fn name(&self) -> &'static str {
        match self {
            Difficulty::Easy => "Easy",
            Difficulty::Medium => "Medium",
            Difficulty::Hard => "Hard",
        }
    }
}

const BAD_WORDS: &[&str] = &[
    "fuck", "shit", "cunt", "bastard", "bellend", "wanker", "piss", "asshole", "dick",
];

fn main() {
    println!("🎉 Welcome to Guess the Number! 🎉");
    println!("Persistent leaderboard • Multiple difficulties • Warmth hints • Custom roaster banter\n");

    let roaster = ask_roaster();
    println!("\nYou've chosen {} as your roaster. Get ready!\n", roaster.name());

    let profane = ask_profane();
    if profane {
        println!("Profanity mode: ON – some roasters will get extra spicy. 🔞\n");
    } else {
        println!("Profanity mode: OFF – keeping it family-friendly. 😇\n");
    }

    let mut leaderboards = load_leaderboards();

    loop {
        let difficulty = ask_difficulty();
        let (attempts, guesses) = play_round(difficulty, roaster, profane);

        println!("\n🏆 You nailed it in {} attempt{}!", attempts, if attempts == 1 { "" } else { "s" });
        println!("Your guesses: {}", guesses.iter().map(|g| g.to_string()).collect::<Vec<_>>().join(", "));

        update_leaderboard(&mut leaderboards, difficulty, attempts);
        display_leaderboards(&leaderboards);

        if !ask_play_again() {
            save_leaderboards(&leaderboards);
            println!("\nCheers for playing! Leaderboard saved.");
            break;
        }
        println!();
    }
}

fn play_round(difficulty: Difficulty, roaster: Roaster, profane: bool) -> (u32, Vec<u32>) {
    let (lower, upper) = difficulty.range();
    let secret_number = rand::thread_rng().gen_range(lower..=upper);

    let mut attempts = 0u32;
    let mut guesses = Vec::new();
    let mut previous_diff: Option<u32> = None;

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
        // [Other roasters unchanged – converted to Vec<String> in the same way]
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
            ].into_iter().map(String::from).collect(),
            "🎯 Fuiyoh! Correct lah! Uncle Roger proud of you!",
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
            ].into_iter().map(String::from).collect(),
            "🎯 Well done. That was actually very good.",
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
            ].into_iter().map(String::from).collect(),
            "🎯 Yes! Finally – you got there.",
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
            ].into_iter().map(String::from).collect(),
            "🎯 Oh honey, you got it! Fabulous!",
        ),
    };

    // Profanity filter – now works cleanly on Vec<String>
    if !profane {
        low_jibes = low_jibes
            .into_iter()
            .filter(|j| !BAD_WORDS.iter().any(|&w| j.to_lowercase().contains(w)))
            .collect();
        high_jibes = high_jibes
            .into_iter()
            .filter(|j| !BAD_WORDS.iter().any(|&w| j.to_lowercase().contains(w)))
            .collect();

        // Defensive fallback – extremely unlikely to trigger with current data
        if low_jibes.is_empty() {
            low_jibes.push(String::from("Too low!"));
        }
        if high_jibes.is_empty() {
            high_jibes.push(String::from("Too high!"));
        }
    }

    loop {
        print!("Enter your guess ({}-{}): ", lower, upper);
        io::stdout().flush().expect("Failed to flush stdout");

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");

        let guess: u32 = match input.trim().parse() {
            Ok(num) => num,
            Err(_) => {
                println!("👎 That's not even a proper number. Try again.");
                continue;
            }
        };

        if guess < lower || guess > upper {
            println!("👎 Out of range – stick to {}-{}!", lower, upper);
            continue;
        }

        attempts += 1;
        guesses.push(guess);

        let current_diff = guess.abs_diff(secret_number);

        println!("Guess {}: {}", attempts, guess);
        println!(
            "Guesses so far: {}",
            guesses.iter().map(|g| g.to_string()).collect::<Vec<_>>().join(", ")
        );

        match guess.cmp(&secret_number) {
            Ordering::Less => {
                let jibe = low_jibes[rand::thread_rng().gen_range(0..low_jibes.len())].as_str();
                println!("🔥 {jibe}");
            }
            Ordering::Greater => {
                let jibe = high_jibes[rand::thread_rng().gen_range(0..high_jibes.len())].as_str();
                println!("🔥 {jibe}");
            }
            Ordering::Equal => {
                println!("{}", win_message);
                return (attempts, guesses);
            }
        }

        if let Some(prev_diff) = previous_diff {
            if current_diff < prev_diff {
                println!("🌡️  You're getting warmer!");
            } else if current_diff > prev_diff {
                println!("❄️  You're getting colder!");
            } else {
                println!("😐 Same distance – treading water?");
            }
        }

        previous_diff = Some(current_diff);
    }
}

fn ask_roaster() -> Roaster {
    loop {
        println!("Choose your roaster (they'll mock your bad guesses):");
        println!("1. Gordon Ramsay – Brutal British chef burns");
        println!("2. Uncle Roger – Haiyaa! Asian uncle cooking roasts");
        println!("3. Rick Astley – Never gonna give you up... on the puns");
        println!("4. Simon Cowell – Blunt, \"It's a no from me\"");
        println!("5. Nikki Glaser – Sharp, modern comedy roast");
        println!("6. Joan Rivers – Legendary savage fashion burns");
        print!("Your choice (1-6): ");
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
            _ => println!("👎 Please enter 1-6.\n"),
        }
    }
}

// [The rest of the functions (ask_profane, ask_difficulty, load/save_leaderboards,
// update_leaderboard, display_leaderboards, ask_play_again) remain unchanged from the previous version]

fn ask_profane() -> bool {
    loop {
        print!("Enable profanity in roasts? (y/n): ");
        io::stdout().flush().expect("Failed to flush stdout");

        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read line");

        match input.trim().to_lowercase().as_str() {
            "y" | "yes" => return true,
            "n" | "no" => return false,
            _ => println!("👎 Just y or n, please!"),
        }
    }
}

fn ask_difficulty() -> Difficulty {
    loop {
        println!("Choose your difficulty:");
        println!("1. Easy   (1–100)");
        println!("2. Medium (1–500)");
        println!("3. Hard   (1–1000)");
        print!("Your choice (1-3): ");
        io::stdout().flush().expect("Failed to flush stdout");

        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read line");

        match input.trim() {
            "1" => return Difficulty::Easy,
            "2" => return Difficulty::Medium,
            "3" => return Difficulty::Hard,
            _ => println!("👎 Please enter 1, 2, or 3.\n"),
        }
    }
}

fn load_leaderboards() -> HashMap<Difficulty, Vec<(String, u32)>> {
    // [unchanged]
    let mut map: HashMap<Difficulty, Vec<(String, u32)>> = HashMap::new();
    for diff in [Difficulty::Easy, Difficulty::Medium, Difficulty::Hard] {
        map.insert(diff, Vec::new());
    }

    if let Ok(content) = fs::read_to_string("leaderboard.txt") {
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() { continue; }
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() == 3 {
                let diff = match parts[0] {
                    "Easy" => Difficulty::Easy,
                    "Medium" => Difficulty::Medium,
                    "Hard" => Difficulty::Hard,
                    _ => continue,
                };
                let name = parts[1].to_string();
                if let Ok(attempts) = parts[2].parse::<u32>() {
                    map.entry(diff).or_insert_with(Vec::new).push((name, attempts));
                }
            }
        }
    }

    for vec in map.values_mut() {
        vec.sort_by_key(|e| e.1);
        vec.truncate(3);
    }

    map
}

fn save_leaderboards(leaderboards: &HashMap<Difficulty, Vec<(String, u32)>>) {
    // [unchanged]
    let mut content = String::new();
    for (&diff, board) in leaderboards {
        let diff_name = diff.name();
        for (name, attempts) in board {
            content.push_str(&format!("{}|{}|{}\n", diff_name, name, attempts));
        }
    }
    let _ = fs::write("leaderboard.txt", content);
}

fn update_leaderboard(
    leaderboards: &mut HashMap<Difficulty, Vec<(String, u32)>>,
    difficulty: Difficulty,
    attempts: u32,
) {
    // [unchanged]
    let board = leaderboards.entry(difficulty).or_insert_with(Vec::new);

    let threshold = if board.len() < 3 {
        u32::MAX
    } else {
        board.last().unwrap().1
    };

    if board.len() < 3 || attempts <= threshold {
        print!("\n🌟 New top-3 score on {}! Enter your name: ", difficulty.name());
        io::stdout().flush().expect("Failed to flush stdout");

        let mut name = String::new();
        io::stdin().read_line(&mut name).expect("Failed to read name");
        let name = name.trim();
        let name = if name.is_empty() { "Anonymous".to_string() } else { name.to_string() };

        board.push((name, attempts));
        board.sort_by_key(|e| e.1);
        board.truncate(3);

        save_leaderboards(leaderboards);
    } else {
        println!("\nSolid effort, but not quite top-3 material on {} this time.", difficulty.name());
    }
}

fn display_leaderboards(leaderboards: &HashMap<Difficulty, Vec<(String, u32)>>) {
    // [unchanged]
    println!("\n🏅 --- Leaderboards (Top 3 Lowest Attempts) ---");
    for &diff in &[Difficulty::Easy, Difficulty::Medium, Difficulty::Hard] {
        let (_, upper) = diff.range();
        println!("\n{} (1–{}):", diff.name(), upper);
        let board = leaderboards.get(&diff).unwrap();
        if board.is_empty() {
            println!("   No entries yet – be the first!");
        } else {
            for (rank, (name, attempts)) in board.iter().enumerate() {
                println!(
                    "   {}. {} – {} attempt{}",
                    rank + 1,
                    name,
                    attempts,
                    if *attempts == 1 { "" } else { "s" }
                );
            }
        }
    }
    println!("--------------------------------------------------\n");
}

fn ask_play_again() -> bool {
    // [unchanged]
    loop {
        print!("Play another round? (y/n): ");
        io::stdout().flush().expect("Failed to flush stdout");

        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read line");

        match input.trim().to_lowercase().as_str() {
            "y" | "yes" => return true,
            "n" | "no" => return false,
            _ => println!("👎 Just y or n, please!"),
        }
    }
}
