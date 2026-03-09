mod board;
mod display;

use std::io;
use std::time::Instant;

use crossterm::{
    cursor,
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};

use board::{Board, GameStatus, Level};
use display::{
    draw, draw_level_select, read_action, read_menu_action, InputAction, MenuAction,
};

// ─── Level select loop ───────────────────────────────────────────────────────

fn level_select() -> io::Result<Option<Level>> {
    let levels = [Level::Peasant, Level::Knight, Level::Champion];
    let mut selected = 0usize;

    loop {
        draw_level_select(selected)?;

        match read_menu_action()? {
            MenuAction::Up => {
                if selected > 0 { selected -= 1; }
            }
            MenuAction::Down => {
                if selected < levels.len() - 1 { selected += 1; }
            }
            MenuAction::Select(idx) => {
                let actual = if idx == usize::MAX { selected } else { idx };
                if actual < levels.len() {
                    return Ok(Some(levels[actual]));
                }
            }
            MenuAction::Quit => return Ok(None),
            MenuAction::None => {}
        }
    }
}

// ─── Game loop ───────────────────────────────────────────────────────────────

fn play(level: Level) -> io::Result<bool> {
    let mut board = Board::new(level);
    let mut cursor_row: usize = level.rows() / 2;
    let mut cursor_col: usize = level.cols() / 2;
    let start = Instant::now();

    loop {
        let elapsed = start.elapsed().as_secs();
        // Cap the timer once the game is finished.
        let display_secs = elapsed;
        draw(&board, cursor_row, cursor_col, display_secs)?;

        let action = read_action()?;
        match action {
            InputAction::Move(dr, dc) => {
                let new_row = (cursor_row as i32 + dr)
                    .clamp(0, (board.rows - 1) as i32) as usize;
                let new_col = (cursor_col as i32 + dc)
                    .clamp(0, (board.cols - 1) as i32) as usize;
                cursor_row = new_row;
                cursor_col = new_col;
            }
            InputAction::Reveal => {
                board.reveal(cursor_row, cursor_col);
            }
            InputAction::Flag => {
                board.toggle_flag(cursor_row, cursor_col);
            }
            InputAction::Restart => {
                return Ok(true); // signal: restart at level select
            }
            InputAction::Quit => {
                return Ok(false);
            }
            InputAction::None => {}
        }

        // After game ends, keep rendering until the player presses R or Q.
        if board.status != GameStatus::Playing {
            // Draw final state once more with game-over overlay.
            draw(&board, cursor_row, cursor_col, display_secs)?;
            loop {
                match read_action()? {
                    InputAction::Restart => return Ok(true),
                    InputAction::Quit => return Ok(false),
                    _ => {}
                }
            }
        }
    }
}

// ─── Entry point ─────────────────────────────────────────────────────────────

fn main() -> io::Result<()> {
    let mut stdout = io::stdout();

    // Set up the alternate screen + raw mode.
    execute!(stdout, EnterAlternateScreen, cursor::Hide)?;
    terminal::enable_raw_mode()?;

    let result = run_game();

    // Always restore the terminal, even on error.
    terminal::disable_raw_mode()?;
    execute!(stdout, cursor::Show, LeaveAlternateScreen)?;

    result
}

fn run_game() -> io::Result<()> {
    loop {
        // Show level select screen.
        let level = match level_select()? {
            Some(l) => l,
            None => break,
        };

        // Play until the player quits or asks to restart.
        let restart = play(level)?;
        if !restart {
            break;
        }
    }

    Ok(())
}
