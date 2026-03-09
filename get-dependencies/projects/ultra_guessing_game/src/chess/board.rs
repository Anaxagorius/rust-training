use rand::seq::SliceRandom;
use rand::thread_rng;

// ── Piece ──────────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PieceKind {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Color {
    White, // Player
    Black, // AI
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Piece {
    pub kind:  PieceKind,
    pub color: Color,
}

impl Piece {
    pub fn new(kind: PieceKind, color: Color) -> Self { Piece { kind, color } }
    pub fn is_white(self) -> bool { self.color == Color::White }
    pub fn is_black(self) -> bool { self.color == Color::Black }
}

// ── Turn / Status ─────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Turn { White, Black }

impl Turn {
    pub fn opposite(self) -> Turn {
        match self { Turn::White => Turn::Black, Turn::Black => Turn::White }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum GameStatus {
    Playing,
    Check,         // Current side is in check.
    PlayerWon,     // White checkmated Black.
    AiWon,         // Black checkmated White.
    Stalemate,
    Draw,          // 50-move rule.
}

// ── Move ──────────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq)]
pub struct ChessMove {
    pub from: (usize, usize),
    pub to:   (usize, usize),
    /// Promotion piece kind (always Queen in our simplified AI).
    pub promotion: Option<PieceKind>,
    /// En-passant capture square (if any).
    pub ep_capture: Option<(usize, usize)>,
    /// Castling: rook (from, to) squares.
    pub castle_rook: Option<((usize, usize), (usize, usize))>,
}

impl ChessMove {
    pub fn simple(from: (usize, usize), to: (usize, usize)) -> Self {
        ChessMove { from, to, promotion: None, ep_capture: None, castle_rook: None }
    }
}

// ── Board ─────────────────────────────────────────────────────────────────────

/// Square: `None` = empty, `Some(piece)` = occupied.
pub type Square = Option<Piece>;

#[derive(Clone)]
pub struct Board {
    pub cells: [[Square; 8]; 8],
    pub turn:  Turn,
    pub status: GameStatus,

    // Cursor / selection state (used by the display layer).
    pub selected:    Option<(usize, usize)>,
    pub valid_dests: Vec<ChessMove>,

    // Castling rights (king-side, queen-side) for each color.
    pub castle_white_k: bool,
    pub castle_white_q: bool,
    pub castle_black_k: bool,
    pub castle_black_q: bool,

    /// En-passant target square (the square a pawn can capture into).
    pub en_passant: Option<(usize, usize)>,

    /// Half-move clock for the 50-move rule.
    pub halfmove_clock: u32,

    pub move_count: u32,
}

impl Board {
    /// Standard chess starting position.
    pub fn new() -> Self {
        let mut cells: [[Square; 8]; 8] = [[None; 8]; 8];

        // Black pieces (rows 0–1).
        let back_row = [
            PieceKind::Rook, PieceKind::Knight, PieceKind::Bishop, PieceKind::Queen,
            PieceKind::King, PieceKind::Bishop, PieceKind::Knight, PieceKind::Rook,
        ];
        for (col, &kind) in back_row.iter().enumerate() {
            cells[0][col] = Some(Piece::new(kind, Color::Black));
            cells[1][col] = Some(Piece::new(PieceKind::Pawn, Color::Black));
        }

        // White pieces (rows 6–7).
        for (col, &kind) in back_row.iter().enumerate() {
            cells[7][col] = Some(Piece::new(kind, Color::White));
            cells[6][col] = Some(Piece::new(PieceKind::Pawn, Color::White));
        }

        Board {
            cells,
            turn:  Turn::White,
            status: GameStatus::Playing,
            selected:    None,
            valid_dests: Vec::new(),
            castle_white_k: true,
            castle_white_q: true,
            castle_black_k: true,
            castle_black_q: true,
            en_passant: None,
            halfmove_clock: 0,
            move_count: 0,
        }
    }

    // ── Query helpers ─────────────────────────────────────────────────────────

    pub fn valid_dest_positions(&self) -> Vec<(usize, usize)> {
        self.valid_dests.iter().map(|m| m.to).collect()
    }

    /// All legal moves for the current turn (checks are filtered out).
    pub fn gen_moves(&self) -> Vec<ChessMove> {
        gen_all_legal_moves(self, self.turn)
    }

    // ── Selection ────────────────────────────────────────────────────────────

    /// Try to select the piece at `(row, col)`. Returns `true` on success.
    pub fn select(&mut self, row: usize, col: usize) -> bool {
        if self.turn != Turn::White { return false; }
        match self.cells[row][col] {
            Some(p) if p.is_white() => {
                let moves: Vec<ChessMove> = gen_all_legal_moves(self, Turn::White)
                    .into_iter()
                    .filter(|m| m.from == (row, col))
                    .collect();
                if moves.is_empty() { return false; }
                self.selected    = Some((row, col));
                self.valid_dests = moves;
                true
            }
            _ => false,
        }
    }

    pub fn deselect(&mut self) {
        self.selected    = None;
        self.valid_dests.clear();
    }

    /// Try to move the selected piece to `(row, col)`. Returns `true` on success.
    pub fn move_selected_to(&mut self, row: usize, col: usize) -> bool {
        // Prefer promotion to Queen automatically.
        let m = self.valid_dests.iter().find(|m| {
            m.to == (row, col) && (m.promotion.is_none() || m.promotion == Some(PieceKind::Queen))
        }).cloned();
        if let Some(m) = m {
            self.deselect();
            self.apply_move(&m);
            true
        } else {
            false
        }
    }

    // ── Move application ──────────────────────────────────────────────────────

    /// Apply a (pseudo-)legal move and update board state.
    pub fn apply_move(&mut self, m: &ChessMove) {
        let piece = match self.cells[m.from.0][m.from.1] {
            Some(p) => p,
            None    => return,
        };

        // Reset or increment halfmove clock.
        if piece.kind == PieceKind::Pawn || self.cells[m.to.0][m.to.1].is_some() {
            self.halfmove_clock = 0;
        } else {
            self.halfmove_clock += 1;
        }

        // En-passant capture.
        if let Some(ep) = m.ep_capture {
            self.cells[ep.0][ep.1] = None;
        }

        // Move the piece.
        self.cells[m.from.0][m.from.1] = None;
        self.cells[m.to.0][m.to.1] = if let Some(promo) = m.promotion {
            Some(Piece::new(promo, piece.color))
        } else {
            Some(piece)
        };

        // Castling – move the rook.
        if let Some((rook_from, rook_to)) = m.castle_rook {
            let rook = self.cells[rook_from.0][rook_from.1];
            self.cells[rook_from.0][rook_from.1] = None;
            self.cells[rook_to.0][rook_to.1]     = rook;
        }

        // Update castling rights.
        match (piece.kind, piece.color) {
            (PieceKind::King, Color::White) => {
                self.castle_white_k = false;
                self.castle_white_q = false;
            }
            (PieceKind::King, Color::Black) => {
                self.castle_black_k = false;
                self.castle_black_q = false;
            }
            (PieceKind::Rook, Color::White) => {
                if m.from == (7, 7) { self.castle_white_k = false; }
                if m.from == (7, 0) { self.castle_white_q = false; }
            }
            (PieceKind::Rook, Color::Black) => {
                if m.from == (0, 7) { self.castle_black_k = false; }
                if m.from == (0, 0) { self.castle_black_q = false; }
            }
            _ => {}
        }

        // Update en-passant target.
        if piece.kind == PieceKind::Pawn {
            let dr = (m.to.0 as i32 - m.from.0 as i32).abs();
            if dr == 2 {
                let ep_row = (m.from.0 + m.to.0) / 2;
                self.en_passant = Some((ep_row, m.to.1));
            } else {
                self.en_passant = None;
            }
        } else {
            self.en_passant = None;
        }

        self.move_count += 1;
        self.turn = self.turn.opposite();

        // Update game status.
        self.update_status();
    }

    fn update_status(&mut self) {
        if self.halfmove_clock >= 100 {
            self.status = GameStatus::Draw;
            return;
        }
        let legal = gen_all_legal_moves(self, self.turn);
        let in_check = is_in_check(&self.cells, self.turn);
        if legal.is_empty() {
            if in_check {
                self.status = match self.turn {
                    Turn::White => GameStatus::AiWon,
                    Turn::Black => GameStatus::PlayerWon,
                };
            } else {
                self.status = GameStatus::Stalemate;
            }
        } else if in_check {
            self.status = GameStatus::Check;
        } else {
            self.status = GameStatus::Playing;
        }
    }

    // ── AI ────────────────────────────────────────────────────────────────────

    /// Let the AI (Black) pick and apply its best move (minimax depth 3).
    pub fn ai_move(&mut self) {
        if self.status != GameStatus::Playing && self.status != GameStatus::Check {
            return;
        }
        let moves = gen_all_legal_moves(self, Turn::Black);
        if moves.is_empty() { return; }

        let depth = 3;
        let mut best_score = i32::MIN + 1;
        let mut best_indices: Vec<usize> = Vec::new();

        for (i, m) in moves.iter().enumerate() {
            let mut sim = self.clone();
            sim.apply_move(m);
            let score = minimax(&sim, depth - 1, i32::MIN + 1, i32::MAX, false);
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

// ── Check detection ───────────────────────────────────────────────────────────

/// Returns `true` if `turn`'s king is under attack.
pub fn is_in_check(cells: &[[Square; 8]; 8], turn: Turn) -> bool {
    // Find the king.
    let king_color = match turn { Turn::White => Color::White, Turn::Black => Color::Black };
    let mut king_pos = None;
    'outer: for row in 0..8 {
        for col in 0..8 {
            if let Some(p) = cells[row][col] {
                if p.color == king_color && p.kind == PieceKind::King {
                    king_pos = Some((row, col));
                    break 'outer;
                }
            }
        }
    }
    let king_pos = match king_pos { Some(p) => p, None => return false };
    let opp_turn = match turn { Turn::White => Turn::Black, Turn::Black => Turn::White };

    // Generate all pseudo-legal moves for the opponent.
    for row in 0..8 {
        for col in 0..8 {
            if let Some(p) = cells[row][col] {
                let p_turn = match p.color { Color::White => Turn::White, Color::Black => Turn::Black };
                if p_turn != opp_turn { continue; }
                let moves = pseudo_moves(row, col, cells, p, None);
                if moves.iter().any(|m| m.to == king_pos) {
                    return true;
                }
            }
        }
    }
    false
}

// ── Legal move generation ─────────────────────────────────────────────────────

fn gen_all_legal_moves(board: &Board, turn: Turn) -> Vec<ChessMove> {
    let color = match turn { Turn::White => Color::White, Turn::Black => Color::Black };
    let mut result = Vec::new();

    for row in 0..8 {
        for col in 0..8 {
            if let Some(p) = board.cells[row][col] {
                if p.color != color { continue; }
                let pseudo = pseudo_moves(row, col, &board.cells, p, board.en_passant.as_ref().copied());
                for m in pseudo {
                    // Filter: must not leave own king in check.
                    let mut sim_cells = board.cells;
                    apply_move_to_cells(&mut sim_cells, &m);
                    if !is_in_check(&sim_cells, turn) {
                        result.push(m);
                    }
                }
            }
        }
    }

    // Castling (only when not currently in check).
    if !is_in_check(&board.cells, turn) {
        result.extend(gen_castling_moves(board, turn));
    }

    result
}

fn apply_move_to_cells(cells: &mut [[Square; 8]; 8], m: &ChessMove) {
    let piece = match cells[m.from.0][m.from.1] {
        Some(p) => p,
        None    => return,
    };
    if let Some(ep) = m.ep_capture {
        cells[ep.0][ep.1] = None;
    }
    cells[m.from.0][m.from.1] = None;
    cells[m.to.0][m.to.1] = if let Some(promo) = m.promotion {
        Some(Piece::new(promo, piece.color))
    } else {
        Some(piece)
    };
    if let Some((rf, rt)) = m.castle_rook {
        let rook = cells[rf.0][rf.1];
        cells[rf.0][rf.1] = None;
        cells[rt.0][rt.1] = rook;
    }
}

// ── Castling ──────────────────────────────────────────────────────────────────

fn gen_castling_moves(board: &Board, turn: Turn) -> Vec<ChessMove> {
    let mut result = Vec::new();
    match turn {
        Turn::White => {
            let row = 7;
            // King-side.
            if board.castle_white_k
                && board.cells[row][5].is_none()
                && board.cells[row][6].is_none()
                && !square_attacked(&board.cells, row, 5, Turn::Black)
                && !square_attacked(&board.cells, row, 6, Turn::Black)
            {
                result.push(ChessMove {
                    from: (row, 4), to: (row, 6),
                    promotion: None, ep_capture: None,
                    castle_rook: Some(((row, 7), (row, 5))),
                });
            }
            // Queen-side.
            if board.castle_white_q
                && board.cells[row][1].is_none()
                && board.cells[row][2].is_none()
                && board.cells[row][3].is_none()
                && !square_attacked(&board.cells, row, 3, Turn::Black)
                && !square_attacked(&board.cells, row, 2, Turn::Black)
            {
                result.push(ChessMove {
                    from: (row, 4), to: (row, 2),
                    promotion: None, ep_capture: None,
                    castle_rook: Some(((row, 0), (row, 3))),
                });
            }
        }
        Turn::Black => {
            let row = 0;
            // King-side.
            if board.castle_black_k
                && board.cells[row][5].is_none()
                && board.cells[row][6].is_none()
                && !square_attacked(&board.cells, row, 5, Turn::White)
                && !square_attacked(&board.cells, row, 6, Turn::White)
            {
                result.push(ChessMove {
                    from: (row, 4), to: (row, 6),
                    promotion: None, ep_capture: None,
                    castle_rook: Some(((row, 7), (row, 5))),
                });
            }
            // Queen-side.
            if board.castle_black_q
                && board.cells[row][1].is_none()
                && board.cells[row][2].is_none()
                && board.cells[row][3].is_none()
                && !square_attacked(&board.cells, row, 3, Turn::White)
                && !square_attacked(&board.cells, row, 2, Turn::White)
            {
                result.push(ChessMove {
                    from: (row, 4), to: (row, 2),
                    promotion: None, ep_capture: None,
                    castle_rook: Some(((row, 0), (row, 3))),
                });
            }
        }
    }
    result
}

fn square_attacked(cells: &[[Square; 8]; 8], row: usize, col: usize, by: Turn) -> bool {
    let color = match by { Turn::White => Color::White, Turn::Black => Color::Black };
    for r in 0..8 {
        for c in 0..8 {
            if let Some(p) = cells[r][c] {
                if p.color != color { continue; }
                let moves = pseudo_moves(r, c, cells, p, None);
                if moves.iter().any(|m| m.to == (row, col)) {
                    return true;
                }
            }
        }
    }
    false
}

// ── Pseudo-legal move generation ──────────────────────────────────────────────

fn pseudo_moves(
    row: usize, col: usize,
    cells: &[[Square; 8]; 8],
    piece: Piece,
    ep: Option<(usize, usize)>,
) -> Vec<ChessMove> {
    match piece.kind {
        PieceKind::Pawn   => pawn_moves(row, col, cells, piece.color, ep),
        PieceKind::Knight => knight_moves(row, col, cells, piece.color),
        PieceKind::Bishop => sliding_moves(row, col, cells, piece.color, &[(-1,-1),(-1,1),(1,-1),(1,1)]),
        PieceKind::Rook   => sliding_moves(row, col, cells, piece.color, &[(-1,0),(1,0),(0,-1),(0,1)]),
        PieceKind::Queen  => {
            let mut m = sliding_moves(row, col, cells, piece.color, &[(-1,-1),(-1,1),(1,-1),(1,1)]);
            m.extend(sliding_moves(row, col, cells, piece.color, &[(-1,0),(1,0),(0,-1),(0,1)]));
            m
        }
        PieceKind::King => king_moves(row, col, cells, piece.color),
    }
}

fn pawn_moves(
    row: usize, col: usize,
    cells: &[[Square; 8]; 8],
    color: Color,
    ep: Option<(usize, usize)>,
) -> Vec<ChessMove> {
    let mut moves = Vec::new();
    let dir: i32 = match color { Color::White => -1, Color::Black => 1 };
    let start_row: usize = match color { Color::White => 6, Color::Black => 1 };
    let promo_row: usize = match color { Color::White => 0, Color::Black => 7 };

    let nr = row as i32 + dir;
    if nr < 0 || nr > 7 { return moves; }
    let nr = nr as usize;

    // Single push.
    if cells[nr][col].is_none() {
        if nr == promo_row {
            for &k in &[PieceKind::Queen, PieceKind::Rook, PieceKind::Bishop, PieceKind::Knight] {
                moves.push(ChessMove { from: (row,col), to: (nr,col), promotion: Some(k), ep_capture: None, castle_rook: None });
            }
        } else {
            moves.push(ChessMove::simple((row, col), (nr, col)));
            // Double push.
            if row == start_row {
                let nr2 = (row as i32 + 2 * dir) as usize;
                if cells[nr2][col].is_none() {
                    moves.push(ChessMove::simple((row, col), (nr2, col)));
                }
            }
        }
    }

    // Captures.
    for dc in [-1i32, 1] {
        let nc = col as i32 + dc;
        if nc < 0 || nc > 7 { continue; }
        let nc = nc as usize;
        if let Some(target) = cells[nr][nc] {
            if target.color != color {
                if nr == promo_row {
                    for &k in &[PieceKind::Queen, PieceKind::Rook, PieceKind::Bishop, PieceKind::Knight] {
                        moves.push(ChessMove { from: (row,col), to: (nr,nc), promotion: Some(k), ep_capture: None, castle_rook: None });
                    }
                } else {
                    moves.push(ChessMove::simple((row, col), (nr, nc)));
                }
            }
        }
        // En-passant.
        if let Some(ep_sq) = ep {
            if ep_sq == (nr, nc) {
                let captured_row = row; // The captured pawn is on the same row as the capturing pawn.
                moves.push(ChessMove {
                    from: (row, col), to: (nr, nc),
                    promotion: None,
                    ep_capture: Some((captured_row, nc)),
                    castle_rook: None,
                });
            }
        }
    }
    moves
}

fn knight_moves(row: usize, col: usize, cells: &[[Square; 8]; 8], color: Color) -> Vec<ChessMove> {
    let mut moves = Vec::new();
    let jumps: [(i32,i32); 8] = [(-2,-1),(-2,1),(-1,-2),(-1,2),(1,-2),(1,2),(2,-1),(2,1)];
    for (dr, dc) in jumps {
        let nr = row as i32 + dr;
        let nc = col as i32 + dc;
        if nr < 0 || nr > 7 || nc < 0 || nc > 7 { continue; }
        let (nr, nc) = (nr as usize, nc as usize);
        match cells[nr][nc] {
            None                         => moves.push(ChessMove::simple((row,col),(nr,nc))),
            Some(p) if p.color != color  => moves.push(ChessMove::simple((row,col),(nr,nc))),
            _ => {}
        }
    }
    moves
}

fn sliding_moves(
    row: usize, col: usize,
    cells: &[[Square; 8]; 8],
    color: Color,
    dirs: &[(i32,i32)],
) -> Vec<ChessMove> {
    let mut moves = Vec::new();
    for &(dr, dc) in dirs {
        let mut nr = row as i32 + dr;
        let mut nc = col as i32 + dc;
        while nr >= 0 && nr < 8 && nc >= 0 && nc < 8 {
            let (r, c) = (nr as usize, nc as usize);
            match cells[r][c] {
                None => {
                    moves.push(ChessMove::simple((row,col),(r,c)));
                }
                Some(p) if p.color != color => {
                    moves.push(ChessMove::simple((row,col),(r,c)));
                    break;
                }
                Some(_) => break,
            }
            nr += dr;
            nc += dc;
        }
    }
    moves
}

fn king_moves(row: usize, col: usize, cells: &[[Square; 8]; 8], color: Color) -> Vec<ChessMove> {
    let mut moves = Vec::new();
    for dr in -1i32..=1 {
        for dc in -1i32..=1 {
            if dr == 0 && dc == 0 { continue; }
            let nr = row as i32 + dr;
            let nc = col as i32 + dc;
            if nr < 0 || nr > 7 || nc < 0 || nc > 7 { continue; }
            let (nr, nc) = (nr as usize, nc as usize);
            match cells[nr][nc] {
                None                         => moves.push(ChessMove::simple((row,col),(nr,nc))),
                Some(p) if p.color != color  => moves.push(ChessMove::simple((row,col),(nr,nc))),
                _ => {}
            }
        }
    }
    moves
}

// ── Evaluation ────────────────────────────────────────────────────────────────

/// Material values in centipawns.
fn piece_value(kind: PieceKind) -> i32 {
    match kind {
        PieceKind::Pawn   => 100,
        PieceKind::Knight => 320,
        PieceKind::Bishop => 330,
        PieceKind::Rook   => 500,
        PieceKind::Queen  => 900,
        PieceKind::King   => 20000,
    }
}

// Piece-square tables (White's perspective, row 0 = rank 8).
const PAWN_TABLE: [[i32; 8]; 8] = [
    [ 0,  0,  0,  0,  0,  0,  0,  0],
    [50, 50, 50, 50, 50, 50, 50, 50],
    [10, 10, 20, 30, 30, 20, 10, 10],
    [ 5,  5, 10, 25, 25, 10,  5,  5],
    [ 0,  0,  0, 20, 20,  0,  0,  0],
    [ 5, -5,-10,  0,  0,-10, -5,  5],
    [ 5, 10, 10,-20,-20, 10, 10,  5],
    [ 0,  0,  0,  0,  0,  0,  0,  0],
];

const KNIGHT_TABLE: [[i32; 8]; 8] = [
    [-50,-40,-30,-30,-30,-30,-40,-50],
    [-40,-20,  0,  0,  0,  0,-20,-40],
    [-30,  0, 10, 15, 15, 10,  0,-30],
    [-30,  5, 15, 20, 20, 15,  5,-30],
    [-30,  0, 15, 20, 20, 15,  0,-30],
    [-30,  5, 10, 15, 15, 10,  5,-30],
    [-40,-20,  0,  5,  5,  0,-20,-40],
    [-50,-40,-30,-30,-30,-30,-40,-50],
];

const BISHOP_TABLE: [[i32; 8]; 8] = [
    [-20,-10,-10,-10,-10,-10,-10,-20],
    [-10,  0,  0,  0,  0,  0,  0,-10],
    [-10,  0,  5, 10, 10,  5,  0,-10],
    [-10,  5,  5, 10, 10,  5,  5,-10],
    [-10,  0, 10, 10, 10, 10,  0,-10],
    [-10, 10, 10, 10, 10, 10, 10,-10],
    [-10,  5,  0,  0,  0,  0,  5,-10],
    [-20,-10,-10,-10,-10,-10,-10,-20],
];

const ROOK_TABLE: [[i32; 8]; 8] = [
    [ 0,  0,  0,  0,  0,  0,  0,  0],
    [ 5, 10, 10, 10, 10, 10, 10,  5],
    [-5,  0,  0,  0,  0,  0,  0, -5],
    [-5,  0,  0,  0,  0,  0,  0, -5],
    [-5,  0,  0,  0,  0,  0,  0, -5],
    [-5,  0,  0,  0,  0,  0,  0, -5],
    [-5,  0,  0,  0,  0,  0,  0, -5],
    [ 0,  0,  0,  5,  5,  0,  0,  0],
];

const QUEEN_TABLE: [[i32; 8]; 8] = [
    [-20,-10,-10, -5, -5,-10,-10,-20],
    [-10,  0,  0,  0,  0,  0,  0,-10],
    [-10,  0,  5,  5,  5,  5,  0,-10],
    [ -5,  0,  5,  5,  5,  5,  0, -5],
    [  0,  0,  5,  5,  5,  5,  0, -5],
    [-10,  5,  5,  5,  5,  5,  0,-10],
    [-10,  0,  5,  0,  0,  0,  0,-10],
    [-20,-10,-10, -5, -5,-10,-10,-20],
];

const KING_TABLE: [[i32; 8]; 8] = [
    [-30,-40,-40,-50,-50,-40,-40,-30],
    [-30,-40,-40,-50,-50,-40,-40,-30],
    [-30,-40,-40,-50,-50,-40,-40,-30],
    [-30,-40,-40,-50,-50,-40,-40,-30],
    [-20,-30,-30,-40,-40,-30,-30,-20],
    [-10,-20,-20,-20,-20,-20,-20,-10],
    [ 20, 20,  0,  0,  0,  0, 20, 20],
    [ 20, 30, 10,  0,  0, 10, 30, 20],
];

fn pst_value(kind: PieceKind, row: usize, col: usize, color: Color) -> i32 {
    // For Black, mirror the table vertically.
    let r = match color { Color::White => row, Color::Black => 7 - row };
    match kind {
        PieceKind::Pawn   => PAWN_TABLE[r][col],
        PieceKind::Knight => KNIGHT_TABLE[r][col],
        PieceKind::Bishop => BISHOP_TABLE[r][col],
        PieceKind::Rook   => ROOK_TABLE[r][col],
        PieceKind::Queen  => QUEEN_TABLE[r][col],
        PieceKind::King   => KING_TABLE[r][col],
    }
}

/// Static evaluation from Black's perspective (positive = Black is better).
fn evaluate(board: &Board) -> i32 {
    match &board.status {
        GameStatus::AiWon      => return 100_000,
        GameStatus::PlayerWon  => return -100_000,
        GameStatus::Stalemate | GameStatus::Draw => return 0,
        _ => {}
    }

    let mut score = 0i32;
    for row in 0..8 {
        for col in 0..8 {
            if let Some(p) = board.cells[row][col] {
                let v = piece_value(p.kind) + pst_value(p.kind, row, col, p.color);
                if p.is_black() {
                    score += v;
                } else {
                    score -= v;
                }
            }
        }
    }
    score
}

// ── Minimax AI ────────────────────────────────────────────────────────────────

fn minimax(board: &Board, depth: u8, mut alpha: i32, mut beta: i32, maximising: bool) -> i32 {
    if depth == 0
        || matches!(board.status,
            GameStatus::AiWon | GameStatus::PlayerWon | GameStatus::Stalemate | GameStatus::Draw)
    {
        return evaluate(board);
    }

    let moves = board.gen_moves();
    if moves.is_empty() { return evaluate(board); }

    if maximising {
        // Black is maximising.
        let mut value = i32::MIN + 1;
        for m in &moves {
            let mut sim = board.clone();
            sim.apply_move(m);
            value = value.max(minimax(&sim, depth - 1, alpha, beta, false));
            alpha = alpha.max(value);
            if alpha >= beta { break; }
        }
        value
    } else {
        // White is minimising.
        let mut value = i32::MAX;
        for m in &moves {
            let mut sim = board.clone();
            sim.apply_move(m);
            value = value.min(minimax(&sim, depth - 1, alpha, beta, true));
            beta = beta.min(value);
            if beta <= alpha { break; }
        }
        value
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starting_position_has_correct_pieces() {
        let board = Board::new();
        // White pawns on row 6.
        for col in 0..8 {
            assert_eq!(board.cells[6][col], Some(Piece::new(PieceKind::Pawn, Color::White)));
        }
        // Black pawns on row 1.
        for col in 0..8 {
            assert_eq!(board.cells[1][col], Some(Piece::new(PieceKind::Pawn, Color::Black)));
        }
        // Middle rows are empty.
        for row in 2..6 {
            for col in 0..8 {
                assert_eq!(board.cells[row][col], None);
            }
        }
    }

    #[test]
    fn white_pawn_has_two_starting_moves() {
        let board = Board::new();
        let moves = gen_all_legal_moves(&board, Turn::White);
        // 16 pawn moves (each of 8 pawns has 2 moves) + 4 knight moves = 20.
        assert_eq!(moves.len(), 20);
    }

    #[test]
    fn not_in_check_at_start() {
        let board = Board::new();
        assert!(!is_in_check(&board.cells, Turn::White));
        assert!(!is_in_check(&board.cells, Turn::Black));
    }

    #[test]
    fn apply_move_advances_turn() {
        let mut board = Board::new();
        assert_eq!(board.turn, Turn::White);
        let m = ChessMove::simple((6, 4), (4, 4)); // e4
        board.apply_move(&m);
        assert_eq!(board.turn, Turn::Black);
    }
}
