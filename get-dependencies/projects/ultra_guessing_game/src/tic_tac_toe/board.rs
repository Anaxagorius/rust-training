use rand::Rng;

// ── Cell ──────────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Cell {
    Empty,
    /// Human player.
    X,
    /// AI opponent.
    O,
}

// ── Game Status ───────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GameStatus {
    Playing,
    PlayerWon,
    AiWon,
    Draw,
}

// ── Board ─────────────────────────────────────────────────────────────────────

pub struct Board {
    pub cells: [[Cell; 3]; 3],
    pub status: GameStatus,
    /// How many moves the player made in total (for the "Flawless" achievement).
    pub player_moves: u32,
}

impl Board {
    pub fn new() -> Self {
        Board {
            cells: [[Cell::Empty; 3]; 3],
            status: GameStatus::Playing,
            player_moves: 0,
        }
    }

    /// Attempt to place the player's mark (X) at `(row, col)`.
    /// Returns `true` if the move was accepted.
    pub fn player_move(&mut self, row: usize, col: usize) -> bool {
        if self.status != GameStatus::Playing || self.cells[row][col] != Cell::Empty {
            return false;
        }
        self.cells[row][col] = Cell::X;
        self.player_moves += 1;
        self.update_status();
        true
    }

    /// Let the AI pick and apply its move.
    /// The AI plays the minimax-optimal move 80 % of the time and a random
    /// valid move 20 % of the time, making the game occasionally winnable.
    pub fn ai_move(&mut self) {
        if self.status != GameStatus::Playing {
            return;
        }

        let mut rng = rand::thread_rng();
        // `r#gen()` is required because `gen` is a reserved keyword in Rust 2024 edition.
        let random_chance: f64 = rng.r#gen();

        let (row, col) = if random_chance < 0.20 {
            // Random valid move.
            let empties: Vec<(usize, usize)> = (0..3)
                .flat_map(|r| (0..3).map(move |c| (r, c)))
                .filter(|&(r, c)| self.cells[r][c] == Cell::Empty)
                .collect();
            empties[rng.gen_range(0..empties.len())]
        } else {
            best_move(&self.cells)
        };

        self.cells[row][col] = Cell::O;
        self.update_status();
    }

    fn update_status(&mut self) {
        if let Some(winner) = check_winner(&self.cells) {
            self.status = match winner {
                Cell::X => GameStatus::PlayerWon,
                Cell::O => GameStatus::AiWon,
                Cell::Empty => unreachable!(),
            };
        } else if self.cells.iter().flatten().all(|&c| c != Cell::Empty) {
            self.status = GameStatus::Draw;
        }
        // otherwise still Playing
    }

    pub fn is_over(&self) -> bool {
        self.status != GameStatus::Playing
    }
}

// ── Win detection ─────────────────────────────────────────────────────────────

/// Returns `Some(winner_cell)` if there is a winner, else `None`.
pub fn check_winner(cells: &[[Cell; 3]; 3]) -> Option<Cell> {
    // Rows and columns.
    for i in 0..3 {
        if cells[i][0] != Cell::Empty
            && cells[i][0] == cells[i][1]
            && cells[i][1] == cells[i][2]
        {
            return Some(cells[i][0]);
        }
        if cells[0][i] != Cell::Empty
            && cells[0][i] == cells[1][i]
            && cells[1][i] == cells[2][i]
        {
            return Some(cells[0][i]);
        }
    }
    // Diagonals.
    if cells[0][0] != Cell::Empty
        && cells[0][0] == cells[1][1]
        && cells[1][1] == cells[2][2]
    {
        return Some(cells[0][0]);
    }
    if cells[0][2] != Cell::Empty
        && cells[0][2] == cells[1][1]
        && cells[1][1] == cells[2][0]
    {
        return Some(cells[0][2]);
    }
    None
}

// ── Minimax ───────────────────────────────────────────────────────────────────

/// Score the board from the AI (O) perspective.
fn score(cells: &[[Cell; 3]; 3]) -> i32 {
    match check_winner(cells) {
        Some(Cell::O) =>  10,
        Some(Cell::X) => -10,
        _             =>   0,
    }
}

fn minimax(cells: &mut [[Cell; 3]; 3], depth: u8, is_maximising: bool) -> i32 {
    let s = score(cells);
    if s != 0 {
        return s;
    }
    let any_empty = cells.iter().flatten().any(|&c| c == Cell::Empty);
    if !any_empty {
        return 0; // Draw.
    }

    if is_maximising {
        let mut best = i32::MIN;
        'outer: for r in 0..3 {
            for c in 0..3 {
                if cells[r][c] == Cell::Empty {
                    cells[r][c] = Cell::O;
                    best = best.max(minimax(cells, depth + 1, false));
                    cells[r][c] = Cell::Empty;
                    if best == 10 {
                        break 'outer; // Can't do better.
                    }
                }
            }
        }
        best
    } else {
        let mut best = i32::MAX;
        'outer: for r in 0..3 {
            for c in 0..3 {
                if cells[r][c] == Cell::Empty {
                    cells[r][c] = Cell::X;
                    best = best.min(minimax(cells, depth + 1, true));
                    cells[r][c] = Cell::Empty;
                    if best == -10 {
                        break 'outer;
                    }
                }
            }
        }
        best
    }
}

/// Returns the best `(row, col)` for the AI (O).
fn best_move(cells: &[[Cell; 3]; 3]) -> (usize, usize) {
    let mut cells = *cells;
    let mut best_val = i32::MIN;
    let mut best_pos = (0, 0);

    for r in 0..3 {
        for c in 0..3 {
            if cells[r][c] == Cell::Empty {
                cells[r][c] = Cell::O;
                let val = minimax(&mut cells, 0, false);
                cells[r][c] = Cell::Empty;
                if val > best_val {
                    best_val = val;
                    best_pos = (r, c);
                }
            }
        }
    }
    best_pos
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_board_is_empty() {
        let board = Board::new();
        for row in &board.cells {
            for &cell in row {
                assert_eq!(cell, Cell::Empty);
            }
        }
        assert_eq!(board.status, GameStatus::Playing);
    }

    #[test]
    fn player_wins_row() {
        let mut board = Board::new();
        board.cells[0] = [Cell::X, Cell::X, Cell::X];
        board.update_status();
        assert_eq!(board.status, GameStatus::PlayerWon);
    }

    #[test]
    fn ai_wins_column() {
        let mut board = Board::new();
        board.cells[0][0] = Cell::O;
        board.cells[1][0] = Cell::O;
        board.cells[2][0] = Cell::O;
        board.update_status();
        assert_eq!(board.status, GameStatus::AiWon);
    }

    #[test]
    fn draw_detected() {
        let mut board = Board::new();
        // X O X
        // X X O
        // O X O
        board.cells = [
            [Cell::X, Cell::O, Cell::X],
            [Cell::X, Cell::X, Cell::O],
            [Cell::O, Cell::X, Cell::O],
        ];
        board.update_status();
        assert_eq!(board.status, GameStatus::Draw);
    }

    #[test]
    fn player_move_rejected_on_occupied_cell() {
        let mut board = Board::new();
        board.cells[1][1] = Cell::O;
        assert!(!board.player_move(1, 1));
        assert_eq!(board.player_moves, 0);
    }

    #[test]
    fn player_move_accepted_on_empty_cell() {
        let mut board = Board::new();
        assert!(board.player_move(0, 0));
        assert_eq!(board.cells[0][0], Cell::X);
        assert_eq!(board.player_moves, 1);
    }

    #[test]
    fn minimax_blocks_player_win() {
        // Player has X at (0,0) and (0,1); AI must block at (0,2).
        let cells = [
            [Cell::X, Cell::X, Cell::Empty],
            [Cell::Empty, Cell::O, Cell::Empty],
            [Cell::Empty, Cell::Empty, Cell::Empty],
        ];
        let (r, c) = best_move(&cells);
        assert_eq!((r, c), (0, 2), "AI must block the player's winning row");
    }

    #[test]
    fn minimax_takes_winning_move() {
        // AI has O at (2,0) and (2,1); it should take (2,2) to win.
        let cells = [
            [Cell::X, Cell::X, Cell::Empty],
            [Cell::Empty, Cell::X, Cell::Empty],
            [Cell::O, Cell::O, Cell::Empty],
        ];
        let (r, c) = best_move(&cells);
        assert_eq!((r, c), (2, 2), "AI must take the winning move");
    }
}
