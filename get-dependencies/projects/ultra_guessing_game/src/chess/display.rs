use std::io::{self, Write};
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyModifiers},
    queue,
    style::{Attribute, Color, Print, ResetColor, SetAttribute, SetForegroundColor},
    terminal::{self, ClearType},
};

use super::board::{Board, Color as PColor, GameStatus, PieceKind, Turn};

// ── Input ─────────────────────────────────────────────────────────────────────

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
        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                // Ctrl-C / Ctrl-Q → quit.
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    match key.code {
                        KeyCode::Char('c') | KeyCode::Char('q') => return Ok(InputAction::Quit),
                        _ => {}
                    }
                }
                let action = match key.code {
                    KeyCode::Up    | KeyCode::Char('k') => InputAction::Move(-1,  0),
                    KeyCode::Down  | KeyCode::Char('j') => InputAction::Move( 1,  0),
                    KeyCode::Left  | KeyCode::Char('h') => InputAction::Move( 0, -1),
                    KeyCode::Right | KeyCode::Char('l') => InputAction::Move( 0,  1),
                    KeyCode::Enter | KeyCode::Char(' ') => InputAction::Confirm,
                    KeyCode::Esc                        => InputAction::Deselect,
                    KeyCode::Char('r') | KeyCode::Char('R') => InputAction::Restart,
                    KeyCode::Char('q') | KeyCode::Char('Q') => InputAction::Quit,
                    _ => InputAction::None,
                };
                return Ok(action);
            }
        }
    }
}

// ── Rendering ─────────────────────────────────────────────────────────────────

/// Render the chess TUI to stdout.
pub fn draw(board: &Board, cursor_row: usize, cursor_col: usize) -> io::Result<()> {
    let mut out = io::stdout();
    queue!(out, cursor::MoveTo(0, 0), terminal::Clear(ClearType::All))?;

    draw_header(&mut out)?;
    draw_status(&mut out, board)?;
    draw_board(&mut out, board, cursor_row, cursor_col)?;
    draw_footer(&mut out, &board.status)?;

    out.flush()
}

fn draw_header(out: &mut impl Write) -> io::Result<()> {
    queue!(
        out,
        SetAttribute(Attribute::Bold),
        SetForegroundColor(Color::Yellow),
        Print("  ╔══════════════════════════════════════════╗\n"),
        Print("  ║          ♔  IRON AGE CHESS  ♚            ║\n"),
        Print("  ╚══════════════════════════════════════════╝\n\n"),
        ResetColor,
        SetAttribute(Attribute::Reset),
    )
}

fn draw_status(out: &mut impl Write, board: &Board) -> io::Result<()> {
    // Legend.
    queue!(
        out,
        SetForegroundColor(Color::White),
        SetAttribute(Attribute::Bold),
        Print("  You: "),
        ResetColor,
        SetAttribute(Attribute::Reset),
        Print("White  ♙♖♘♗♕♔"),
        Print("    "),
        SetForegroundColor(Color::Cyan),
        SetAttribute(Attribute::Bold),
        Print("AI: "),
        ResetColor,
        SetAttribute(Attribute::Reset),
        Print("Black  ♟♜♞♝♛♚\n"),
    )?;

    let (white_mat, black_mat) = material_counts(board);
    let turn_str = match &board.status {
        GameStatus::Check => match board.turn {
            Turn::White => "\x1b[1m\x1b[33m  ⚠  CHECK! Your turn\x1b[0m",
            Turn::Black => "\x1b[1m\x1b[33m  ⚠  CHECK! AI in check\x1b[0m",
        },
        GameStatus::Playing => match board.turn {
            Turn::White => "\x1b[1m\x1b[32m  ▶  Your turn (White)\x1b[0m",
            Turn::Black => "     AI is thinking… (Black)",
        },
        GameStatus::PlayerWon  => "\x1b[1m\x1b[32m  🏆  Checkmate – You WIN!\x1b[0m",
        GameStatus::AiWon      => "\x1b[1m\x1b[31m  💀  Checkmate – AI wins!\x1b[0m",
        GameStatus::Stalemate  => "\x1b[1m\x1b[33m  🤝  Stalemate – Draw!\x1b[0m",
        GameStatus::Draw       => "\x1b[1m\x1b[33m  🤝  50-move rule – Draw!\x1b[0m",
    };
    queue!(
        out,
        Print(format!(
            "{}\n  Material: \x1b[37mWhite {}\x1b[0m  |  \x1b[36mBlack {}\x1b[0m\n\n",
            turn_str, white_mat, black_mat
        ))
    )
}

fn material_counts(board: &Board) -> (i32, i32) {
    let mut w = 0i32;
    let mut b = 0i32;
    for row in &board.cells {
        for sq in row {
            if let Some(p) = sq {
                let v = match p.kind {
                    PieceKind::Pawn   => 1,
                    PieceKind::Knight | PieceKind::Bishop => 3,
                    PieceKind::Rook   => 5,
                    PieceKind::Queen  => 9,
                    PieceKind::King   => 0,
                };
                if p.color == PColor::White { w += v; } else { b += v; }
            }
        }
    }
    (w, b)
}

fn piece_glyph(kind: PieceKind, color: PColor) -> &'static str {
    match (color, kind) {
        (PColor::White, PieceKind::Pawn)   => "♙",
        (PColor::White, PieceKind::Rook)   => "♖",
        (PColor::White, PieceKind::Knight) => "♘",
        (PColor::White, PieceKind::Bishop) => "♗",
        (PColor::White, PieceKind::Queen)  => "♕",
        (PColor::White, PieceKind::King)   => "♔",
        (PColor::Black, PieceKind::Pawn)   => "♟",
        (PColor::Black, PieceKind::Rook)   => "♜",
        (PColor::Black, PieceKind::Knight) => "♞",
        (PColor::Black, PieceKind::Bishop) => "♝",
        (PColor::Black, PieceKind::Queen)  => "♛",
        (PColor::Black, PieceKind::King)   => "♚",
    }
}

fn draw_board(
    out: &mut impl Write,
    board: &Board,
    cursor_row: usize,
    cursor_col: usize,
) -> io::Result<()> {
    let selected    = board.selected;
    let valid_dests = board.valid_dest_positions();

    // Column letters.
    queue!(out, Print("     a   b   c   d   e   f   g   h\n"))?;
    queue!(out, Print("   ┌───┬───┬───┬───┬───┬───┬───┬───┐\n"))?;

    for row in 0..8usize {
        let rank = 8 - row;
        queue!(out, Print(format!(" {} │", rank)))?;

        for col in 0..8usize {
            let is_cursor   = cursor_row == row && cursor_col == col;
            let is_selected = selected == Some((row, col));
            let is_dest     = valid_dests.contains(&(row, col));
            let is_light    = (row + col) % 2 == 0;

            // Background colour selection.
            let bg = if is_selected {
                Color::DarkYellow
            } else if is_cursor {
                Color::DarkMagenta
            } else if is_dest {
                Color::DarkGreen
            } else if is_light {
                Color::Rgb { r: 200, g: 150, b: 80 }
            } else {
                Color::Rgb { r: 100, g: 60, b: 20 }
            };

            queue!(
                out,
                crossterm::style::SetBackgroundColor(bg),
            )?;

            match board.cells[row][col] {
                None => {
                    if is_dest {
                        // Show a dot for reachable empty squares.
                        queue!(
                            out,
                            SetForegroundColor(Color::Green),
                            Print(" · "),
                        )?;
                    } else {
                        queue!(out, Print("   "))?;
                    }
                }
                Some(p) => {
                    let fg = match p.color {
                        PColor::White => Color::White,
                        PColor::Black => Color::Cyan,
                    };
                    let glyph = piece_glyph(p.kind, p.color);
                    queue!(
                        out,
                        SetForegroundColor(fg),
                        SetAttribute(Attribute::Bold),
                        Print(format!(" {} ", glyph)),
                        SetAttribute(Attribute::Reset),
                    )?;
                }
            }

            queue!(
                out,
                crossterm::style::ResetColor,
                Print("│"),
            )?;
        }

        queue!(out, Print(format!(" {}\n", rank)))?;

        if row < 7 {
            queue!(out, Print("   ├───┼───┼───┼───┼───┼───┼───┼───┤\n"))?;
        }
    }

    queue!(out, Print("   └───┴───┴───┴───┴───┴───┴───┴───┘\n"))?;
    queue!(out, Print("     a   b   c   d   e   f   g   h\n\n"))?;

    Ok(())
}

fn draw_footer(out: &mut impl Write, status: &GameStatus) -> io::Result<()> {
    let is_over = matches!(
        status,
        GameStatus::PlayerWon | GameStatus::AiWon | GameStatus::Stalemate | GameStatus::Draw
    );

    queue!(
        out,
        SetForegroundColor(Color::DarkGrey),
        Print("  Controls: "),
        ResetColor,
        Print("Arrow keys / hjkl: move cursor"),
        Print("  │  "),
        Print("Enter/Space: select / move"),
        Print("  │  "),
        Print("Esc: deselect"),
        Print("\n  "),
    )?;

    if is_over {
        queue!(out, Print("R: new game  │  Q: quit\n"))?;
    } else {
        queue!(out, Print("Q: quit\n"))?;
    }
    Ok(())
}
