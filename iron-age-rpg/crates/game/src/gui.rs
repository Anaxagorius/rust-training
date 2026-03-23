// ── Iron Age RPG – GUI ────────────────────────────────────────────────────────
//
// A full eframe/egui front-end that wraps the existing CLI game engine.
// All sub-windows (inventory, character, quests, equipment, map) open as
// floating egui::Windows that are constrained to the app viewport — nothing
// escapes the main container.

use eframe::egui::{
    self, Align, Color32, FontId, Frame, Layout, Margin, RichText, Rounding,
    Stroke, Vec2, ScrollArea, TextEdit,
};

use crate::game_state::GameState;
use crate::commands::{self, CommandResult};
use crate::display;

// ── Colour palette ────────────────────────────────────────────────────────────
const COL_BG:          Color32 = Color32::from_rgb(18,  15,  12);   // near-black stone
const COL_PANEL:       Color32 = Color32::from_rgb(28,  22,  16);   // dark oak
const COL_PANEL_DARK:  Color32 = Color32::from_rgb(20,  16,  12);   // darker oak
const COL_BORDER:      Color32 = Color32::from_rgb(90,  65,  35);   // aged bronze
const COL_HEADER_BG:   Color32 = Color32::from_rgb(35,  25,  12);   // header strip
const COL_TEXT:        Color32 = Color32::from_rgb(220, 200, 160);  // parchment
const COL_TEXT_DIM:    Color32 = Color32::from_rgb(140, 120,  80);  // faded ink
const COL_GOLD:        Color32 = Color32::from_rgb(255, 210,  60);  // gold coin
const COL_HP:          Color32 = Color32::from_rgb(200,  60,  60);  // blood red
const COL_STA:         Color32 = Color32::from_rgb( 70, 180, 100);  // forest green
const COL_MP:          Color32 = Color32::from_rgb( 80, 130, 240);  // arcane blue
const COL_BTN:         Color32 = Color32::from_rgb( 55,  40,  22);  // button fill
const COL_BTN_HOVER:   Color32 = Color32::from_rgb( 90,  65,  35);  // button hover
const COL_BTN_DIR:     Color32 = Color32::from_rgb( 45,  35,  18);  // direction btn
const COL_DANGER:      Color32 = Color32::from_rgb(200,  80,  40);  // warning
const COL_SAFE:        Color32 = Color32::from_rgb( 70, 180, 100);  // safe
const COL_INPUT_BG:    Color32 = Color32::from_rgb( 25,  20,  14);  // input field
const COL_ACCENT:      Color32 = Color32::from_rgb(180, 130,  55);  // bronze accent
const COL_MSG_SYS:     Color32 = Color32::from_rgb(160, 140, 100);  // system / faded
const COL_MSG_COMBAT:  Color32 = Color32::from_rgb(220,  80,  60);  // combat text
const COL_MSG_LOOT:    Color32 = Color32::from_rgb(255, 210,  60);  // loot text
const COL_MSG_QUEST:   Color32 = Color32::from_rgb(130, 200, 255);  // quest text
const COL_WINDOW_BG:   Color32 = Color32::from_rgb(22,  17,  11);   // sub-window

// ── Message log entry ─────────────────────────────────────────────────────────
#[derive(Clone)]
struct LogEntry {
    text: String,
    kind: LogKind,
}

#[derive(Clone, PartialEq)]
enum LogKind {
    Normal,
    System,
    Combat,
    Loot,
    Quest,
    Location,
}

impl LogEntry {
    fn colour(&self) -> Color32 {
        match self.kind {
            LogKind::Normal   => COL_TEXT,
            LogKind::System   => COL_MSG_SYS,
            LogKind::Combat   => COL_MSG_COMBAT,
            LogKind::Loot     => COL_MSG_LOOT,
            LogKind::Quest    => COL_MSG_QUEST,
            LogKind::Location => COL_ACCENT,
        }
    }

    fn classify(text: &str) -> LogKind {
        let lo = text.to_lowercase();
        if lo.contains("⚔") || lo.contains("battle") || lo.contains("attack") || lo.contains("damage") || lo.contains("slain") || lo.contains("hp:") {
            LogKind::Combat
        } else if lo.contains("received:") || lo.contains("loot") || lo.contains("gold") || lo.contains("reward") || lo.contains("crafted") {
            LogKind::Loot
        } else if lo.contains("quest") || lo.contains("journal") || lo.contains("objective") || lo.contains("completed!") {
            LogKind::Quest
        } else if lo.contains("──") || lo.contains("exits:") || lo.contains("people here:") {
            LogKind::Location
        } else if lo.contains("unknown command") || lo.contains("type 'help'") {
            LogKind::System
        } else {
            LogKind::Normal
        }
    }
}

// ── Sub-window toggle state ───────────────────────────────────────────────────
#[derive(Default)]
struct PanelState {
    inventory:  bool,
    character:  bool,
    quests:     bool,
    equipment:  bool,
    map:        bool,
    help:       bool,
    shop:       bool,
}

// ── Main app ──────────────────────────────────────────────────────────────────
pub struct IronAgeApp {
    state:          GameState,
    log:            Vec<LogEntry>,
    input:          String,
    scroll_to_end:  bool,
    panels:         PanelState,
    shop_npc_id:    String,   // last npc id used for shop
    last_location_id: String, // tracks location changes
}

impl IronAgeApp {
    pub fn new() -> Self {
        let state = GameState::new_game();
        let mut app = Self {
            state,
            log: Vec::new(),
            input: String::new(),
            scroll_to_end: true,
            panels: PanelState::default(),
            shop_npc_id: String::new(),
            last_location_id: String::new(),
        };

        // Intro messages
        for line in display::title_screen().lines() {
            app.push_log(line, LogKind::System);
        }
        for line in display::intro_text().lines() {
            app.push_log(line, LogKind::Normal);
        }

        // Initial location display
        if let Some(loc) = app.state.world.current_location() {
            let loc_text = display::location_display(loc);
            app.last_location_id = loc.id.clone();
            for line in loc_text.lines() {
                app.push_log(line, LogKind::Location);
            }
        }

        app
    }

    fn push_log(&mut self, text: &str, kind: LogKind) {
        for line in text.lines() {
            self.log.push(LogEntry { text: line.to_owned(), kind: kind.clone() });
        }
        self.scroll_to_end = true;
    }

    fn push_result(&mut self, text: &str) {
        // Auto-classify lines for colour coding
        for line in text.lines() {
            let kind = LogEntry::classify(line);
            self.log.push(LogEntry { text: line.to_owned(), kind });
        }
        self.scroll_to_end = true;
    }

    fn send_command(&mut self, cmd: &str) {
        let trimmed = cmd.trim().to_string();
        if trimmed.is_empty() { return; }

        // Echo command in log
        self.push_log(&format!("> {}", trimmed), LogKind::System);

        match commands::handle_command(&trimmed, &mut self.state) {
            CommandResult::Message(msg) => {
                self.push_result(&msg);
                // If location changed, track it
                if let Some(loc) = self.state.world.current_location() {
                    if loc.id != self.last_location_id {
                        self.last_location_id = loc.id.clone();
                    }
                }
            }
            CommandResult::Quit => {
                self.push_log("Farewell, traveller. May your blade stay sharp.", LogKind::System);
            }
        }
    }

    fn direction_button(ui: &mut egui::Ui, label: &str) -> bool {
        let btn = egui::Button::new(
            RichText::new(label)
                .font(FontId::proportional(13.0))
                .color(COL_TEXT),
        )
        .fill(COL_BTN_DIR)
        .stroke(Stroke::new(1.0, COL_BORDER))
        .min_size(Vec2::new(36.0, 28.0));
        ui.add(btn).clicked()
    }

    fn action_button(ui: &mut egui::Ui, label: &str) -> bool {
        let btn = egui::Button::new(
            RichText::new(label)
                .font(FontId::proportional(12.0))
                .color(COL_TEXT),
        )
        .fill(COL_BTN)
        .stroke(Stroke::new(1.0, COL_BORDER))
        .min_size(Vec2::new(100.0, 24.0));
        ui.add(btn).clicked()
    }

    fn panel_button(ui: &mut egui::Ui, label: &str, active: bool) -> bool {
        let fill = if active { COL_BTN_HOVER } else { COL_BTN };
        let text_col = if active { COL_GOLD } else { COL_TEXT };
        let btn = egui::Button::new(
            RichText::new(label)
                .font(FontId::proportional(12.0))
                .color(text_col),
        )
        .fill(fill)
        .stroke(Stroke::new(1.0, COL_BORDER))
        .min_size(Vec2::new(100.0, 24.0));
        ui.add(btn).clicked()
    }

    fn bar(ui: &mut egui::Ui, label: &str, val: i32, max: i32, col: Color32) {
        let frac = if max > 0 { (val as f32 / max as f32).clamp(0.0, 1.0) } else { 0.0 };
        ui.horizontal(|ui| {
            ui.label(RichText::new(label).font(FontId::proportional(11.0)).color(COL_TEXT_DIM));
            let (rect, _) = ui.allocate_exact_size(Vec2::new(80.0, 10.0), egui::Sense::hover());
            let bg_rect = rect;
            let fill_rect = egui::Rect::from_min_size(
                rect.min,
                Vec2::new(rect.width() * frac, rect.height()),
            );
            ui.painter().rect_filled(bg_rect, Rounding::same(2.0), COL_PANEL_DARK);
            ui.painter().rect_filled(fill_rect, Rounding::same(2.0), col);
            ui.painter().rect_stroke(bg_rect, Rounding::same(2.0), Stroke::new(1.0, COL_BORDER));
            ui.label(
                RichText::new(format!("{}/{}", val, max))
                    .font(FontId::proportional(10.0))
                    .color(COL_TEXT_DIM),
            );
        });
    }
}

impl eframe::App for IronAgeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // ── Apply dark fantasy style ───────────────────────────────────────────
        let mut style = (*ctx.style()).clone();
        style.visuals.dark_mode = true;
        style.visuals.window_fill = COL_WINDOW_BG;
        style.visuals.panel_fill  = COL_PANEL;
        style.visuals.extreme_bg_color = COL_INPUT_BG;
        style.visuals.override_text_color = Some(COL_TEXT);
        style.visuals.window_stroke = Stroke::new(1.0, COL_BORDER);
        style.visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, COL_BORDER);
        ctx.set_style(style);
        ctx.request_repaint_after(std::time::Duration::from_millis(100));

        // ── Shortcuts ─────────────────────────────────────────────────────────
        // We'll collect deferred commands so we don't borrow `self` twice.
        let mut deferred: Option<String> = None;

        // ── TOP HUD ──────────────────────────────────────────────────────────
        egui::TopBottomPanel::top("hud")
            .frame(Frame::none().fill(COL_HEADER_BG).inner_margin(Margin::symmetric(8.0, 6.0)))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    // Title
                    ui.label(
                        RichText::new("IRON AGE  R.P.G.")
                            .font(FontId::proportional(18.0))
                            .color(COL_GOLD),
                    );
                    ui.separator();

                    // Bars
                    let c = &self.state.player.character;
                    Self::bar(ui, "HP", c.hp, c.max_hp, COL_HP);
                    ui.add_space(6.0);
                    Self::bar(ui, "STA", c.stamina, c.max_stamina, COL_STA);
                    ui.add_space(6.0);
                    Self::bar(ui, "MP", c.mana, c.max_mana, COL_MP);
                    ui.add_space(6.0);
                    ui.label(
                        RichText::new(format!("{} gold", self.state.gold))
                            .font(FontId::proportional(12.0))
                            .color(COL_GOLD),
                    );
                    ui.add_space(6.0);
                    let c = &self.state.player.character;
                    ui.label(
                        RichText::new(format!("Lv.{} {}  XP→{}", c.level, c.name,
                            iron_age_character::Character::xp_for_level(c.level + 1)
                                .saturating_sub(c.experience)))
                            .font(FontId::proportional(11.0))
                            .color(COL_TEXT_DIM),
                    );

                    // Location safety badge
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let (safe, loc_name) = self.state.world.current_location()
                            .map(|l| (l.is_safe, l.name.clone()))
                            .unwrap_or((true, "???".into()));
                        let (badge_col, badge_text) = if safe {
                            (COL_SAFE, "[ Safe Zone ]")
                        } else {
                            (COL_DANGER, "[ Dangerous ]")
                        };
                        ui.label(
                            RichText::new(badge_text)
                                .font(FontId::proportional(11.0))
                                .color(badge_col),
                        );
                        ui.separator();
                        ui.label(
                            RichText::new(&loc_name)
                                .font(FontId::proportional(12.0))
                                .color(COL_ACCENT),
                        );
                    });
                });
            });

        // ── BOTTOM INPUT BAR ──────────────────────────────────────────────────
        egui::TopBottomPanel::bottom("input_bar")
            .frame(Frame::none().fill(COL_PANEL_DARK).inner_margin(Margin::symmetric(8.0, 6.0)))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(">")
                            .font(FontId::proportional(14.0))
                            .color(COL_GOLD),
                    );
                    let resp = ui.add_sized(
                        Vec2::new(ui.available_width() - 80.0, 24.0),
                        TextEdit::singleline(&mut self.input)
                            .font(FontId::proportional(13.0))
                            .text_color(COL_TEXT)
                            .frame(true),
                    );
                    if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        let cmd = self.input.clone();
                        self.input.clear();
                        deferred = Some(cmd);
                        resp.request_focus(); // keep focus on input
                    }
                    if ui.add(
                        egui::Button::new(RichText::new("Send").color(COL_TEXT))
                            .fill(COL_BTN)
                            .stroke(Stroke::new(1.0, COL_BORDER)),
                    ).clicked() {
                        let cmd = self.input.clone();
                        self.input.clear();
                        deferred = Some(cmd);
                    }
                    // Re-focus input after button click
                    if !resp.has_focus() {
                        resp.request_focus();
                    }
                });
            });

        // ── LEFT SIDEBAR: compass + actions ──────────────────────────────────
        egui::SidePanel::left("left_panel")
            .resizable(false)
            .exact_width(116.0)
            .frame(Frame::none().fill(COL_PANEL).inner_margin(Margin::same(6.0)))
            .show(ctx, |ui| {
                ui.add_space(4.0);
                // Title
                ui.label(
                    RichText::new("NAVIGATE")
                        .font(FontId::proportional(10.0))
                        .color(COL_ACCENT),
                );
                ui.separator();

                // ── Compass rose ────────────────────────────────────────────
                // Get available exits for this location
                let exits: Vec<String> = self.state.world.current_location()
                    .map(|l| l.exits.iter().map(|e| e.direction.to_lowercase()).collect())
                    .unwrap_or_default();

                let has = |dir: &str| exits.contains(&dir.to_lowercase().to_string());

                egui::Grid::new("compass")
                    .spacing(Vec2::new(2.0, 2.0))
                    .show(ui, |ui| {
                        // Row 1: NW, N, NE
                        if has("northwest") {
                            if Self::direction_button(ui, "NW") { deferred = Some("go northwest".into()); }
                        } else { ui.label(RichText::new("   ").font(FontId::proportional(13.0))); }
                        if has("north") {
                            if Self::direction_button(ui, "N") { deferred = Some("go north".into()); }
                        } else { ui.label(RichText::new("  ").font(FontId::proportional(13.0))); }
                        if has("northeast") {
                            if Self::direction_button(ui, "NE") { deferred = Some("go northeast".into()); }
                        } else { ui.label(RichText::new("   ").font(FontId::proportional(13.0))); }
                        ui.end_row();

                        // Row 2: W, center marker, E
                        if has("west") {
                            if Self::direction_button(ui, "W") { deferred = Some("go west".into()); }
                        } else { ui.label(RichText::new("  ").font(FontId::proportional(13.0))); }
                        ui.label(
                            RichText::new("*")
                                .font(FontId::proportional(16.0))
                                .color(COL_ACCENT),
                        );
                        if has("east") {
                            if Self::direction_button(ui, "E") { deferred = Some("go east".into()); }
                        } else { ui.label(RichText::new("  ").font(FontId::proportional(13.0))); }
                        ui.end_row();

                        // Row 3: SW, S, SE
                        if has("southwest") {
                            if Self::direction_button(ui, "SW") { deferred = Some("go southwest".into()); }
                        } else { ui.label(RichText::new("   ").font(FontId::proportional(13.0))); }
                        if has("south") {
                            if Self::direction_button(ui, "S") { deferred = Some("go south".into()); }
                        } else { ui.label(RichText::new("  ").font(FontId::proportional(13.0))); }
                        if has("southeast") {
                            if Self::direction_button(ui, "SE") { deferred = Some("go southeast".into()); }
                        } else { ui.label(RichText::new("   ").font(FontId::proportional(13.0))); }
                        ui.end_row();

                        // Row 4: enter/up/down for sub-locations
                        if has("up") {
                            if Self::direction_button(ui, "↑ Up") { deferred = Some("go up".into()); }
                        } else { ui.label(RichText::new("").font(FontId::proportional(10.0))); }
                        if has("enter") || has("in") {
                            let dir = if has("enter") { "enter" } else { "in" };
                            if Self::direction_button(ui, "IN") { deferred = Some(format!("go {}", dir)); }
                        } else { ui.label(RichText::new("").font(FontId::proportional(10.0))); }
                        if has("down") {
                            if Self::direction_button(ui, "↓ Dn") { deferred = Some("go down".into()); }
                        } else { ui.label(RichText::new("").font(FontId::proportional(10.0))); }
                        ui.end_row();
                    });

                ui.add_space(8.0);
                ui.label(
                    RichText::new("ACTIONS")
                        .font(FontId::proportional(10.0))
                        .color(COL_ACCENT),
                );
                ui.separator();

                // ── Quick action buttons ─────────────────────────────────────
                if Self::action_button(ui, "Look") { deferred = Some("look".into()); }
                if Self::action_button(ui, "Search") { deferred = Some("search".into()); }
                if Self::action_button(ui, "Attack") { deferred = Some("attack".into()); }
                if Self::action_button(ui, "Flee") { deferred = Some("flee".into()); }
                if Self::action_button(ui, "Rest") { deferred = Some("rest".into()); }
                if Self::action_button(ui, "Talk...") {
                    // open input pre-filled
                    self.input = "talk ".to_string();
                }

                ui.add_space(8.0);
                ui.label(
                    RichText::new("PANELS")
                        .font(FontId::proportional(10.0))
                        .color(COL_ACCENT),
                );
                ui.separator();

                if Self::panel_button(ui, "Inventory", self.panels.inventory) {
                    self.panels.inventory = !self.panels.inventory;
                }
                if Self::panel_button(ui, "Character", self.panels.character) {
                    self.panels.character = !self.panels.character;
                }
                if Self::panel_button(ui, "Equipment", self.panels.equipment) {
                    self.panels.equipment = !self.panels.equipment;
                }
                if Self::panel_button(ui, "Quests", self.panels.quests) {
                    self.panels.quests = !self.panels.quests;
                }
                if Self::panel_button(ui, "World Map", self.panels.map) {
                    self.panels.map = !self.panels.map;
                }
                if Self::panel_button(ui, "Help", self.panels.help) {
                    self.panels.help = !self.panels.help;
                }

                ui.add_space(8.0);
                ui.label(
                    RichText::new("GAME")
                        .font(FontId::proportional(10.0))
                        .color(COL_ACCENT),
                );
                ui.separator();
                if Self::action_button(ui, "Save") { deferred = Some("save".into()); }
                if Self::action_button(ui, "Load") { deferred = Some("load".into()); }
            });

        // ── RIGHT SIDEBAR: location info ──────────────────────────────────────
        egui::SidePanel::right("right_panel")
            .resizable(false)
            .exact_width(200.0)
            .frame(Frame::none().fill(COL_PANEL).inner_margin(Margin::same(6.0)))
            .show(ctx, |ui| {
                ui.add_space(4.0);
                ui.label(
                    RichText::new("LOCATION")
                        .font(FontId::proportional(10.0))
                        .color(COL_ACCENT),
                );
                ui.separator();

                if let Some(loc) = self.state.world.current_location() {
                    // Location name
                    ui.label(
                        RichText::new(&loc.name)
                            .font(FontId::proportional(14.0))
                            .color(COL_GOLD),
                    );
                    // Region type
                    ui.label(
                        RichText::new(format!("[{}]", format!("{:?}", loc.region_type)))
                            .font(FontId::proportional(10.0))
                            .color(COL_TEXT_DIM),
                    );
                    ui.add_space(4.0);

                    // Description (wrapped)
                    ScrollArea::vertical()
                        .id_source("loc_desc_scroll")
                        .max_height(80.0)
                        .show(ui, |ui| {
                            ui.label(
                                RichText::new(&loc.description)
                                    .font(FontId::proportional(11.0))
                                    .color(COL_TEXT_DIM),
                            );
                        });

                    ui.add_space(4.0);
                    ui.separator();

                    // Exits
                    if !loc.exits.is_empty() {
                        ui.label(
                            RichText::new("EXITS")
                                .font(FontId::proportional(10.0))
                                .color(COL_ACCENT),
                        );
                        for exit in &loc.exits {
                            let lock_icon = if exit.is_locked { " [locked]" } else { "" };
                            ui.horizontal(|ui| {
                                let dir_upper = exit.direction.to_uppercase();
                                ui.label(
                                    RichText::new(format!("• {}{}", dir_upper, lock_icon))
                                        .font(FontId::proportional(11.0))
                                        .color(if exit.is_locked { COL_DANGER } else { COL_TEXT }),
                                );
                            });
                        }
                        ui.add_space(4.0);
                        ui.separator();
                    }

                    // NPCs
                    if !loc.npc_ids.is_empty() {
                        ui.label(
                            RichText::new("PEOPLE HERE")
                                .font(FontId::proportional(10.0))
                                .color(COL_ACCENT),
                        );
                        for npc_id in &loc.npc_ids {
                            let display_name = npc_id.replace('_', " ");
                            let display_name_cap: String = display_name
                                .split_whitespace()
                                .map(|w| {
                                    let mut c = w.chars();
                                    match c.next() {
                                        None => String::new(),
                                        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                                    }
                                })
                                .collect::<Vec<_>>()
                                .join(" ");
                            let npc_id_clone = npc_id.clone();
                            if ui.add(
                                egui::Button::new(
                                    RichText::new(format!(">> {}", display_name_cap))
                                        .font(FontId::proportional(11.0))
                                        .color(COL_TEXT),
                                )
                                .fill(Color32::TRANSPARENT)
                                .stroke(Stroke::NONE),
                            ).clicked() {
                                deferred = Some(format!("talk {}", npc_id_clone));
                            }
                        }
                        ui.add_space(4.0);
                        ui.separator();
                    }

                    // Enemy presence
                    if !loc.enemy_spawn_ids.is_empty() && !loc.is_safe {
                        ui.label(
                            RichText::new("!! HOSTILES NEARBY")
                                .font(FontId::proportional(11.0))
                                .color(COL_DANGER),
                        );
                        if ui.add(
                            egui::Button::new(
                                RichText::new("[ Attack! ]")
                                    .font(FontId::proportional(12.0))
                                    .color(COL_DANGER),
                            )
                            .fill(COL_BTN_DIR)
                            .stroke(Stroke::new(1.0, COL_DANGER)),
                        ).clicked() {
                            deferred = Some("attack".into());
                        }
                        ui.add_space(4.0);
                        ui.separator();
                    }

                    // Crafting station
                    if let Some(station) = &loc.has_crafting_station {
                        ui.label(
                            RichText::new(format!("[Station: {}]", station))
                                .font(FontId::proportional(11.0))
                                .color(COL_ACCENT),
                        );
                        ui.add_space(4.0);
                        ui.separator();
                    }
                }

                // ── Player quick stats ─────────────────────────────────────
                ui.add_space(4.0);
                ui.label(
                    RichText::new("STATS")
                        .font(FontId::proportional(10.0))
                        .color(COL_ACCENT),
                );
                ui.separator();
                let c = &self.state.player.character;
                let stats = [
                    ("STR", c.stats.strength),
                    ("INT", c.stats.intelligence),
                    ("WIS", c.stats.wisdom),
                    ("CON", c.stats.constitution),
                    ("DEX", c.stats.dexterity),
                    ("CHA", c.stats.charisma),
                ];
                egui::Grid::new("stat_grid")
                    .num_columns(3)
                    .spacing(Vec2::new(4.0, 2.0))
                    .show(ui, |ui| {
                        for (i, (name, val)) in stats.iter().enumerate() {
                            ui.label(
                                RichText::new(format!("{}: {}", name, val))
                                    .font(FontId::proportional(11.0))
                                    .color(COL_TEXT_DIM),
                            );
                            if i % 2 == 1 { ui.end_row(); }
                        }
                    });

                if c.stat_points > 0 || c.skill_points > 0 {
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new(format!(
                            "Stat pts: {}  Skill pts: {}",
                            c.stat_points, c.skill_points
                        ))
                        .font(FontId::proportional(11.0))
                        .color(COL_GOLD),
                    );
                }

                // Status effects
                if !c.status_effects.is_empty() {
                    ui.add_space(4.0);
                    ui.separator();
                    ui.label(
                        RichText::new("STATUS")
                            .font(FontId::proportional(10.0))
                            .color(COL_ACCENT),
                    );
                    for eff in &c.status_effects {
                        ui.label(
                            RichText::new(format!("• {:?}", eff))
                                .font(FontId::proportional(10.0))
                                .color(COL_MSG_QUEST),
                        );
                    }
                }

                // Turn counter
                ui.add_space(6.0);
                ui.with_layout(Layout::bottom_up(Align::LEFT), |ui| {
                    ui.label(
                        RichText::new(format!("Turn: {}", self.state.turn))
                            .font(FontId::proportional(10.0))
                            .color(COL_TEXT_DIM),
                    );
                });
            });

        // ── CENTRAL LOG ───────────────────────────────────────────────────────
        egui::CentralPanel::default()
            .frame(Frame::none().fill(COL_BG).inner_margin(Margin::same(6.0)))
            .show(ctx, |ui| {
                // Section header
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("── Chronicle ──")
                            .font(FontId::proportional(12.0))
                            .color(COL_ACCENT),
                    );
                    if ui.add(
                        egui::Button::new(
                            RichText::new("Clear").font(FontId::proportional(10.0)).color(COL_TEXT_DIM),
                        )
                        .fill(Color32::TRANSPARENT)
                        .stroke(Stroke::NONE)
                        .small(),
                    ).clicked() {
                        self.log.clear();
                    }
                });
                ui.add_space(2.0);

                let scroll_area = ScrollArea::vertical()
                    .id_source("main_log")
                    .auto_shrink([false, false])
                    .stick_to_bottom(true);

                scroll_area.show(ui, |ui| {
                    for entry in &self.log {
                        ui.label(
                            RichText::new(&entry.text)
                                .font(FontId::monospace(12.0))
                                .color(entry.colour()),
                        );
                    }
                    if self.scroll_to_end {
                        ui.scroll_to_cursor(Some(Align::BOTTOM));
                        self.scroll_to_end = false;
                    }
                });
            });

        // ── SUB-WINDOWS (float within viewport) ───────────────────────────────

        // ── Inventory window ──────────────────────────────────────────────────
        let mut inv_open = self.panels.inventory;
        egui::Window::new("Inventory")
            .open(&mut inv_open)
            .resizable(true)
            .default_size(Vec2::new(380.0, 320.0))
            .frame(Frame::window(&ctx.style()).fill(COL_WINDOW_BG).stroke(Stroke::new(1.0, COL_BORDER)))
            .show(ctx, |ui| {
                ui.label(
                    RichText::new(format!(
                        "Slots: {}/{}  |  Gold: {}",
                        self.state.inventory.items.len(),
                        self.state.inventory.max_slots,
                        self.state.gold
                    ))
                    .font(FontId::proportional(12.0))
                    .color(COL_GOLD),
                );
                ui.separator();
                ScrollArea::vertical().max_height(240.0).show(ui, |ui| {
                    if self.state.inventory.items.is_empty() {
                        ui.label(RichText::new("(empty)").color(COL_TEXT_DIM));
                    } else {
                        for item in &self.state.inventory.items {
                            let equip_tag = if item.equip_slot.is_some() { " [equip]" } else { "" };
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(format!(
                                        "• {} x{}{}",
                                        item.name, item.quantity, equip_tag
                                    ))
                                    .font(FontId::proportional(12.0))
                                    .color(COL_TEXT),
                                );
                                // Use / Equip buttons
                                let item_id_click = item.id.clone();
                                let item_id_equip = item.id.clone();
                                let has_equip = item.equip_slot.is_some();
                                let is_consumable = matches!(
                                    item.item_type,
                                    iron_age_inventory::ItemType::HealthPotion
                                    | iron_age_inventory::ItemType::StaminaPotion
                                    | iron_age_inventory::ItemType::AntidotePotion
                                    | iron_age_inventory::ItemType::ClarityPotion
                                    | iron_age_inventory::ItemType::FortifyPotion
                                );
                                if is_consumable {
                                    if ui.small_button("Use").clicked() {
                                        deferred = Some(format!("use {}", item_id_click));
                                    }
                                }
                                if has_equip {
                                    if ui.small_button("Equip").clicked() {
                                        deferred = Some(format!("equip {}", item_id_equip));
                                    }
                                }
                            });
                            ui.label(
                                RichText::new(format!("  └ {}", item.description))
                                    .font(FontId::proportional(10.0))
                                    .color(COL_TEXT_DIM),
                            );
                        }
                    }
                });
            });
        self.panels.inventory = inv_open;

        // ── Character window ──────────────────────────────────────────────────
        let mut char_open = self.panels.character;
        egui::Window::new("Character Sheet")
            .open(&mut char_open)
            .resizable(true)
            .default_size(Vec2::new(360.0, 400.0))
            .frame(Frame::window(&ctx.style()).fill(COL_WINDOW_BG).stroke(Stroke::new(1.0, COL_BORDER)))
            .show(ctx, |ui| {
                let sheet = display::character_sheet(&self.state.player.character, self.state.gold);
                ScrollArea::vertical().show(ui, |ui| {
                    for line in sheet.lines() {
                        let col = if line.contains("Level") || line.contains("──") {
                            COL_GOLD
                        } else if line.contains("HP:") || line.contains("Stamina:") {
                            COL_TEXT
                        } else {
                            COL_TEXT_DIM
                        };
                        ui.label(RichText::new(line).font(FontId::monospace(12.0)).color(col));
                    }
                });

                ui.separator();
                let c = &self.state.player.character;
                if c.stat_points > 0 {
                    ui.label(
                        RichText::new(format!("{} stat points to spend — use 'alloc <stat> [n]' in the command bar", c.stat_points))
                            .font(FontId::proportional(11.0))
                            .color(COL_GOLD),
                    );
                    ui.horizontal_wrapped(|ui| {
                        for (lbl, stat_cmd) in &[
                            ("STR+1", "alloc str"),
                            ("INT+1", "alloc int"),
                            ("WIS+1", "alloc wis"),
                            ("CON+1", "alloc con"),
                            ("DEX+1", "alloc dex"),
                            ("CHA+1", "alloc cha"),
                        ] {
                            if ui.button(RichText::new(*lbl).color(COL_TEXT)).clicked() {
                                deferred = Some(stat_cmd.to_string());
                            }
                        }
                    });
                }
            });
        self.panels.character = char_open;

        // ── Equipment window ──────────────────────────────────────────────────
        let mut eq_open = self.panels.equipment;
        egui::Window::new("Equipment")
            .open(&mut eq_open)
            .resizable(true)
            .default_size(Vec2::new(380.0, 360.0))
            .frame(Frame::window(&ctx.style()).fill(COL_WINDOW_BG).stroke(Stroke::new(1.0, COL_BORDER)))
            .show(ctx, |ui| {
                let eq_text = display::equipment_display(&self.state.equipment);
                ScrollArea::vertical().show(ui, |ui| {
                    for line in eq_text.lines() {
                        let col = if line.contains("──") { COL_GOLD } else { COL_TEXT };
                        ui.label(RichText::new(line).font(FontId::monospace(12.0)).color(col));
                    }
                });
                ui.separator();
                ui.label(
                    RichText::new("Unequip slots: mainhand · offhand · helmet · shoulders · torso · leggings · cape · amulet · ring1 · ring2")
                        .font(FontId::proportional(10.0))
                        .color(COL_TEXT_DIM),
                );
                ui.horizontal_wrapped(|ui| {
                    for slot in &["mainhand","offhand","helmet","shoulders","torso","leggings","cape","amulet","ring1","ring2"] {
                        if ui.small_button(RichText::new(*slot).color(COL_TEXT_DIM)).clicked() {
                            deferred = Some(format!("unequip {}", slot));
                        }
                    }
                });
            });
        self.panels.equipment = eq_open;

        // ── Quest log window ──────────────────────────────────────────────────
        let mut q_open = self.panels.quests;
        egui::Window::new("Quest Journal")
            .open(&mut q_open)
            .resizable(true)
            .default_size(Vec2::new(420.0, 380.0))
            .frame(Frame::window(&ctx.style()).fill(COL_WINDOW_BG).stroke(Stroke::new(1.0, COL_BORDER)))
            .show(ctx, |ui| {
                let active = self.state.quest_log.active_quests();
                let q_text = display::quest_log_display(&active);
                ScrollArea::vertical().show(ui, |ui| {
                    for line in q_text.lines() {
                        let col = if line.starts_with('[') || line.contains("──") {
                            COL_GOLD
                        } else if line.contains('✓') {
                            COL_SAFE
                        } else if line.contains('○') {
                            COL_TEXT_DIM
                        } else {
                            COL_TEXT
                        };
                        ui.label(RichText::new(line).font(FontId::monospace(12.0)).color(col));
                    }
                });
            });
        self.panels.quests = q_open;

        // ── World map window ──────────────────────────────────────────────────
        let mut map_open = self.panels.map;
        egui::Window::new("World Map")
            .open(&mut map_open)
            .resizable(true)
            .default_size(Vec2::new(440.0, 420.0))
            .frame(Frame::window(&ctx.style()).fill(COL_WINDOW_BG).stroke(Stroke::new(1.0, COL_BORDER)))
            .show(ctx, |ui| {
                let map_text = display::world_map_display(&self.state.world);
                ScrollArea::vertical().show(ui, |ui| {
                    for line in map_text.lines() {
                        let col = if line.starts_with("▶") {
                            COL_GOLD
                        } else if line.contains("[locked]") {
                            COL_DANGER
                        } else if line.contains("──") {
                            COL_ACCENT
                        } else {
                            COL_TEXT_DIM
                        };
                        ui.label(RichText::new(line).font(FontId::monospace(12.0)).color(col));
                    }
                });
            });
        self.panels.map = map_open;

        // ── Help window ───────────────────────────────────────────────────────
        let mut help_open = self.panels.help;
        egui::Window::new("Help")
            .open(&mut help_open)
            .resizable(true)
            .default_size(Vec2::new(480.0, 460.0))
            .frame(Frame::window(&ctx.style()).fill(COL_WINDOW_BG).stroke(Stroke::new(1.0, COL_BORDER)))
            .show(ctx, |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    for line in display::help_text().lines() {
                        let col = if line.contains("──") {
                            COL_GOLD
                        } else if line.starts_with("  ") {
                            let parts: Vec<&str> = line.splitn(2, "—").collect();
                            if parts.len() == 2 {
                                ui.horizontal(|ui| {
                                    ui.label(
                                        RichText::new(parts[0])
                                            .font(FontId::monospace(12.0))
                                            .color(COL_ACCENT),
                                    );
                                    ui.label(
                                        RichText::new(format!("—{}", parts[1]))
                                            .font(FontId::monospace(12.0))
                                            .color(COL_TEXT_DIM),
                                    );
                                });
                                continue;
                            }
                            COL_TEXT
                        } else {
                            COL_TEXT_DIM
                        };
                        ui.label(RichText::new(line).font(FontId::monospace(12.0)).color(col));
                    }
                });
            });
        self.panels.help = help_open;

        // ── Process deferred command ──────────────────────────────────────────
        if let Some(cmd) = deferred {
            self.send_command(&cmd);
        }
    }
}

// ── Entry point ────────────────────────────────────────────────────────────────
pub fn run() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Iron Age RPG")
            .with_inner_size([1200.0, 750.0])
            .with_min_inner_size([900.0, 600.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Iron Age RPG",
        options,
        Box::new(|_cc| Box::new(IronAgeApp::new())),
    ).expect("Failed to launch Iron Age RPG GUI");
}
