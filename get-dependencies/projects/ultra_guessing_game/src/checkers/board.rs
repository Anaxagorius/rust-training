use rand::seq::SliceRandom;
use rand::thread_rng;

// ── Piece ─────────────────────────────────────────────────────────────────────

/// A single square on the 8 × 8 board.
/// Playable (dark) squares have `(row + col) % 2 == 1`.
/// The player's pieces move *up* (toward row 0); the AI's move *down* (toward row 7).
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Piece {
    Empty,
    Player,      // Red regular piece.
    PlayerKing,  // Red king.
    Ai,          // Black regular piece.
    AiKing,      // Black king.
}

impl Piece {
    #[inline] pub fn is_player(self) -> bool { matches!(self, Piece::Player | Piece::PlayerKing) }
    #[inline] pub fn is_ai(self)     -> bool { matches!(self, Piece::Ai    | Piece::AiKing)     }
    #[inline] pub fn is_king(self)   -> bool { matches!(self, Piece::PlayerKing | Piece::AiKing) }
    #[inline] pub fn is_empty(self)  -> bool { self == Piece::Empty }
}

// ── Turn / Status ─────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Turn { Player, Ai }

#[derive(Clone, PartialEq, Debug)]
pub enum GameStatus { Playing, PlayerWon, AiWon }

// ── Move ──────────────────────────────────────────────────────────────────────

/// A single (possibly multi-jump) checkers move.
#[derive(Clone, Debug, PartialEq)]
pub struct CheckersMove {
    /// Starting square.
    pub from: (usize, usize),
    /// Final landing square (after all jumps in the sequence).
    pub to: (usize, usize),
    /// Every opponent square captured along the way.
    pub captures: Vec<(usize, usize)>,
}

// ── Board ─────────────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct Board {
    pub cells: [[Piece; 8]; 8],
    pub turn: Turn,
    pub status: GameStatus,
    /// The piece the human player has currently selected.
    pub selected: Option<(usize, usize)>,
    /// Valid moves available for the selected piece.
    pub valid_dests: Vec<CheckersMove>,
    pub move_count: u32,
    pub player_pieces: u8,
    pub ai_pieces: u8,
}

impl Board {
    /// Standard 12-vs-12 starting position.
    pub fn new() -> Self {
        let mut cells = [[Piece::Empty; 8]; 8];
        // AI occupies the top three rows on dark squares.
        for row in 0..3 {
            for col in 0..8 {
                if (row + col) % 2 == 1 {
                    cells[row][col] = Piece::Ai;
                }
            }
        }
        // Player occupies the bottom three rows on dark squares.
        for row in 5..8 {
            for col in 0..8 {
                if (row + col) % 2 == 1 {
                    cells[row][col] = Piece::Player;
                }
            }
        }
        Board {
            cells,
            turn: Turn::Player,
            status: GameStatus::Playing,
            selected: None,
            valid_dests: Vec::new(),
            move_count: 0,
            player_pieces: 12,
            ai_pieces: 12,
        }
    }

    // ── Query helpers ─────────────────────────────────────────────────────────

    /// All legal moves for the current turn.  Captures are mandatory.
    pub fn gen_moves(&self) -> Vec<CheckersMove> {
        gen_all_moves(&self.cells, self.turn)
    }

    /// Squares the selected piece is allowed to land on.
    pub fn valid_dest_positions(&self) -> Vec<(usize, usize)> {
        self.valid_dests.iter().map(|m| m.to).collect()
    }

    // ── Selection ────────────────────────────────────────────────────────────

    /// Attempt to select the piece at `(row, col)`.  Returns `true` on success.
    pub fn select(&mut self, row: usize, col: usize) -> bool {
        if !self.cells[row][col].is_player() || self.turn != Turn::Player {
            return false;
        }
        let all = gen_all_moves(&self.cells, Turn::Player);
        let dests: Vec<CheckersMove> = all.into_iter().filter(|m| m.from == (row, col)).collect();
        if dests.is_empty() {
            return false;
        }
        self.selected = Some((row, col));
        self.valid_dests = dests;
        true
    }

    pub fn deselect(&mut self) {
        self.selected = None;
        self.valid_dests.clear();
    }

    /// Try to move the selected piece to `(row, col)`.  Returns `true` on success.
    pub fn move_selected_to(&mut self, row: usize, col: usize) -> bool {
        let m = self.valid_dests.iter().find(|m| m.to == (row, col)).cloned();
        if let Some(m) = m {
            self.deselect();
            self.apply_move(&m);
            true
        } else {
            false
        }
    }

    // ── Move application ──────────────────────────────────────────────────────

    /// Apply a validated move (no legality check).
    pub fn apply_move(&mut self, m: &CheckersMove) {
        let piece = self.cells[m.from.0][m.from.1];
        self.cells[m.from.0][m.from.1] = Piece::Empty;
        for &(cr, cc) in &m.captures {
            self.cells[cr][cc] = Piece::Empty;
        }
        self.cells[m.to.0][m.to.1] = promote(piece, m.to.0);

        // Recount pieces.
        let (mut p, mut a) = (0u8, 0u8);
        for row in &self.cells {
            for &cell in row {
                if cell.is_player() { p += 1; }
                if cell.is_ai()     { a += 1; }
            }
        }
        self.player_pieces = p;
        self.ai_pieces     = a;

        self.move_count += 1;
        self.turn = match self.turn { Turn::Player => Turn::Ai, Turn::Ai => Turn::Player };
        self.check_status();
    }

    fn check_status(&mut self) {
        if self.ai_pieces == 0 {
            self.status = GameStatus::PlayerWon;
        } else if self.player_pieces == 0 {
            self.status = GameStatus::AiWon;
        } else if gen_all_moves(&self.cells, self.turn).is_empty() {
            self.status = match self.turn {
                Turn::Player => GameStatus::AiWon,
                Turn::Ai     => GameStatus::PlayerWon,
            };
        }
    }

    // ── AI ────────────────────────────────────────────────────────────────────

    /// Let the AI pick and apply its best move (minimax, depth 5).
    pub fn ai_move(&mut self) {
        if self.status != GameStatus::Playing { return; }
        let moves = self.gen_moves();
        if moves.is_empty() { return; }

        let mut best_score = i32::MIN + 1;
        let mut best_indices: Vec<usize> = Vec::new();

        for (i, m) in moves.iter().enumerate() {
            let mut sim = self.clone();
            sim.apply_move(m);
            let score = minimax(&sim, 5, i32::MIN + 1, i32::MAX);
            if score > best_score {
                best_score = score;
                best_indices.clear();
                best_indices.push(i);
            } else if score == best_score {
                best_indices.push(i);
            }
        }

        let idx = *best_indices.choose(&mut thread_rng()).unwrap_or(&0);
        let m = moves[idx].clone();
        self.apply_move(&m);
    }
}

// ── Move generation ───────────────────────────────────────────────────────────

fn promote(piece: Piece, row: usize) -> Piece {
    match piece {
        Piece::Player if row == 0 => Piece::PlayerKing,
        Piece::Ai     if row == 7 => Piece::AiKing,
        p => p,
    }
}

/// Generate all legal moves for `turn`.  Captures take mandatory priority.
fn gen_all_moves(cells: &[[Piece; 8]; 8], turn: Turn) -> Vec<CheckersMove> {
    let mut captures: Vec<CheckersMove> = Vec::new();
    let mut regular: Vec<CheckersMove>  = Vec::new();

    for row in 0..8 {
        for col in 0..8 {
            let piece = cells[row][col];
            let mine = match turn {
                Turn::Player => piece.is_player(),
                Turn::Ai     => piece.is_ai(),
            };
            if !mine { continue; }

            // Collect jump sequences from this piece.
            let mut caps: Vec<CheckersMove> = Vec::new();
            jumps_from(row, col, cells, turn, piece, &[], &[], &mut caps, (row, col));

            if !caps.is_empty() {
                captures.extend(caps);
            } else {
                regular.extend(regular_moves(row, col, cells, turn, piece));
            }
        }
    }

    if !captures.is_empty() { captures } else { regular }
}

/// Single-step (non-capture) moves for one piece.
fn regular_moves(
    row: usize, col: usize,
    cells: &[[Piece; 8]; 8],
    turn: Turn, piece: Piece,
) -> Vec<CheckersMove> {
    let dirs: &[(i32, i32)] = if piece.is_king() {
        &[(-1, -1), (-1, 1), (1, -1), (1, 1)]
    } else {
        match turn {
            Turn::Player => &[(-1i32, -1i32), (-1, 1)],
            Turn::Ai     => &[(1i32,  -1i32), (1,  1)],
        }
    };
    let mut moves = Vec::new();
    for &(dr, dc) in dirs {
        let nr = row as i32 + dr;
        let nc = col as i32 + dc;
        if nr >= 0 && nr < 8 && nc >= 0 && nc < 8 {
            let (nr, nc) = (nr as usize, nc as usize);
            if cells[nr][nc].is_empty() {
                moves.push(CheckersMove { from: (row, col), to: (nr, nc), captures: vec![] });
            }
        }
    }
    moves
}

/// Recursively generate all jump sequences from `(row, col)`.
///
/// * `captured`  – opponent squares already captured in this chain.
/// * `visited`   – landing squares already visited in this chain (prevents cycles).
/// * `result`    – accumulate completed moves here.
/// * `origin`    – the piece's true starting square (for `from` in the final move).
fn jumps_from(
    row: usize, col: usize,
    cells: &[[Piece; 8]; 8],
    turn: Turn, piece: Piece,
    captured: &[(usize, usize)],
    visited:  &[(usize, usize)],
    result:   &mut Vec<CheckersMove>,
    origin:   (usize, usize),
) {
    // After landing the piece may have been promoted – respect the new directions.
    let is_king = piece.is_king()
        || (turn == Turn::Player && row == 0)
        || (turn == Turn::Ai    && row == 7);

    let dirs: &[(i32, i32)] = if is_king {
        &[(-1, -1), (-1, 1), (1, -1), (1, 1)]
    } else {
        match turn {
            Turn::Player => &[(-1i32, -1i32), (-1, 1)],
            Turn::Ai     => &[(1i32,  -1i32), (1,  1)],
        }
    };

    for &(dr, dc) in dirs {
        let mr = row as i32 + dr;
        let mc = col as i32 + dc;
        let lr = row as i32 + 2 * dr;
        let lc = col as i32 + 2 * dc;

        if mr < 0 || mr >= 8 || mc < 0 || mc >= 8 { continue; }
        if lr < 0 || lr >= 8 || lc < 0 || lc >= 8 { continue; }

        let mid  = (mr as usize, mc as usize);
        let land = (lr as usize, lc as usize);

        // Mid must be an uncaptured opponent.
        let mid_piece = cells[mid.0][mid.1];
        let is_opp = match turn {
            Turn::Player => mid_piece.is_ai(),
            Turn::Ai     => mid_piece.is_player(),
        };
        if !is_opp || captured.contains(&mid) { continue; }

        // Landing must be empty (the origin square has already been vacated).
        if !cells[land.0][land.1].is_empty() && land != origin { continue; }

        // Prevent revisiting a square we've already landed on.
        if visited.contains(&land) { continue; }

        let mut new_captured = captured.to_vec();
        new_captured.push(mid);
        let mut new_visited = visited.to_vec();
        new_visited.push(land);

        // Simulate the jump.
        let mut new_cells = *cells;
        new_cells[row][col]     = Piece::Empty;
        new_cells[mid.0][mid.1] = Piece::Empty;
        let land_piece = promote(piece, land.0);
        new_cells[land.0][land.1] = land_piece;

        // King promotion ends the multi-jump turn.
        let kinged_here = land_piece.is_king() && !piece.is_king();

        if kinged_here {
            result.push(CheckersMove { from: origin, to: land, captures: new_captured });
        } else {
            let prev_len = result.len();
            jumps_from(
                land.0, land.1, &new_cells, turn, piece,
                &new_captured, &new_visited, result, origin,
            );
            if result.len() == prev_len {
                // No further jump found – this landing is the final position.
                result.push(CheckersMove { from: origin, to: land, captures: new_captured });
            }
        }
    }
}

// ── Minimax AI ────────────────────────────────────────────────────────────────

/// Minimax with alpha-beta pruning.  The AI maximises; the player minimises.
fn minimax(board: &Board, depth: u8, mut alpha: i32, mut beta: i32) -> i32 {
    if depth == 0 || board.status != GameStatus::Playing {
        return evaluate(board);
    }
    let moves = board.gen_moves();
    if moves.is_empty() {
        return evaluate(board);
    }

    match board.turn {
        Turn::Ai => {
            let mut value = i32::MIN + 1;
            for m in &moves {
                let mut sim = board.clone();
                sim.apply_move(m);
                value = value.max(minimax(&sim, depth - 1, alpha, beta));
                alpha = alpha.max(value);
                if alpha >= beta { break; }
            }
            value
        }
        Turn::Player => {
            let mut value = i32::MAX;
            for m in &moves {
                let mut sim = board.clone();
                sim.apply_move(m);
                value = value.min(minimax(&sim, depth - 1, alpha, beta));
                beta = beta.min(value);
                if beta <= alpha { break; }
            }
            value
        }
    }
}

/// Static evaluation: positive values favour the AI.
fn evaluate(board: &Board) -> i32 {
    match board.status {
        GameStatus::AiWon    => return  10_000,
        GameStatus::PlayerWon => return -10_000,
        GameStatus::Playing  => {}
    }
    let mut score = 0i32;
    for (row, row_cells) in board.cells.iter().enumerate() {
        for (col, &piece) in row_cells.iter().enumerate() {
            match piece {
                Piece::Ai => {
                    score += 100;
                    score += row as i32 * 3;               // advancement
                    if col > 0 && col < 7 { score += 5; } // centre control
                }
                Piece::AiKing => { score += 160; }
                Piece::Player => {
                    score -= 100;
                    score -= (7 - row as i32) * 3;        // advancement toward row 0
                    if col > 0 && col < 7 { score -= 5; }
                }
                Piece::PlayerKing => { score -= 160; }
                Piece::Empty => {}
            }
        }
    }
    score
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_board_has_correct_piece_counts() {
        let b = Board::new();
        assert_eq!(b.player_pieces, 12);
        assert_eq!(b.ai_pieces, 12);
    }

    #[test]
    fn player_moves_first() {
        let b = Board::new();
        assert_eq!(b.turn, Turn::Player);
    }

    #[test]
    fn initial_player_has_seven_moves() {
        let b = Board::new();
        assert_eq!(b.gen_moves().len(), 7);
    }

    #[test]
    fn simple_move_switches_turn() {
        let mut b = Board::new();
        let m = b.gen_moves().into_iter().next().unwrap();
        b.apply_move(&m);
        assert_eq!(b.turn, Turn::Ai);
    }

    #[test]
    fn select_requires_player_piece() {
        let mut b = Board::new();
        // Row 0 col 1 holds an AI piece at the start.
        assert!(!b.select(0, 1));
    }

    #[test]
    fn capture_removes_opponent_and_updates_count() {
        let mut b = Board::new();
        b.cells = [[Piece::Empty; 8]; 8];
        b.cells[4][2] = Piece::Player;
        b.cells[3][3] = Piece::Ai;
        b.turn = Turn::Player;
        b.player_pieces = 1;
        b.ai_pieces = 1;

        let moves = b.gen_moves();
        assert!(!moves.is_empty(), "should find a capture");
        assert!(moves[0].captures.contains(&(3, 3)));
        b.apply_move(&moves[0]);
        assert_eq!(b.cells[3][3], Piece::Empty, "captured piece removed");
        assert_eq!(b.cells[2][4], Piece::Player, "player piece at landing");
        assert_eq!(b.ai_pieces, 0);
    }

    #[test]
    fn player_promoted_to_king_at_row_0() {
        let mut b = Board::new();
        b.cells = [[Piece::Empty; 8]; 8];
        b.cells[1][2] = Piece::Player;
        b.cells[7][0] = Piece::Ai; // keep ai_pieces > 0
        b.turn = Turn::Player;
        b.player_pieces = 1;
        b.ai_pieces = 1;

        let moves = b.gen_moves();
        let king_move = moves.iter().find(|m| m.to.0 == 0);
        if let Some(m) = king_move {
            let dest_col = m.to.1;
            b.apply_move(m);
            assert_eq!(b.cells[0][dest_col], Piece::PlayerKing);
        }
    }

    #[test]
    fn no_pieces_means_game_over() {
        let mut b = Board::new();
        b.cells = [[Piece::Empty; 8]; 8];
        b.cells[0][1] = Piece::Player;
        b.turn = Turn::Ai;
        b.player_pieces = 1;
        b.ai_pieces = 0;
        b.check_status();
        assert_eq!(b.status, GameStatus::PlayerWon);
    }

    #[test]
    fn mandatory_capture_overrides_regular_moves() {
        // Piece at (4,2) can move regularly to (3,1)/(3,3),
        // but can also jump over AI at (3,3) to (2,4).
        let mut b = Board::new();
        b.cells = [[Piece::Empty; 8]; 8];
        b.cells[4][2] = Piece::Player;
        b.cells[3][3] = Piece::Ai;
        b.cells[6][0] = Piece::Player; // second player piece (can only do regular moves)
        b.turn = Turn::Player;
        b.player_pieces = 2;
        b.ai_pieces = 1;

        let moves = b.gen_moves();
        // All returned moves must be captures.
        assert!(moves.iter().all(|m| !m.captures.is_empty()),
            "only captures should be returned when captures are available");
    }
}
