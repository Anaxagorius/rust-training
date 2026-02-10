// Cargo.toml dependencies (add these to your project):
// [dependencies]
// eframe = { version = "0.28.1", features = ["default"] }
// rand = "0.8.5"

use eframe::egui;
use eframe::egui::{ComboBox, RichText, ScrollArea, Ui};
use rand::Rng;
use std::collections::HashMap;
use std::fs;

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

#[derive(Clone, Copy, PartialEq)]
enum Roaster {
    Ramsay,
    UncleRoger,
    RickAstley,
    SimonCowell,
    NikkiGlaser,
    JoanRivers,
    AnthonyJeselnik,
    NickDiPaolo,
    AmySchumer,
    GilbertGottfried,
    NormMacdonald,
    LisaLampanelli,
    RichardPryor,
    DonRickles,
    GregGiraldo,
    JeffRoss,
}

impl Roaster {
    const ALL: [Roaster; 16] = [
        Roaster::Ramsay,
        Roaster::UncleRoger,
        Roaster::RickAstley,
        Roaster::SimonCowell,
        Roaster::NikkiGlaser,
        Roaster::JoanRivers,
        Roaster::AnthonyJeselnik,
        Roaster::NickDiPaolo,
        Roaster::AmySchumer,
        Roaster::GilbertGottfried,
        Roaster::NormMacdonald,
        Roaster::LisaLampanelli,
        Roaster::RichardPryor,
        Roaster::DonRickles,
        Roaster::GregGiraldo,
        Roaster::JeffRoss,
    ];

    fn name(&self) -> &'static str {
        match self {
            Roaster::Ramsay => "Gordon Ramsay",
            Roaster::UncleRoger => "Uncle Roger",
            Roaster::RickAstley => "Rick Astley",
            Roaster::SimonCowell => "Simon Cowell",
            Roaster::NikkiGlaser => "Nikki Glaser",
            Roaster::JoanRivers => "Joan Rivers",
            Roaster::AnthonyJeselnik => "Anthony Jeselnik",
            Roaster::NickDiPaolo => "Nick DiPaolo",
            Roaster::AmySchumer => "Amy Schumer",
            Roaster::GilbertGottfried => "Gilbert Gottfried",
            Roaster::NormMacdonald => "Norm Macdonald",
            Roaster::LisaLampanelli => "Lisa Lampanelli",
            Roaster::RichardPryor => "Richard Pryor",
            Roaster::DonRickles => "Don Rickles",
            Roaster::GregGiraldo => "Greg Giraldo",
            Roaster::JeffRoss => "Jeff Ross",
        }
    }

    fn description(&self) -> &'static str {
        match self {
            Roaster::Ramsay => "Brutal chef burns",
            Roaster::UncleRoger => "Haiyaa! Cooking roasts",
            Roaster::RickAstley => "Never gonna give you up puns",
            Roaster::SimonCowell => "\"It's a no from me\"",
            Roaster::NikkiGlaser => "Sharp modern roast",
            Roaster::JoanRivers => "Savage fashion insults",
            Roaster::AnthonyJeselnik => "Dark, twisted deadpan",
            Roaster::NickDiPaolo => "Edgy, no-filter rants",
            Roaster::AmySchumer => "Bold, self-deprecating",
            Roaster::GilbertGottfried => "Loud screechy offense",
            Roaster::NormMacdonald => "Dry, absurd wit",
            Roaster::LisaLampanelli => "Savage insult queen",
            Roaster::RichardPryor => "Raw legendary fire",
            Roaster::DonRickles => "Classic hockey puck insults",
            Roaster::GregGiraldo => "Intelligent sharp roasts",
            Roaster::JeffRoss => "The Roastmaster General",
        }
    }
}

const BAD_WORDS: &[&str] = &[
    "fuck", "shit", "cunt", "bastard", "bellend", "wanker", "piss", "asshole", "dick", "fag", "retard",
];

struct GuessApp {
    state: AppState,
    leaderboards: HashMap<Difficulty, Vec<(String, u32)>>,

    roaster: Roaster,
    profane: bool,
    low_jibes: Vec<String>,
    high_jibes: Vec<String>,
    win_message: String,
}

enum AppState {
    Startup,
    DifficultySelect,
    Playing(PlayingState),
}

struct PlayingState {
    difficulty: Difficulty,
    secret: u32,
    attempts: u32,
    guesses: Vec<u32>,
    messages: Vec<String>,
    input: String,
    previous_diff: Option<u32>,
    show_name_input: bool,
    name_buffer: String,
}

impl Default for GuessApp {
    fn default() -> Self {
        Self {
            state: AppState::Startup,
            leaderboards: HashMap::new(),
            roaster: Roaster::Ramsay,
            profane: false,
            low_jibes: vec![],
            high_jibes: vec![],
            win_message: String::new(),
        }
    }
}

enum Action {
    StartGame,
    StartRound(Difficulty),
    ProcessGuess(u32, u32),
    FinalizeEntry,
}

impl eframe::App for GuessApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(format!("Roaster: {}", self.roaster.name())).strong());
                ui.separator();
                ui.label(RichText::new(format!("Profanity: {}", if self.profane { "ON 🔞" } else { "OFF 😇" })).strong());
            });
        });

        egui::SidePanel::right("leaderboard_panel").default_width(250.0).show(ctx, |ui| {
            ui.heading(RichText::new("🏅 Leaderboards (Top 3)").strong());
            ScrollArea::vertical().show(ui, |ui: &mut Ui| {
                for &diff in &[Difficulty::Easy, Difficulty::Medium, Difficulty::Hard] {
                    let (_, upper) = diff.range();
                    ui.collapsing(format!("{} (1–{})", diff.name(), upper), |ui| {
                        let board = self.leaderboards.entry(diff).or_default();
                        if board.is_empty() {
                            ui.label(egui::RichText::new("No entries yet").italics());
                        } else {
                            for (i, (name, attempts)) in board.iter().enumerate() {
                                ui.label(format!("{}. {} – {} attempt{}", i + 1, name, attempts, if *attempts == 1 { "" } else { "s" }));
                            }
                        }
                    });
                }
            });
        });

        // Handle actions that need mutable access to self
        let mut action = None;
        
        egui::CentralPanel::default().show(ctx, |ui| {
            match &mut self.state {
                AppState::Startup => {
                    ui.vertical_centered(|ui| {
                        ui.heading(RichText::new("🎉 Guess the Number – Roast Edition 🎉").size(32.0));
                        ui.add_space(30.0);
                        ui.label(RichText::new("Select your roaster and settings, then start the game.").size(18.0));
                    });

                    ui.add_space(20.0);
                    ui.label("Roaster:");
                    ComboBox::from_id_source("roaster_combo")
                        .width(400.0)
                        .selected_text(format!("{} – {}", self.roaster.name(), self.roaster.description()))
                        .show_ui(ui, |ui: &mut Ui| {
                            for r in Roaster::ALL {
                                ui.selectable_value(&mut self.roaster, r, format!("{} – {}", r.name(), r.description()));
                            }
                        });

                    ui.add_space(15.0);
                    ui.horizontal(|ui| {
                        ui.label("Profanity mode:");
                        ui.checkbox(&mut self.profane, "Enable 🔞");
                    });

                    ui.add_space(40.0);
                    ui.vertical_centered(|ui| {
                        if ui.button(RichText::new("Start Game").size(24.0)).clicked() {
                            action = Some(Action::StartGame);
                        }
                    });
                }
                AppState::DifficultySelect => {
                    ui.vertical_centered(|ui| {
                        ui.heading(RichText::new("Choose Difficulty").size(28.0));
                        ui.add_space(40.0);
                        if ui.button(RichText::new("Easy (1–100)").size(20.0)).clicked() {
                            action = Some(Action::StartRound(Difficulty::Easy));
                        }
                        ui.add_space(10.0);
                        if ui.button(RichText::new("Medium (1–500)").size(20.0)).clicked() {
                            action = Some(Action::StartRound(Difficulty::Medium));
                        }
                        ui.add_space(10.0);
                        if ui.button(RichText::new("Hard (1–1000)").size(20.0)).clicked() {
                            action = Some(Action::StartRound(Difficulty::Hard));
                        }
                    });
                }
                AppState::Playing(ps) => {
                    let (lower, upper) = ps.difficulty.range();
                    ui.heading(RichText::new(format!("Guess the number ({lower}–{upper})")).size(26.0));
                    ui.label(format!("Attempts: {}", ps.attempts));

                    ui.add_space(15.0);
                    ui.horizontal(|ui| {
                        ui.label("Your guess:");
                        let response = ui.text_edit_singleline(&mut ps.input);
                        if ui.button("Submit").clicked() || (response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))) {
                            if !ps.input.trim().is_empty() {
                                action = Some(Action::ProcessGuess(lower, upper));
                            }
                        }
                    });

                    ui.add_space(15.0);
                    ui.separator();
                    ui.add_space(10.0);

                    ScrollArea::vertical()
                        .id_source("messages_scroll")
                        .auto_shrink([false, true])
                        .stick_to_bottom(true)
                        .show(ui, |ui: &mut Ui| {
                            for msg in &ps.messages {
                                ui.label(msg);
                            }
                        });

                    if ps.show_name_input {
                        ui.add_space(15.0);
                        ui.separator();
                        ui.add_space(10.0);
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("🌟 Top-3 score! Enter name:").strong());
                            ui.text_edit_singleline(&mut ps.name_buffer);
                            if ui.button("Submit").clicked() {
                                action = Some(Action::FinalizeEntry);
                            }
                        });
                    }

                    ui.add_space(30.0);
                    ui.horizontal(|ui| {
                        if ui.button("New Round").clicked() {
                            self.state = AppState::DifficultySelect;
                        }
                        if ui.button("Change Roaster/Settings").clicked() {
                            self.state = AppState::Startup;
                        }
                    });
                }
            }
        });
        
        // Execute actions after UI is done
        if let Some(action) = action {
            match action {
                Action::StartGame => {
                    let (mut low, mut high, win) = load_jibes(self.roaster);
                    if !self.profane {
                        low.retain(|j| !BAD_WORDS.iter().any(|&w| j.to_lowercase().contains(w)));
                        high.retain(|j| !BAD_WORDS.iter().any(|&w| j.to_lowercase().contains(w)));
                        if low.is_empty() { low.push("Too low!".to_string()); }
                        if high.is_empty() { high.push("Too high!".to_string()); }
                    }
                    self.low_jibes = low;
                    self.high_jibes = high;
                    self.win_message = win.to_string();
                    self.leaderboards = load_leaderboards();
                    self.state = AppState::DifficultySelect;
                }
                Action::StartRound(difficulty) => {
                    self.start_new_round(difficulty);
                }
                Action::ProcessGuess(lower, upper) => {
                    // Extract the data we need before calling the method
                    let mut temp_state = std::mem::replace(&mut self.state, AppState::Startup);
                    if let AppState::Playing(ref mut ps) = temp_state {
                        self.process_guess(ps, lower, upper);
                    }
                    self.state = temp_state;
                }
                Action::FinalizeEntry => {
                    // Extract the data we need before calling the method
                    let mut temp_state = std::mem::replace(&mut self.state, AppState::Startup);
                    if let AppState::Playing(ref mut ps) = temp_state {
                        self.finalize_leaderboard_entry(ps);
                    }
                    self.state = temp_state;
                }
            }
        }
    }
}

impl GuessApp {
    fn start_new_round(&mut self, difficulty: Difficulty) {
        let (lower, upper) = difficulty.range();
        let secret = rand::thread_rng().gen_range(lower..=upper);
        self.state = AppState::Playing(PlayingState {
            difficulty,
            secret,
            attempts: 0,
            guesses: vec![],
            messages: vec!["New round started! Good luck... you'll need it.".to_string()],
            input: String::new(),
            previous_diff: None,
            show_name_input: false,
            name_buffer: String::new(),
        });
    }

    fn process_guess(&mut self, ps: &mut PlayingState, lower: u32, upper: u32) {
        let guess_str = ps.input.trim().to_string();
        ps.input.clear();

        if let Ok(guess) = guess_str.parse::<u32>() {
            if guess < lower || guess > upper {
                ps.messages.push(format!("👎 Out of range – must be {lower}–{upper}!"));
                return;
            }

            ps.attempts += 1;
            ps.guesses.push(guess);

            let current_diff = guess.abs_diff(ps.secret);
            ps.messages.push(format!("Guess {}: {}", ps.attempts, guess));

            if guess < ps.secret {
                let jibe = self.low_jibes[rand::thread_rng().gen_range(0..self.low_jibes.len())].clone();
                ps.messages.push(format!("🔥 {jibe}"));
            } else if guess > ps.secret {
                let jibe = self.high_jibes[rand::thread_rng().gen_range(0..self.high_jibes.len())].clone();
                ps.messages.push(format!("🔥 {jibe}"));
            } else {
                ps.messages.push(self.win_message.clone());
                ps.messages.push(format!("🏆 You win in {} attempt{}!", ps.attempts, if ps.attempts == 1 { "" } else { "s" }));
                ps.messages.push(format!("Your guesses: {}", ps.guesses.iter().map(|g| g.to_string()).collect::<Vec<_>>().join(", ")));
                self.handle_win(ps);
                return;
            }

            if let Some(prev) = ps.previous_diff {
                if current_diff < prev {
                    ps.messages.push("🌡️ You're getting warmer!".to_string());
                } else if current_diff > prev {
                    ps.messages.push("❄️ You're getting colder!".to_string());
                } else {
                    ps.messages.push("😐 Same distance – treading water?".to_string());
                }
            }
            ps.previous_diff = Some(current_diff);
        } else {
            ps.messages.push("👎 Not a valid number!".to_string());
        }
    }

    fn handle_win(&mut self, ps: &mut PlayingState) {
        let board = self.leaderboards.entry(ps.difficulty).or_default();
        let threshold = if board.len() < 3 { u32::MAX } else { board.last().unwrap().1 };

        if board.len() < 3 || ps.attempts <= threshold {
            ps.show_name_input = true;
        } else {
            ps.messages.push(format!("Solid effort, but not top-3 material on {}!", ps.difficulty.name()));
        }
    }

    fn finalize_leaderboard_entry(&mut self, ps: &mut PlayingState) {
        let name = if ps.name_buffer.trim().is_empty() {
            "Anonymous".to_string()
        } else {
            ps.name_buffer.trim().to_string()
        };
        let board = self.leaderboards.entry(ps.difficulty).or_default();
        board.push((name, ps.attempts));
        board.sort_by_key(|e| e.1);
        board.truncate(3);
        save_leaderboards(&self.leaderboards);
        ps.show_name_input = false;
        ps.messages.push("🏅 Leaderboard updated!".to_string());
    }
}

fn load_jibes(roaster: Roaster) -> (Vec<String>, Vec<String>, &'static str) {
    match roaster {
        Roaster::Ramsay => (
            vec![
                "Too small! You absolute donkey!".to_string(),
                "What are you—an idiot sandwich guessing low?".to_string(),
                "Too small! My gran could do better, and she's dead!".to_string(),
                "Hey, panini head, wake up and guess higher!".to_string(),
                "Too low! You stupid donut!".to_string(),
                "That's so low it's raw—still mooing!".to_string(),
                "Too small! You fucking idiot—aim up!".to_string(),
                "Blimey, that's colder than my freezer. Try harder!".to_string(),
                "Too low! You muppet, you're embarrassing yourself!".to_string(),
                "You wanker—guess higher!".to_string(),
                "Piss off with that low rubbish!".to_string(),
            ],
            vec![
                "Too high! You overcooked donkey!".to_string(),
                "Lower! It's so high it's burnt to a crisp!".to_string(),
                "Too big! Greedy panini head!".to_string(),
                "Reel it in, you absolute plank!".to_string(),
                "That's so high it's fucking charred!".to_string(),
                "Way too big! Piss off with that guess!".to_string(),
                "Lower! You donut, you're scorching everything!".to_string(),
                "You absolute bastard—lower!".to_string(),
            ],
            "🎯 Bang on! Finally – about bloody time!",
        ),
        Roaster::UncleRoger => (
            vec![
                "Haiyaa! Too low lah! So weak!".to_string(),
                "Why you guess so low? No strength at all!".to_string(),
                "Aiyah! Too small – you fry rice like this ah?".to_string(),
                "Too low! Emotionally damage my wok!".to_string(),
                "Haiyaa! Guess higher lah, don't be so sad!".to_string(),
                "So low... you put no MSG in your guess?".to_string(),
                "Aiyo! Too low – children guess better!".to_string(),
                "Why so weak? Lift your guess higher!".to_string(),
                "Haiyaa! This guess no flavor – too low!".to_string(),
                "Too small lah! Uncle Roger disappointed!".to_string(),
            ],
            vec![
                "Fuiyoh! Too high lah! Overcook already!".to_string(),
                "Aiyah! Too big – you put too much MSG!".to_string(),
                "Haiyaa! Way too high – wok on fire!".to_string(),
                "Too high! You deep fry until burnt ah?".to_string(),
                "Fuiyoh! Reel it in – too much oil!".to_string(),
                "So high... you make Uncle Roger scream!".to_string(),
                "Aiyo! Too big – lower lah, don't be crazy!".to_string(),
                "Haiyaa! This guess over-seasoned!".to_string(),
            ],
            "🎯 Fuiyoh! Correct lah! Uncle Roger proud of you!",
        ),
        Roaster::RickAstley => (
            vec![
                "Too low! But I'm never gonna let you down... so guess higher!".to_string(),
                "Never gonna give you up... but that guess is too small!".to_string(),
                "We're no strangers to bad guesses – aim up!".to_string(),
                "Too low! Never gonna run around and desert the right number!".to_string(),
                "Never gonna make you cry... unless you keep guessing low!".to_string(),
                "A full commitment's what I'm thinking of – guess higher!".to_string(),
            ],
            vec![
                "Too high! Never gonna run around with big numbers!".to_string(),
                "That guess is too big – never gonna give you up!".to_string(),
                "Never gonna let you down... by guessing lower!".to_string(),
                "Too high! You've got to make me understand – reel it in!".to_string(),
                "Guess lower – never gonna give this roast up!".to_string(),
            ],
            "🎯 Never gonna give you up... you finally got it! Well played!",
        ),
        Roaster::SimonCowell => (
            vec![
                "Too low. That was absolutely dreadful.".to_string(),
                "It's a no from me – guess higher.".to_string(),
                "Honestly, that guess was terrible.".to_string(),
                "Far too low. I didn't like it at all.".to_string(),
                "Dreadful. Absolutely dreadful.".to_string(),
                "Too low! Not good enough, I'm afraid.".to_string(),
            ],
            vec![
                "Too high. Over the top.".to_string(),
                "It's a no from me – reel it in.".to_string(),
                "Way too high. Honestly, terrible.".to_string(),
                "Too big! I didn't like it.".to_string(),
            ],
            "🎯 Well done. That was actually very good.",
        ),
        Roaster::NikkiGlaser => (
            vec![
                "Too low – that's disappointing.".to_string(),
                "Too small! Come on, aim higher.".to_string(),
                "Too low! You're undershooting, babe.".to_string(),
                "Too low – fucking embarrassing.".to_string(),
                "Guess higher, you idiot.".to_string(),
                "That's so low it's pathetic.".to_string(),
                "Too low! What the fuck?".to_string(),
            ],
            vec![
                "Too high – greedy much?".to_string(),
                "Way too big! Reel it in.".to_string(),
                "Too high! Calm down.".to_string(),
                "Too big! Fucking relax.".to_string(),
                "Lower! Jesus Christ.".to_string(),
            ],
            "🎯 Yes! Finally – you got there.",
        ),
        Roaster::JoanRivers => (
            vec![
                "Too low! Can we talk? That guess is hideous.".to_string(),
                "Oh honey, too low – that's tragic.".to_string(),
                "Too small, darling – it fell off the ugly tree.".to_string(),
                "Guess higher! That was atrocious.".to_string(),
                "Can we talk? Too fucking low.".to_string(),
            ],
            vec![
                "Too high! That's overdone, darling.".to_string(),
                "Way too big – calm down.".to_string(),
                "Too high! It looks ridiculous.".to_string(),
                "Lower! Fucking terrible.".to_string(),
            ],
            "🎯 Oh honey, you got it! Fabulous!",
        ),
        Roaster::AnthonyJeselnik => (
            vec![
                "Too low. Your guess has the life expectancy of a Jeselnik relationship.".to_string(),
                "Too low – that's the number of people who care.".to_string(),
                "Too low. I was hoping for tragedy, but this is just sad.".to_string(),
                "Too low! Like the bar I set for humanity.".to_string(),
                "Too low. Dead wrong – my favorite kind.".to_string(),
                "Too low. That's almost as disappointing as finding out your parents tried.".to_string(),
            ],
            vec![
                "Too high. Optimism is cute – on other people.".to_string(),
                "Too high! Reaching for something you'll never touch.".to_string(),
                "Too high. That's the highest you'll ever get.".to_string(),
                "Too high – classic overconfidence before the fall.".to_string(),
                "Too high. Your guess peaked too early.".to_string(),
            ],
            "🎯 You got it. The twist? You still lose at life.",
        ),
        Roaster::NickDiPaolo => (
            vec![
                "Too low, you moron!".to_string(),
                "Too low! What are you, stupid?".to_string(),
                "Too low – that's weak sauce.".to_string(),
                "Too low, you fuckin' idiot!".to_string(),
                "Too low! Grow a pair and guess higher!".to_string(),
                "Too low – embarrassing.".to_string(),
            ],
            vec![
                "Too high! Calm the fuck down!".to_string(),
                "Too high, you greedy bastard!".to_string(),
                "Too high – reel it in, jackass!".to_string(),
                "Too high! You're killing me here.".to_string(),
                "Too high, dumbass!".to_string(),
            ],
            "🎯 You got it – miracles do happen.",
        ),
        Roaster::AmySchumer => (
            vec![
                "Too low – story of my life.".to_string(),
                "Too low! I would've guessed higher, but I'm not good at this either.".to_string(),
                "Too low – that's like my standards on a Tuesday.".to_string(),
                "Too low, you basic bitch.".to_string(),
                "Too low! Even I steal better than that.".to_string(),
                "Too low – that's sad, babe.".to_string(),
            ],
            vec![
                "Too high – overcompensating much?".to_string(),
                "Too high! Greedy, aren't we?".to_string(),
                "Too high – calm your tits.".to_string(),
                "Too high, you try-hard.".to_string(),
                "Too high – that's ambitious for you.".to_string(),
            ],
            "🎯 You win! I did not see that coming.",
        ),
        Roaster::GilbertGottfried => (
            vec![
                "TOO LOW!!! AHHHH!!!".to_string(),
                "TOO LOW, YOU IDIOT!!!".to_string(),
                "TOO LOOOOOOW!!! SCREECH!!!".to_string(),
                "TOO LOW!!! WHAT IS WRONG WITH YOU?!!".to_string(),
                "TOO LOW!!! DISGUSTING!!!".to_string(),
            ],
            vec![
                "TOO HIGH!!! AHHHHH!!!".to_string(),
                "TOO HIIIIIIIGH!!! MORON!!!".to_string(),
                "TOO HIGH!!! STOP IT!!!".to_string(),
                "TOO HIGH!!! YOU'RE KILLING ME!!!".to_string(),
            ],
            "🎯 YOU GOT IT!!! FINALLY!!! AHHHH!!!",
        ),
        Roaster::NormMacdonald => (
            vec![
                "Too low. You know, back in the old country...".to_string(),
                "Too low. Or so the Germans would have us believe.".to_string(),
                "Too low. That's the old joke.".to_string(),
                "Too low, folks. Real low.".to_string(),
                "Too low. I once knew a guy who guessed too low... he died.".to_string(),
            ],
            vec![
                "Too high. Now that's just silly.".to_string(),
                "Too high. You're funnier than you look.".to_string(),
                "Too high. That's what she said... wait, no.".to_string(),
                "Too high, my friend. Way too high.".to_string(),
            ],
            "🎯 You got it. Well, I'll be a son of a gun.",
        ),
        Roaster::LisaLampanelli => (
            vec![
                "Too low, you fat fuck!".to_string(),
                "Too low – that's pathetic, sweetheart.".to_string(),
                "Too low, you disgusting pig!".to_string(),
                "Too low! Go eat a sandwich and guess higher!".to_string(),
                "Too low, you worthless piece of shit!".to_string(),
            ],
            vec![
                "Too high, you greedy bastard!".to_string(),
                "Too high – tone it down, asshole!".to_string(),
                "Too high, you cocky prick!".to_string(),
                "Too high! Calm your tits!".to_string(),
            ],
            "🎯 You got it, you magnificent bastard!",
        ),
        Roaster::RichardPryor => (
            vec![
                "Too low, motherfucker!".to_string(),
                "Too low! Shit, that's cold.".to_string(),
                "Too low – you jivin' me?".to_string(),
                "Too low, damn!".to_string(),
                "Too low! That's some weak-ass guessing.".to_string(),
            ],
            vec![
                "Too high, nigga!".to_string(),
                "Too high! Slow your roll!".to_string(),
                "Too high – you trippin'!".to_string(),
                "Too high, goddamn!".to_string(),
            ],
            "🎯 You got it! Hot damn!",
        ),
        Roaster::DonRickles => (
            vec![
                "Too low, you hockey puck!".to_string(),
                "Too low, you dummy!".to_string(),
                "Too low, you bum!".to_string(),
                "Too low, you big shot!".to_string(),
                "Too low, you moron!".to_string(),
                "Too low, you schmuck!".to_string(),
            ],
            vec![
                "Too high, you hockey puck!".to_string(),
                "Too high, you dummy!".to_string(),
                "Too high, you stiff!".to_string(),
                "Too high, you palooka!".to_string(),
            ],
            "🎯 You got it, you hockey puck! Nice work.",
        ),
        Roaster::GregGiraldo => (
            vec![
                "Too low. That's just sad.".to_string(),
                "Too low – you're killing me here.".to_string(),
                "Too low! What a waste of talent.".to_string(),
                "Too low. You're better than this.".to_string(),
                "Too low – embarrassing.".to_string(),
            ],
            vec![
                "Too high. Greedy much?".to_string(),
                "Too high! Overreaching as usual.".to_string(),
                "Too high – dial it back.".to_string(),
                "Too high. You're not that good.".to_string(),
            ],
            "🎯 You got it. Not bad, not bad.",
        ),
        Roaster::JeffRoss => (
            vec![
                "Too low! You look like a low guess feels.".to_string(),
                "Too low – you're bombing harder than usual.".to_string(),
                "Too low! Even your haircut guessed higher.".to_string(),
                "Too low, you fat fuck!".to_string(),
                "Too low – ugly and wrong.".to_string(),
            ],
            vec![
                "Too high! Reel it in, fatty!".to_string(),
                "Too high – overcompensating again?".to_string(),
                "Too high! Your ego guessed that.".to_string(),
                "Too high – calm down, loser.".to_string(),
            ],
            "🎯 You got it! The Roastmaster is impressed... slightly.",
        ),
    }
}

fn load_leaderboards() -> HashMap<Difficulty, Vec<(String, u32)>> {
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
                    map.entry(diff).or_default().push((name, attempts));
                }
            }
        }
    }

    for board in map.values_mut() {
        board.sort_by_key(|e| e.1);
        board.truncate(3);
    }

    map
}

fn save_leaderboards(leaderboards: &HashMap<Difficulty, Vec<(String, u32)>>) {
    let mut content = String::new();
    for (&diff, board) in leaderboards {
        for (name, attempts) in board {
            content.push_str(&format!("{}|{}|{}\n", diff.name(), name, attempts));
        }
    }
    let _ = fs::write("leaderboard.txt", content);
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([900.0, 700.0])
            .with_resizable(true),
        ..Default::default()
    };

    eframe::run_native(
        "Guess the Number – Roast Edition",
        options,
        Box::new(|_cc| Ok(Box::new(GuessApp::default()))),
    )
}