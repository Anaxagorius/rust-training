// ── Ultra Game Suite – GUI Launcher ──────────────────────────────────────────
//
// Opens a native egui window showing the game menu with animations.
// Returns the index of the chosen game (1–10) or None if the user closes.

use eframe::egui::{self, Color32, FontId, Frame, Pos2, RichText, Sense, Vec2};
use std::sync::{Arc, Mutex};
use std::time::Instant;

// ── Star particle ─────────────────────────────────────────────────────────────
struct Star {
    x: f32,
    y: f32,
    size: f32,
    speed: f32,
    brightness: f32,
    phase: f32,
}

impl Star {
    fn new(rng: &mut impl rand::Rng, width: f32, height: f32) -> Self {
        Star {
            x: rng.gen_range(0.0..width),
            y: rng.gen_range(0.0..height),
            size: rng.gen_range(1.0..3.5_f32),
            speed: rng.gen_range(0.3..1.2_f32),
            brightness: rng.gen_range(0.4..1.0_f32),
            phase: rng.gen_range(0.0..std::f32::consts::TAU),
        }
    }
}

// ── Firework spark ────────────────────────────────────────────────────────────
struct Spark {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    life: f32,      // 1.0 → 0.0
    color: Color32,
}

// ── App state ─────────────────────────────────────────────────────────────────
pub struct LauncherApp {
    /// Selected game index (1–10) when user clicks a button.
    pub selection: Arc<Mutex<Option<u8>>>,

    start: Instant,
    stars: Vec<Star>,
    sparks: Vec<Spark>,
    last_firework: f32, // time of last firework burst
    hovered_game: Option<usize>,
    window_size: Vec2,
}

// ── Game entries ──────────────────────────────────────────────────────────────
struct GameEntry {
    icon: &'static str,
    name: &'static str,
    desc: &'static str,
    color: Color32,
}

fn game_entries() -> Vec<GameEntry> {
    vec![
        GameEntry { icon: "🎲", name: "Number Guessing",          desc: "Guess the secret number with roaster commentary!",      color: Color32::from_rgb(100, 200, 255) },
        GameEntry { icon: "💀", name: "Hangman",                  desc: "Guess the hidden word before you're hanged!",           color: Color32::from_rgb(220, 120, 80)  },
        GameEntry { icon: "🟩", name: "Wordle",                   desc: "Crack the 5-letter word in just 6 tries!",              color: Color32::from_rgb(100, 200, 120) },
        GameEntry { icon: "💣", name: "Ultra Minesweeper",        desc: "Clear the cursed ruins without hitting a trap!",        color: Color32::from_rgb(255, 180, 60)  },
        GameEntry { icon: "♟",  name: "Ultra Checkers",           desc: "Outmanoeuvre the Minimax AI on the ancient board!",     color: Color32::from_rgb(180, 130, 255) },
        GameEntry { icon: "♔",  name: "Ultra Chess",              desc: "Face the AI across the 64-square battlefield!",         color: Color32::from_rgb(255, 220, 80)  },
        GameEntry { icon: "✕",  name: "Ultra Tic Tac Toe",        desc: "Outwit the AI on the classic 3×3 grid!",                color: Color32::from_rgb(80, 200, 220)  },
        GameEntry { icon: "🃏", name: "Ultra Blackjack",          desc: "Beat the dealer to 21 – natural blackjack pays 3:2!",   color: Color32::from_rgb(255, 100, 130) },
        GameEntry { icon: "♠",  name: "Ultra Poker",              desc: "Texas Hold'em with animated cards & roaster opponents!", color: Color32::from_rgb(140, 220, 140) },
        GameEntry { icon: "🎴", name: "Ultra Crazy Eights",        desc: "Match suit or rank – play an 8 as wild to go first!",   color: Color32::from_rgb(255, 160, 100) },
    ]
}

// ── Helper: HSV → Color32 ─────────────────────────────────────────────────────
fn hsv(h: f32, s: f32, v: f32) -> Color32 {
    let h = h.rem_euclid(360.0);
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;
    let (r, g, b) = match h as u32 {
        0..=59    => (c, x, 0.0),
        60..=119  => (x, c, 0.0),
        120..=179 => (0.0, c, x),
        180..=239 => (0.0, x, c),
        240..=299 => (x, 0.0, c),
        _         => (c, 0.0, x),
    };
    Color32::from_rgb(
        ((r + m) * 255.0) as u8,
        ((g + m) * 255.0) as u8,
        ((b + m) * 255.0) as u8,
    )
}

impl LauncherApp {
    pub fn new(selection: Arc<Mutex<Option<u8>>>) -> Self {
        let mut rng = rand::thread_rng();
        let w = 920.0_f32;
        let h = 700.0_f32;
        let stars = (0..120).map(|_| Star::new(&mut rng, w, h)).collect();
        LauncherApp {
            selection,
            start: Instant::now(),
            stars,
            sparks: Vec::new(),
            last_firework: -5.0,
            hovered_game: None,
            window_size: Vec2::new(w, h),
        }
    }

    fn emit_firework(&mut self, cx: f32, cy: f32, t: f32) {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let hue: f32 = rng.gen_range(0.0..360.0);
        for _ in 0..60 {
            let angle: f32 = rng.gen_range(0.0..std::f32::consts::TAU);
            let speed: f32 = rng.gen_range(60.0..220.0);
            self.sparks.push(Spark {
                x: cx,
                y: cy,
                vx: angle.cos() * speed,
                vy: angle.sin() * speed,
                life: 1.0,
                color: hsv(hue + rng.gen_range(-30.0..30.0), 1.0, 1.0),
            });
        }
        self.last_firework = t;
    }
}

impl eframe::App for LauncherApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        use rand::Rng;
        let t = self.start.elapsed().as_secs_f32();
        let dt = ctx.input(|i| i.stable_dt).min(0.05);

        // Request continuous repaint for animations.
        ctx.request_repaint();

        let screen = ctx.screen_rect();
        self.window_size = screen.size();
        let w = screen.width();
        let h = screen.height();

        // ── Update stars ─────────────────────────────────────────────────────
        for star in &mut self.stars {
            star.y += star.speed * dt * 30.0;
            if star.y > h {
                star.y = 0.0;
                star.x = rand::thread_rng().gen_range(0.0..w);
            }
        }

        // ── Update sparks ────────────────────────────────────────────────────
        for spark in &mut self.sparks {
            spark.x += spark.vx * dt;
            spark.y += spark.vy * dt;
            spark.vy += 120.0 * dt; // gravity
            spark.life -= dt * 1.1;
        }
        self.sparks.retain(|s| s.life > 0.0);

        // ── Periodic fireworks ────────────────────────────────────────────────
        if t - self.last_firework > 4.0 {
            let mut rng = rand::thread_rng();
            let fx = rng.gen_range(w * 0.2..w * 0.8);
            let fy = rng.gen_range(h * 0.1..h * 0.4);
            self.emit_firework(fx, fy, t);
        }

        // ── Background painter ────────────────────────────────────────────────
        let painter = ctx.layer_painter(egui::LayerId::background());

        // Deep space gradient background
        let dark_bg = Color32::from_rgb(8, 5, 20);
        painter.rect_filled(screen, 0.0, dark_bg);
        // Subtle purple nebula clouds
        painter.add(egui::Shape::circle_filled(
            Pos2::new(w * 0.3, h * 0.15),
            h * 0.35,
            Color32::from_rgba_premultiplied(60, 20, 80, 18),
        ));
        painter.add(egui::Shape::circle_filled(
            Pos2::new(w * 0.75, h * 0.25),
            h * 0.28,
            Color32::from_rgba_premultiplied(20, 40, 90, 15),
        ));

        // Stars
        for star in &self.stars {
            let twinkle = (t * star.speed + star.phase).sin() * 0.5 + 0.5;
            let alpha = ((star.brightness * 0.7 + twinkle * 0.3) * 255.0) as u8;
            let c = Color32::from_rgba_premultiplied(220, 220, 255, alpha);
            painter.circle_filled(Pos2::new(star.x, star.y), star.size * 0.5, c);
        }

        // Sparks (fireworks)
        for spark in &self.sparks {
            let alpha = (spark.life * 220.0) as u8;
            let c = Color32::from_rgba_premultiplied(
                spark.color.r(), spark.color.g(), spark.color.b(), alpha);
            painter.circle_filled(Pos2::new(spark.x, spark.y), 2.5, c);
            // trail
            let trail_x = spark.x - spark.vx * dt * 3.0;
            let trail_y = spark.y - spark.vy * dt * 3.0;
            painter.line_segment(
                [Pos2::new(trail_x, trail_y), Pos2::new(spark.x, spark.y)],
                egui::Stroke::new(1.5, c),
            );
        }

        // ── Central UI panel ─────────────────────────────────────────────────
        let mut clicked: Option<u8> = None;

        egui::CentralPanel::default()
            .frame(Frame::none())
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(18.0);

                    // ── Animated rainbow title ────────────────────────────────
                    let title_hue = (t * 50.0) % 360.0;
                    let pulse = (t * 2.0).sin() * 0.08 + 0.92;
                    let title_color = hsv(title_hue, 0.9, pulse);

                    ui.label(
                        RichText::new("⚡  ULTRA GAME SUITE  ⚡")
                            .font(FontId::proportional(42.0))
                            .color(title_color)
                            .strong(),
                    );

                    // subtitle
                    let sub_hue = (t * 40.0 + 180.0) % 360.0;
                    ui.label(
                        RichText::new("v9.0  •  Choose your game")
                            .font(FontId::proportional(16.0))
                            .color(hsv(sub_hue, 0.6, 0.85)),
                    );

                    ui.add_space(10.0);

                    // ── Divider line ──────────────────────────────────────────
                    let div_rect = ui.available_rect_before_wrap();
                    let dy = div_rect.top();
                    painter.line_segment(
                        [Pos2::new(div_rect.left() + 20.0, dy),
                         Pos2::new(div_rect.right() - 20.0, dy)],
                        egui::Stroke::new(1.5, Color32::from_rgb(80, 60, 140)),
                    );
                    ui.add_space(6.0);

                    // ── Game grid (3 columns) ─────────────────────────────────
                    let games = game_entries();
                    let cols = 3usize;
                    let mut game_idx = 0usize;
                    let mut new_hovered: Option<usize> = None;

                    while game_idx < games.len() {
                        ui.horizontal(|ui| {
                            ui.add_space(20.0);
                            for col in 0..cols {
                                let gi = game_idx + col;
                                if gi >= games.len() { break; }
                                let entry = &games[gi];
                                let is_hovered = self.hovered_game == Some(gi);

                                // Card hover glow factor
                                let hovered_pulse = if is_hovered {
                                    (t * 4.0).sin() * 0.15 + 0.85
                                } else { 0.7 };
                                let card_base = if is_hovered {
                                    Color32::from_rgba_premultiplied(
                                        (entry.color.r() as f32 * 0.3) as u8,
                                        (entry.color.g() as f32 * 0.3) as u8,
                                        (entry.color.b() as f32 * 0.3) as u8,
                                        220)
                                } else {
                                    Color32::from_rgba_premultiplied(25, 18, 50, 210)
                                };

                                let border_color = if is_hovered {
                                    Color32::from_rgba_premultiplied(
                                        (entry.color.r() as f32 * hovered_pulse) as u8,
                                        (entry.color.g() as f32 * hovered_pulse) as u8,
                                        (entry.color.b() as f32 * hovered_pulse) as u8,
                                        255)
                                } else {
                                    Color32::from_rgb(60, 45, 90)
                                };

                                let (response, painter_local) = ui.allocate_painter(
                                    Vec2::new(270.0, 100.0),
                                    Sense::click(),
                                );
                                let rect = response.rect;

                                // Card shadow when hovered
                                if is_hovered {
                                    painter_local.rect_filled(
                                        rect.expand(4.0),
                                        12.0,
                                        Color32::from_rgba_premultiplied(
                                            entry.color.r() / 4,
                                            entry.color.g() / 4,
                                            entry.color.b() / 4,
                                            80),
                                    );
                                }

                                // Card body + border
                                painter_local.rect_filled(rect, 10.0, card_base);
                                painter_local.rect_stroke(rect, 10.0,
                                    egui::Stroke::new(
                                        if is_hovered { 2.0 } else { 1.0 },
                                        border_color));

                                // Icon
                                painter_local.text(
                                    rect.left_top() + Vec2::new(12.0, 12.0),
                                    egui::Align2::LEFT_TOP,
                                    entry.icon,
                                    FontId::proportional(28.0),
                                    if is_hovered { entry.color } else {
                                        Color32::from_rgb(
                                            (entry.color.r() as f32 * 0.8) as u8,
                                            (entry.color.g() as f32 * 0.8) as u8,
                                            (entry.color.b() as f32 * 0.8) as u8,
                                        )
                                    },
                                );

                                // Game number badge
                                painter_local.text(
                                    rect.right_top() + Vec2::new(-10.0, 10.0),
                                    egui::Align2::RIGHT_TOP,
                                    format!("{}", gi + 1),
                                    FontId::proportional(13.0),
                                    Color32::from_rgba_premultiplied(200, 200, 200, 160),
                                );

                                // Game name
                                let name_color = if is_hovered { Color32::WHITE } else {
                                    Color32::from_rgb(200, 190, 220)
                                };
                                painter_local.text(
                                    rect.left_top() + Vec2::new(12.0, 48.0),
                                    egui::Align2::LEFT_TOP,
                                    entry.name,
                                    FontId::proportional(14.0),
                                    name_color,
                                );

                                // Description (word-wrapped)
                                let desc_lines = wrap_text(entry.desc, 32);
                                for (li, line) in desc_lines.iter().enumerate() {
                                    painter_local.text(
                                        rect.left_top() + Vec2::new(12.0, 68.0 + li as f32 * 14.0),
                                        egui::Align2::LEFT_TOP,
                                        line,
                                        FontId::proportional(10.5),
                                        Color32::from_rgba_premultiplied(170, 160, 190, 200),
                                    );
                                }

                                if response.hovered() { new_hovered = Some(gi); }
                                if response.clicked() { clicked = Some((gi + 1) as u8); }

                                if col < cols - 1 { ui.add_space(8.0); }
                            }
                        });
                        game_idx += cols;
                        ui.add_space(10.0);
                    }

                    self.hovered_game = new_hovered;

                    // ── Bottom bar ────────────────────────────────────────────
                    ui.add_space(8.0);
                    let hint_hue = (t * 35.0 + 90.0) % 360.0;
                    ui.label(
                        RichText::new("✨  10 roasters  •  28 achievements  •  per-round timer  •  session stats  ✨")
                            .font(FontId::proportional(11.5))
                            .color(hsv(hint_hue, 0.5, 0.7)),
                    );
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new("Click a game card to launch  •  Close window to exit")
                            .font(FontId::proportional(10.0))
                            .color(Color32::from_rgb(100, 90, 120)),
                    );
                });
            });

        // ── Handle game selection outside the closure ─────────────────────────
        if let Some(idx) = clicked {
            if let Ok(mut sel) = self.selection.lock() {
                *sel = Some(idx);
            }
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
}

// ── Simple word-wrap helper ───────────────────────────────────────────────────
fn wrap_text(text: &str, max_chars: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        if current.is_empty() {
            current.push_str(word);
        } else if current.len() + 1 + word.len() <= max_chars {
            current.push(' ');
            current.push_str(word);
        } else {
            lines.push(current.clone());
            current = word.to_string();
        }
    }
    if !current.is_empty() { lines.push(current); }
    lines
}

// ── Public entry-point ────────────────────────────────────────────────────────
/// Launch the GUI launcher window and return the selected game index (1–10),
/// or `None` if the user closed the window without choosing.
pub fn run_launcher() -> Option<u8> {
    let selection = Arc::new(Mutex::new(None::<u8>));
    let sel_clone = Arc::clone(&selection);

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Ultra Game Suite")
            .with_inner_size([920.0, 700.0])
            .with_min_inner_size([600.0, 500.0]),
        vsync: true,
        ..Default::default()
    };

    let _ = eframe::run_native(
        "Ultra Game Suite",
        options,
        Box::new(move |_cc| Box::new(LauncherApp::new(sel_clone))),
    );

    // Retrieve whatever the user selected (may be None if they closed the window)
    selection.lock().ok().and_then(|g| *g)
}
