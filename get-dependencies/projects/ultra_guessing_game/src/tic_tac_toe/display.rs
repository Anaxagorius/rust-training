use std::io::{self, Write};
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyModifiers},
    queue,
    style::{Attribute, Color, Print, ResetColor, SetAttribute, SetForegroundColor},
    terminal::{self, ClearType},
};

use super::board::{Board, Cell, GameStatus};

// ── Input ─────────────────────────────────────────────────────────────────────

pub enum InputAction {
    Move(i32, i32),
    Confirm,
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

// ── Rendering ─────────────────────────────────────────────────────────────────

/// Render the full Tic Tac Toe TUI.
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
        SetForegroundColor(Color::Cyan),
        Print("  ╔══════════════════════════════════════════╗\n"),
        Print("  ║        ✕  IRON AGE TIC TAC TOE  ○        ║\n"),
        Print("  ╚══════════════════════════════════════════╝\n\n"),
        ResetColor,
        SetAttribute(Attribute::Reset),
    )
}

fn draw_status(stdout: &mut impl Write, board: &Board) -> io::Result<()> {
    queue!(
        stdout,
        SetForegroundColor(Color::Red),
        SetAttribute(Attribute::Bold),
        Print("  You: X"),
        ResetColor,
        SetAttribute(Attribute::Reset),
        Print("    "),
        SetForegroundColor(Color::Cyan),
        SetAttribute(Attribute::Bold),
        Print("AI: O"),
        ResetColor,
        SetAttribute(Attribute::Reset),
        Print("\n"),
    )?;

    let turn_str = match board.status {
        GameStatus::Playing => format!(
            "  \x1b[1m\x1b[33m▶ Your turn\x1b[0m  (move {} placed)\n\n",
            board.player_moves
        ),
        _ => String::new(),
    };
    if !turn_str.is_empty() {
        queue!(stdout, Print(turn_str))?;
    } else {
        queue!(stdout, Print("\n"))?;
    }
    Ok(())
}

fn draw_board(
    stdout: &mut impl Write,
    board: &Board,
    cursor_row: usize,
    cursor_col: usize,
) -> io::Result<()> {
    // Column labels
    queue!(
        stdout,
        SetForegroundColor(Color::DarkGrey),
        Print("       1     2     3\n"),
        ResetColor,
    )?;

    for row in 0..3 {
        // Row separator (except before first row)
        if row > 0 {
            queue!(
                stdout,
                SetForegroundColor(Color::DarkGrey),
                Print("     ──────┼──────┼──────\n"),
                ResetColor,
            )?;
        }

        // Row label
        queue!(
            stdout,
            SetForegroundColor(Color::DarkGrey),
            Print(format!("  {}  ", (b'A' + row as u8) as char)),
            ResetColor,
        )?;

        for col in 0..3 {
            if col > 0 {
                queue!(
                    stdout,
                    SetForegroundColor(Color::DarkGrey),
                    Print("│"),
                    ResetColor,
                )?;
            }

            let is_cursor = row == cursor_row && col == cursor_col;
            let cell = board.cells[row][col];

            if is_cursor && board.status == GameStatus::Playing {
                queue!(stdout, SetAttribute(Attribute::Reverse))?;
            }

            match cell {
                Cell::Empty => {
                    if is_cursor && board.status == GameStatus::Playing {
                        queue!(
                            stdout,
                            SetForegroundColor(Color::Yellow),
                            Print("  ·  "),
                            ResetColor,
                        )?;
                    } else {
                        queue!(stdout, Print("     "))?;
                    }
                }
                Cell::X => {
                    queue!(
                        stdout,
                        SetForegroundColor(Color::Red),
                        SetAttribute(Attribute::Bold),
                        Print("  X  "),
                        ResetColor,
                        SetAttribute(Attribute::Reset),
                    )?;
                }
                Cell::O => {
                    queue!(
                        stdout,
                        SetForegroundColor(Color::Cyan),
                        SetAttribute(Attribute::Bold),
                        Print("  O  "),
                        ResetColor,
                        SetAttribute(Attribute::Reset),
                    )?;
                }
            }

            if is_cursor && board.status == GameStatus::Playing {
                queue!(stdout, SetAttribute(Attribute::Reset))?;
            }
        }

        queue!(stdout, Print("\n"))?;
    }

    queue!(stdout, Print("\n"))?;
    Ok(())
}

fn draw_footer(stdout: &mut impl Write, status: &GameStatus) -> io::Result<()> {
    match status {
        GameStatus::Playing => {
            queue!(
                stdout,
                SetForegroundColor(Color::DarkGrey),
                Print("  Arrows / WASD / hjkl: move cursor  |  Enter / Space: place X\n"),
                Print("  R: restart  |  Q: quit\n"),
                ResetColor,
            )?;
        }
        GameStatus::PlayerWon => {
            queue!(
                stdout,
                SetForegroundColor(Color::Yellow),
                SetAttribute(Attribute::Bold),
                Print("  ⚔  You won! The AI couldn't stop you!\n"),
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
                Print("  ☠  The AI wins! Better luck next time!\n"),
                SetAttribute(Attribute::Reset),
                ResetColor,
                SetForegroundColor(Color::DarkGrey),
                Print("  Press R to play again  |  Q to quit\n"),
                ResetColor,
            )?;
        }
        GameStatus::Draw => {
            queue!(
                stdout,
                SetForegroundColor(Color::Magenta),
                SetAttribute(Attribute::Bold),
                Print("  🤝  It's a draw! Well matched!\n"),
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
