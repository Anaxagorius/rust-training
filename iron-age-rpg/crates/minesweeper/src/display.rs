use std::io::{self, Write};
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyModifiers},
    queue,
    style::{Attribute, Color, Print, ResetColor, SetAttribute, SetForegroundColor},
    terminal::{self, ClearType},
};

use crate::board::{Board, GameStatus, Level};

// ─── Iron-Age themed symbols ────────────────────────────────────────────────

/// Unrevealed ground (safe or unknown).
const HIDDEN: &str = "▓";
/// Rune marker planted by the player.
const FLAG: &str = "⚑";
/// A goblin trap (shown only on game over).
const TRAP: &str = "✸";
/// The trap that ended the game.
const TRAP_HIT: &str = "☠";
/// Safe revealed ground with no adjacent traps.
const SAFE: &str = "·";

/// Colour for each adjacency count, matching the classic colour scheme.
fn adj_colour(n: u8) -> Color {
    match n {
        1 => Color::Blue,
        2 => Color::DarkGreen,
        3 => Color::Red,
        4 => Color::DarkBlue,
        5 => Color::DarkRed,
        6 => Color::Cyan,
        7 => Color::Magenta,
        8 => Color::Grey,
        _ => Color::White,
    }
}

/// Roman-numeral labels for adjacency counts 1–8 (fitting the Iron Age theme).
fn roman(n: u8) -> &'static str {
    match n {
        1 => "I",
        2 => "II",
        3 => "III",
        4 => "IV",
        5 => "V",
        6 => "VI",
        7 => "VII",
        8 => "VIII",
        _ => " ",
    }
}

// ─── Top-level rendering ────────────────────────────────────────────────────

/// Draw / refresh the entire game screen.
pub fn draw(board: &Board, cursor_row: usize, cursor_col: usize, elapsed_secs: u64)
    -> io::Result<()>
{
    let mut stdout = io::stdout();

    queue!(stdout, cursor::MoveTo(0, 0), terminal::Clear(ClearType::All))?;

    draw_header(&mut stdout, board, elapsed_secs)?;
    draw_grid(&mut stdout, board, cursor_row, cursor_col)?;
    draw_footer(&mut stdout, &board.status)?;

    stdout.flush()
}

fn draw_header(stdout: &mut impl Write, board: &Board, elapsed_secs: u64) -> io::Result<()> {
    // Title banner
    queue!(
        stdout,
        SetAttribute(Attribute::Bold),
        SetForegroundColor(Color::Yellow),
        Print("  ╔══════════════════════════════════════════╗\n"),
        Print("  ║   ⚔  IRON AGE MINESWEEPER: RUINS OF PERIL  ⚔   ║\n"),
        Print("  ╚══════════════════════════════════════════╝\n"),
        ResetColor,
        SetAttribute(Attribute::Reset),
    )?;

    // Status bar
    let remaining = board.traps_remaining();
    let mins = elapsed_secs / 60;
    let secs = elapsed_secs % 60;
    queue!(
        stdout,
        SetForegroundColor(Color::DarkYellow),
        Print(format!("  Level: {:20}  ", board.level.name())),
        SetForegroundColor(Color::Red),
        Print(format!("{} {} remaining  ", FLAG, remaining)),
        SetForegroundColor(Color::Cyan),
        Print(format!("⏱ {:02}:{:02}\n\n", mins, secs)),
        ResetColor,
    )
}

fn draw_grid(
    stdout: &mut impl Write,
    board: &Board,
    cursor_row: usize,
    cursor_col: usize,
) -> io::Result<()> {
    // Column index header
    queue!(stdout, SetForegroundColor(Color::DarkGrey), Print("     "))?;
    for c in 0..board.cols {
        queue!(stdout, Print(format!("{:2}", (c + 1) % 100)))?;
    }
    queue!(stdout, Print("\n"), ResetColor)?;

    for row in 0..board.rows {
        // Row index
        queue!(
            stdout,
            SetForegroundColor(Color::DarkGrey),
            Print(format!("  {:2} ", row + 1)),
            ResetColor,
        )?;

        for col in 0..board.cols {
            let cell = board.cell(row, col);
            let is_cursor = row == cursor_row && col == cursor_col;

            if is_cursor {
                queue!(stdout, SetAttribute(Attribute::Reverse))?;
            }

            if cell.revealed {
                if cell.is_trap {
                    // Distinguish the trap that was hit vs others revealed on loss.
                    let symbol = if board.status == GameStatus::Lost && is_cursor {
                        TRAP_HIT
                    } else {
                        TRAP
                    };
                    queue!(
                        stdout,
                        SetForegroundColor(Color::Red),
                        Print(format!(" {}", symbol)),
                        ResetColor,
                    )?;
                } else if cell.adjacent == 0 {
                    queue!(
                        stdout,
                        SetForegroundColor(Color::DarkGrey),
                        Print(format!(" {}", SAFE)),
                        ResetColor,
                    )?;
                } else {
                    let label = roman(cell.adjacent);
                    // Pad to 2 wide, right-align.
                    let padded = if label.len() == 1 {
                        format!(" {}", label)
                    } else {
                        format!("{}", label)
                    };
                    queue!(
                        stdout,
                        SetForegroundColor(adj_colour(cell.adjacent)),
                        SetAttribute(Attribute::Bold),
                        Print(padded),
                        ResetColor,
                        SetAttribute(Attribute::Reset),
                    )?;
                }
            } else if cell.flagged {
                queue!(
                    stdout,
                    SetForegroundColor(Color::Magenta),
                    SetAttribute(Attribute::Bold),
                    Print(format!(" {}", FLAG)),
                    ResetColor,
                    SetAttribute(Attribute::Reset),
                )?;
            } else {
                queue!(
                    stdout,
                    SetForegroundColor(Color::DarkGrey),
                    Print(format!(" {}", HIDDEN)),
                    ResetColor,
                )?;
            }

            if is_cursor {
                queue!(stdout, SetAttribute(Attribute::Reset))?;
            }
        }

        queue!(stdout, Print("\n"))?;
    }

    Ok(())
}

fn draw_footer(stdout: &mut impl Write, status: &GameStatus) -> io::Result<()> {
    queue!(stdout, Print("\n"))?;
    match status {
        GameStatus::Playing => {
            queue!(
                stdout,
                SetForegroundColor(Color::DarkGrey),
                Print("  Arrows: move  |  Space/Enter: reveal  |  F: flag  |  R: restart  |  Q: quit\n"),
                ResetColor,
            )?;
        }
        GameStatus::Lost => {
            queue!(
                stdout,
                SetForegroundColor(Color::Red),
                SetAttribute(Attribute::Bold),
                Print("  ☠  You trod upon a goblin trap!  Your legend ends here.\n"),
                SetAttribute(Attribute::Reset),
                ResetColor,
                SetForegroundColor(Color::DarkGrey),
                Print("  Press R to try again  |  Q to quit\n"),
                ResetColor,
            )?;
        }
        GameStatus::Won => {
            queue!(
                stdout,
                SetForegroundColor(Color::Yellow),
                SetAttribute(Attribute::Bold),
                Print("  ⚔  Victory!  The ruins are cleared.  Your name echoes through the ages!\n"),
                SetAttribute(Attribute::Reset),
                ResetColor,
                SetForegroundColor(Color::DarkGrey),
                Print("  Press R to play again  |  Q to quit\n"),
                ResetColor,
            )?;
        }
    }
    Ok(())
}

// ─── Level selection screen ─────────────────────────────────────────────────

pub fn draw_level_select(selected: usize) -> io::Result<()> {
    let mut stdout = io::stdout();
    queue!(stdout, cursor::MoveTo(0, 0), terminal::Clear(ClearType::All))?;

    queue!(
        stdout,
        SetAttribute(Attribute::Bold),
        SetForegroundColor(Color::Yellow),
        Print("\n  ╔══════════════════════════════════════════╗\n"),
        Print("  ║   ⚔  IRON AGE MINESWEEPER: RUINS OF PERIL  ⚔   ║\n"),
        Print("  ╚══════════════════════════════════════════╝\n\n"),
        ResetColor,
        SetAttribute(Attribute::Reset),
        SetForegroundColor(Color::DarkYellow),
        Print("  Choose your trial, adventurer:\n\n"),
        ResetColor,
    )?;

    let levels = [Level::Peasant, Level::Knight, Level::Champion];
    for (i, lvl) in levels.iter().enumerate() {
        let cursor_mark = if i == selected { "▶" } else { " " };
        let (rows, cols, traps) = (lvl.rows(), lvl.cols(), lvl.traps());

        if i == selected {
            queue!(stdout, SetAttribute(Attribute::Bold), SetForegroundColor(Color::Yellow))?;
        } else {
            queue!(stdout, SetForegroundColor(Color::White))?;
        }

        queue!(
            stdout,
            Print(format!("  {} [{}]  {}\n", cursor_mark, i + 1, lvl.name())),
        )?;

        queue!(stdout, SetForegroundColor(Color::DarkGrey), SetAttribute(Attribute::Reset))?;
        queue!(
            stdout,
            Print(format!("       {}×{} grid, {} goblin traps\n", rows, cols, traps)),
            Print(format!("       {}\n\n", lvl.flavour())),
            ResetColor,
        )?;
    }

    queue!(
        stdout,
        SetForegroundColor(Color::DarkGrey),
        Print("  Up/Down: navigate  |  Enter: begin  |  1/2/3: quick select  |  Q: quit\n"),
        ResetColor,
    )?;

    stdout.flush()
}

// ─── Input reading ──────────────────────────────────────────────────────────

pub enum InputAction {
    Move(i32, i32),
    Reveal,
    Flag,
    Restart,
    Quit,
    None,
}

pub fn read_action() -> io::Result<InputAction> {
    loop {
        match event::read()? {
            Event::Key(key) => {
                if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
                    return Ok(InputAction::Quit);
                }
                return Ok(match key.code {
                    KeyCode::Up    | KeyCode::Char('w') | KeyCode::Char('k') =>
                        InputAction::Move(-1, 0),
                    KeyCode::Down  | KeyCode::Char('s') | KeyCode::Char('j') =>
                        InputAction::Move(1, 0),
                    KeyCode::Left  | KeyCode::Char('a') | KeyCode::Char('h') =>
                        InputAction::Move(0, -1),
                    KeyCode::Right | KeyCode::Char('d') | KeyCode::Char('l') =>
                        InputAction::Move(0, 1),
                    KeyCode::Enter | KeyCode::Char(' ') => InputAction::Reveal,
                    KeyCode::Char('f') | KeyCode::Char('F') => InputAction::Flag,
                    KeyCode::Char('r') | KeyCode::Char('R') => InputAction::Restart,
                    KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc =>
                        InputAction::Quit,
                    _ => InputAction::None,
                });
            }
            Event::Resize(_, _) => return Ok(InputAction::None),
            _ => {}
        }
    }
}

/// Input reading for the level-select screen.
pub enum MenuAction {
    Up,
    Down,
    Select(usize),
    Quit,
    None,
}

pub fn read_menu_action() -> io::Result<MenuAction> {
    loop {
        match event::read()? {
            Event::Key(key) => {
                if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
                    return Ok(MenuAction::Quit);
                }
                return Ok(match key.code {
                    KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('k') =>
                        MenuAction::Up,
                    KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('j') =>
                        MenuAction::Down,
                    KeyCode::Enter | KeyCode::Char(' ') => MenuAction::Select(usize::MAX),
                    KeyCode::Char('1') => MenuAction::Select(0),
                    KeyCode::Char('2') => MenuAction::Select(1),
                    KeyCode::Char('3') => MenuAction::Select(2),
                    KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc =>
                        MenuAction::Quit,
                    _ => MenuAction::None,
                });
            }
            Event::Resize(_, _) => return Ok(MenuAction::None),
            _ => {}
        }
    }
}
