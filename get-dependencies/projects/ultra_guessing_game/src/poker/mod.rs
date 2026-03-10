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
#[derive(Clone, Copy, PartialEq, Eq, Debug, PartialOrd, Ord)]
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
}

// ── Rank ──────────────────────────────────────────────────────────────────────
#[derive(Clone, Copy, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub enum Rank {
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
    Ace,
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
    fn value(&self) -> u8 {
        match self {
            Rank::Two   => 2,
            Rank::Three => 3,
            Rank::Four  => 4,
            Rank::Five  => 5,
            Rank::Six   => 6,
            Rank::Seven => 7,
            Rank::Eight => 8,
            Rank::Nine  => 9,
            Rank::Ten   => 10,
            Rank::Jack  => 11,
            Rank::Queen => 12,
            Rank::King  => 13,
            Rank::Ace   => 14,
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
    fn display(&self) -> String {
        format!(
            "{}{}{}{}{}",
            self.suit.color(), BOLD,
            self.rank.label(),
            self.suit.symbol(),
            RESET,
        )
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
            Rank::Two, Rank::Three, Rank::Four, Rank::Five, Rank::Six,
            Rank::Seven, Rank::Eight, Rank::Nine, Rank::Ten,
            Rank::Jack, Rank::Queen, Rank::King, Rank::Ace,
        ];
        let mut cards: Vec<Card> = suits.iter()
            .flat_map(|&s| ranks.iter().map(move |&r| Card { rank: r, suit: s }))
            .collect();
        cards.shuffle(&mut thread_rng());
        Deck { cards }
    }

    pub fn deal(&mut self) -> Card {
        self.cards.pop().expect("deck is empty")
    }

    #[allow(dead_code)]
    pub fn remaining(&self) -> usize {
        self.cards.len()
    }
}

// ── Hand Ranking ──────────────────────────────────────────────────────────────
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum HandRank {
    HighCard(Vec<u8>),
    OnePair(u8, Vec<u8>),
    TwoPair(u8, u8, u8),       // high pair, low pair, kicker
    ThreeOfAKind(u8, Vec<u8>),
    Straight(u8),               // highest card
    Flush(Vec<u8>),
    FullHouse(u8, u8),          // trips rank, pair rank
    FourOfAKind(u8, u8),        // quad rank, kicker
    StraightFlush(u8),
    RoyalFlush,
}

impl HandRank {
    fn name(&self) -> &'static str {
        match self {
            HandRank::HighCard(_)       => "High Card",
            HandRank::OnePair(_, _)     => "One Pair",
            HandRank::TwoPair(_, _, _)  => "Two Pair",
            HandRank::ThreeOfAKind(_, _)=> "Three of a Kind",
            HandRank::Straight(_)       => "Straight",
            HandRank::Flush(_)          => "Flush",
            HandRank::FullHouse(_, _)   => "Full House",
            HandRank::FourOfAKind(_, _) => "Four of a Kind",
            HandRank::StraightFlush(_)  => "Straight Flush",
            HandRank::RoyalFlush        => "Royal Flush",
        }
    }
    fn emoji(&self) -> &'static str {
        match self {
            HandRank::HighCard(_)        => "🃏",
            HandRank::OnePair(_, _)      => "🎴",
            HandRank::TwoPair(_, _, _)   => "🎭",
            HandRank::ThreeOfAKind(_, _) => "🎯",
            HandRank::Straight(_)        => "📈",
            HandRank::Flush(_)           => "💧",
            HandRank::FullHouse(_, _)    => "🏠",
            HandRank::FourOfAKind(_, _)  => "💎",
            HandRank::StraightFlush(_)   => "⚡",
            HandRank::RoyalFlush         => "👑",
        }
    }
}

/// Evaluate the best 5-card hand from any 7 cards (hole + community).
pub fn best_hand(cards: &[Card]) -> HandRank {
    let n = cards.len();
    if n < 5 {
        panic!("Need at least 5 cards");
    }
    // Generate all combinations of 5 from n
    let mut best: Option<HandRank> = None;
    for i in 0..n {
        for j in (i + 1)..n {
            for k in (j + 1)..n {
                for l in (k + 1)..n {
                    for m in (l + 1)..n {
                        let five = [cards[i], cards[j], cards[k], cards[l], cards[m]];
                        let rank = evaluate_five(&five);
                        if best.is_none() || rank > *best.as_ref().unwrap() {
                            best = Some(rank);
                        }
                    }
                }
            }
        }
    }
    best.unwrap()
}

fn evaluate_five(cards: &[Card; 5]) -> HandRank {
    let mut vals: Vec<u8> = cards.iter().map(|c| c.rank.value()).collect();
    vals.sort_unstable_by(|a, b| b.cmp(a)); // descending

    let is_flush = cards.windows(2).all(|w| w[0].suit == w[1].suit);

    // Check for straight (including A-2-3-4-5 wheel)
    let is_straight = {
        let mut sorted = vals.clone();
        sorted.dedup();
        if sorted.len() != 5 {
            false // has duplicate ranks – can't be a straight
        } else if sorted == vec![14, 5, 4, 3, 2] {
            // Wheel (A-2-3-4-5); treat as straight with high card 5
            true
        } else {
            sorted[0] - sorted[4] == 4
        }
    };

    let wheel = vals == [14, 5, 4, 3, 2];
    let straight_high = if wheel { 5 } else { vals[0] };

    if is_flush && is_straight {
        return if straight_high == 14 {
            HandRank::RoyalFlush
        } else {
            HandRank::StraightFlush(straight_high)
        };
    }

    // Count ranks
    let mut counts: Vec<(u8, u8)> = {
        let mut map: std::collections::HashMap<u8, u8> = std::collections::HashMap::new();
        for &v in &vals {
            *map.entry(v).or_insert(0) += 1;
        }
        let mut v: Vec<(u8, u8)> = map.into_iter().collect();
        // sort by count desc, then rank desc
        v.sort_by(|a, b| b.1.cmp(&a.1).then(b.0.cmp(&a.0)));
        v
    };

    match counts.as_slice() {
        [(r, 4), (k, 1)] => return HandRank::FourOfAKind(*r, *k),
        [(r, 4), ..] => {
            let kicker = counts.iter().find(|(_, c)| *c == 1).map(|(r, _)| *r).unwrap_or(0);
            return HandRank::FourOfAKind(*r, kicker);
        }
        [(trips, 3), (pair, 2)] => return HandRank::FullHouse(*trips, *pair),
        _ => {}
    }

    if is_flush {
        return HandRank::Flush(vals.clone());
    }
    if is_straight {
        return HandRank::Straight(straight_high);
    }

    // Re-read counts for pairs/trips
    counts.sort_by(|a, b| b.1.cmp(&a.1).then(b.0.cmp(&a.0)));

    match counts.as_slice() {
        [(trips, 3), rest @ ..] => {
            let kickers: Vec<u8> = rest.iter().map(|(r, _)| *r).collect();
            return HandRank::ThreeOfAKind(*trips, kickers);
        }
        [(hp, 2), (lp, 2), (k, 1)] => return HandRank::TwoPair(*hp, *lp, *k),
        [(p, 2), rest @ ..] => {
            let kickers: Vec<u8> = rest.iter().flat_map(|(r, c)| std::iter::repeat(*r).take(*c as usize)).collect();
            return HandRank::OnePair(*p, kickers);
        }
        _ => {}
    }

    HandRank::HighCard(vals)
}

// ── Card-flip animation (same style as blackjack) ─────────────────────────────
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
    let r = card.rank.label();
    let s = card.suit.symbol();
    let c = card.suit.color();
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
    let mid = format!(" │{}{}{}{}{}│", " ".repeat(lpad), c, s, RESET, " ".repeat(rpad));
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
            (7, true, 55), (5, true, 45), (3, true, 35), (1, true, 30),
            (0, true, 25), (0, false, 25), (1, false, 30), (3, false, 35),
            (5, false, 45), (7, false, 90),
        ]
    } else {
        &[
            (0, false, 20), (1, false, 30), (3, false, 40),
            (5, false, 50), (7, false, 90),
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

// ── Difficulty ────────────────────────────────────────────────────────────────
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AiDifficulty {
    Easy,
    Medium,
    Hard,
    Expert,
}

impl AiDifficulty {
    fn name(&self) -> &'static str {
        match self {
            AiDifficulty::Easy   => "Easy",
            AiDifficulty::Medium => "Medium",
            AiDifficulty::Hard   => "Hard",
            AiDifficulty::Expert => "Expert",
        }
    }
    fn emoji(&self) -> &'static str {
        match self {
            AiDifficulty::Easy   => "😊",
            AiDifficulty::Medium => "😤",
            AiDifficulty::Hard   => "💀",
            AiDifficulty::Expert => "👹",
        }
    }
}

// ── Betting round ─────────────────────────────────────────────────────────────
#[derive(Clone, Copy, Debug, PartialEq)]
enum BettingRound {
    PreFlop,
    Flop,
    Turn,
    River,
}

impl BettingRound {
    fn name(&self) -> &'static str {
        match self {
            BettingRound::PreFlop => "Pre-Flop",
            BettingRound::Flop    => "Flop",
            BettingRound::Turn    => "Turn",
            BettingRound::River   => "River",
        }
    }
}

// ── AI player action ──────────────────────────────────────────────────────────
#[derive(Clone, Copy, Debug, PartialEq)]
enum AiAction {
    Fold,
    Check,
    Call,
    Raise(u32),
}

// ── Player state ──────────────────────────────────────────────────────────────
#[derive(Debug)]
struct Player {
    name: String,
    chips: u32,
    hole: Vec<Card>,
    is_human: bool,
    folded: bool,
    current_bet: u32,
    roaster_idx: Option<usize>,
    all_in: bool,
}

impl Player {
    fn new_human(name: &str, chips: u32) -> Self {
        Player {
            name: name.to_string(),
            chips,
            hole: Vec::new(),
            is_human: true,
            folded: false,
            current_bet: 0,
            roaster_idx: None,
            all_in: false,
        }
    }

    fn new_ai(name: &str, chips: u32, roaster_idx: usize) -> Self {
        Player {
            name: name.to_string(),
            chips,
            hole: Vec::new(),
            is_human: false,
            folded: false,
            current_bet: 0,
            roaster_idx: Some(roaster_idx),
            all_in: false,
        }
    }

    fn is_active(&self) -> bool {
        !self.folded && !self.all_in
    }
}

// ── Hand strength for AI decision-making ─────────────────────────────────────

/// Quick pre-flop hand strength estimate (0.0 = worst, 1.0 = best).
fn preflop_strength(hole: &[Card]) -> f64 {
    if hole.len() < 2 {
        return 0.0;
    }
    let r0 = hole[0].rank.value();
    let r1 = hole[1].rank.value();
    let same_suit = hole[0].suit == hole[1].suit;
    let hi = r0.max(r1);
    let lo = r0.min(r1);
    let paired = r0 == r1;
    let gap = (hi - lo) as f64;

    let mut score: f64 = (hi as f64 + lo as f64) / 28.0; // 0..1 rough

    if paired {
        score += 0.25 + (hi as f64 / 14.0) * 0.15;
    }
    if same_suit { score += 0.08; }
    if gap <= 1.0 && !paired { score += 0.05; } // connected

    score.clamp(0.0, 1.0)
}

/// Post-flop hand strength mapped to 0.0..1.0.
fn postflop_strength(rank: &HandRank) -> f64 {
    match rank {
        HandRank::HighCard(_)        => 0.05,
        HandRank::OnePair(_, _)      => 0.20,
        HandRank::TwoPair(_, _, _)   => 0.38,
        HandRank::ThreeOfAKind(_, _) => 0.52,
        HandRank::Straight(_)        => 0.65,
        HandRank::Flush(_)           => 0.72,
        HandRank::FullHouse(_, _)    => 0.84,
        HandRank::FourOfAKind(_, _)  => 0.93,
        HandRank::StraightFlush(_)   => 0.98,
        HandRank::RoyalFlush         => 1.00,
    }
}

// ── AI Decision ───────────────────────────────────────────────────────────────

fn ai_decision(
    player: &Player,
    community: &[Card],
    pot: u32,
    to_call: u32,
    difficulty: AiDifficulty,
    rng: &mut impl rand::Rng,
) -> AiAction {
    let strength = if community.is_empty() {
        preflop_strength(&player.hole)
    } else {
        let mut all_cards = player.hole.clone();
        all_cards.extend_from_slice(community);
        let rank = best_hand(&all_cards);
        postflop_strength(&rank)
    };

    // Noise: Easy bots are random, Expert bots are precise
    let noise: f64 = match difficulty {
        AiDifficulty::Easy   => rng.gen_range(-0.35..0.35),
        AiDifficulty::Medium => rng.gen_range(-0.20..0.20),
        AiDifficulty::Hard   => rng.gen_range(-0.10..0.10),
        AiDifficulty::Expert => rng.gen_range(-0.04..0.04),
    };
    let effective = (strength + noise).clamp(0.0, 1.0);

    // Bluff chance
    let bluff_chance: f64 = match difficulty {
        AiDifficulty::Easy   => 0.04,
        AiDifficulty::Medium => 0.08,
        AiDifficulty::Hard   => 0.13,
        AiDifficulty::Expert => 0.18,
    };

    let bluffing = rng.r#gen::<f64>() < bluff_chance;
    let adjusted = if bluffing { (effective + 0.30).min(1.0) } else { effective };

    let can_check = to_call == 0;

    if adjusted < 0.15 && !bluffing {
        if can_check { AiAction::Check } else { AiAction::Fold }
    } else if adjusted < 0.35 {
        if can_check {
            AiAction::Check
        } else if to_call <= player.chips / 4 {
            AiAction::Call
        } else {
            AiAction::Fold
        }
    } else if adjusted < 0.60 {
        if can_check {
            // Occasionally bet small
            if rng.r#gen::<f64>() < 0.40 {
                let bet = (pot / 3).max(1).min(player.chips);
                AiAction::Raise(bet)
            } else {
                AiAction::Check
            }
        } else {
            AiAction::Call
        }
    } else if adjusted < 0.80 {
        // Bet / raise half pot
        let bet = (pot / 2).max(1).min(player.chips);
        AiAction::Raise(bet)
    } else {
        // Strong hand – pot-size bet or all-in
        let bet = pot.max(1).min(player.chips);
        AiAction::Raise(bet)
    }
}

// ── Roaster lines for poker ───────────────────────────────────────────────────

#[allow(dead_code)]
struct PokerRoasterLines {
    on_fold:         &'static str,
    on_check:        &'static str,
    on_call:         &'static str,
    on_raise:        &'static str,
    on_player_win:   &'static str,
    on_player_lose:  &'static str,
    on_player_bust:  &'static str,
    on_bad_hand:     &'static str,
    on_good_hand:    &'static str,
    on_bluff_caught: &'static str,
    taunt:           &'static str,
}

fn poker_roaster_lines(roaster_idx: usize) -> PokerRoasterLines {
    match roaster_idx {
        // Gordon Ramsay
        0 => PokerRoasterLines {
            on_fold:         "I fold. Even I know when I'm beaten, you donkey.",
            on_check:        "Check. Playing it careful, like a properly rested dough.",
            on_call:         "Call. I'll match that, you absolute panini head.",
            on_raise:        "RAISE! Come on then – let's see if you can handle the heat!",
            on_player_win:   "🏆 You WON? You absolute LEGEND! That hand was BEAUTIFUL!",
            on_player_lose:  "💀 Pathetic. Absolutely pathetic. Even raw chicken plays better.",
            on_player_bust:  "💸 You're out of chips, you donut! Absolutely shameful!",
            on_bad_hand:     "Look at that rubbish hand – about as useful as a wet napkin.",
            on_good_hand:    "Fuiyoh- I mean, bloody hell, that's a CRACKING hand!",
            on_bluff_caught: "You were BLUFFING?! You cheeky little— I almost respect it!",
            taunt:           "Is that all you've got?! My gran bets more than that!",
        },
        // Uncle Roger
        1 => PokerRoasterLines {
            on_fold:         "Haiyaa! I fold. Not worth Uncle Roger's time.",
            on_check:        "Check lah. Uncle Roger waiting for better card.",
            on_call:         "Call! Haiyaa, I match your bet. No MSG needed here.",
            on_raise:        "RAISE! Fuiyoh! Uncle Roger going all in on this flavour!",
            on_player_win:   "🏆 Fuiyoh! You WIN! Uncle Roger very proud! MSG approved!",
            on_player_lose:  "💀 Haiyaa! You lose! Emotionally damage Uncle Roger so much!",
            on_player_bust:  "💸 Aiyah! No chips left! You spend like Jamie Oliver buy spice!",
            on_bad_hand:     "This hand worse than Jamie Oliver fried rice. No flavour!",
            on_good_hand:    "Fuiyoh! This hand better than Uncle Roger's wok! Haiyaa!",
            on_bluff_caught: "You bluff Uncle Roger?! Haiyaa! So sneaky lah!",
            taunt:           "Uncle Roger has better hand, no cap. Haiyaa!",
        },
        // Rick Astley
        2 => PokerRoasterLines {
            on_fold:         "I fold. Never gonna give you my chips on THAT hand.",
            on_check:        "Check. Never gonna rush into a bet without good reason.",
            on_call:         "Call! Never gonna let you win without a fight!",
            on_raise:        "RAISE! Never gonna give you up – I'm raising the stakes!",
            on_player_win:   "🏆 You WIN! Never gonna let you down – you played perfectly!",
            on_player_lose:  "💀 Never gonna make you cry... but ouch, you lost badly.",
            on_player_bust:  "💸 Never gonna run around with those chips anymore – they're GONE.",
            on_bad_hand:     "Never gonna pretend that's a good hand. It's rough.",
            on_good_hand:    "Never gonna tell a lie: that hand is INCREDIBLE.",
            on_bluff_caught: "A full commitment to bluffing you were thinking of! Caught ya!",
            taunt:           "Never gonna fold on THIS hand. You've known the rules!",
        },
        // Simon Cowell
        3 => PokerRoasterLines {
            on_fold:         "I fold. That was dreadful. I'm removing myself from this situation.",
            on_check:        "Check. Safe, uninspiring. Typical.",
            on_call:         "Call. Mildly acceptable decision.",
            on_raise:        "Raise. Now THIS is a bold move. Risky, but bold.",
            on_player_win:   "🏆 You win. That was... actually good. I'm genuinely impressed.",
            on_player_lose:  "💀 You lose. Absolutely dreadful. Predictably terrible.",
            on_player_bust:  "💸 Out of chips. I knew this would happen. Ghastly.",
            on_bad_hand:     "That hand is one of the worst I've ever seen. Embarrassing.",
            on_good_hand:    "That hand is... extraordinary. I'll give you that.",
            on_bluff_caught: "You were bluffing. I almost fell for it. Almost. Not quite.",
            taunt:           "It's a no from me on your strategy. Dreadful.",
        },
        // Nikki Glaser
        4 => PokerRoasterLines {
            on_fold:         "I fold! Not dying on this hill, babe.",
            on_check:        "Check. Keeping it cool, like I do.",
            on_call:         "Call! I'll match that, babe. Don't test me.",
            on_raise:        "RAISE! Oh, we're doing this! Let's GO, babe!",
            on_player_win:   "🏆 You WON! Yes babe, you absolutely cooked!",
            on_player_lose:  "💀 You LOST! That's embarrassing. For you.",
            on_player_bust:  "💸 You're broke! That's... that's a lot. Even for me.",
            on_bad_hand:     "That hand is giving nothing. Like, nothing, babe.",
            on_good_hand:    "OKAY. That hand is actually incredible. I'm shook.",
            on_bluff_caught: "You were bluffing?! You sneaky little– respect honestly.",
            taunt:           "Bold move betting that much with THAT hand, babe.",
        },
        // Joan Rivers
        5 => PokerRoasterLines {
            on_fold:         "I fold, darling. Not even Joan plays bad cards.",
            on_check:        "Check, honey. Patience is a virtue. Unlike that outfit.",
            on_call:         "Call! Darling, I'll match your bet and raise my standards.",
            on_raise:        "RAISE! Oh darling, can we talk? I'm RAISING!",
            on_player_win:   "🏆 You WIN! Oh darling, you look GORGEOUS winning!",
            on_player_lose:  "💀 You LOSE! Honey, that was tragic. Simply tragic.",
            on_player_bust:  "💸 Broke! Oh honey, even my divorce was less painful.",
            on_bad_hand:     "That hand looks like my first husband – a complete disaster.",
            on_good_hand:    "Can we talk? That hand is FABULOUS, darling!",
            on_bluff_caught: "You bluffed! Honey, the deception! I'm almost flattered.",
            taunt:           "Darling, with that hand? You've got nerve. I'll give you that.",
        },
        // CaseOh
        6 => PokerRoasterLines {
            on_fold:         "FOLD! CHAT! Not dying for this hand! L hand anyway!",
            on_check:        "Check! CHAT! Playing it cool! This is CONTENT!",
            on_call:         "CALL! CHAT! Matching the bet! This is SPICY!",
            on_raise:        "YOOO RAISE! CHAT! WE GOING BIG! THIS IS CONTENT!",
            on_player_win:   "🏆 YOOO THEY WIN! CHAT! THIS PERSON ATE! GG GG!",
            on_player_lose:  "💀 CHAT! THEY LOST! L! MASSIVE L! RATIO!",
            on_player_bust:  "💸 CHAT! THEY'RE BROKE! ALL THE CHIPS ARE GONE! MASSIVE L!",
            on_bad_hand:     "CHAT! That hand is COOKED! Actual trash fr!",
            on_good_hand:    "YOOO CHAT! THAT HAND IS BUSSIN! THIS IS FIRE!",
            on_bluff_caught: "CAUGHT IN A BLUFF! CHAT THEY WERE TROLLING! CLIP IT!",
            taunt:           "CHAT! They really bet that much with THAT hand? L!",
        },
        // Gen X
        7 => PokerRoasterLines {
            on_fold:         "Fold. Whatever. Not worth my time.",
            on_check:        "Check. Playing it safe. Not that it matters.",
            on_call:         "Call. Whatever, I guess.",
            on_raise:        "Raise. Bold. Probably pointless. Whatever.",
            on_player_win:   "You win. Cool. Not that I care. Good job or whatever.",
            on_player_lose:  "You lose. Whatever. This game's rigged anyway.",
            on_player_bust:  "Broke. Classic. This always happens. Whatever.",
            on_bad_hand:     "That hand is terrible. Not that I'm surprised.",
            on_good_hand:    "Whoa. That's actually a good hand. Whatever.",
            on_bluff_caught: "Bluffing. How very. Classic move. Whatever.",
            taunt:           "Whatever. Bet more. Or don't. I don't care.",
        },
        // Millennial
        8 => PokerRoasterLines {
            on_fold:         "I fold! This hand is not giving what it needs to give, bestie!",
            on_check:        "Check! Manifesting good cards, bestie!",
            on_call:         "Call! That's a whole mood, but I'll match it!",
            on_raise:        "RAISE! Slay! I'm raising and I will NOT be gaslit!",
            on_player_win:   "🏆 YASSS! You WON! That's so iconic! I'm literally crying!",
            on_player_lose:  "💀 You LOST bestie! I'm having a moment! My anxiety!",
            on_player_bust:  "💸 BROKE bestie! That hit different (badly)! I'm deceased!",
            on_bad_hand:     "That hand is giving broke millennial energy, bestie. No.",
            on_good_hand:    "BESTIE! That hand is ICONIC! I'm literally shook!",
            on_bluff_caught: "You were BLUFFING?! The audacity! The disrespect! Iconic though.",
            taunt:           "That bet is giving main character energy! Bestie, really?",
        },
        // Gen Z
        _ => PokerRoasterLines {
            on_fold:         "Folding fr fr. This hand is NOT it bestie. L hand.",
            on_check:        "Check no cap. Waiting for the right vibe fr.",
            on_call:         "Call! Matching that bet deadass. We go fr fr!",
            on_raise:        "RAISE! No cap we going up! This is bussin fr fr!",
            on_player_win:   "🏆 W! You cooked! No cap that was lowkey fire! Purr!",
            on_player_lose:  "💀 L bestie. You fell off. That's an L + ratio.",
            on_player_bust:  "💸 BROKE fr fr! All the chips gone! Massive L bestie!",
            on_bad_hand:     "That hand is mid at best. Lowkey trash ngl.",
            on_good_hand:    "No cap that hand is BUSSIN! That's actually fire fr!",
            on_bluff_caught: "You were bluffing?! That's so slay fr. Caught though. L.",
            taunt:           "Bro really bet that much? That's kinda sus ngl.",
        },
    }
}

// ── Display helpers ───────────────────────────────────────────────────────────

fn display_cards(cards: &[Card]) -> String {
    cards.iter().map(|c| c.display()).collect::<Vec<_>>().join("  ")
}

fn display_card_backs(count: usize) -> String {
    (0..count)
        .map(|_| format!("{}{}??{}", YELLOW, BOLD, RESET))
        .collect::<Vec<_>>()
        .join("  ")
}

fn display_table(
    players: &[Player],
    community: &[Card],
    pot: u32,
    round: BettingRound,
    player_hand_rank: Option<&HandRank>,
) {
    println!("\n{}", col(CYAN, "─".repeat(62)));
    println!("  {} {}  │  {}: {}",
        col(BOLD, "🃏 Pot:"),
        col(YELLOW, format!("{} chips", pot)),
        col(BOLD, round.name()),
        col(CYAN, match community.len() {
            0 => "Waiting for flop...".to_string(),
            n => format!("{} community card{}", n, if n == 1 { "" } else { "s" }),
        }),
    );
    if !community.is_empty() {
        println!("  {} {}",
            col(BOLD, "Community:"),
            display_cards(community),
        );
    }
    println!("{}", col(CYAN, "─".repeat(62)));

    let human = players.iter().find(|p| p.is_human).unwrap();
    print!("  {} {}",
        col(BOLD, format!("  You ({}):", human.name)),
        display_cards(&human.hole),
    );
    if let Some(rank) = player_hand_rank {
        print!("  {} {} {}",
            col(BOLD, "→"),
            rank.emoji(),
            col(GREEN, rank.name()),
        );
    }
    println!("  {} {} chips  {}",
        col(MAGENTA, "💰"),
        col(YELLOW, human.chips),
        if human.folded { col(RED, "[FOLDED]") } else { String::new() },
    );

    for p in players.iter().filter(|p| !p.is_human) {
        let status = if p.folded {
            col(RED, " [FOLDED]".to_string())
        } else if p.all_in {
            col(MAGENTA, " [ALL-IN]".to_string())
        } else {
            String::new()
        };
        println!("  {} {} {}  {} {} chips{}",
            col(BOLD, format!("  {}:", p.name)),
            display_card_backs(2),
            " ".repeat(8),
            col(MAGENTA, "💰"),
            col(YELLOW, p.chips),
            status,
        );
    }
    println!("{}", col(CYAN, "─".repeat(62)));
}

// ── Betting round logic ───────────────────────────────────────────────────────

/// Returns false if player folds or quits.
fn betting_round(
    players: &mut Vec<Player>,
    pot: &mut u32,
    community: &[Card],
    round: BettingRound,
    big_blind: u32,
    difficulty: AiDifficulty,
    roaster_lines: &[PokerRoasterLines],
    rng: &mut impl rand::Rng,
) -> bool {
    // current_bet is the amount required to stay in (above current_bet of each player)
    let mut current_call: u32 = match round {
        BettingRound::PreFlop => big_blind,
        _ => 0,
    };

    // Reset per-round bets (except pre-flop blinds already posted)
    for p in players.iter_mut() {
        if round != BettingRound::PreFlop {
            p.current_bet = 0;
        }
    }

    // We need at least one active player to proceed
    let active_count = players.iter().filter(|p| p.is_active() && !p.folded).count();
    if active_count <= 1 {
        return true;
    }

    // Action order: start from player index 0 (simplified; real Hold'em is position-based)
    let n = players.len();
    let mut action_queue: Vec<usize> = (0..n).collect();
    let mut i = 0;
    let mut last_raiser: Option<usize> = None;

    // Detect the current hand rank for human display
    fn current_rank(community: &[Card], hole: &[Card]) -> Option<HandRank> {
        if community.is_empty() {
            return None;
        }
        let mut all = hole.to_vec();
        all.extend_from_slice(community);
        Some(best_hand(&all))
    }

    loop {
        if action_queue.is_empty() {
            break;
        }
        if i >= action_queue.len() {
            break;
        }

        let idx = action_queue[i];
        let player = &players[idx];

        if player.folded || player.all_in {
            i += 1;
            continue;
        }

        let to_call = current_call.saturating_sub(player.current_bet);
        let is_human = player.is_human;

        if is_human {
            // Display table before player acts
            let rank = current_rank(community, &players[idx].hole);
            display_table(players, community, *pot, round, rank.as_ref());

            let player = &players[idx];
            let can_check = to_call == 0;
            println!("\n  {} Your turn! {} chips in pot.",
                col(BOLD, "🃏"),
                col(YELLOW, *pot),
            );
            if can_check {
                println!("  Options: {} | {} | {} | {}",
                    col(YELLOW, "check (c)"),
                    col(YELLOW, "raise <amount> (r <n>)"),
                    col(YELLOW, "fold (f)"),
                    col(YELLOW, "quit (q)"),
                );
            } else {
                println!("  Call amount: {} chips  Options: {} | {} | {} | {}",
                    col(CYAN, to_call),
                    col(YELLOW, "call (c)"),
                    col(YELLOW, "raise <amount> (r <n>)"),
                    col(YELLOW, "fold (f)"),
                    col(YELLOW, "quit (q)"),
                );
            }
            let chips = player.chips;

            loop {
                print!("  > ");
                io::stdout().flush().expect("flush");
                let input = read_line();
                let parts: Vec<&str> = input.trim().splitn(2, ' ').collect();
                match parts[0] {
                    "c" | "check" | "call" => {
                        if to_call == 0 {
                            println!("{}", col(CYAN, "  ✓ Check."));
                        } else {
                            let actual_call = to_call.min(chips);
                            players[idx].chips -= actual_call;
                            players[idx].current_bet += actual_call;
                            *pot += actual_call;
                            if actual_call < to_call {
                                players[idx].all_in = true;
                                println!("{}", col(MAGENTA, format!("  ♠ All-in for {} chips!", actual_call)));
                            } else {
                                println!("{}", col(GREEN, format!("  ✓ Called {} chips.", actual_call)));
                            }
                        }
                        break;
                    }
                    "r" | "raise" | "bet" => {
                        if parts.len() < 2 {
                            println!("  Specify amount, e.g. 'r 50'");
                            continue;
                        }
                        match parts[1].parse::<u32>() {
                            Ok(amount) if amount > 0 && amount <= chips => {
                                let total = to_call + amount;
                                let paid = total.min(chips);
                                players[idx].chips -= paid;
                                players[idx].current_bet += paid;
                                *pot += paid;
                                current_call = players[idx].current_bet;
                                last_raiser = Some(idx);
                                if paid == chips {
                                    players[idx].all_in = true;
                                    println!("{}", col(MAGENTA, format!("  ♠ All-in for {} chips!", paid)));
                                } else {
                                    println!("{}", col(GREEN, format!("  ✓ Raised {} chips (total bet: {}).", amount, paid)));
                                }
                                // Re-add all active players after this raiser
                                action_queue = (0..n)
                                    .filter(|&j| {
                                        let p = &players[j];
                                        !p.folded && !p.all_in
                                    })
                                    .collect();
                                i = action_queue.iter().position(|&j| j == idx).map(|x| x + 1).unwrap_or(0);
                                break;
                            }
                            Ok(_) => println!("  Amount must be between 1 and {}.", chips),
                            Err(_) => println!("  Invalid amount."),
                        }
                        continue;
                    }
                    "f" | "fold" => {
                        players[idx].folded = true;
                        println!("{}", col(RED, "  ✗ You folded."));
                        // Check if only one player left
                        let active: Vec<usize> = (0..n)
                            .filter(|&j| !players[j].folded)
                            .collect();
                        if active.len() == 1 {
                            return false; // human folded, someone else wins
                        }
                        break;
                    }
                    "q" | "quit" => {
                        players[idx].folded = true;
                        return false;
                    }
                    _ => {
                        println!("  Unknown command. Try: c (check/call), r <amount> (raise), f (fold)");
                    }
                }
            }
        } else {
            // AI player
            let hole = players[idx].hole.clone();
            let ai_idx = idx;
            let rl_idx = players[idx].roaster_idx.unwrap_or(9);
            let rl = &roaster_lines[rl_idx % roaster_lines.len()];
            let action = ai_decision(&players[ai_idx], community, *pot, to_call, difficulty, rng);

            match action {
                AiAction::Fold => {
                    players[idx].folded = true;
                    println!("  {} {}  [{}]",
                        col(BOLD, format!("{}:", players[idx].name)),
                        col(RED, "Folds"),
                        col(MAGENTA, rl.on_fold),
                    );
                    let _ = hole;
                }
                AiAction::Check => {
                    if to_call > 0 {
                        // Can't really check – call or fold
                        let actual_call = to_call.min(players[idx].chips);
                        players[idx].chips -= actual_call;
                        players[idx].current_bet += actual_call;
                        *pot += actual_call;
                        println!("  {} {}  [{}]",
                            col(BOLD, format!("{}:", players[idx].name)),
                            col(CYAN, format!("Calls {} chips", actual_call)),
                            col(MAGENTA, rl.on_call),
                        );
                        if players[idx].chips == 0 {
                            players[idx].all_in = true;
                        }
                    } else {
                        println!("  {} {}  [{}]",
                            col(BOLD, format!("{}:", players[idx].name)),
                            col(CYAN, "Checks"),
                            col(MAGENTA, rl.on_check),
                        );
                    }
                }
                AiAction::Call => {
                    let actual_call = to_call.min(players[idx].chips);
                    players[idx].chips -= actual_call;
                    players[idx].current_bet += actual_call;
                    *pot += actual_call;
                    println!("  {} {}  [{}]",
                        col(BOLD, format!("{}:", players[idx].name)),
                        col(CYAN, format!("Calls {} chips", actual_call)),
                        col(MAGENTA, rl.on_call),
                    );
                    if players[idx].chips == 0 {
                        players[idx].all_in = true;
                    }
                }
                AiAction::Raise(amount) => {
                    let total = to_call + amount;
                    let paid = total.min(players[idx].chips);
                    players[idx].chips -= paid;
                    players[idx].current_bet += paid;
                    *pot += paid;
                    current_call = players[idx].current_bet;
                    last_raiser = Some(idx);
                    println!("  {} {}  [{}]",
                        col(BOLD, format!("{}:", players[idx].name)),
                        col(YELLOW, format!("Raises by {} (total: {})", amount, paid)),
                        col(MAGENTA, rl.on_raise),
                    );
                    if players[idx].chips == 0 {
                        players[idx].all_in = true;
                    }
                    // Everyone else needs to act again
                    action_queue = (0..n)
                        .filter(|&j| !players[j].folded && !players[j].all_in)
                        .collect();
                    i = action_queue.iter().position(|&j| j == idx).map(|x| x + 1).unwrap_or(0);
                    continue;
                }
            }
        }

        i += 1;

        // Check if we should stop: only one active non-folded player
        let still_in: Vec<usize> = (0..n).filter(|&j| !players[j].folded).collect();
        if still_in.len() == 1 {
            break;
        }

        // If all active players have matched the current_call, stop
        let all_matched = (0..n).filter(|&j| !players[j].folded && !players[j].all_in)
            .all(|j| players[j].current_bet >= current_call);
        if all_matched && last_raiser.map_or(true, |lr| action_queue.iter().position(|&j| j == lr).map_or(true, |pos| i > pos)) {
            // Everyone has had a chance after the last raise
            break;
        }
    }

    true
}

// ── Showdown ──────────────────────────────────────────────────────────────────

/// Reveal AI hands at showdown and determine winner(s). Returns the winner index.
fn showdown(
    players: &mut Vec<Player>,
    community: &[Card],
    pot: u32,
    roaster_lines: &[PokerRoasterLines],
) -> usize {
    println!("\n{}", col(BOLD, "═".repeat(62)));
    println!("{}", col(YELLOW, "  🎭  SHOWDOWN!  🎭"));
    println!("{}", col(BOLD, "═".repeat(62)));

    let active: Vec<usize> = (0..players.len()).filter(|&i| !players[i].folded).collect();

    if active.len() == 1 {
        let winner = active[0];
        println!("  {} wins the pot of {} chips (everyone else folded)!",
            col(BOLD, &players[winner].name),
            col(YELLOW, pot),
        );
        players[winner].chips += pot;
        return winner;
    }

    let mut best_rank: Option<HandRank> = None;
    let mut best_idx = active[0];

    for &i in &active {
        let mut all_cards = players[i].hole.clone();
        all_cards.extend_from_slice(community);
        let rank = best_hand(&all_cards);
        println!("  {} {} → {} {}   {}",
            col(BOLD, format!("{}:", players[i].name)),
            display_cards(&players[i].hole),
            rank.emoji(),
            col(CYAN, rank.name()),
            if players[i].is_human { "" } else {
                roaster_lines.get(players[i].roaster_idx.unwrap_or(9) % roaster_lines.len())
                    .map_or("", |rl| rl.on_good_hand)
            },
        );
        if best_rank.is_none() || rank > *best_rank.as_ref().unwrap() {
            best_rank = Some(rank);
            best_idx = i;
        }
    }

    println!();
    println!("  {} {} {} wins {} chips! {} {}",
        col(YELLOW, "🏆"),
        col(BOLD, &players[best_idx].name),
        if players[best_idx].is_human { col(GREEN, "(YOU)".to_string()) } else { String::new() },
        col(YELLOW, pot),
        best_rank.as_ref().map(|r| r.emoji()).unwrap_or(""),
        best_rank.as_ref().map(|r| col(CYAN, r.name())).unwrap_or_default(),
    );

    players[best_idx].chips += pot;
    best_idx
}

// ── Ask buy-in ────────────────────────────────────────────────────────────────

fn ask_buy_in() -> u32 {
    println!("\n{}", col(BOLD, "💰 Buy-In Options:"));
    println!("  1. {}  –  Small stakes, casual game", col(YELLOW, "500 chips"));
    println!("  2. {}  –  Standard buy-in", col(YELLOW, "1,000 chips"));
    println!("  3. {}  –  High roller", col(YELLOW, "2,500 chips"));
    println!("  4. {}  –  VIP table", col(YELLOW, "5,000 chips"));

    loop {
        print!("\n  Your choice (1-4): ");
        io::stdout().flush().expect("flush");
        match read_line().as_str() {
            "1" => return 500,
            "2" => return 1_000,
            "3" => return 2_500,
            "4" => return 5_000,
            _   => println!("  Please enter 1, 2, 3, or 4."),
        }
    }
}

fn ask_difficulty() -> AiDifficulty {
    println!("\n{}", col(BOLD, "🎮 AI Difficulty:"));
    println!("  1. {} Easy   – Opponents play loosely, fold often, rarely bluff", AiDifficulty::Easy.emoji());
    println!("  2. {} Medium – Balanced play with occasional surprises", AiDifficulty::Medium.emoji());
    println!("  3. {} Hard   – Solid hand evaluation, strategic betting", AiDifficulty::Hard.emoji());
    println!("  4. {} Expert – Tight-aggressive, reads hands, bluffs smartly", AiDifficulty::Expert.emoji());

    loop {
        print!("\n  Your choice (1-4): ");
        io::stdout().flush().expect("flush");
        match read_line().as_str() {
            "1" => return AiDifficulty::Easy,
            "2" => return AiDifficulty::Medium,
            "3" => return AiDifficulty::Hard,
            "4" => return AiDifficulty::Expert,
            _   => println!("  Please enter 1, 2, 3, or 4."),
        }
    }
}

fn ask_opponent_count() -> usize {
    println!("\n{}", col(BOLD, "👥 Number of AI Opponents (1–3):"));
    println!("  1. One opponent   – heads up");
    println!("  2. Two opponents  – trio match");
    println!("  3. Three opponents– full table");
    loop {
        print!("\n  Your choice (1-3): ");
        io::stdout().flush().expect("flush");
        match read_line().as_str() {
            "1" => return 1,
            "2" => return 2,
            "3" => return 3,
            _   => println!("  Please enter 1, 2, or 3."),
        }
    }
}

// ── Names for AI roasters ─────────────────────────────────────────────────────

const AI_NAMES: &[(&str, usize)] = &[
    ("Gordon Ramsay",  0),
    ("Uncle Roger",    1),
    ("Rick Astley",    2),
    ("Simon Cowell",   3),
    ("Nikki Glaser",   4),
    ("Joan Rivers",    5),
    ("CaseOh",         6),
    ("Gen X",          7),
    ("Millennial",     8),
    ("Gen Z",          9),
];

// ── Main entry point ──────────────────────────────────────────────────────────

/// Play one session of Texas Hold'em.
/// Returns `(won_a_hand, got_royal_flush, elapsed_secs)`.
pub fn play(roaster_idx: usize, profane: bool) -> (bool, bool, u64) {
    let _ = profane; // reserved for future profanity-filtered commentary

    println!("\n{}", col(CYAN, "─".repeat(62)));
    println!("{}", col(BOLD, "  ♠  IRON AGE TEXAS HOLD'EM POKER  ♠"));
    println!("{}", col(CYAN, "─".repeat(62)));
    println!("  Texas Hold'em rules – blind bets, community cards, showdown.");
    println!("  Commands during betting: {} (check/call)  │  {} (raise)  │  {} (fold)  │  {} (quit)",
        col(YELLOW, "c"),
        col(YELLOW, "r <amount>"),
        col(YELLOW, "f"),
        col(YELLOW, "q"),
    );

    let buy_in = ask_buy_in();
    let difficulty = ask_difficulty();
    let opp_count = ask_opponent_count();

    println!("\n{} {} buy-in  │  {} difficulty  │  {} opponent{}",
        col(CYAN, "  ♠"),
        col(YELLOW, format!("{} chip", buy_in)),
        col(BOLD, difficulty.name()),
        opp_count,
        if opp_count == 1 { "" } else { "s" },
    );

    // Build roaster lines lookup array
    let all_roaster_lines: Vec<PokerRoasterLines> = (0..10)
        .map(poker_roaster_lines)
        .collect();

    // Choose AI opponents – exclude the current human's roaster
    let mut available: Vec<(&str, usize)> = AI_NAMES.iter()
        .filter(|(_, idx)| *idx != roaster_idx)
        .cloned()
        .collect();
    available.shuffle(&mut thread_rng());
    let opponents: Vec<(&str, usize)> = available.into_iter().take(opp_count).collect();

    let session_start = Instant::now();
    let mut won_any = false;
    let mut got_royal_flush = false;
    let small_blind = buy_in / 100;
    let big_blind   = small_blind * 2;

    // Build player list
    let mut players: Vec<Player> = Vec::new();
    players.push(Player::new_human("You", buy_in));
    for (name, ridx) in &opponents {
        players.push(Player::new_ai(name, buy_in, *ridx));
    }

    let mut rng = thread_rng();
    let mut hand_num = 0u32;
    let mut dealer_btn = 0usize; // dealer button position

    loop {
        // Check if only one player has chips
        let still_in: Vec<usize> = (0..players.len())
            .filter(|&i| players[i].chips > 0)
            .collect();
        if still_in.len() <= 1 {
            if still_in.len() == 1 && still_in[0] == 0 {
                println!("\n{}", col(GREEN, "🏆 All opponents are broke! You win the session!"));
                won_any = true;
            } else if still_in.is_empty() || still_in[0] != 0 {
                println!("\n{}", col(RED, "💸 You're out of chips! Better luck next time."));
                // Show who holds all the chips
                for (i, p) in players.iter().enumerate() {
                    if i != 0 && p.chips > 0 {
                        println!("  {} ended with {} chips.", col(BOLD, &p.name), col(YELLOW, p.chips));
                    }
                }
            }
            break;
        }

        hand_num += 1;
        println!("\n{}", col(BOLD, format!("═══  Hand #{hand_num}  ═══")));
        println!("  Blinds: {} / {}  │  Difficulty: {}",
            col(YELLOW, small_blind),
            col(YELLOW, big_blind),
            col(CYAN, difficulty.name()),
        );

        // Reset hands and fold status
        for p in players.iter_mut() {
            p.hole.clear();
            p.folded = p.chips == 0; // sit out if broke
            p.current_bet = 0;
            p.all_in = false;
        }

        let mut deck = Deck::new_shuffled();
        let mut community: Vec<Card> = Vec::new();
        let mut pot: u32 = 0;

        // Rotate dealer button
        dealer_btn = (dealer_btn + 1) % players.len();
        let sb_idx  = (dealer_btn + 1) % players.len();
        let bb_idx  = (dealer_btn + 2) % players.len();

        // Post blinds (skip broke players)
        let sb_amount = small_blind.min(players[sb_idx].chips);
        let bb_amount = big_blind.min(players[bb_idx].chips);

        players[sb_idx].chips       -= sb_amount;
        players[sb_idx].current_bet  = sb_amount;
        pot += sb_amount;

        players[bb_idx].chips       -= bb_amount;
        players[bb_idx].current_bet  = bb_amount;
        pot += bb_amount;

        println!("  {} posts small blind ({}),  {} posts big blind ({})",
            col(BOLD, &players[sb_idx].name), col(YELLOW, sb_amount),
            col(BOLD, &players[bb_idx].name), col(YELLOW, bb_amount),
        );

        // Deal hole cards
        println!("\n  {} Dealing hole cards…", col(CYAN, "🂠"));
        for i in 0..players.len() {
            if players[i].folded { continue; }
            let c1 = deck.deal();
            let c2 = deck.deal();
            if players[i].is_human {
                println!("  {} Your hole cards:", col(BOLD, "🃏"));
                animate_deal_card(&c1);
                animate_deal_card(&c2);
            }
            players[i].hole.push(c1);
            players[i].hole.push(c2);
        }

        // Show initial table
        let initial_rank = {
            let human = players.iter().find(|p| p.is_human).unwrap();
            if human.hole.len() >= 2 {
                let mut all = human.hole.clone();
                all.extend_from_slice(&community);
                if all.len() >= 5 { Some(best_hand(&all)) } else { None }
            } else { None }
        };
        display_table(&players, &community, pot, BettingRound::PreFlop, initial_rank.as_ref());

        // ── Pre-Flop ──────────────────────────────────────────────────────────
        let cont = betting_round(
            &mut players, &mut pot, &community,
            BettingRound::PreFlop, big_blind, difficulty,
            &all_roaster_lines, &mut rng,
        );
        if !cont {
            // Someone won by everyone else folding
            let winner = showdown(&mut players, &community, pot, &all_roaster_lines);
            if winner == 0 { won_any = true; }
            if !ask_play_again_poker() { break; }
            continue;
        }

        // Check if only one remains after pre-flop
        {
            let remaining: Vec<usize> = (0..players.len()).filter(|&i| !players[i].folded).collect();
            if remaining.len() == 1 {
                let winner = showdown(&mut players, &community, pot, &all_roaster_lines);
                if winner == 0 { won_any = true; }
                if !ask_play_again_poker() { break; }
                continue;
            }
        }

        // ── Flop ─────────────────────────────────────────────────────────────
        println!("\n  {} Dealing the flop…", col(CYAN, "🂠"));
        deck.deal(); // burn card
        for _ in 0..3 {
            let c = deck.deal();
            animate_deal_card(&c);
            community.push(c);
        }
        println!("  {} {}",
            col(BOLD, "Flop:"),
            display_cards(&community),
        );

        let cont = betting_round(
            &mut players, &mut pot, &community,
            BettingRound::Flop, big_blind, difficulty,
            &all_roaster_lines, &mut rng,
        );
        if !cont {
            let winner = showdown(&mut players, &community, pot, &all_roaster_lines);
            if winner == 0 { won_any = true; }
            if !ask_play_again_poker() { break; }
            continue;
        }
        {
            let remaining: Vec<usize> = (0..players.len()).filter(|&i| !players[i].folded).collect();
            if remaining.len() == 1 {
                let winner = showdown(&mut players, &community, pot, &all_roaster_lines);
                if winner == 0 { won_any = true; }
                if !ask_play_again_poker() { break; }
                continue;
            }
        }

        // ── Turn ──────────────────────────────────────────────────────────────
        println!("\n  {} Dealing the turn…", col(CYAN, "🂠"));
        deck.deal(); // burn
        let turn = deck.deal();
        animate_deal_card(&turn);
        community.push(turn);
        println!("  {} {}",
            col(BOLD, "Turn:"),
            display_cards(&community),
        );

        let cont = betting_round(
            &mut players, &mut pot, &community,
            BettingRound::Turn, big_blind, difficulty,
            &all_roaster_lines, &mut rng,
        );
        if !cont {
            let winner = showdown(&mut players, &community, pot, &all_roaster_lines);
            if winner == 0 { won_any = true; }
            if !ask_play_again_poker() { break; }
            continue;
        }
        {
            let remaining: Vec<usize> = (0..players.len()).filter(|&i| !players[i].folded).collect();
            if remaining.len() == 1 {
                let winner = showdown(&mut players, &community, pot, &all_roaster_lines);
                if winner == 0 { won_any = true; }
                if !ask_play_again_poker() { break; }
                continue;
            }
        }

        // ── River ─────────────────────────────────────────────────────────────
        println!("\n  {} Dealing the river…", col(CYAN, "🂠"));
        deck.deal(); // burn
        let river = deck.deal();
        animate_deal_card(&river);
        community.push(river);
        println!("  {} {}",
            col(BOLD, "River:"),
            display_cards(&community),
        );

        let cont = betting_round(
            &mut players, &mut pot, &community,
            BettingRound::River, big_blind, difficulty,
            &all_roaster_lines, &mut rng,
        );
        // Proceed to showdown regardless (unless quit)

        let winner = showdown(&mut players, &community, pot, &all_roaster_lines);
        if winner == 0 {
            won_any = true;
            // Check for royal flush achievement
            let mut all_cards = players[0].hole.clone();
            all_cards.extend_from_slice(&community);
            if all_cards.len() >= 5 {
                let rank = best_hand(&all_cards);
                if rank == HandRank::RoyalFlush {
                    got_royal_flush = true;
                    println!("{}", col(YELLOW, "👑 ROYAL FLUSH! Legendary hand!"));
                }
            }
        }

        // Print commentary from AI opponents
        for p in players.iter().filter(|p| !p.is_human && !p.folded) {
            let rl = &all_roaster_lines[p.roaster_idx.unwrap_or(9) % all_roaster_lines.len()];
            let line = if winner == 0 { rl.on_player_win } else { rl.on_player_lose };
            println!("  {} {}", col(BOLD, format!("{}:", p.name)), col(MAGENTA, line));
        }

        let _ = cont; // used for early exits above

        if !ask_play_again_poker() {
            break;
        }
    }

    let elapsed = session_start.elapsed().as_secs();
    (won_any, got_royal_flush, elapsed)
}

fn ask_play_again_poker() -> bool {
    loop {
        print!("\n  🔄 Play another hand? (y/n): ");
        io::stdout().flush().expect("flush");
        match read_line().as_str() {
            "y" | "yes" => return true,
            "n" | "no"  => return false,
            _           => println!("  Please enter y or n."),
        }
    }
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn c(rank: Rank, suit: Suit) -> Card { Card { rank, suit } }

    #[test]
    fn deck_has_52_cards() {
        let d = Deck::new_shuffled();
        assert_eq!(d.cards.len(), 52);
    }

    #[test]
    fn deck_deal_reduces_count() {
        let mut d = Deck::new_shuffled();
        d.deal();
        assert_eq!(d.remaining(), 51);
    }

    #[test]
    fn hand_rank_royal_flush() {
        let cards = vec![
            c(Rank::Ace,   Suit::Spades),
            c(Rank::King,  Suit::Spades),
            c(Rank::Queen, Suit::Spades),
            c(Rank::Jack,  Suit::Spades),
            c(Rank::Ten,   Suit::Spades),
        ];
        assert_eq!(best_hand(&cards), HandRank::RoyalFlush);
    }

    #[test]
    fn hand_rank_straight_flush() {
        let cards = vec![
            c(Rank::Nine,  Suit::Hearts),
            c(Rank::Eight, Suit::Hearts),
            c(Rank::Seven, Suit::Hearts),
            c(Rank::Six,   Suit::Hearts),
            c(Rank::Five,  Suit::Hearts),
        ];
        assert_eq!(best_hand(&cards), HandRank::StraightFlush(9));
    }

    #[test]
    fn hand_rank_four_of_a_kind() {
        let cards = vec![
            c(Rank::Ace, Suit::Spades),
            c(Rank::Ace, Suit::Hearts),
            c(Rank::Ace, Suit::Clubs),
            c(Rank::Ace, Suit::Diamonds),
            c(Rank::King, Suit::Spades),
        ];
        assert_eq!(best_hand(&cards), HandRank::FourOfAKind(14, 13));
    }

    #[test]
    fn hand_rank_full_house() {
        let cards = vec![
            c(Rank::King, Suit::Spades),
            c(Rank::King, Suit::Hearts),
            c(Rank::King, Suit::Clubs),
            c(Rank::Ace,  Suit::Spades),
            c(Rank::Ace,  Suit::Hearts),
        ];
        assert_eq!(best_hand(&cards), HandRank::FullHouse(13, 14));
    }

    #[test]
    fn hand_rank_flush() {
        let cards = vec![
            c(Rank::Ace,   Suit::Clubs),
            c(Rank::Jack,  Suit::Clubs),
            c(Rank::Nine,  Suit::Clubs),
            c(Rank::Seven, Suit::Clubs),
            c(Rank::Two,   Suit::Clubs),
        ];
        assert!(matches!(best_hand(&cards), HandRank::Flush(_)));
    }

    #[test]
    fn hand_rank_straight() {
        let cards = vec![
            c(Rank::Ten,  Suit::Spades),
            c(Rank::Nine, Suit::Hearts),
            c(Rank::Eight,Suit::Clubs),
            c(Rank::Seven,Suit::Diamonds),
            c(Rank::Six,  Suit::Spades),
        ];
        assert_eq!(best_hand(&cards), HandRank::Straight(10));
    }

    #[test]
    fn hand_rank_wheel_straight() {
        // A-2-3-4-5 wheel
        let cards = vec![
            c(Rank::Ace,  Suit::Spades),
            c(Rank::Two,  Suit::Hearts),
            c(Rank::Three,Suit::Clubs),
            c(Rank::Four, Suit::Diamonds),
            c(Rank::Five, Suit::Spades),
        ];
        assert_eq!(best_hand(&cards), HandRank::Straight(5));
    }

    #[test]
    fn hand_rank_three_of_a_kind() {
        let cards = vec![
            c(Rank::Queen, Suit::Spades),
            c(Rank::Queen, Suit::Hearts),
            c(Rank::Queen, Suit::Clubs),
            c(Rank::Ace,   Suit::Spades),
            c(Rank::King,  Suit::Hearts),
        ];
        assert!(matches!(best_hand(&cards), HandRank::ThreeOfAKind(12, _)));
    }

    #[test]
    fn hand_rank_two_pair() {
        let cards = vec![
            c(Rank::Ace,   Suit::Spades),
            c(Rank::Ace,   Suit::Hearts),
            c(Rank::King,  Suit::Spades),
            c(Rank::King,  Suit::Hearts),
            c(Rank::Queen, Suit::Clubs),
        ];
        assert_eq!(best_hand(&cards), HandRank::TwoPair(14, 13, 12));
    }

    #[test]
    fn hand_rank_one_pair() {
        let cards = vec![
            c(Rank::Ace,   Suit::Spades),
            c(Rank::Ace,   Suit::Hearts),
            c(Rank::King,  Suit::Spades),
            c(Rank::Queen, Suit::Hearts),
            c(Rank::Jack,  Suit::Clubs),
        ];
        assert!(matches!(best_hand(&cards), HandRank::OnePair(14, _)));
    }

    #[test]
    fn hand_rank_high_card() {
        let cards = vec![
            c(Rank::Ace,   Suit::Spades),
            c(Rank::King,  Suit::Hearts),
            c(Rank::Jack,  Suit::Clubs),
            c(Rank::Nine,  Suit::Diamonds),
            c(Rank::Seven, Suit::Spades),
        ];
        assert!(matches!(best_hand(&cards), HandRank::HighCard(_)));
    }

    #[test]
    fn best_hand_from_seven() {
        // Hole: A♠ K♠  Community: Q♠ J♠ 10♠ 2♥ 3♦ → Royal Flush
        let cards = vec![
            c(Rank::Ace,   Suit::Spades),
            c(Rank::King,  Suit::Spades),
            c(Rank::Queen, Suit::Spades),
            c(Rank::Jack,  Suit::Spades),
            c(Rank::Ten,   Suit::Spades),
            c(Rank::Two,   Suit::Hearts),
            c(Rank::Three, Suit::Diamonds),
        ];
        assert_eq!(best_hand(&cards), HandRank::RoyalFlush);
    }

    #[test]
    fn higher_hand_wins_comparison() {
        assert!(HandRank::RoyalFlush > HandRank::StraightFlush(13));
        assert!(HandRank::FourOfAKind(14, 2) > HandRank::FullHouse(13, 14));
        assert!(HandRank::OnePair(14, vec![13, 12, 11]) > HandRank::HighCard(vec![14, 13, 12, 11, 9]));
    }

    #[test]
    fn preflop_strength_pocket_aces() {
        let hole = vec![c(Rank::Ace, Suit::Spades), c(Rank::Ace, Suit::Hearts)];
        let s = preflop_strength(&hole);
        assert!(s > 0.7, "pocket aces should score > 0.7, got {}", s);
    }

    #[test]
    fn preflop_strength_72_offsuit() {
        let hole = vec![c(Rank::Seven, Suit::Spades), c(Rank::Two, Suit::Hearts)];
        let s = preflop_strength(&hole);
        assert!(s < 0.45, "7-2 offsuit should score < 0.45, got {}", s);
    }
}
