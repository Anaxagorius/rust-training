use std::io::{self, Write};
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyModifiers},
    queue,
    style::{Attribute, Color, Print, ResetColor, SetAttribute, SetForegroundColor},
    terminal::{self, ClearType},
};

use super::board::{Board, GameStatus, Piece, Turn};

// ── Symbols ───────────────────────────────────────────────────────────────────

/// Player's regular piece.
const PLAYER_REG:  &str = "o";
/// Player's king.
const PLAYER_KING: &str = "O";
/// AI's regular piece.
const AI_REG:  &str = "x";
/// AI's king.
const AI_KING: &str = "X";
/// Light (non-playable) square fill.
const LIGHT:   &str = "##";
/// Dark empty square.
const DARK_EMPTY: &str = "  ";

// ── Top-level rendering ───────────────────────────────────────────────────────

/// Render the entire checkers TUI.
pub fn draw(board: &Board, cursor_row: usize, cursor_col: usize) -> io::Result<()> {
    let mut stdout = io::stdout();
    queue!(stdout, cursor::MoveTo(0, 0), terminal::Clear(ClearType::All))?;

    draw_header(&mut stdout)?;
    draw_status(&mut stdout, board)?;
    draw_board(&mut stdout, board, cursor_row, cursor_col)?;
    draw_footer(&mut stdout, &board.status)?;

    stdout.flush()
}

fn draw_header(stdout: &mut impl Write) -> io::Result<()> {
    queue!(
        stdout,
        SetAttribute(Attribute::Bold),
        SetForegroundColor(Color::Yellow),
        Print("  ╔══════════════════════════════════════════╗\n"),
        Print("  ║        ♟  IRON AGE CHECKERS  ♟           ║\n"),
        Print("  ╚══════════════════════════════════════════╝\n\n"),
        ResetColor,
        SetAttribute(Attribute::Reset),
    )
}

fn draw_status(stdout: &mut impl Write, board: &Board) -> io::Result<()> {
    // Piece legend.
    queue!(
        stdout,
        SetForegroundColor(Color::Red),
        SetAttribute(Attribute::Bold),
        Print(format!("  You ({}): o=piece  O=king", PLAYER_REG)),
        ResetColor,
        SetAttribute(Attribute::Reset),
        Print("    "),
        SetForegroundColor(Color::Cyan),
        SetAttribute(Attribute::Bold),
        Print(format!("AI ({}): x=piece  X=king", AI_REG)),
        ResetColor,
        SetAttribute(Attribute::Reset),
        Print("\n"),
    )?;

    // Piece counts and whose turn it is.
    let turn_str = match board.turn {
        Turn::Player => {
            format!(
                "  \x1b[1m\x1b[31m▶ Your turn\x1b[0m        Pieces: \x1b[31m{} ×{}\x1b[0m  |  \x1b[36m{} ×{}\x1b[0m\n\n",
                PLAYER_REG, board.player_pieces, AI_REG, board.ai_pieces
            )
        }
        Turn::Ai => {
            format!(
                "    AI is thinking…    Pieces: \x1b[31m{} ×{}\x1b[0m  |  \x1b[36m{} ×{}\x1b[0m\n\n",
                PLAYER_REG, board.player_pieces, AI_REG, board.ai_pieces
            )
        }
    };
    queue!(stdout, Print(turn_str))
}

fn draw_board(
    stdout: &mut impl Write,
    board: &Board,
    cursor_row: usize,
    cursor_col: usize,
) -> io::Result<()> {
    let selected      = board.selected;
    let valid_dests   = board.valid_dest_positions();

    // Column header  (a–h).
    queue!(stdout, SetForegroundColor(Color::DarkGrey), Print("      "))?;
    for c in 0..8u8 {
        queue!(stdout, Print(format!(" {:2}", (b'a' + c) as char)))?;
    }
    queue!(stdout, Print("\n"), ResetColor)?;

    for row in 0..8 {
        // Row label (1 = top, 8 = bottom).
        queue!(
            stdout,
            SetForegroundColor(Color::DarkGrey),
            Print(format!("  {:2}  ", row + 1)),
            ResetColor,
        )?;

        for col in 0..8 {
            let is_cursor   = row == cursor_row && col == cursor_col;
            let is_selected = selected == Some((row, col));
            let is_dest     = valid_dests.contains(&(row, col));
            let is_dark     = (row + col) % 2 == 1;
            let piece       = board.cells[row][col];

            // Apply cursor inversion first.
            if is_cursor {
                queue!(stdout, SetAttribute(Attribute::Reverse))?;
            }

            if !is_dark {
                // Light square – always just fill.
                queue!(stdout, SetForegroundColor(Color::DarkGrey), Print(LIGHT), ResetColor)?;
            } else if is_selected {
                // The piece the player has picked – highlight in yellow with the correct symbol.
                queue!(
                    stdout,
                    SetForegroundColor(Color::Yellow),
                    SetAttribute(Attribute::Bold),
                    Print(format!("{} ", piece_symbol(piece))),
                    ResetColor,
                    SetAttribute(Attribute::Reset),
                )?;
            } else if is_dest {
                // Valid landing square – show in green.
                let sym = piece_symbol(piece);
                queue!(
                    stdout,
                    SetForegroundColor(Color::Green),
                    SetAttribute(Attribute::Bold),
                    Print(if piece.is_empty() { "· ".to_string() } else { format!("{} ", sym) }),
                    ResetColor,
                    SetAttribute(Attribute::Reset),
                )?;
            } else {
                // Normal square.
                match piece {
                    Piece::Empty => {
                        queue!(stdout, Print(DARK_EMPTY))?;
                    }
                    Piece::Player => {
                        queue!(
                            stdout,
                            SetForegroundColor(Color::Red),
                            Print(format!("{} ", PLAYER_REG)),
                            ResetColor,
                        )?;
                    }
                    Piece::PlayerKing => {
                        queue!(
                            stdout,
                            SetForegroundColor(Color::Red),
                            SetAttribute(Attribute::Bold),
                            Print(format!("{} ", PLAYER_KING)),
                            ResetColor,
                            SetAttribute(Attribute::Reset),
                        )?;
                    }
                    Piece::Ai => {
                        queue!(
                            stdout,
                            SetForegroundColor(Color::Cyan),
                            Print(format!("{} ", AI_REG)),
                            ResetColor,
                        )?;
                    }
                    Piece::AiKing => {
                        queue!(
                            stdout,
                            SetForegroundColor(Color::Cyan),
                            SetAttribute(Attribute::Bold),
                            Print(format!("{} ", AI_KING)),
                            ResetColor,
                            SetAttribute(Attribute::Reset),
                        )?;
                    }
                }
            }

            if is_cursor {
                queue!(stdout, SetAttribute(Attribute::Reset))?;
            }
        }

        queue!(stdout, Print("\n"))?;
    }

    Ok(())
}

fn piece_symbol(piece: Piece) -> &'static str {
    match piece {
        Piece::Player     => PLAYER_REG,
        Piece::PlayerKing => PLAYER_KING,
        Piece::Ai         => AI_REG,
        Piece::AiKing     => AI_KING,
        Piece::Empty      => " ",
    }
}

fn draw_footer(stdout: &mut impl Write, status: &GameStatus) -> io::Result<()> {
    queue!(stdout, Print("\n"))?;
    match status {
        GameStatus::Playing => {
            queue!(
                stdout,
                SetForegroundColor(Color::DarkGrey),
                Print("  Arrows: move cursor  |  Enter/Space: select / confirm move\n"),
                Print("  Esc: deselect         |  Q: quit game\n"),
                ResetColor,
            )?;
        }
        GameStatus::PlayerWon => {
            queue!(
                stdout,
                SetForegroundColor(Color::Yellow),
                SetAttribute(Attribute::Bold),
                Print("  ⚔  Victory!  You vanquished the AI overlord!\n"),
                SetAttribute(Attribute::Reset),
                ResetColor,
                SetForegroundColor(Color::DarkGrey),
                Print("  Press R to play again  |  Q to quit\n"),
                ResetColor,
            )?;
        }
        GameStatus::AiWon => {
            queue!(
                stdout,
                SetForegroundColor(Color::Red),
                SetAttribute(Attribute::Bold),
                Print("  ☠  Defeated!  The AI outwitted you.  Train harder!\n"),
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

// ── Input reading ─────────────────────────────────────────────────────────────

pub enum InputAction {
    Move(i32, i32),
    Confirm,
    Deselect,
    Restart,
    Quit,
    None,
}

pub fn read_action() -> io::Result<InputAction> {
    loop {
        match event::read()? {
            Event::Key(key) => {
                if key.modifiers.contains(KeyModifiers::CONTROL)
                    && key.code == KeyCode::Char('c')
                {
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
                    KeyCode::Enter | KeyCode::Char(' ') => InputAction::Confirm,
                    KeyCode::Esc                         => InputAction::Deselect,
                    KeyCode::Char('r') | KeyCode::Char('R') => InputAction::Restart,
                    KeyCode::Char('q') | KeyCode::Char('Q') => InputAction::Quit,
                    _ => InputAction::None,
                });
            }
            Event::Resize(_, _) => return Ok(InputAction::None),
            _ => {}
        }
    }
}
