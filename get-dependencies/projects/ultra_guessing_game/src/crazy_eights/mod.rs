use crossterm::{cursor, queue, terminal::ClearType};
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::fmt;
use std::io::{self, Write};
use std::thread;
use std::time::{Duration, Instant};

// ── ANSI helpers ──────────────────────────────────────────────────────────────
const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const MAGENTA: &str = "\x1b[35m";
const CYAN: &str = "\x1b[36m";

fn col(color: &str, text: impl fmt::Display) -> String {
    format!("{}{}{}", color, text, RESET)
}

fn read_line() -> String {
    let mut buf = String::new();
    io::stdin().read_line(&mut buf).expect("Failed to read line");
    buf.trim().to_lowercase().to_string()
}

// ── Suit ──────────────────────────────────────────────────────────────────────
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Suit {
    Clubs,
    Diamonds,
    Hearts,
    Spades,
}

impl Suit {
    fn symbol(&self) -> &'static str {
        match self {
            Suit::Clubs    => "♣",
            Suit::Diamonds => "♦",
            Suit::Hearts   => "♥",
            Suit::Spades   => "♠",
        }
    }

    fn color(&self) -> &'static str {
        match self {
            Suit::Hearts | Suit::Diamonds => RED,
            Suit::Clubs  | Suit::Spades   => RESET,
        }
    }

    fn name(&self) -> &'static str {
        match self {
            Suit::Clubs    => "Clubs",
            Suit::Diamonds => "Diamonds",
            Suit::Hearts   => "Hearts",
            Suit::Spades   => "Spades",
        }
    }
}

impl fmt::Display for Suit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}{}", self.color(), self.symbol(), RESET)
    }
}

// ── Rank ──────────────────────────────────────────────────────────────────────
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Rank {
    Ace,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
}

impl Rank {
    fn label(&self) -> &'static str {
        match self {
            Rank::Ace   => "A",
            Rank::Two   => "2",
            Rank::Three => "3",
            Rank::Four  => "4",
            Rank::Five  => "5",
            Rank::Six   => "6",
            Rank::Seven => "7",
            Rank::Eight => "8",
            Rank::Nine  => "9",
            Rank::Ten   => "10",
            Rank::Jack  => "J",
            Rank::Queen => "Q",
            Rank::King  => "K",
        }
    }

    /// Points value for scoring (held cards at end of round).
    fn points(&self) -> u32 {
        match self {
            Rank::Eight                             => 50,
            Rank::Jack | Rank::Queen | Rank::King   => 10,
            Rank::Ace                               => 1,
            Rank::Two                               => 2,
            Rank::Three                             => 3,
            Rank::Four                              => 4,
            Rank::Five                              => 5,
            Rank::Six                               => 6,
            Rank::Seven                             => 7,
            Rank::Nine                              => 9,
            Rank::Ten                               => 10,
        }
    }
}

// ── Card ──────────────────────────────────────────────────────────────────────
#[derive(Clone, Copy, Debug)]
pub struct Card {
    pub rank: Rank,
    pub suit: Suit,
}

impl Card {
    /// One-line display e.g. "A♥"
    fn display(&self) -> String {
        format!(
            "{}{}{}{}{}",
            self.suit.color(), BOLD,
            self.rank.label(),
            self.suit.symbol(),
            RESET,
        )
    }

    /// Whether this card can be played on the given top-of-discard.
    /// `current_suit` overrides the discard suit when an 8 was last played.
    fn can_play_on(&self, top: &Card, current_suit: Suit) -> bool {
        if self.rank == Rank::Eight {
            return true; // 8s are always wild
        }
        self.suit == current_suit || self.rank == top.rank
    }
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display())
    }
}

// ── Deck ──────────────────────────────────────────────────────────────────────
pub struct Deck {
    cards: Vec<Card>,
}

impl Deck {
    pub fn new_shuffled() -> Self {
        let suits = [Suit::Clubs, Suit::Diamonds, Suit::Hearts, Suit::Spades];
        let ranks = [
            Rank::Ace, Rank::Two, Rank::Three, Rank::Four, Rank::Five,
            Rank::Six, Rank::Seven, Rank::Eight, Rank::Nine, Rank::Ten,
            Rank::Jack, Rank::Queen, Rank::King,
        ];
        let mut cards: Vec<Card> = suits
            .iter()
            .flat_map(|&s| ranks.iter().map(move |&r| Card { rank: r, suit: s }))
            .collect();
        cards.shuffle(&mut thread_rng());
        Deck { cards }
    }

    pub fn deal(&mut self) -> Option<Card> {
        self.cards.pop()
    }

    pub fn remaining(&self) -> usize {
        self.cards.len()
    }

    /// Reshuffle the discard pile back into the deck (keeping top card).
    pub fn reshuffle_from_discard(&mut self, discard: &mut Vec<Card>) {
        if discard.len() <= 1 {
            return;
        }
        let top = discard.pop().unwrap();
        self.cards.append(discard);
        discard.push(top);
        self.cards.shuffle(&mut thread_rng());
        println!("{}", col(YELLOW, "♻️  Deck exhausted – reshuffling discard pile into the deck…"));
    }
}

// ── Card-flip animation (matches blackjack pattern) ───────────────────────────
const CARD_HEIGHT: u16 = 5;

fn back_frame(inner_w: usize) -> Vec<String> {
    if inner_w == 0 {
        return (0..CARD_HEIGHT as usize)
            .map(|_| format!(" {}│{}", YELLOW, RESET))
            .collect();
    }
    let bar  = "─".repeat(inner_w);
    let fill = format!("{}{}{}", YELLOW, "░".repeat(inner_w), RESET);
    vec![
        format!(" ┌{}┐", bar),
        format!(" │{}│", fill),
        format!(" │{}│", fill),
        format!(" │{}│", fill),
        format!(" └{}┘", bar),
    ]
}

fn face_frame(card: &Card, inner_w: usize) -> Vec<String> {
    let r     = card.rank.label();
    let s     = card.suit.symbol();
    let c     = card.suit.color();
    let r_len = r.len();

    if inner_w == 0 {
        return (0..CARD_HEIGHT as usize)
            .map(|_| format!(" {}│{}", c, RESET))
            .collect();
    }

    let bar = "─".repeat(inner_w);

    let top = if inner_w >= r_len {
        format!(" │{}{}{}{}{}│", c, BOLD, r, RESET, " ".repeat(inner_w - r_len))
    } else {
        format!(" │{}│", " ".repeat(inner_w))
    };

    let lpad = inner_w.saturating_sub(1) / 2;
    let rpad = inner_w.saturating_sub(1 + lpad);
    let mid = format!(" │{}{}{}{}{}│",
        " ".repeat(lpad), c, s, RESET, " ".repeat(rpad));

    let bot = if inner_w >= r_len {
        format!(" │{}{}{}{}{}│", " ".repeat(inner_w - r_len), c, BOLD, r, RESET)
    } else {
        format!(" │{}│", " ".repeat(inner_w))
    };

    vec![
        format!(" ┌{}┐", bar),
        top,
        mid,
        bot,
        format!(" └{}┘", bar),
    ]
}

fn clear_card_area(out: &mut impl Write) {
    queue!(out, cursor::MoveUp(CARD_HEIGHT)).unwrap();
    for _ in 0..CARD_HEIGHT {
        queue!(out,
            cursor::MoveToColumn(0),
            crossterm::terminal::Clear(ClearType::CurrentLine),
            cursor::MoveDown(1)
        ).unwrap();
    }
    queue!(out, cursor::MoveUp(CARD_HEIGHT)).unwrap();
}

fn animate_flip(card: &Card, reveal: bool) {
    let phases: &[(usize, bool, u64)] = if reveal {
        &[
            (7, true,  55),
            (5, true,  45),
            (3, true,  35),
            (1, true,  30),
            (0, true,  25),
            (0, false, 25),
            (1, false, 30),
            (3, false, 35),
            (5, false, 45),
            (7, false, 90),
        ]
    } else {
        &[
            (0, false, 20),
            (1, false, 30),
            (3, false, 40),
            (5, false, 50),
            (7, false, 90),
        ]
    };

    let mut out = io::stdout();
    let mut first = true;

    for &(w, is_back, delay_ms) in phases {
        if !first {
            queue!(out, cursor::MoveUp(CARD_HEIGHT)).unwrap();
        }
        first = false;

        let lines = if is_back { back_frame(w) } else { face_frame(card, w) };
        for line in &lines {
            queue!(out,
                cursor::MoveToColumn(0),
                crossterm::terminal::Clear(ClearType::CurrentLine),
                crossterm::style::Print(format!("  {}\n", line))
            ).unwrap();
        }
        out.flush().unwrap();

        if delay_ms > 0 {
            thread::sleep(Duration::from_millis(delay_ms));
        }
    }

    clear_card_area(&mut out);
    out.flush().unwrap();
}

fn animate_deal_card(card: &Card) {
    animate_flip(card, false);
}

// ── Display helpers ───────────────────────────────────────────────────────────

fn display_hand(hand: &[Card]) -> String {
    hand.iter()
        .enumerate()
        .map(|(i, c)| format!("[{}]{}", col(CYAN, i + 1), c.display()))
        .collect::<Vec<_>>()
        .join("  ")
}

fn display_opponent_hand_size(name: &str, count: usize) -> String {
    format!(
        "  {} {} {} card{}",
        col(MAGENTA, name),
        col(YELLOW, "▓".repeat(count.min(20))),
        col(BOLD, count),
        if count == 1 { "" } else { "s" }
    )
}

// ── AI opponent ───────────────────────────────────────────────────────────────

fn ai_choose_card(hand: &[Card], top: &Card, current_suit: Suit) -> Option<usize> {
    // Prefer non-8 playable cards first (save 8s as a last resort).
    if let Some(i) = hand.iter().position(|c| c.rank != Rank::Eight && c.can_play_on(top, current_suit)) {
        return Some(i);
    }
    // Fall back to playing an 8.
    hand.iter().position(|c| c.rank == Rank::Eight && c.can_play_on(top, current_suit))
}

/// AI picks a new suit when playing an 8: the suit it has the most of.
fn ai_choose_suit(hand: &[Card]) -> Suit {
    let mut counts = [0usize; 4];
    for card in hand {
        counts[match card.suit {
            Suit::Clubs    => 0,
            Suit::Diamonds => 1,
            Suit::Hearts   => 2,
            Suit::Spades   => 3,
        }] += 1;
    }
    let best = counts.iter().enumerate().max_by_key(|&(_, &c)| c).map(|(i, _)| i).unwrap_or(0);
    [Suit::Clubs, Suit::Diamonds, Suit::Hearts, Suit::Spades][best]
}

// ── Game state ────────────────────────────────────────────────────────────────

struct GameState {
    deck: Deck,
    discard: Vec<Card>,
    /// current_suit tracks the active suit (changed by eights).
    current_suit: Suit,
    /// Player hand
    player_hand: Vec<Card>,
    /// AI hands
    ai_hands: Vec<Vec<Card>>,
    ai_names: Vec<&'static str>,
    /// Direction: true = forward (0→1→2→…), false = reverse.
    forward: bool,
}

impl GameState {
    fn top(&self) -> &Card {
        self.discard.last().expect("discard pile empty")
    }

    fn draw_card(&mut self) -> Option<Card> {
        if self.deck.remaining() == 0 {
            self.deck.reshuffle_from_discard(&mut self.discard);
        }
        self.deck.deal()
    }

    fn play_card(&mut self, hand_idx: usize, card_idx: usize) -> Card {
        let hand = if hand_idx == 0 {
            &mut self.player_hand
        } else {
            &mut self.ai_hands[hand_idx - 1]
        };
        let card = hand.remove(card_idx);
        self.current_suit = card.suit; // updated to played suit (or chosen suit for 8)
        self.discard.push(card);
        card
    }
}

// ── Print table ───────────────────────────────────────────────────────────────

fn print_table(gs: &GameState) {
    let top = gs.top();
    println!();
    println!("{}", col(CYAN, "─".repeat(60)));
    // Opponent card counts
    for (i, name) in gs.ai_names.iter().enumerate() {
        println!("{}", display_opponent_hand_size(name, gs.ai_hands[i].len()));
    }
    println!();
    println!(
        "  {} {}  {}active suit: {}{}",
        col(BOLD, "Discard:"),
        top.display(),
        BOLD,
        col(match gs.current_suit {
            Suit::Hearts   => RED,
            Suit::Diamonds => RED,
            _              => RESET,
        }, format!("{} {}", gs.current_suit.symbol(), gs.current_suit.name())),
        RESET
    );
    println!("  {} cards left in deck", col(YELLOW, gs.deck.remaining()));
    println!();
    println!(
        "  {} {}",
        col(BOLD, "Your hand:"),
        display_hand(&gs.player_hand)
    );
    println!("{}", col(CYAN, "─".repeat(60)));
}

// ── Round loop ────────────────────────────────────────────────────────────────

/// Returns the index of the winner (0 = player, 1..= AI indices).
fn play_round(gs: &mut GameState) -> usize {
    let num_players = 1 + gs.ai_hands.len();
    let mut current = 0usize; // 0 = player, 1..= AI
    let mut skip_next = false;

    loop {
        // ── Check for winner ──────────────────────────────────────────────
        if gs.player_hand.is_empty() {
            return 0;
        }
        for (i, hand) in gs.ai_hands.iter().enumerate() {
            if hand.is_empty() {
                return i + 1;
            }
        }

        let top = *gs.top();
        let suit = gs.current_suit;

        if skip_next {
            skip_next = false;
            current = advance(current, gs.forward, num_players);
            continue;
        }

        if current == 0 {
            // ── Player turn ───────────────────────────────────────────────
            print_table(gs);

            let playable: Vec<usize> = gs.player_hand.iter().enumerate()
                .filter(|(_, c)| c.can_play_on(&top, suit))
                .map(|(i, _)| i)
                .collect();

            if playable.is_empty() {
                println!("{}", col(YELLOW, "⚠️  No playable card – you must draw."));
                if let Some(card) = gs.draw_card() {
                    println!("  You drew: {}", card.display());
                    animate_deal_card(&card);
                    gs.player_hand.push(card);
                } else {
                    println!("{}", col(RED, "  Deck is empty – passing your turn."));
                }
                current = advance(current, gs.forward, num_players);
                continue;
            }

            // Player chooses a card.
            loop {
                print!(
                    "  Your turn – play a card ({}) or {} to draw: ",
                    col(GREEN, playable.iter().map(|i| (i + 1).to_string()).collect::<Vec<_>>().join(",")),
                    col(YELLOW, "'d'")
                );
                io::stdout().flush().unwrap();
                let input = read_line();

                if input == "d" {
                    if let Some(card) = gs.draw_card() {
                        println!("  You drew: {}", card.display());
                        animate_deal_card(&card);
                        gs.player_hand.push(card);
                    } else {
                        println!("{}", col(RED, "  Deck is empty – passing."));
                    }
                    break;
                }

                let idx: usize = match input.parse::<usize>() {
                    Ok(n) if n >= 1 && n <= gs.player_hand.len() => n - 1,
                    _ => {
                        println!("{}", col(RED, "  Invalid choice."));
                        continue;
                    }
                };

                if !playable.contains(&idx) {
                    println!("{}", col(RED, "  That card can't be played on the current discard. Match suit or rank, or play an 8."));
                    continue;
                }

                let played = gs.play_card(0, idx);
                println!("  You played: {}", played.display());
                animate_deal_card(&played);

                // Handle special cards
                if played.rank == Rank::Eight {
                    gs.current_suit = ask_player_suit();
                    println!("  {} You declared: {} {}{}",
                        col(BOLD, "🎴 Wild Eight!"),
                        col(match gs.current_suit {
                            Suit::Hearts | Suit::Diamonds => RED,
                            _ => RESET,
                        }, gs.current_suit.symbol()),
                        col(BOLD, gs.current_suit.name()),
                        RESET
                    );
                } else if played.rank == Rank::Two {
                    // Draw two: next player draws 2
                    let next = advance(current, gs.forward, num_players);
                    println!("{}", col(MAGENTA, "  🃏 Draw Two! Next player draws 2 cards."));
                    for _ in 0..2 {
                        if let Some(c) = gs.draw_card() {
                            if next == 0 {
                                println!("  You draw: {}", c.display());
                                gs.player_hand.push(c);
                            } else {
                                gs.ai_hands[next - 1].push(c);
                            }
                        }
                    }
                } else if played.rank == Rank::Queen {
                    println!("{}", col(MAGENTA, "  👑 Skip! Next player loses their turn."));
                    skip_next = true;
                } else if played.rank == Rank::Ace {
                    gs.forward = !gs.forward;
                    println!("{}", col(MAGENTA, "  🔄 Reverse! Direction of play flipped."));
                }

                break;
            }
        } else {
            // ── AI turn ───────────────────────────────────────────────────
            let ai_idx = current - 1;
            let name = gs.ai_names[ai_idx];

            let hand_clone = gs.ai_hands[ai_idx].clone();
            if let Some(card_idx) = ai_choose_card(&hand_clone, &top, suit) {
                let played = gs.play_card(current, card_idx);
                println!("  {} plays: {}", col(MAGENTA, name), played.display());
                animate_deal_card(&played);

                if played.rank == Rank::Eight {
                    let new_suit = ai_choose_suit(&gs.ai_hands[ai_idx]);
                    gs.current_suit = new_suit;
                    println!("  {} {} declared suit: {} {}",
                        col(MAGENTA, name),
                        col(BOLD, "🎴 plays EIGHT!"),
                        col(match new_suit {
                            Suit::Hearts | Suit::Diamonds => RED,
                            _ => RESET,
                        }, new_suit.symbol()),
                        col(BOLD, new_suit.name())
                    );
                } else if played.rank == Rank::Two {
                    let next = advance(current, gs.forward, num_players);
                    println!("{}", col(MAGENTA, format!("  🃏 {} plays Draw Two!", name)));
                    for _ in 0..2 {
                        if let Some(c) = gs.draw_card() {
                            if next == 0 {
                                println!("  You draw: {}", c.display());
                                gs.player_hand.push(c);
                            } else {
                                gs.ai_hands[next - 1].push(c);
                            }
                        }
                    }
                } else if played.rank == Rank::Queen {
                    println!("{}", col(MAGENTA, format!("  👑 {} plays Skip!", name)));
                    skip_next = true;
                } else if played.rank == Rank::Ace {
                    gs.forward = !gs.forward;
                    println!("{}", col(MAGENTA, format!("  🔄 {} plays Reverse!", name)));
                }

                // Uno-style "last card" alert
                if gs.ai_hands[ai_idx].len() == 1 {
                    println!("{}", col(YELLOW, format!("  ⚠️  {} has ONE card left!", name)));
                }
            } else {
                // AI must draw
                if let Some(card) = gs.draw_card() {
                    println!("  {} draws a card.", col(MAGENTA, name));
                    gs.ai_hands[ai_idx].push(card);
                } else {
                    println!("  {} passes (deck empty).", col(MAGENTA, name));
                }
            }
        }

        current = advance(current, gs.forward, num_players);
    }
}

fn advance(current: usize, forward: bool, total: usize) -> usize {
    if forward {
        (current + 1) % total
    } else {
        (current + total - 1) % total
    }
}

fn ask_player_suit() -> Suit {
    loop {
        print!("  Choose a suit – (c)lubs, (d)iamonds, (h)earts, (s)pades: ");
        io::stdout().flush().unwrap();
        match read_line().as_str() {
            "c" | "clubs"    => return Suit::Clubs,
            "d" | "diamonds" => return Suit::Diamonds,
            "h" | "hearts"   => return Suit::Hearts,
            "s" | "spades"   => return Suit::Spades,
            _                => println!("{}", col(RED, "  Please enter c, d, h, or s.")),
        }
    }
}

// ── Scoring ───────────────────────────────────────────────────────────────────

fn score_hand(hand: &[Card]) -> u32 {
    hand.iter().map(|c| c.rank.points()).sum()
}

fn print_scores(player_score: u32, ai_scores: &[u32], ai_names: &[&str]) {
    println!("\n{}", col(BOLD, "📊 Scores after this round:"));
    println!("  {} {} pts", col(GREEN, "You:"), player_score);
    for (i, &score) in ai_scores.iter().enumerate() {
        println!("  {} {} pts", col(MAGENTA, ai_names[i]), score);
    }
}

// ── Session entry point ───────────────────────────────────────────────────────

/// Returns `(player_won_session, played_8_to_win, elapsed_secs)`.
pub fn play(_roaster_idx: usize, _profane: bool) -> (bool, bool, u64) {
    let start = Instant::now();

    println!("{}", col(CYAN, "╔════════════════════════════════════════════════════════════╗"));
    println!("{}", col(CYAN, "║") + &col(BOLD, "         🎴  IRON AGE CRAZY EIGHTS  🎴                   ") + &col(CYAN, "║"));
    println!("{}", col(CYAN, "╚════════════════════════════════════════════════════════════╝"));
    println!();
    println!("{}", col(BOLD, "  Rules:"));
    println!("  • Match the suit or rank of the top card to play.");
    println!("  • {} are wild – play on anything and choose a new suit.", col(YELLOW, "Eights (8)"));
    println!("  • {} – next player draws 2 cards.", col(MAGENTA, "Twos (2)"));
    println!("  • {} – skip the next player.", col(MAGENTA, "Queens (Q)"));
    println!("  • {} – reverse direction of play.", col(MAGENTA, "Aces (A)"));
    println!("  • First to empty their hand wins the round!");
    println!("  • Lowest cumulative score after 3 rounds wins the session.");
    println!("  • Cards left in hand score: 8=50, J/Q/K=10, A=1, others=face value.");
    println!();

    // Ask number of AI opponents (1-3)
    let num_ai = ask_num_opponents();

    let ai_names_pool: &[&str] = &[
        "Iron Rex", "Lady Vex", "Baron Grim",
    ];
    let ai_names: Vec<&str> = ai_names_pool[..num_ai].to_vec();

    let target_score = 200u32;
    println!("\n{}", col(CYAN, format!("  Playing to {} points. Lowest score wins!", target_score)));
    println!("  {} opponent{}: {}",
        num_ai,
        if num_ai == 1 { "" } else { "s" },
        ai_names.iter().map(|n| col(MAGENTA, n)).collect::<Vec<_>>().join(", ")
    );

    let mut player_total   = 0u32;
    let mut ai_totals: Vec<u32> = vec![0; num_ai];

    let mut session_won   = false;
    let mut played_8_to_win = false;
    let mut round = 0u32;

    loop {
        round += 1;
        println!("\n{}", col(YELLOW, format!("━━━━━━━━━━━━━━━━  Round {}  ━━━━━━━━━━━━━━━━", round)));

        // Build deck and deal 7 cards each.
        let mut deck = Deck::new_shuffled();
        let player_hand: Vec<Card> = (0..7).filter_map(|_| deck.deal()).collect();
        let ai_hands: Vec<Vec<Card>> = (0..num_ai)
            .map(|_| (0..7).filter_map(|_| deck.deal()).collect())
            .collect();

        // Flip first card to start discard (re-flip if it's an 8).
        let first_card = loop {
            let c = deck.deal().expect("not enough cards");
            if c.rank != Rank::Eight {
                break c;
            }
            // Put the 8 back and reshuffle.
            deck.cards.push(c);
            deck.cards.shuffle(&mut thread_rng());
        };
        let current_suit = first_card.suit;
        let discard = vec![first_card];

        println!("\n  Dealing cards…");
        for c in &player_hand {
            animate_deal_card(c);
        }
        println!("  {} Starting card: {}", col(BOLD, "🎴"), first_card.display());

        let mut gs = GameState {
            deck,
            discard,
            current_suit,
            player_hand,
            ai_hands,
            ai_names: ai_names.clone(),
            forward: true,
        };

        let winner = play_round(&mut gs);

        // Tally scores for losers (hand points added to their total).
        let player_round_score = score_hand(&gs.player_hand);
        let ai_round_scores: Vec<u32> = gs.ai_hands.iter().map(|h| score_hand(h)).collect();

        if winner == 0 {
            println!("\n{}", col(GREEN, "🎉 You won this round!"));
            // Check if player won by playing an 8.
            if let Some(top) = gs.discard.last() {
                if top.rank == Rank::Eight {
                    played_8_to_win = true;
                }
            }
            // Opponents get their hand scores added to their totals.
            for (i, &s) in ai_round_scores.iter().enumerate() {
                ai_totals[i] += s;
            }
        } else {
            let winner_name = ai_names[winner - 1];
            println!("\n{}", col(RED, format!("😔 {} won this round.", winner_name)));
            // Player and other losers score their hand points.
            player_total += player_round_score;
            for (i, &s) in ai_round_scores.iter().enumerate() {
                if i + 1 != winner {
                    ai_totals[i] += s;
                }
            }
        }

        print_scores(player_total, &ai_totals, &ai_names);

        // Check if anyone has reached/exceeded target.
        let player_eliminated = player_total >= target_score;
        let all_ai_eliminated = ai_totals.iter().all(|&s| s >= target_score);

        if player_eliminated && !all_ai_eliminated {
            // Player busted out.
            println!("\n{}", col(RED, format!("💀 You've hit {} points – you're out!", target_score)));
            break;
        }

        if all_ai_eliminated && !player_eliminated {
            println!("\n{}", col(GREEN, format!("🏆 All opponents hit {} points – YOU WIN THE SESSION!", target_score)));
            session_won = true;
            break;
        }

        if player_eliminated && all_ai_eliminated {
            // Everyone busted – lowest score wins.
            let player_is_lowest = ai_totals.iter().all(|&s| player_total <= s);
            if player_is_lowest {
                println!("\n{}", col(GREEN, "🏆 Everyone busted, but you have the lowest score – you win!"));
                session_won = true;
            } else {
                println!("\n{}", col(RED, "😔 Everyone busted, and you don't have the lowest score – you lose."));
            }
            break;
        }

        // Ask to continue.
        print!("\n  {} to play another round, {} to quit: ",
            col(GREEN, "'y'"), col(RED, "'n'"));
        io::stdout().flush().unwrap();
        if read_line() != "y" {
            // Session ends early – player wins if they have the lowest score.
            let player_is_lowest = ai_totals.iter().all(|&s| player_total <= s);
            session_won = player_is_lowest;
            break;
        }
    }

    let elapsed = start.elapsed().as_secs();

    println!("\n{}", col(CYAN, "─".repeat(60)));
    if session_won {
        println!("{}", col(GREEN, "🎊 Congratulations – you won the Crazy Eights session!"));
    } else {
        println!("{}", col(RED, "💀 Better luck next session!"));
    }

    (session_won, played_8_to_win, elapsed)
}

fn ask_num_opponents() -> usize {
    loop {
        print!("  How many AI opponents? (1-3): ");
        io::stdout().flush().unwrap();
        match read_line().as_str() {
            "1" => return 1,
            "2" => return 2,
            "3" => return 3,
            _   => println!("{}", col(RED, "  Please enter 1, 2, or 3.")),
        }
    }
}
