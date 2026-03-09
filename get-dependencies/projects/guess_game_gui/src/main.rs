// Cargo.toml dependencies (add these to your project):
// [dependencies]
// eframe = { version = "0.28.1", features = ["default"] }
// rand = "0.8.5"

use eframe::egui::{self, Color32, ComboBox, RichText, ScrollArea, Ui};
use rand::Rng;
use std::collections::HashMap;
use std::fs;

// ─── Difficulty ─────────────────────────────────────────────────────────────

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
enum Difficulty {
    Easy,
    Medium,
    Hard,
    Insane,
}

impl Difficulty {
    const ALL: [Difficulty; 4] = [
        Difficulty::Easy,
        Difficulty::Medium,
        Difficulty::Hard,
        Difficulty::Insane,
    ];

    fn range(&self) -> (u32, u32) {
        match self {
            Difficulty::Easy => (1, 100),
            Difficulty::Medium => (1, 500),
            Difficulty::Hard => (1, 1_000),
            Difficulty::Insane => (1, 10_000),
        }
    }

    fn name(&self) -> &'static str {
        match self {
            Difficulty::Easy => "Easy",
            Difficulty::Medium => "Medium",
            Difficulty::Hard => "Hard",
            Difficulty::Insane => "Insane",
        }
    }

    fn emoji(&self) -> &'static str {
        match self {
            Difficulty::Easy => "😊",
            Difficulty::Medium => "😤",
            Difficulty::Hard => "💀",
            Difficulty::Insane => "👹",
        }
    }

    fn color(&self) -> Color32 {
        match self {
            Difficulty::Easy => Color32::from_rgb(80, 200, 120),
            Difficulty::Medium => Color32::from_rgb(255, 190, 50),
            Difficulty::Hard => Color32::from_rgb(230, 80, 80),
            Difficulty::Insane => Color32::from_rgb(180, 60, 220),
        }
    }

    fn description(&self) -> &'static str {
        match self {
            Difficulty::Easy => "1–100 · Perfect for beginners",
            Difficulty::Medium => "1–500 · A fair challenge",
            Difficulty::Hard => "1–1,000 · For the brave",
            Difficulty::Insane => "1–10,000 · Are you psychic?",
        }
    }
}

// ─── Roaster ────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq)]
enum Roaster {
    // Classic roasters
    Ramsay,
    UncleRoger,
    RickAstley,
    SimonCowell,
    NikkiGlaser,
    JoanRivers,
    // Stand-up legends
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
    // Internet / Gen culture (from ultra_guessing_game)
    CaseOh,
    GenX,
    Millennial,
    GenZ,
}

impl Roaster {
    const ALL: [Roaster; 20] = [
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
        Roaster::CaseOh,
        Roaster::GenX,
        Roaster::Millennial,
        Roaster::GenZ,
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
            Roaster::CaseOh => "CaseOh",
            Roaster::GenX => "Gen X Teen",
            Roaster::Millennial => "Millennial",
            Roaster::GenZ => "Gen Z",
        }
    }

    fn emoji(&self) -> &'static str {
        match self {
            Roaster::Ramsay => "🔪",
            Roaster::UncleRoger => "🍚",
            Roaster::RickAstley => "🎵",
            Roaster::SimonCowell => "❌",
            Roaster::NikkiGlaser => "💅",
            Roaster::JoanRivers => "👗",
            Roaster::AnthonyJeselnik => "☠️",
            Roaster::NickDiPaolo => "🤬",
            Roaster::AmySchumer => "🍷",
            Roaster::GilbertGottfried => "📢",
            Roaster::NormMacdonald => "🃏",
            Roaster::LisaLampanelli => "👑",
            Roaster::RichardPryor => "🔥",
            Roaster::DonRickles => "🏒",
            Roaster::GregGiraldo => "🎤",
            Roaster::JeffRoss => "🏆",
            Roaster::CaseOh => "🎮",
            Roaster::GenX => "🙄",
            Roaster::Millennial => "📱",
            Roaster::GenZ => "💀",
        }
    }

    fn description(&self) -> &'static str {
        match self {
            Roaster::Ramsay => "Brutal British chef burns 🔪",
            Roaster::UncleRoger => "Haiyaa! Asian uncle cooking roasts 🍚",
            Roaster::RickAstley => "Never gonna give you up… on the puns 🎵",
            Roaster::SimonCowell => "Blunt, \"It's a no from me\" ❌",
            Roaster::NikkiGlaser => "Sharp, modern comedy roast 💅",
            Roaster::JoanRivers => "Legendary savage fashion burns 👗",
            Roaster::AnthonyJeselnik => "Dark, twisted deadpan ☠️",
            Roaster::NickDiPaolo => "Edgy, no-filter rants 🤬",
            Roaster::AmySchumer => "Bold, self-deprecating 🍷",
            Roaster::GilbertGottfried => "Loud screechy offense 📢",
            Roaster::NormMacdonald => "Dry, absurd wit 🃏",
            Roaster::LisaLampanelli => "Savage insult queen 👑",
            Roaster::RichardPryor => "Raw legendary fire 🔥",
            Roaster::DonRickles => "Classic hockey puck insults 🏒",
            Roaster::GregGiraldo => "Intelligent sharp roasts 🎤",
            Roaster::JeffRoss => "The Roastmaster General 🏆",
            Roaster::CaseOh => "Chaotic YouTube energy & food trauma 🎮",
            Roaster::GenX => "Whatever, this is lame anyway 🙄",
            Roaster::Millennial => "Yas queen, but also anxious & broke 📱",
            Roaster::GenZ => "No cap, this slaps fr fr 💀",
        }
    }

    fn intro(&self) -> &'static str {
        match self {
            Roaster::Ramsay => "🔪 Gordon Ramsay: \"Right, you donut. Let's see if you can count!\"",
            Roaster::UncleRoger => "🍚 Uncle Roger: \"Haiyaa! You better not disappoint Uncle Roger!\"",
            Roaster::RickAstley => "🎵 Rick Astley: \"Never gonna give you up on this game!\"",
            Roaster::SimonCowell => "❌ Simon Cowell: \"Let's see if you're any good at this.\"",
            Roaster::NikkiGlaser => "💅 Nikki Glaser: \"Oh honey, this should be interesting…\"",
            Roaster::JoanRivers => "👗 Joan Rivers: \"Can we talk? Let's see those guessing skills!\"",
            Roaster::AnthonyJeselnik => "☠️ Jeselnik: \"I've seen tragedy. This might qualify.\"",
            Roaster::NickDiPaolo => "🤬 Nick DiPaolo: \"Alright genius, try not to embarrass yourself.\"",
            Roaster::AmySchumer => "🍷 Amy Schumer: \"Let's see… I've done worse things than this game.\"",
            Roaster::GilbertGottfried => "📢 Gilbert: \"I AM GILBERT GOTTFRIED AND YOU WILL GUESS NOW!!!\"",
            Roaster::NormMacdonald => "🃏 Norm: \"Well. Here we are. I once played this game. I lost.\"",
            Roaster::LisaLampanelli => "👑 Lisa: \"Alright sweetheart, let's see what you've got!\"",
            Roaster::RichardPryor => "🔥 Richard Pryor: \"I'm watchin' you, and I ain't impressed yet.\"",
            Roaster::DonRickles => "🏒 Don Rickles: \"You hockey puck – let's see you guess right!\"",
            Roaster::GregGiraldo => "🎤 Greg Giraldo: \"Okay smart guy, here's your chance to shine.\"",
            Roaster::JeffRoss => "🏆 Jeff Ross: \"The Roastmaster is watching. Don't bomb.\"",
            Roaster::CaseOh => "🎮 CaseOh: \"CHAT! CHAT! Watch me destroy this person at guessing!\"",
            Roaster::GenX => "🙄 Gen X: \"Whatever, this is probably rigged anyway.\"",
            Roaster::Millennial => "📱 Millennial: \"OMG this is giving early 2000s vibes! Let's do this!\"",
            Roaster::GenZ => "💀 Gen Z: \"Bestie, this about to be a whole vibe, no cap.\"",
        }
    }

    /// Accent color shown next to roaster messages.
    fn color(&self) -> Color32 {
        match self {
            Roaster::Ramsay => Color32::from_rgb(220, 50, 50),
            Roaster::UncleRoger => Color32::from_rgb(255, 200, 0),
            Roaster::RickAstley => Color32::from_rgb(80, 160, 230),
            Roaster::SimonCowell => Color32::from_rgb(180, 180, 180),
            Roaster::NikkiGlaser => Color32::from_rgb(240, 120, 200),
            Roaster::JoanRivers => Color32::from_rgb(200, 100, 220),
            Roaster::AnthonyJeselnik => Color32::from_rgb(80, 80, 80),
            Roaster::NickDiPaolo => Color32::from_rgb(200, 80, 40),
            Roaster::AmySchumer => Color32::from_rgb(220, 170, 100),
            Roaster::GilbertGottfried => Color32::from_rgb(255, 100, 0),
            Roaster::NormMacdonald => Color32::from_rgb(150, 150, 200),
            Roaster::LisaLampanelli => Color32::from_rgb(230, 60, 130),
            Roaster::RichardPryor => Color32::from_rgb(255, 140, 0),
            Roaster::DonRickles => Color32::from_rgb(100, 180, 100),
            Roaster::GregGiraldo => Color32::from_rgb(100, 200, 220),
            Roaster::JeffRoss => Color32::from_rgb(220, 180, 0),
            Roaster::CaseOh => Color32::from_rgb(140, 100, 220),
            Roaster::GenX => Color32::from_rgb(160, 160, 160),
            Roaster::Millennial => Color32::from_rgb(100, 200, 180),
            Roaster::GenZ => Color32::from_rgb(180, 80, 255),
        }
    }
}

// ─── Profanity filter ────────────────────────────────────────────────────────

const BAD_WORDS: &[&str] = &[
    "fuck", "shit", "cunt", "bastard", "bellend", "wanker", "piss", "asshole",
    "dick", "fag", "retard", "nigga", "motherfucker",
];

// ─── Message system ──────────────────────────────────────────────────────────

/// A single chat-style message with its display color.
struct Message {
    text: String,
    color: Color32,
}

impl Message {
    fn new(text: impl Into<String>, color: Color32) -> Self {
        Self { text: text.into(), color }
    }

    fn system(text: impl Into<String>) -> Self {
        Self::new(text, Color32::from_rgb(160, 160, 160))
    }

    fn guess_info(text: impl Into<String>) -> Self {
        Self::new(text, Color32::WHITE)
    }

    fn too_low(text: impl Into<String>) -> Self {
        Self::new(text, Color32::from_rgb(100, 180, 255))
    }

    fn too_high(text: impl Into<String>) -> Self {
        Self::new(text, Color32::from_rgb(255, 110, 80))
    }

    fn win(text: impl Into<String>) -> Self {
        Self::new(text, Color32::from_rgb(255, 215, 0))
    }

    fn warmer(text: impl Into<String>) -> Self {
        Self::new(text, Color32::from_rgb(255, 165, 50))
    }

    fn colder(text: impl Into<String>) -> Self {
        Self::new(text, Color32::from_rgb(140, 210, 255))
    }

    fn error(text: impl Into<String>) -> Self {
        Self::new(text, Color32::from_rgb(255, 80, 80))
    }

    fn leaderboard(text: impl Into<String>) -> Self {
        Self::new(text, Color32::from_rgb(255, 215, 0))
    }
}

// ─── App state ───────────────────────────────────────────────────────────────

struct GuessApp {
    state: AppState,
    leaderboards: HashMap<Difficulty, Vec<(String, u32)>>,
    roaster: Roaster,
    profane: bool,
    low_jibes: Vec<String>,
    high_jibes: Vec<String>,
    win_message: &'static str,
    session_games: u32,
    session_attempts: u32,
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
    messages: Vec<Message>,
    input: String,
    previous_diff: Option<u32>,
    show_name_input: bool,
    name_buffer: String,
    won: bool,
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
            win_message: "",
            session_games: 0,
            session_attempts: 0,
        }
    }
}

// ─── Actions (deferred to avoid borrow conflicts) ────────────────────────────

enum Action {
    StartGame,
    StartRound(Difficulty),
    ProcessGuess(u32, u32),
    FinalizeEntry,
}

// ─── eframe::App impl ────────────────────────────────────────────────────────

impl eframe::App for GuessApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // ── Top status bar ────────────────────────────────────────────────
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let r = self.roaster;
                ui.label(
                    RichText::new(format!("{} {}", r.emoji(), r.name()))
                        .color(r.color())
                        .strong(),
                );
                ui.separator();
                ui.label(
                    RichText::new(if self.profane { "🔞 Profanity ON" } else { "😇 Profanity OFF" })
                        .strong(),
                );
                if self.session_games > 0 {
                    ui.separator();
                    ui.label(
                        RichText::new(format!(
                            "📊 {} game{} · {:.1} avg attempts",
                            self.session_games,
                            if self.session_games == 1 { "" } else { "s" },
                            self.session_attempts as f32 / self.session_games as f32,
                        ))
                        .color(Color32::from_rgb(180, 220, 255)),
                    );
                }
            });
        });

        // ── Right leaderboard panel ───────────────────────────────────────
        egui::SidePanel::right("leaderboard_panel")
            .default_width(260.0)
            .show(ctx, |ui| {
                ui.heading(RichText::new("🏅 Top-5 Leaderboard").strong());
                ui.separator();
                ScrollArea::vertical().show(ui, |ui: &mut Ui| {
                    for &diff in &Difficulty::ALL {
                        let (_, upper) = diff.range();
                        ui.collapsing(
                            RichText::new(format!(
                                "{} {} (1–{})",
                                diff.emoji(),
                                diff.name(),
                                upper
                            ))
                            .color(diff.color()),
                            |ui| {
                                let board = self.leaderboards.entry(diff).or_default();
                                if board.is_empty() {
                                    ui.label(
                                        RichText::new("No entries yet – be the first! 💤")
                                            .italics()
                                            .color(Color32::GRAY),
                                    );
                                } else {
                                    for (i, (name, attempts)) in board.iter().enumerate() {
                                        let medal = ["🥇", "🥈", "🥉", "  4.", "  5."][i.min(4)];
                                        ui.label(format!(
                                            "{} {} – {} attempt{}",
                                            medal,
                                            name,
                                            attempts,
                                            if *attempts == 1 { "" } else { "s" }
                                        ));
                                    }
                                }
                            },
                        );
                    }
                });
            });

        // ── Central panel ─────────────────────────────────────────────────
        let mut action: Option<Action> = None;

        egui::CentralPanel::default().show(ctx, |ui| {
            match &mut self.state {
                // ── Startup screen ────────────────────────────────────────
                AppState::Startup => {
                    ui.vertical_centered(|ui| {
                        ui.add_space(10.0);
                        ui.label(
                            RichText::new("🎲  ULTRA GUESSING GAME  🎲")
                                .size(36.0)
                                .color(Color32::from_rgb(255, 215, 0))
                                .strong(),
                        );
                        ui.label(
                            RichText::new("Now with 420% more roasts")
                                .size(16.0)
                                .color(Color32::from_rgb(180, 180, 180))
                                .italics(),
                        );
                        ui.add_space(8.0);
                        ui.separator();
                        ui.add_space(8.0);

                        // Feature list
                        for feat in &[
                            "✨ 20 unique roasters with personality",
                            "🏆 Persistent top-5 leaderboards across 4 difficulties",
                            "🌡️  Hot/cold warmth hints after every guess",
                            "🔥 Optional profanity mode",
                            "📊 Session statistics tracking",
                        ] {
                            ui.label(RichText::new(*feat).color(Color32::from_rgb(200, 220, 200)));
                        }
                        ui.add_space(16.0);
                    });

                    ui.separator();
                    ui.add_space(12.0);
                    ui.label(RichText::new("Choose your roaster:").size(15.0).strong());
                    ui.add_space(6.0);
                    ComboBox::from_id_source("roaster_combo")
                        .width(500.0)
                        .selected_text(format!(
                            "{} {}  –  {}",
                            self.roaster.emoji(),
                            self.roaster.name(),
                            self.roaster.description()
                        ))
                        .show_ui(ui, |ui: &mut Ui| {
                            for r in Roaster::ALL {
                                ui.selectable_value(
                                    &mut self.roaster,
                                    r,
                                    RichText::new(format!(
                                        "{} {}  –  {}",
                                        r.emoji(),
                                        r.name(),
                                        r.description()
                                    ))
                                    .color(r.color()),
                                );
                            }
                        });

                    ui.add_space(14.0);
                    ui.horizontal(|ui| {
                        ui.label("Profanity mode:");
                        ui.checkbox(&mut self.profane, "Enable 🔞 (spicy roasts)");
                    });

                    ui.add_space(30.0);
                    ui.vertical_centered(|ui| {
                        if ui
                            .button(
                                RichText::new("🚀  Start Game")
                                    .size(26.0)
                                    .color(Color32::from_rgb(255, 215, 0)),
                            )
                            .clicked()
                        {
                            action = Some(Action::StartGame);
                        }
                    });
                }

                // ── Difficulty select ─────────────────────────────────────
                AppState::DifficultySelect => {
                    // Roaster intro banner
                    let intro = self.roaster.intro();
                    egui::Frame::none()
                        .fill(Color32::from_rgb(30, 30, 45))
                        .inner_margin(egui::Margin::symmetric(12.0, 10.0))
                        .rounding(egui::Rounding::same(8.0))
                        .show(ui, |ui| {
                            ui.label(
                                RichText::new(intro)
                                    .size(15.0)
                                    .color(self.roaster.color())
                                    .italics(),
                            );
                        });

                    ui.add_space(20.0);
                    ui.vertical_centered(|ui| {
                        ui.label(
                            RichText::new("🎮  Choose Your Difficulty")
                                .size(28.0)
                                .strong(),
                        );
                        ui.add_space(20.0);

                        for &diff in &Difficulty::ALL {
                            let btn_text = RichText::new(format!(
                                "{}  {}   –   {}",
                                diff.emoji(),
                                diff.name(),
                                diff.description()
                            ))
                            .size(20.0)
                            .color(diff.color());

                            if ui
                                .add_sized([380.0, 52.0], egui::Button::new(btn_text))
                                .clicked()
                            {
                                action = Some(Action::StartRound(diff));
                            }
                            ui.add_space(10.0);
                        }
                    });
                }

                // ── Playing ───────────────────────────────────────────────
                AppState::Playing(ps) => {
                    let (lower, upper) = ps.difficulty.range();

                    // Header
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(format!(
                                "{} {}  –  Guess {} to {}",
                                ps.difficulty.emoji(),
                                ps.difficulty.name(),
                                lower,
                                upper,
                            ))
                            .size(22.0)
                            .color(ps.difficulty.color())
                            .strong(),
                        );
                        ui.with_layout(
                            egui::Layout::right_to_left(egui::Align::Center),
                            |ui| {
                                ui.label(
                                    RichText::new(format!("Attempts: {}", ps.attempts))
                                        .size(16.0)
                                        .color(Color32::from_rgb(200, 200, 200)),
                                );
                            },
                        );
                    });

                    ui.add_space(12.0);

                    // Guess input (hidden after winning)
                    if !ps.won {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("💭 Your guess:").size(15.0));
                            let resp = ui.add(
                                egui::TextEdit::singleline(&mut ps.input)
                                    .desired_width(120.0)
                                    .hint_text(format!("{lower}–{upper}")),
                            );
                            if ui
                                .button(RichText::new("Submit").size(15.0))
                                .clicked()
                                || (resp.lost_focus()
                                    && ui.input(|i| i.key_pressed(egui::Key::Enter)))
                            {
                                if !ps.input.trim().is_empty() {
                                    action = Some(Action::ProcessGuess(lower, upper));
                                }
                            }
                        });
                    }

                    ui.add_space(12.0);
                    ui.separator();
                    ui.add_space(8.0);

                    // Message log (color-coded)
                    ScrollArea::vertical()
                        .id_source("messages_scroll")
                        .auto_shrink([false, true])
                        .stick_to_bottom(true)
                        .show(ui, |ui: &mut Ui| {
                            for msg in &ps.messages {
                                ui.label(
                                    RichText::new(&msg.text).color(msg.color).size(14.0),
                                );
                            }
                        });

                    // Name entry for leaderboard
                    if ps.show_name_input {
                        ui.add_space(12.0);
                        ui.separator();
                        ui.add_space(8.0);
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new("🌟 Top-5 score! Enter your name:")
                                    .color(Color32::from_rgb(255, 215, 0))
                                    .strong(),
                            );
                            ui.text_edit_singleline(&mut ps.name_buffer);
                            if ui.button("Save").clicked() {
                                action = Some(Action::FinalizeEntry);
                            }
                        });
                    }

                    ui.add_space(20.0);
                    ui.horizontal(|ui| {
                        if ui.button(RichText::new("🔄 New Round").size(15.0)).clicked() {
                            self.state = AppState::DifficultySelect;
                        }
                        ui.add_space(10.0);
                        if ui
                            .button(RichText::new("⚙️ Change Roaster").size(15.0))
                            .clicked()
                        {
                            self.state = AppState::Startup;
                        }
                    });
                }
            }
        });

        // ── Execute deferred actions ──────────────────────────────────────
        if let Some(act) = action {
            match act {
                Action::StartGame => {
                    let (mut low, mut high, win) = load_jibes(self.roaster);
                    if !self.profane {
                        low.retain(|j| {
                            !BAD_WORDS.iter().any(|&w| j.to_lowercase().contains(w))
                        });
                        high.retain(|j| {
                            !BAD_WORDS.iter().any(|&w| j.to_lowercase().contains(w))
                        });
                        if low.is_empty() { low.push("Too low!".to_string()); }
                        if high.is_empty() { high.push("Too high!".to_string()); }
                    }
                    self.low_jibes = low;
                    self.high_jibes = high;
                    self.win_message = win;
                    self.leaderboards = load_leaderboards();
                    self.state = AppState::DifficultySelect;
                }
                Action::StartRound(difficulty) => {
                    self.start_new_round(difficulty);
                }
                Action::ProcessGuess(lower, upper) => {
                    let mut temp = std::mem::replace(&mut self.state, AppState::Startup);
                    if let AppState::Playing(ref mut ps) = temp {
                        self.process_guess(ps, lower, upper);
                    }
                    self.state = temp;
                }
                Action::FinalizeEntry => {
                    let mut temp = std::mem::replace(&mut self.state, AppState::Startup);
                    if let AppState::Playing(ref mut ps) = temp {
                        self.finalize_leaderboard_entry(ps);
                    }
                    self.state = temp;
                }
            }
        }
    }
}

// ─── GuessApp methods ────────────────────────────────────────────────────────

impl GuessApp {
    fn start_new_round(&mut self, difficulty: Difficulty) {
        let (lower, upper) = difficulty.range();
        let secret = rand::thread_rng().gen_range(lower..=upper);
        let intro_msg = format!(
            "💡 {} Mode: Guess between {} and {}. I've picked a number – time to prove yourself!",
            difficulty.name(),
            lower,
            upper,
        );
        self.state = AppState::Playing(PlayingState {
            difficulty,
            secret,
            attempts: 0,
            guesses: vec![],
            messages: vec![Message::system(intro_msg)],
            input: String::new(),
            previous_diff: None,
            show_name_input: false,
            name_buffer: String::new(),
            won: false,
        });
    }

    fn process_guess(&mut self, ps: &mut PlayingState, lower: u32, upper: u32) {
        let guess_str = ps.input.trim().to_string();
        ps.input.clear();

        let Ok(guess) = guess_str.parse::<u32>() else {
            ps.messages.push(Message::error("👎 Not a valid number!"));
            return;
        };

        if guess < lower || guess > upper {
            ps.messages.push(Message::error(format!(
                "👎 Out of range – must be {lower}–{upper}!"
            )));
            return;
        }

        ps.attempts += 1;
        ps.guesses.push(guess);

        let current_diff = guess.abs_diff(ps.secret);
        ps.messages.push(Message::guess_info(format!("Guess #{}: {}", ps.attempts, guess)));

        match guess.cmp(&ps.secret) {
            std::cmp::Ordering::Less => {
                let jibe = self.low_jibes
                    [rand::thread_rng().gen_range(0..self.low_jibes.len())]
                .clone();
                ps.messages.push(Message::too_low(format!("❄️ {jibe}")));
            }
            std::cmp::Ordering::Greater => {
                let jibe = self.high_jibes
                    [rand::thread_rng().gen_range(0..self.high_jibes.len())]
                .clone();
                ps.messages.push(Message::too_high(format!("🔥 {jibe}")));
            }
            std::cmp::Ordering::Equal => {
                ps.won = true;
                self.session_games += 1;
                self.session_attempts += ps.attempts;

                ps.messages.push(Message::win(format!(
                    "🌟🌟🌟 {}  🌟🌟🌟",
                    self.win_message
                )));
                ps.messages.push(Message::win(format!(
                    "🏆 VICTORY! You nailed it in {} attempt{}!",
                    ps.attempts,
                    if ps.attempts == 1 { "" } else { "s" }
                )));

                let perf = if ps.attempts == 1 {
                    "💯 PERFECT! First try! Are you psychic?!"
                } else if ps.attempts <= 3 {
                    "🔥 INCREDIBLE! You're a natural!"
                } else if ps.attempts <= 6 {
                    "👏 Well done! Solid performance!"
                } else if ps.attempts <= 12 {
                    "👍 Not bad! Room for improvement!"
                } else {
                    "😅 Finally! That was… a journey!"
                };
                ps.messages.push(Message::win(perf));

                let history = ps
                    .guesses
                    .iter()
                    .map(|g| g.to_string())
                    .collect::<Vec<_>>()
                    .join(" → ");
                ps.messages.push(Message::system(format!("Your journey: {history}")));

                self.handle_win(ps);
                return;
            }
        }

        // Warmth hint
        if let Some(prev) = ps.previous_diff {
            if current_diff < prev {
                ps.messages.push(Message::warmer("🌡️ Getting WARMER! 🔥"));
            } else if current_diff > prev {
                ps.messages.push(Message::colder("❄️ Getting COLDER! 🧊"));
            } else {
                ps.messages.push(Message::system("😐 Same distance – treading water?"));
            }
        }
        ps.previous_diff = Some(current_diff);

        // Extra hint in Insane mode
        if ps.difficulty == Difficulty::Insane && ps.attempts >= 5 {
            if current_diff <= 100 {
                ps.messages.push(Message::warmer("🎯 SUPER HOT! Within 100!"));
            } else if current_diff <= 500 {
                ps.messages.push(Message::warmer("🔥 Within 500 – keep going!"));
            }
        }
    }

    fn handle_win(&mut self, ps: &mut PlayingState) {
        let board = self.leaderboards.entry(ps.difficulty).or_default();
        let threshold = if board.len() < 5 {
            u32::MAX
        } else {
            board.last().unwrap().1
        };

        if board.len() < 5 || ps.attempts <= threshold {
            ps.show_name_input = true;
        } else {
            ps.messages.push(Message::system(format!(
                "Solid effort! You'd need ≤{} attempts for the top-5 on {}.",
                threshold,
                ps.difficulty.name()
            )));
        }
    }

    fn finalize_leaderboard_entry(&mut self, ps: &mut PlayingState) {
        let name = if ps.name_buffer.trim().is_empty() {
            "Anonymous".to_string()
        } else {
            ps.name_buffer.trim().chars().take(20).collect() // cap at 20 chars to prevent UI overflow
        };
        let board = self.leaderboards.entry(ps.difficulty).or_default();
        board.push((name.clone(), ps.attempts));
        board.sort_by_key(|e| e.1);
        board.truncate(5);
        save_leaderboards(&self.leaderboards);
        ps.show_name_input = false;
        ps.messages.push(Message::leaderboard(format!(
            "🏅 {} added to the {} leaderboard!",
            name,
            ps.difficulty.name()
        )));
    }
}

// ─── Jibe data ───────────────────────────────────────────────────────────────

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
                "Cold as a London winter. Aim up!".to_string(),
                "Too low! Absolute pants.".to_string(),
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
                "Too big! You're taking the mickey.".to_string(),
                "Lower, you wally!".to_string(),
            ],
            "🎯 Bang on! Finally, you got it right – about bloody time!",
        ),
        Roaster::UncleRoger => (
            vec![
                "Haiyaa! Too low lah! So weak!".to_string(),
                "Why you guess so low? No strength at all!".to_string(),
                "Haiyah! Too small – you fry rice like this ah?".to_string(),
                "Too low! Emotionally damage my wok!".to_string(),
                "Haiyaa! Guess higher lah, don't be so sad!".to_string(),
                "So low... you put no MSG in your guess?".to_string(),
                "Aiyo! Too low – children guess better!".to_string(),
                "Why so weak? Lift your guess higher!".to_string(),
                "Haiyaa! This guess no flavor – too low!".to_string(),
                "Too small lah! Uncle Roger disappointed!".to_string(),
                "Aiyah! Guess low like no confidence!".to_string(),
                "Aiyo! You guess like Jamie Oliver cook rice!".to_string(),
            ],
            vec![
                "Fuiyoh! Too high lah! Overcook already!".to_string(),
                "Haiyah! Too big – you put too much MSG!".to_string(),
                "Haiyaa! Way too high – wok on fire!".to_string(),
                "Too high! You deep fry until burnt ah?".to_string(),
                "Fuiyoh! Reel it in – too much oil!".to_string(),
                "So high... you make Uncle Roger scream!".to_string(),
                "Aiyo! Too big – lower lah, don't be crazy!".to_string(),
                "Haiyaa! This guess over-seasoned!".to_string(),
                "Too high! Like putting ketchup in fried rice!".to_string(),
            ],
            "🎯 Fuiyoh! Correct lah! Uncle Roger proud of you! MSG approved!",
        ),
        Roaster::RickAstley => (
            vec![
                "Too low! But I'm never gonna let you down... so guess higher!".to_string(),
                "Never gonna give you up... but that guess is too small!".to_string(),
                "We're no strangers to bad guesses – aim up!".to_string(),
                "Too low! Never gonna run around and desert the right number!".to_string(),
                "Never gonna make you cry... unless you keep guessing low!".to_string(),
                "That guess is too small – never gonna say goodbye to roasting!".to_string(),
                "Never gonna tell a lie... your guess is low!".to_string(),
                "A full commitment's what I'm thinking of – guess higher!".to_string(),
            ],
            vec![
                "Too high! Never gonna run around with big numbers!".to_string(),
                "That guess is too big – never gonna give you up!".to_string(),
                "Never gonna let you down... by guessing lower!".to_string(),
                "Too high! You've got to make me understand – reel it in!".to_string(),
                "Never gonna desert you... with overshoots like that!".to_string(),
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
                "That was one of the worst guesses I've seen. Higher.".to_string(),
                "Dreadful. Absolutely dreadful.".to_string(),
                "Too low! Not good enough, I'm afraid.".to_string(),
                "I don't mean to be rude, but that's pants.".to_string(),
                "That guess was forgettable – too small.".to_string(),
                "No. Just no. Try higher.".to_string(),
            ],
            vec![
                "Too high. Over the top.".to_string(),
                "It's a no from me – reel it in.".to_string(),
                "That was far too much.".to_string(),
                "Way too high. Honestly, terrible.".to_string(),
                "Too big! I didn't like it.".to_string(),
                "That's just not right – lower.".to_string(),
                "Absolutely dreadful. Lower please.".to_string(),
                "Ghastly. Simply ghastly.".to_string(),
            ],
            "🎯 Well done. That was actually very good. I'm impressed.",
        ),
        Roaster::NikkiGlaser => (
            vec![
                "Too low – that's disappointing.".to_string(),
                "Too small! Come on, aim higher.".to_string(),
                "That's like my standards – way too low.".to_string(),
                "Too low! You're undershooting, babe.".to_string(),
                "Ouch, too low – that's sad.".to_string(),
                "Too small! Step your game up.".to_string(),
                "Too low – fucking embarrassing.".to_string(),
                "Guess higher, you idiot.".to_string(),
                "That's so low it's pathetic.".to_string(),
                "Babe, no. Higher.".to_string(),
            ],
            vec![
                "Too high – greedy much?".to_string(),
                "Way too big! Reel it in.".to_string(),
                "That's overcompensating – lower.".to_string(),
                "Too high! Calm down.".to_string(),
                "Overshot it – classic overreach.".to_string(),
                "Too big! Fucking relax.".to_string(),
                "That's way too high, babe.".to_string(),
                "Honey, that's too much.".to_string(),
            ],
            "🎯 Yes! Finally – you got there. Proud of you, babe!",
        ),
        Roaster::JoanRivers => (
            vec![
                "Too low! Can we talk? That guess is hideous.".to_string(),
                "Oh honey, too low – that's tragic.".to_string(),
                "That number looks like it needs work – higher!".to_string(),
                "Too small, darling – it fell off the ugly tree.".to_string(),
                "Guess higher! That was atrocious.".to_string(),
                "That's so low it's disgusting.".to_string(),
                "Higher! That guess is a disaster.".to_string(),
                "Can we talk? Too fucking low.".to_string(),
            ],
            vec![
                "Too high! That's overdone, darling.".to_string(),
                "Way too big – calm down.".to_string(),
                "Too high! It looks ridiculous.".to_string(),
                "Reel it in – that's hideous.".to_string(),
                "Too big! Oh honey, no.".to_string(),
                "That's over the top – tragic.".to_string(),
                "Lower! Fucking terrible.".to_string(),
                "Honey, that's a crime against numbers.".to_string(),
            ],
            "🎯 Oh honey, you got it! Fabulous! Simply divine!",
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
        Roaster::CaseOh => (
            vec![
                "CHAT! Too low! This person is TROLLING!".to_string(),
                "Bro, that's so low! CHAT is laughing at you!".to_string(),
                "Too small! You're getting timed out for that guess!".to_string(),
                "CHAT CHAT CHAT! Too low! This is embarrassing!".to_string(),
                "Nah bro, higher! You're making me look bad!".to_string(),
                "Too low! That's it, I'm eating another burger out of stress!".to_string(),
                "WHAT?! Too low! CHAT, spam L's!".to_string(),
                "Bro, that's lower than my K/D ratio! Higher!".to_string(),
                "Too small! I'm literally malding right now!".to_string(),
                "CHAT! This person needs help! Too low!".to_string(),
            ],
            vec![
                "TOO HIGH! CHAT, they're trolling!".to_string(),
                "Bro went way too high! Lower!".to_string(),
                "CHAT CHAT! Too big! This is crazy!".to_string(),
                "Nah bro, reel it in! Way too high!".to_string(),
                "Too high! I'm stress eating Takis over this!".to_string(),
                "WHAT?! Lower! CHAT, clip that!".to_string(),
                "Too big! You're as wrong as my diet!".to_string(),
                "Bro, lower! This is painful to watch!".to_string(),
                "CHAT! Too high! Someone help this person!".to_string(),
            ],
            "🎯 YOOOOO! CHAT! THEY GOT IT! GG! That was actually fire!",
        ),
        Roaster::GenX => (
            vec![
                "Too low. Whatever.".to_string(),
                "Like, too small. Not that I care.".to_string(),
                "Too low. This is lame anyway.".to_string(),
                "That guess sucks. Go higher.".to_string(),
                "Too low. As if.".to_string(),
                "Ugh, too small. Try harder, I guess.".to_string(),
                "Too low. Talk to the hand.".to_string(),
                "That's low. Whatever, guess higher.".to_string(),
                "Too small. This is so bogus.".to_string(),
                "Too low. Gag me with a spoon.".to_string(),
            ],
            vec![
                "Too high. Whatever.".to_string(),
                "Way too big. Lower, I guess.".to_string(),
                "Too high. This is so lame.".to_string(),
                "That's high. Lower. Not that I care.".to_string(),
                "Too big. As if I care. Lower.".to_string(),
                "Ugh, too high. Try lower.".to_string(),
                "Too high. Whatevs.".to_string(),
                "Way too high. Bogus guess.".to_string(),
            ],
            "🎯 Cool, you got it. Whatever. I guess that's good or something.",
        ),
        Roaster::Millennial => (
            vec![
                "Too low bestie! That's not giving what it needs to give!".to_string(),
                "OMG too small! Guess higher, I'm literally dying!".to_string(),
                "Too low! This is NOT the vibe! Higher please!".to_string(),
                "Bestie... too low. I can't even. Go higher!".to_string(),
                "Too small! That's so cringe! Higher!".to_string(),
                "Oof, too low! That hit different (badly). Higher!".to_string(),
                "Too low! Periodt! Guess higher!".to_string(),
                "No cap that's too low! Higher bestie!".to_string(),
                "Too small! That's giving broke millennial energy! Up!".to_string(),
                "Too low! I'm having an existential crisis! Higher!".to_string(),
            ],
            vec![
                "Too high bestie! Lower! I'm literally shaking!".to_string(),
                "Way too big! That's giving try-hard energy! Lower!".to_string(),
                "Too high! Sis, no! Bring it down!".to_string(),
                "Bestie... too high. I can't. Lower please!".to_string(),
                "Too big! That's so extra! Lower!".to_string(),
                "Oof, too high! That's not the tea! Lower!".to_string(),
                "Too high! This ain't it, chief! Down!".to_string(),
                "Way too big bestie! Lower or I'm unfollowing!".to_string(),
                "Too high! My therapist will hear about this! Lower!".to_string(),
            ],
            "🎯 YASSS QUEEN! You did that! I'm so proud! That's so slay! 💅",
        ),
        Roaster::GenZ => (
            vec![
                "Too low bestie! That's giving L energy fr! Higher!".to_string(),
                "Nah that's too small! No cap, aim up!".to_string(),
                "Too low! Bestie you're cooked! Higher fr fr!".to_string(),
                "Bro that's mid and too low! Up!".to_string(),
                "Too small! That's not bussin! Higher!".to_string(),
                "Low key too low! High key need higher!".to_string(),
                "Too low! Deadass guess higher!".to_string(),
                "That ain't it bestie! Too low! Up!".to_string(),
                "Too small! This ain't giving! Higher fr!".to_string(),
                "Nah bro, too low! Periodt! Guess up!".to_string(),
            ],
            vec![
                "Too high bestie! That's doing too much! Lower!".to_string(),
                "Nah that's too big! No cap, down!".to_string(),
                "Too high! Bro you're cooked! Lower fr!".to_string(),
                "That's too much! Not bussin! Lower!".to_string(),
                "Way too high! That's sus! Down!".to_string(),
                "High key too high! Low key need lower!".to_string(),
                "Too high! Deadass lower bestie!".to_string(),
                "That ain't it! Too high! Down fr!".to_string(),
                "Too big! This ain't the vibe! Lower!".to_string(),
                "Nah bro, too high! Periodt! Lower!".to_string(),
            ],
            "🎯 YOOO YOU ATE THAT UP! No cap that was bussin! Purr bestie! 💅💀",
        ),
    }
}

// ─── Leaderboard persistence ─────────────────────────────────────────────────

fn load_leaderboards() -> HashMap<Difficulty, Vec<(String, u32)>> {
    let mut map: HashMap<Difficulty, Vec<(String, u32)>> = HashMap::new();
    for diff in Difficulty::ALL {
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
                    "Insane" => Difficulty::Insane,
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
        board.truncate(5);
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

// ─── Entry point ─────────────────────────────────────────────────────────────

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([980.0, 720.0])
            .with_resizable(true)
            .with_title("🎲 ULTRA GUESSING GAME – Roast Edition"),
        ..Default::default()
    };

    eframe::run_native(
        "🎲 ULTRA GUESSING GAME – Roast Edition",
        options,
        Box::new(|_cc| Ok(Box::new(GuessApp::default()))),
    )
}