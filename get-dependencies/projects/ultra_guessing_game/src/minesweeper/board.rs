use rand::seq::SliceRandom;
use rand::thread_rng;

/// Difficulty levels – each maps to a board size and trap count.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Level {
    /// 8 × 8 grid, 10 cursed traps – suitable for a peasant.
    Peasant,
    /// 16 × 16 grid, 40 cursed traps – a knight's trial.
    Knight,
    /// 30 × 16 grid, 99 cursed traps – only champions dare.
    Champion,
}

impl Level {
    pub fn rows(&self) -> usize {
        match self {
            Level::Peasant => 8,
            Level::Knight => 16,
            Level::Champion => 16,
        }
    }

    pub fn cols(&self) -> usize {
        match self {
            Level::Peasant => 8,
            Level::Knight => 16,
            Level::Champion => 30,
        }
    }

    pub fn traps(&self) -> usize {
        match self {
            Level::Peasant => 10,
            Level::Knight => 40,
            Level::Champion => 99,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Level::Peasant => "Peasant's Trial",
            Level::Knight => "Knight's Gauntlet",
            Level::Champion => "Champion's Ordeal",
        }
    }

    pub fn flavour(&self) -> &'static str {
        match self {
            Level::Peasant => "A modest ruin – even a farmhand might survive.",
            Level::Knight => "The goblin-riddled fortress – only the bold venture here.",
            Level::Champion => "The cursed battlefield – legends are forged or broken within.",
        }
    }
}

/// The state of a single cell.
#[derive(Clone, Debug)]
pub struct Cell {
    /// True if this cell conceals a goblin trap.
    pub is_trap: bool,
    /// Number of adjacent traps (0–8).
    pub adjacent: u8,
    /// True once the player has revealed this cell.
    pub revealed: bool,
    /// True when the player has planted a rune marker here.
    pub flagged: bool,
}

impl Default for Cell {
    fn default() -> Self {
        Cell { is_trap: false, adjacent: 0, revealed: false, flagged: false }
    }
}

/// The overall state of a single game.
#[derive(Clone, Debug, PartialEq)]
pub enum GameStatus {
    Playing,
    Won,
    Lost,
}

pub struct Board {
    pub level: Level,
    pub rows: usize,
    pub cols: usize,
    pub cells: Vec<Cell>,
    pub status: GameStatus,
    /// Number of rune markers currently placed.
    pub flags_placed: usize,
    /// Total number of traps hidden on the board.
    pub total_traps: usize,
    /// True after the first reveal – traps are placed on first move so the
    /// starting cell is always safe.
    pub initialized: bool,
}

impl Board {
    /// Create a blank (un-initialized) board for the given level.
    pub fn new(level: Level) -> Self {
        let rows = level.rows();
        let cols = level.cols();
        Board {
            level,
            rows,
            cols,
            cells: vec![Cell::default(); rows * cols],
            status: GameStatus::Playing,
            flags_placed: 0,
            total_traps: level.traps(),
            initialized: false,
        }
    }

    fn idx(&self, row: usize, col: usize) -> usize {
        row * self.cols + col
    }

    /// Place traps randomly, guaranteeing `safe_row/safe_col` and its
    /// neighbours are trap-free (classic first-click safety rule).
    pub fn initialize(&mut self, safe_row: usize, safe_col: usize) {
        let rows = self.rows;
        let cols = self.cols;

        // Build the set of forbidden positions (first click + its neighbours).
        let mut forbidden = std::collections::HashSet::new();
        for dr in -1i32..=1 {
            for dc in -1i32..=1 {
                let r = safe_row as i32 + dr;
                let c = safe_col as i32 + dc;
                if r >= 0 && r < rows as i32 && c >= 0 && c < cols as i32 {
                    forbidden.insert((r as usize, c as usize));
                }
            }
        }

        // Candidate positions for traps.
        let mut candidates: Vec<(usize, usize)> = (0..rows)
            .flat_map(|r| (0..cols).map(move |c| (r, c)))
            .filter(|pos| !forbidden.contains(pos))
            .collect();

        candidates.shuffle(&mut thread_rng());
        let trap_count = self.total_traps.min(candidates.len());

        for &(r, c) in &candidates[..trap_count] {
            let idx = self.idx(r, c);
            self.cells[idx].is_trap = true;
        }

        // Compute adjacency numbers.
        for row in 0..rows {
            for col in 0..cols {
                let idx = self.idx(row, col);
                if !self.cells[idx].is_trap {
                    let adj = self.count_adjacent_traps(row, col);
                    self.cells[idx].adjacent = adj;
                }
            }
        }

        self.initialized = true;
    }

    fn count_adjacent_traps(&self, row: usize, col: usize) -> u8 {
        let mut count = 0u8;
        for dr in -1i32..=1 {
            for dc in -1i32..=1 {
                if dr == 0 && dc == 0 { continue; }
                let r = row as i32 + dr;
                let c = col as i32 + dc;
                if r >= 0 && r < self.rows as i32 && c >= 0 && c < self.cols as i32 {
                    if self.cells[self.idx(r as usize, c as usize)].is_trap {
                        count += 1;
                    }
                }
            }
        }
        count
    }

    /// Reveal a cell.  Returns false if the cell was already revealed or
    /// flagged (no-op).  Sets status to Lost if a trap is hit.
    pub fn reveal(&mut self, row: usize, col: usize) -> bool {
        if self.status != GameStatus::Playing { return false; }

        // Initialize board on first reveal.
        if !self.initialized {
            self.initialize(row, col);
        }

        let idx = self.idx(row, col);
        if self.cells[idx].revealed || self.cells[idx].flagged {
            return false;
        }

        self.cells[idx].revealed = true;

        if self.cells[idx].is_trap {
            self.status = GameStatus::Lost;
            // Reveal all traps for the game-over display.
            for cell in &mut self.cells {
                if cell.is_trap { cell.revealed = true; }
            }
            return true;
        }

        // Flood-fill if zero adjacent traps.
        if self.cells[idx].adjacent == 0 {
            self.flood_reveal(row, col);
        }

        self.check_win();
        true
    }

    /// BFS flood-fill: reveal connected zero-adjacent cells and their borders.
    fn flood_reveal(&mut self, start_row: usize, start_col: usize) {
        let mut queue = std::collections::VecDeque::new();
        queue.push_back((start_row, start_col));

        while let Some((row, col)) = queue.pop_front() {
            for dr in -1i32..=1 {
                for dc in -1i32..=1 {
                    if dr == 0 && dc == 0 { continue; }
                    let r = row as i32 + dr;
                    let c = col as i32 + dc;
                    if r < 0 || r >= self.rows as i32 || c < 0 || c >= self.cols as i32 {
                        continue;
                    }
                    let nr = r as usize;
                    let nc = c as usize;
                    let nidx = self.idx(nr, nc);
                    if !self.cells[nidx].revealed && !self.cells[nidx].is_trap
                        && !self.cells[nidx].flagged {
                        self.cells[nidx].revealed = true;
                        if self.cells[nidx].adjacent == 0 {
                            queue.push_back((nr, nc));
                        }
                    }
                }
            }
        }
    }

    /// Toggle a rune-marker (flag) on an unrevealed cell.
    pub fn toggle_flag(&mut self, row: usize, col: usize) {
        if self.status != GameStatus::Playing { return; }
        let idx = self.idx(row, col);
        if self.cells[idx].revealed { return; }
        if self.cells[idx].flagged {
            self.cells[idx].flagged = false;
            self.flags_placed = self.flags_placed.saturating_sub(1);
        } else {
            self.cells[idx].flagged = true;
            self.flags_placed += 1;
        }
    }

    /// Win condition: all non-trap cells revealed.
    fn check_win(&mut self) {
        let unrevealed_safe = self.cells.iter()
            .filter(|c| !c.is_trap && !c.revealed)
            .count();
        if unrevealed_safe == 0 {
            self.status = GameStatus::Won;
        }
    }

    pub fn cell(&self, row: usize, col: usize) -> &Cell {
        &self.cells[self.idx(row, col)]
    }

    /// How many traps remain un-flagged (can be negative if over-flagged).
    pub fn traps_remaining(&self) -> i32 {
        self.total_traps as i32 - self.flags_placed as i32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn board_dimensions_match_level() {
        for level in [Level::Peasant, Level::Knight, Level::Champion] {
            let b = Board::new(level);
            assert_eq!(b.rows, level.rows());
            assert_eq!(b.cols, level.cols());
            assert_eq!(b.cells.len(), b.rows * b.cols);
        }
    }

    #[test]
    fn first_reveal_never_hits_trap() {
        // Statistical test – run many times on the smallest board.
        for _ in 0..200 {
            let mut b = Board::new(Level::Peasant);
            b.reveal(3, 3);
            assert_ne!(b.status, GameStatus::Lost, "first reveal should never be a trap");
        }
    }

    #[test]
    fn flag_toggle_works() {
        let mut b = Board::new(Level::Peasant);
        b.toggle_flag(0, 0);
        assert!(b.cell(0, 0).flagged);
        assert_eq!(b.flags_placed, 1);
        b.toggle_flag(0, 0);
        assert!(!b.cell(0, 0).flagged);
        assert_eq!(b.flags_placed, 0);
    }

    #[test]
    fn cannot_reveal_flagged_cell() {
        let mut b = Board::new(Level::Peasant);
        b.toggle_flag(0, 0);
        let changed = b.reveal(0, 0);
        assert!(!changed);
    }

    #[test]
    fn trap_count_correct_after_init() {
        let mut b = Board::new(Level::Peasant);
        b.initialize(3, 3);
        let actual = b.cells.iter().filter(|c| c.is_trap).count();
        assert_eq!(actual, Level::Peasant.traps());
    }

    #[test]
    fn adjacency_counts_are_consistent() {
        let mut b = Board::new(Level::Peasant);
        b.initialize(0, 0);
        for row in 0..b.rows {
            for col in 0..b.cols {
                if !b.cell(row, col).is_trap {
                    let expected = b.count_adjacent_traps(row, col);
                    assert_eq!(b.cell(row, col).adjacent, expected);
                }
            }
        }
    }

    #[test]
    fn traps_remaining_reflects_flags() {
        let mut b = Board::new(Level::Peasant);
        assert_eq!(b.traps_remaining(), 10);
        b.toggle_flag(0, 0);
        assert_eq!(b.traps_remaining(), 9);
        b.toggle_flag(0, 0);
        assert_eq!(b.traps_remaining(), 10);
    }
}
