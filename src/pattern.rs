use std::ops::{BitAnd, BitOr, Not};

/// Bit field of Sudoku cells.
///
/// Row-major order.  Bit 0 of the first `u32` is the top-left cell; bit 16 of
/// the third `u32` is the bottom-right cell.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Pattern(pub [u32; 3]);

impl Pattern {
    pub const EMPTY: Pattern = Pattern([0, 0, 0]);
    pub const FULL: Pattern = Pattern([0xFFFFFFFF, 0xFFFFFFFF, 0x1FFFF]);

    /// Does the pattern contain the cell?
    pub fn has(self, row: usize, col: usize) -> bool {
        let idx = 9 * row + col;
        (self.0[idx / 32] & (1 << (idx % 32))) != 0
    }

    /// Remove the cell, returning true if it was previously in the pattern.
    pub fn remove(&mut self, row: usize, col: usize) -> bool {
        let old = self.has(row, col);
        let idx = 9 * row + col;
        self.0[idx / 32] &= !(1 << (idx % 32));
        old
    }

    /// New pattern also containing the given cell.
    #[must_use]
    #[allow(unused_parens)]
    pub fn with(mut self, row: usize, col: usize) -> Pattern {
        let idx = 9 * row + col;
        self.0[idx / 32] |= (1 << (idx % 32));
        self
    }

    pub fn is_subset(self, other: Pattern) -> bool {
        (self & other) == self
    }

    pub fn intersects(self, other: Pattern) -> bool {
        (self & other) != Pattern::EMPTY
    }
}

impl BitAnd for Pattern {
    type Output = Pattern;
    fn bitand(self, rhs: Self) -> Pattern {
        Pattern([
            self.0[0] & rhs.0[0],
            self.0[1] & rhs.0[1],
            self.0[2] & rhs.0[2],
        ])
    }
}

impl BitOr for Pattern {
    type Output = Pattern;
    fn bitor(self, rhs: Self) -> Pattern {
        Pattern([
            self.0[0] | rhs.0[0],
            self.0[1] | rhs.0[1],
            self.0[2] | rhs.0[2],
        ])
    }
}

impl Not for Pattern {
    type Output = Pattern;
    fn not(self) -> Self::Output {
        Pattern([!self.0[0], !self.0[1], self.0[2] ^ 0x1FFFF])
    }
}

impl std::fmt::Display for Pattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in 0..9 {
            if row == 3 || row == 6 {
                write!(f, "---+---+---\n")?;
            }
            for col in 0..9 {
                if col == 3 || col == 6 {
                    write!(f, "|")?;
                }
                if self.has(row, col) {
                    write!(f, "X")?;
                } else {
                    write!(f, " ")?;
                }
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}
