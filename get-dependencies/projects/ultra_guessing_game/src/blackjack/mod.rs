use crossterm::{cursor, queue, terminal::ClearType};
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::fmt;
use std::io::{self, Write};
use std::thread;
use std::time::{Duration, Instant};

// ── ANSI helpers (mirrored from main.rs) ─────────────────────────────────────
const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const CYAN: &str = "\x1b[36m";
const MAGENTA: &str = "\x1b[35m";

fn col(color: &str, text: impl fmt::Display) -> String {
    format!("{}{}{}", color, text, RESET)
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

    /// Base value. Aces are counted as 11; the hand evaluator adjusts them down.
    fn value(&self) -> u32 {
        match self {
            Rank::Ace                                                  => 11,
            Rank::Two                                                  => 2,
            Rank::Three                                                => 3,
            Rank::Four                                                 => 4,
            Rank::Five                                                 => 5,
            Rank::Six                                                  => 6,
            Rank::Seven                                                => 7,
            Rank::Eight                                                => 8,
            Rank::Nine                                                 => 9,
            Rank::Ten | Rank::Jack | Rank::Queen | Rank::King         => 10,
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
            Rank::Ace,   Rank::Two,  Rank::Three, Rank::Four, Rank::Five,
            Rank::Six,   Rank::Seven,Rank::Eight, Rank::Nine, Rank::Ten,
            Rank::Jack,  Rank::Queen,Rank::King,
        ];
        let mut cards: Vec<Card> = suits
            .iter()
            .flat_map(|&s| ranks.iter().map(move |&r| Card { rank: r, suit: s }))
            .collect();
        cards.shuffle(&mut thread_rng());
        Deck { cards }
    }

    pub fn deal(&mut self) -> Card {
        self.cards.pop().expect("deck is empty")
    }

    pub fn remaining(&self) -> usize {
        self.cards.len()
    }
}

// ── Hand ──────────────────────────────────────────────────────────────────────

pub struct Hand {
    pub cards: Vec<Card>,
}

impl Hand {
    pub fn new() -> Self {
        Hand { cards: Vec::new() }
    }

    pub fn push(&mut self, card: Card) {
        self.cards.push(card);
    }

    /// Total value respecting soft-ace rules.
    pub fn value(&self) -> u32 {
        let mut total: u32 = self.cards.iter().map(|c| c.rank.value()).sum();
        let mut aces = self.cards.iter().filter(|c| c.rank == Rank::Ace).count();
        // Reduce aces from 11 → 1 as needed to avoid bust.
        while total > 21 && aces > 0 {
            total -= 10;
            aces -= 1;
        }
        total
    }

    pub fn is_blackjack(&self) -> bool {
        self.cards.len() == 2 && self.value() == 21
    }

    pub fn is_bust(&self) -> bool {
        self.value() > 21
    }

    /// Render all cards as a comma-separated string, optionally hiding the last.
    fn display_cards(&self, hide_last: bool) -> String {
        self.cards
            .iter()
            .enumerate()
            .map(|(i, c)| {
                if hide_last && i == self.cards.len() - 1 {
                    format!("{}{}??{}", YELLOW, BOLD, RESET)
                } else {
                    c.display()
                }
            })
            .collect::<Vec<_>>()
            .join("  ")
    }
}

// ── Round outcome ─────────────────────────────────────────────────────────────

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum RoundOutcome {
    PlayerWon,
    Push,
    DealerWon,
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn read_line() -> String {
    let mut buf = String::new();
    io::stdin().read_line(&mut buf).expect("Failed to read line");
    buf.trim().to_lowercase().to_string()
}

// ── Card-flip animation ───────────────────────────────────────────────────────

/// Height (in terminal lines) of a single card art block.
const CARD_HEIGHT: u16 = 5;

/// Build the lines for a face-down card frame.
/// `inner_w` is the number of character columns between the `│` borders.
/// When `inner_w == 0` the frame collapses to a bare vertical stroke.
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

/// Build the lines for a face-up card frame.
fn face_frame(card: &Card, inner_w: usize) -> Vec<String> {
    let r     = card.rank.label();    // "A", "2".."K", "10"
    let s     = card.suit.symbol();   // "♣" etc.
    let c     = card.suit.color();    // ANSI colour prefix or RESET
    let r_len = r.len();              // visible columns (1 for most, 2 for "10")

    if inner_w == 0 {
        return (0..CARD_HEIGHT as usize)
            .map(|_| format!(" {}│{}", c, RESET))
            .collect();
    }

    let bar = "─".repeat(inner_w);

    // Top row: rank left-aligned
    let top = if inner_w >= r_len {
        format!(" │{}{}{}{}{}│",
            c, BOLD, r, RESET, " ".repeat(inner_w - r_len))
    } else {
        format!(" │{}│", " ".repeat(inner_w))
    };

    // Middle row: suit symbol centred
    let lpad = inner_w.saturating_sub(1) / 2;
    let rpad = inner_w.saturating_sub(1 + lpad);
    let mid = format!(" │{}{}{}{}{}│",
        " ".repeat(lpad), c, s, RESET, " ".repeat(rpad));

    // Bottom row: rank right-aligned
    let bot = if inner_w >= r_len {
        format!(" │{}{}{}{}{}│",
            " ".repeat(inner_w - r_len), c, BOLD, r, RESET)
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

/// Erase `CARD_HEIGHT` lines above the current cursor position.
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

/// Animate a card flip.  The animation draws `CARD_HEIGHT` lines, animates
/// them in-place via crossterm cursor control, then erases those lines so the
/// caller can proceed to print the updated table cleanly below.
///
/// `reveal`: when `true` the card starts face-down (back) and flips to
/// face-up; when `false` the card is dealt face-up directly (shorter).
fn animate_flip(card: &Card, reveal: bool) {
    // Each phase: (inner content width, show_back, delay_ms)
    let phases: &[(usize, bool, u64)] = if reveal {
        // Start face-down, flip to face-up
        &[
            (7, true,  55),
            (5, true,  45),
            (3, true,  35),
            (1, true,  30),
            (0, true,  25), // edge
            (0, false, 25), // edge (face side appears)
            (1, false, 30),
            (3, false, 35),
            (5, false, 45),
            (7, false, 90),
        ]
    } else {
        // Deal face-up: brief squish from nothing then expand to full
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

    // Clear the animation area so subsequent output starts from a clean slate.
    clear_card_area(&mut out);
    out.flush().unwrap();
}

/// Animate dealing a card face-up (used for player/dealer draws).
fn animate_deal_card(card: &Card) {
    animate_flip(card, false);
}

/// Animate flipping the dealer's hidden card face-up.
fn animate_reveal_card(card: &Card) {
    animate_flip(card, true);
}

// ── Table display ─────────────────────────────────────────────────────────────

fn display_table(player: &Hand, dealer: &Hand, hide_dealer: bool, deck_remaining: usize) {
    let dealer_value_str = if hide_dealer {
        format!("{}{} + ?{}", YELLOW, dealer.cards[0].rank.value(), RESET)
    } else {
        format!("{}{}{}", CYAN, dealer.value(), RESET)
    };

    println!(
        "\n  {} {}  ({})",
        col(BOLD, "Dealer:"),
        dealer.display_cards(hide_dealer),
        dealer_value_str,
    );
    println!(
        "  {} {}  ({})",
        col(BOLD, "You:   "),
        player.display_cards(false),
        col(CYAN, player.value().to_string()),
    );
    println!(
        "  {} {}",
        col(MAGENTA, "🂠 Deck:"),
        col(YELLOW, {
            let s = if deck_remaining == 1 { "" } else { "s" };
            format!("{} card{} remaining", deck_remaining, s)
        }),
    );
}

// ── Roaster commentary ────────────────────────────────────────────────────────

pub struct BlackjackRoasterLines {
    pub on_blackjack:    &'static str,
    pub on_win:          &'static str,
    pub on_push:         &'static str,
    pub on_bust:         &'static str,
    pub on_dealer_win:   &'static str,
    pub on_hit:          &'static str,
    pub on_stand:        &'static str,
    pub dealer_bust:     &'static str,
}

/// Pick lines based on roaster index (0-9 matching Roaster enum order in main.rs).
pub fn roaster_lines(roaster_idx: usize) -> BlackjackRoasterLines {
    match roaster_idx {
        // Gordon Ramsay
        0 => BlackjackRoasterLines {
            on_blackjack:  "🎉 BLACKJACK! You absolute legend! Almost brings a tear to my eye!",
            on_win:        "✅ Well done! You beat the dealer like a proper professional!",
            on_push:       "🤝 A push. Perfectly adequate – like a well-seasoned risotto.",
            on_bust:       "💥 BUST! You idiot sandwich! You went over 21!",
            on_dealer_win: "❌ Dealer wins. Absolutely pathetic. Donkey.",
            on_hit:        "🃏 Bold move. Hitting on that. Let's see if you're not an idiot.",
            on_stand:      "🛑 Standing. Smart, or are you just scared? Either way, bold.",
            dealer_bust:   "🎉 The dealer BUSTED! Even Gordon is impressed!",
        },
        // Uncle Roger
        1 => BlackjackRoasterLines {
            on_blackjack:  "🎉 BLACKJACK! Fuiyoh! Uncle Roger very impressed! MSG approved!",
            on_win:        "✅ You win! Haiyaa, you actually did it! Uncle Roger proud!",
            on_push:       "🤝 Push. Aiyah, nobody wins! Like using sieve as wok lid!",
            on_bust:       "💥 BUST! Haiyaa! You bust like Jamie Oliver's fried rice attempt!",
            on_dealer_win: "❌ Dealer wins. Emotionally damage Uncle Roger so much.",
            on_hit:        "🃏 Hit again! Haiyaa, you sure ah? Okay lah, up to you.",
            on_stand:      "🛑 Stand. Smart move. Uncle Roger approve this decision.",
            dealer_bust:   "🎉 Fuiyoh! Dealer bust! Uncle Roger very excited!",
        },
        // Rick Astley
        2 => BlackjackRoasterLines {
            on_blackjack:  "🎉 BLACKJACK! Never gonna give you up – you got 21! Incredible!",
            on_win:        "✅ You win! Never gonna let you down – you played perfectly!",
            on_push:       "🤝 Push. Never gonna run around – we're even. Balanced.",
            on_bust:       "💥 BUST! Never gonna make you cry... but ouch, that's 22+.",
            on_dealer_win: "❌ Dealer wins. You've known the rules – that's the risk!",
            on_hit:        "🃏 Hit! A full commitment's what I'm thinking of – go for it!",
            on_stand:      "🛑 Stand! Never gonna say goodbye to a safe position.",
            dealer_bust:   "🎉 Dealer busts! Never gonna give this win up!",
        },
        // Simon Cowell
        3 => BlackjackRoasterLines {
            on_blackjack:  "🎉 Blackjack. That was actually... extraordinary. Well done.",
            on_win:        "✅ You win. Decent. I'm mildly impressed – that's rare.",
            on_push:       "🤝 A push. It's a draw from me. Unremarkable.",
            on_bust:       "💥 Bust. That was dreadful. Absolutely dreadful.",
            on_dealer_win: "❌ Dealer wins. It's a no from me. You simply weren't good enough.",
            on_hit:        "🃏 Hitting. Bold, but probably foolish. We'll see.",
            on_stand:      "🛑 Standing. Safe. Uninspiring, but safe.",
            dealer_bust:   "🎉 The dealer busts. Not bad. Not great. You'll take it.",
        },
        // Nikki Glaser
        4 => BlackjackRoasterLines {
            on_blackjack:  "🎉 BLACKJACK! Okay yes, that was impressive. I'll give you that!",
            on_win:        "✅ You win! See, you CAN make good decisions! Proud of you babe!",
            on_push:       "🤝 Push. A draw. Like my patience for bad card players.",
            on_bust:       "💥 Bust! Babe, that was... a choice. A bad one.",
            on_dealer_win: "❌ Dealer wins. That's embarrassing. For you.",
            on_hit:        "🃏 Hitting. Bold. Reckless, but bold.",
            on_stand:      "🛑 Standing. Smart girl/guy. Knew you had it in you.",
            dealer_bust:   "🎉 Dealer busts! Yes! Now THAT is how you win!",
        },
        // Joan Rivers
        5 => BlackjackRoasterLines {
            on_blackjack:  "🎉 BLACKJACK! Oh darling, fabulous! Simply divine! 21 on the nose!",
            on_win:        "✅ You win! Darling you look GORGEOUS winning like that!",
            on_push:       "🤝 Push. A draw, honey. At least you didn't embarrass yourself.",
            on_bust:       "💥 BUST! Oh honey, that hand looks like my ex-husband – a disaster.",
            on_dealer_win: "❌ Dealer wins. Darling, that was tragic. Simply tragic.",
            on_hit:        "🃏 Hitting. Oh honey, you're brave. Or delusional. Hard to tell.",
            on_stand:      "🛑 Standing. Wise choice, darling. Very becoming.",
            dealer_bust:   "🎉 Dealer busts! Oh fabulous! Can we talk about this victory?!",
        },
        // CaseOh
        6 => BlackjackRoasterLines {
            on_blackjack:  "🎉 YOOO BLACKJACK! CHAT! CHAT! THIS PERSON IS A LEGEND! GG!",
            on_win:        "✅ WE WIN! CHAT! THEY ACTUALLY DID IT! LETS GOOO! W!",
            on_push:       "🤝 Push. CHAT it's a tie. Stress eating Takis over this.",
            on_bust:       "💥 CHAT THEY BUSTED! L! MASSIVE L! That hurt to watch bro!",
            on_dealer_win: "❌ Dealer wins. CHAT. That was pain. Actual pain. L + ratio.",
            on_hit:        "🃏 HIT! CHAT they're going for it! This is CONTENT!",
            on_stand:      "🛑 Stand! CHAT they're playing it safe! Smart or scared? Both?",
            dealer_bust:   "🎉 DEALER BUST! YOOOO! CHAT WE EAT! GG!",
        },
        // Gen X
        7 => BlackjackRoasterLines {
            on_blackjack:  "Blackjack. Cool. Whatever. That was actually impressive I guess.",
            on_win:        "You win. Not that it matters. Good job or whatever.",
            on_push:       "Push. A tie. As if anyone cares.",
            on_bust:       "Bust. Whatever. You went over. Classic.",
            on_dealer_win: "Dealer wins. Meh. This game is rigged anyway.",
            on_hit:        "Hitting. Bold. Reckless. Very Gen X of you.",
            on_stand:      "Standing. Playing it safe. How very boring.",
            dealer_bust:   "Dealer busts. Cool. Whatever. A win is a win.",
        },
        // Millennial
        8 => BlackjackRoasterLines {
            on_blackjack:  "🎉 BLACKJACK! Bestie you literally ate that up! I'm OBSESSED!",
            on_win:        "✅ YOU WIN! That's so iconic! I'm literally crying happy tears!",
            on_push:       "🤝 Push bestie. A tie. That's still a vibe though!",
            on_bust:       "💥 BUST! Oh no bestie! That hit different (badly)! I'm deceased!",
            on_dealer_win: "❌ Dealer wins. I'm having a moment. My anxiety is skyrocketing!",
            on_hit:        "🃏 Hitting! That's a whole mood! You're so brave bestie!",
            on_stand:      "🛑 Standing! Smart! That's giving security! Love that for you!",
            dealer_bust:   "🎉 DEALER BUSTED! YASSS! We LOVE to see it! This is everything!",
        },
        // Gen Z
        _ => BlackjackRoasterLines {
            on_blackjack:  "🎉 BLACKJACK FR FR! NO CAP YOU ATE THAT! BUSSIN! PURR BESTIE!",
            on_win:        "✅ W! You cooked! No cap that was lowkey fire! We love to see it!",
            on_push:       "🤝 Push bestie. Tie. Mid outcome ngl but at least you didn't L.",
            on_bust:       "💥 BUST! You are so cooked fr! Massive L bestie! Deadass busted!",
            on_dealer_win: "❌ Dealer wins. That's an L. Ratio. You fell off bestie.",
            on_hit:        "🃏 Hitting! Going for it fr fr! Bussin move or L? We'll see!",
            on_stand:      "🛑 Standing! Playing it safe no cap. Low key smart move bestie.",
            dealer_bust:   "🎉 DEALER BUST! W! You ate and left no crumbs! Periodt!",
        },
    }
}

// ── Main game function ────────────────────────────────────────────────────────

/// Play one or more rounds of Blackjack.
/// Returns `(won_at_least_once, got_natural_blackjack, elapsed_secs)`.
pub fn play(roaster_idx: usize, profane: bool) -> (bool, bool, u64) {
    let lines = roaster_lines(roaster_idx);

    println!("\n{}", col(CYAN, "─".repeat(62)));
    println!("{}", col(BOLD, "  🃏  IRON AGE BLACKJACK  🃏"));
    println!("{}", col(CYAN, "─".repeat(62)));
    println!("  Beat the dealer to 21 without going over.");
    println!("  Commands: {} = take another card  │  {} = keep your hand",
        col(YELLOW, "h / hit"),
        col(YELLOW, "s / stand"),
    );
    println!("  Type {} at any time to quit.\n", col(YELLOW, "q / quit"));

    let session_start = Instant::now();
    let mut won_any     = false;
    let mut got_natural = false;
    let mut chips = 100u32;

    'game: loop {
        if chips == 0 {
            println!("\n{}", col(RED, "💸 You're out of chips! Game over."));
            break;
        }

        println!("\n{} {} chip{}", col(BOLD, "💰 Chips:"), col(YELLOW, chips), if chips == 1 { "" } else { "s" });

        // ── Betting ──────────────────────────────────────────────────────────
        let bet = loop {
            print!("  Place your bet (1–{}): ", chips);
            io::stdout().flush().expect("Failed to flush stdout");
            let raw = read_line();
            if raw == "q" || raw == "quit" {
                break 'game;
            }
            match raw.parse::<u32>() {
                Ok(n) if n >= 1 && n <= chips => break n,
                _ => println!("{}", col(RED, format!("  ❌ Enter a number between 1 and {}.", chips))),
            }
        };

        // ── Deal ─────────────────────────────────────────────────────────────
        let mut deck   = Deck::new_shuffled();
        let mut player = Hand::new();
        let mut dealer = Hand::new();

        println!("  {} Dealing cards...", col(CYAN, "🂠"));

        let c = deck.deal();
        player.push(c);
        animate_deal_card(&c);
        let c = deck.deal();
        dealer.push(c);
        animate_deal_card(&c);
        let c = deck.deal();
        player.push(c);
        animate_deal_card(&c);
        // Dealer's second card is dealt face-down; no flip animation for hidden card
        dealer.push(deck.deal());

        display_table(&player, &dealer, /*hide_dealer=*/true, deck.remaining());

        // ── Natural blackjack check ──────────────────────────────────────────
        if player.is_blackjack() {
            println!("\n{}", col(GREEN, lines.on_blackjack));
            display_table(&player, &dealer, false, deck.remaining());
            if dealer.is_blackjack() {
                println!("{}", col(MAGENTA, "  Dealer also has blackjack – it's a push!"));
                println!("  You keep your {} chip bet.", bet);
            } else {
                let payout = bet + bet / 2; // 3:2 payout; odd bets are rounded down (standard casino practice)
                chips += payout;
                got_natural = true;
                won_any     = true;
                println!("  💰 Blackjack pays 3:2 – you win {} chips! (Total: {})", payout, chips);
            }
            if !ask_play_again_blackjack() { break 'game; }
            continue;
        }

        // ── Player's turn ────────────────────────────────────────────────────
        'player_turn: loop {
            if player.is_bust() {
                println!("\n{}", col(RED, lines.on_bust));
                chips = chips.saturating_sub(bet);
                println!("  You lose {} chip{}. (Total: {})", bet, if bet == 1 { "" } else { "s" }, chips);
                if !ask_play_again_blackjack() { break 'game; }
                continue 'game;
            }

            print!("  {} or {}? (h/s/q): ",
                col(YELLOW, "Hit"),
                col(YELLOW, "Stand"),
            );
            io::stdout().flush().expect("Failed to flush stdout");
            let action = read_line();

            match action.as_str() {
                "h" | "hit" => {
                    println!("{}", col(CYAN, lines.on_hit));
                    let c = deck.deal();
                    player.push(c);
                    animate_deal_card(&c);
                    display_table(&player, &dealer, true, deck.remaining());
                    if player.is_bust() {
                        println!("\n{}", col(RED, lines.on_bust));
                        chips = chips.saturating_sub(bet);
                        println!("  You lose {} chip{}. (Total: {})", bet, if bet == 1 { "" } else { "s" }, chips);
                        if !ask_play_again_blackjack() { break 'game; }
                        continue 'game;
                    }
                    if player.value() == 21 {
                        println!("{}", col(GREEN, "  21 – perfect! Standing automatically."));
                        break 'player_turn;
                    }
                }
                "s" | "stand" => {
                    println!("{}", col(CYAN, lines.on_stand));
                    break 'player_turn;
                }
                "q" | "quit" => break 'game,
                _ => {
                    println!("{}", col(RED, "  ❌ Type 'h' to hit or 's' to stand."));
                }
            }
        }

        // ── Dealer's turn (hit to 17) ────────────────────────────────────────
        println!("\n  {} reveals hidden card…", col(BOLD, "Dealer"));
        animate_reveal_card(&dealer.cards[1]);
        display_table(&player, &dealer, false, deck.remaining());

        while dealer.value() < 17 {
            let c = deck.deal();
            dealer.push(c);
            animate_deal_card(&c);
            display_table(&player, &dealer, false, deck.remaining());
        }

        // ── Determine outcome ────────────────────────────────────────────────
        let player_val = player.value();
        let dealer_val = dealer.value();

        let outcome = if dealer.is_bust() {
            println!("{}", col(GREEN, lines.dealer_bust));
            RoundOutcome::PlayerWon
        } else if player_val > dealer_val {
            RoundOutcome::PlayerWon
        } else if player_val == dealer_val {
            RoundOutcome::Push
        } else {
            RoundOutcome::DealerWon
        };

        match outcome {
            RoundOutcome::PlayerWon => {
                println!("{}", col(GREEN, lines.on_win));
                chips += bet;
                won_any = true;
                println!("  💰 You win {} chip{}! (Total: {})", bet, if bet == 1 { "" } else { "s" }, chips);
            }
            RoundOutcome::Push => {
                println!("{}", col(MAGENTA, lines.on_push));
                println!("  You keep your {} chip bet. (Total: {})", bet, chips);
            }
            RoundOutcome::DealerWon => {
                println!("{}", col(RED, lines.on_dealer_win));
                chips = chips.saturating_sub(bet);
                println!("  You lose {} chip{}. (Total: {})", bet, if bet == 1 { "" } else { "s" }, chips);
            }
        }

        let _ = profane; // reserved for future profanity-filtered commentary

        if !ask_play_again_blackjack() {
            break 'game;
        }
    }

    let elapsed = session_start.elapsed().as_secs();
    (won_any, got_natural, elapsed)
}

fn ask_play_again_blackjack() -> bool {
    loop {
        print!("\n  🔄 Play another hand? (y/n): ");
        io::stdout().flush().expect("Failed to flush stdout");
        match read_line().as_str() {
            "y" | "yes" => return true,
            "n" | "no"  => return false,
            _           => println!("  ❌ Just y or n please."),
        }
    }
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn card(rank: Rank, suit: Suit) -> Card {
        Card { rank, suit }
    }

    #[test]
    fn hand_value_no_aces() {
        let mut h = Hand::new();
        h.push(card(Rank::Seven, Suit::Hearts));
        h.push(card(Rank::Nine,  Suit::Clubs));
        assert_eq!(h.value(), 16);
    }

    #[test]
    fn hand_value_soft_ace() {
        let mut h = Hand::new();
        h.push(card(Rank::Ace,  Suit::Spades));
        h.push(card(Rank::Six,  Suit::Hearts));
        assert_eq!(h.value(), 17); // soft 17
    }

    #[test]
    fn hand_value_hard_ace() {
        let mut h = Hand::new();
        h.push(card(Rank::Ace,   Suit::Spades));
        h.push(card(Rank::Six,   Suit::Hearts));
        h.push(card(Rank::Eight, Suit::Clubs));
        assert_eq!(h.value(), 15); // 11 + 6 + 8 = 25 → reduce ace: 1 + 6 + 8 = 15
    }

    #[test]
    fn blackjack_detection() {
        let mut h = Hand::new();
        h.push(card(Rank::Ace,  Suit::Clubs));
        h.push(card(Rank::King, Suit::Diamonds));
        assert!(h.is_blackjack());
        assert_eq!(h.value(), 21);
    }

    #[test]
    fn three_card_21_is_not_blackjack() {
        let mut h = Hand::new();
        h.push(card(Rank::Seven, Suit::Hearts));
        h.push(card(Rank::Seven, Suit::Clubs));
        h.push(card(Rank::Seven, Suit::Diamonds));
        assert!(!h.is_blackjack());
        assert_eq!(h.value(), 21);
    }

    #[test]
    fn bust_detection() {
        let mut h = Hand::new();
        h.push(card(Rank::King, Suit::Spades));
        h.push(card(Rank::Queen, Suit::Hearts));
        h.push(card(Rank::Five, Suit::Clubs));
        assert!(h.is_bust());
        assert_eq!(h.value(), 25);
    }

    #[test]
    fn two_aces_value() {
        let mut h = Hand::new();
        h.push(card(Rank::Ace, Suit::Hearts));
        h.push(card(Rank::Ace, Suit::Spades));
        // 11 + 11 = 22 → reduce one ace → 11 + 1 = 12
        assert_eq!(h.value(), 12);
    }

    #[test]
    fn deck_has_52_cards() {
        let d = Deck::new_shuffled();
        assert_eq!(d.cards.len(), 52);
    }

    #[test]
    fn deck_remaining_decrements_on_deal() {
        let mut d = Deck::new_shuffled();
        assert_eq!(d.remaining(), 52);
        d.deal();
        assert_eq!(d.remaining(), 51);
        d.deal();
        d.deal();
        assert_eq!(d.remaining(), 49);
    }
}
