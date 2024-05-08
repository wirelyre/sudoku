use crate::{pattern::Pattern, Solution, Template};

/**
 Prepared form of a puzzle, applying logic to the input.

 This completely resolves:

   1. If a cell contains only a single digit, then no other cell in the row,
      column, or box can contain that digit.
   2. If a row, column, or box contains only one of a specific digit, then the
      cell containing that digit contains *only* that digit.

 This is enough to completely solve basic puzzles, and it's real fast too.

 In the best case, this narrows down the search space a lot.  In the worst case,
 barely any logic is done and it's no worse than initializing a search directly
 from the puzzle.

 # Algorithm

 Imagine an empty puzzle with no clues.  Let's create a solution grid: a 9×9
 grid where each cell contains all 9 digits *at the same time*.  Once we know a
 cell cannot contain a digit, we'll erase the digit from the grid.

 We can think of the solution grid as a 9×9×9×1 data structure:

   - 9 rows by
   - 9 columns by
   - 9 digits by
   - 1 `bool` (is the digit possible?)

 The rules of Sudoku establish four kinds of constraints on the possible digits,
 each corresponding to a "slice" of the solution grid selecting 9 `bool`s:

   - **Cell:**  Fix a row and column.  The cell contains exactly one of some digit.
   - **Row:**  Fix a row and digit.  The row contains exactly one of that digit.
   - **Column:**  Fix a column and digit.  The column contains exactly one of that digit.
   - **Box:**  Fix a box and digit.  The box contains exactly one of that digit.

 Notice the nature of these constraints:  Select 9 `bool` entries in the
 solution grid, then ensure that exactly one is `true`.

 This suggests a solving algorithm based on the interactions between cell and
 row/column/box constraints:  If a cell contains exactly one digit, then the
 digit is unique in the row/column/box.  Similarly, if a digit is unique in a
 row/column/box, then it's unique in its cell.  Apply these rules until no more
 progress can be made.

 The code here does this, except the constraints are represented by the number
 of `true` entries in the corresponding slice.  These counts are decremented
 when a digit is found.  When a count hits 1, a new digit has been found, and
 more work is enqueued.
*/

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Possibilities {
    /// [`Pattern`] for each digit
    pub patterns: [Pattern; 9],

    /// Queue of `(row, col, digit)` triples to eliminate
    work_queue: Vec<(u8, u8, u8)>,
    cell_constraints: [[u8; 9]; 9], // [row][col] -> number of digits in cell
    row_constraints: [[u8; 9]; 9],  // [row][dig] -> number of `dig`s in row
    col_constraints: [[u8; 9]; 9],  // [col][dig] -> number of `dig`s in col
    box_constraints: [[u8; 9]; 9],  // [box][dig] -> number of `dig`s in box
                                    // boxes are indexed row-major, like `Pattern` cells
}

/// Error returned when initializing a [`Possibilities`] fails.
#[derive(Clone, Copy, Debug)]
pub struct ImpossiblePuzzle;

impl Possibilities {
    /// Fresh logic machine where every digit is possible in every cell.
    pub fn new() -> Possibilities {
        Possibilities {
            patterns: [Pattern::FULL; 9],

            work_queue: Vec::new(),
            cell_constraints: [[9; 9]; 9],
            row_constraints: [[9; 9]; 9],
            col_constraints: [[9; 9]; 9],
            box_constraints: [[9; 9]; 9],
        }
    }

    /// Remove all other digits from this cell, and apply logic.
    pub fn set(&mut self, row: u8, col: u8, digit: u8) -> Result<(), ImpossiblePuzzle> {
        self.enqueue_others((row as usize, col as usize), digit as usize - 1);
        self.work()
    }

    /// Run work queue until empty.
    fn work(&mut self) -> Result<(), ImpossiblePuzzle> {
        while let Some((row, col, digit)) = self.work_queue.pop() {
            self.eliminate(row as usize, col as usize, digit as usize)?;
        }
        Ok(())
    }

    /// Enqueue removing a single digit from a cell.
    fn enqueue(&mut self, (row, col): (usize, usize), digit: usize) {
        self.work_queue.push((row as u8, col as u8, digit as u8));
    }
    /// Enqueue removing all other digits from a cell.
    fn enqueue_others(&mut self, (row, col): (usize, usize), digit: usize) {
        (0..9)
            .filter(|&d| d != digit)
            .for_each(|d| self.enqueue((row, col), d));
    }
    /// Enqueue removing this digit from all adjacent cells (rest of row, col, box).
    fn enqueue_adjacent(&mut self, (row, col): (usize, usize), digit: usize) {
        for other_col in 0..9 {
            if other_col != col {
                self.enqueue((row, other_col), digit);
            }
        }
        for other_row in 0..9 {
            if other_row != row {
                self.enqueue((other_row, col), digit);
            }
        }
        for (other_row, other_col) in box_cells(row, col) {
            if other_row != row || other_col != col {
                self.enqueue((other_row, other_col), digit);
            }
        }
    }

    /// Eliminate a digit, update constraints, and enqueue work if necessary.
    fn eliminate(&mut self, row: usize, col: usize, digit: usize) -> Result<(), ImpossiblePuzzle> {
        let old = self.patterns[digit].remove(row, col);
        if old == false {
            // digit already eliminated
            return Ok(());
        }

        self.cell_constraints[row][col] -= 1;
        match self.cell_constraints[row][col] {
            0 => return Err(ImpossiblePuzzle),
            1 => self.enqueue_adjacent((row, col), self.find_in_cell(row, col)),
            2.. => {}
        }

        self.row_constraints[row][digit] -= 1;
        match self.row_constraints[row][digit] {
            0 => return Err(ImpossiblePuzzle),
            1 => self.enqueue_others((row, self.find_in_row(row, digit)), digit),
            2.. => {}
        }

        self.col_constraints[col][digit] -= 1;
        match self.col_constraints[col][digit] {
            0 => return Err(ImpossiblePuzzle),
            1 => self.enqueue_others((self.find_in_col(col, digit), col), digit),
            2.. => {}
        }

        let box_ = row / 3 * 3 + col / 3;
        self.box_constraints[box_][digit] -= 1;
        match self.box_constraints[box_][digit] {
            0 => return Err(ImpossiblePuzzle),
            1 => self.enqueue_others(self.find_in_box(row, col, digit), digit),
            2.. => {}
        }

        Ok(())
    }

    /// Find unique digit in cell.
    fn find_in_cell(&self, row: usize, col: usize) -> usize {
        (0..9)
            .find(|&d| self.patterns[d].has(row, col))
            .expect("no digit in cell")
    }
    /// Find unique digit in row.
    fn find_in_row(&self, row: usize, digit: usize) -> usize {
        (0..9)
            .find(|col| self.patterns[digit].has(row, *col))
            .expect("no digit in row")
    }
    /// Find unique digit in column.
    fn find_in_col(&self, col: usize, digit: usize) -> usize {
        (0..9)
            .find(|row| self.patterns[digit].has(*row, col))
            .expect("no digit in column")
    }
    /// Find row and column of given digit in box containing given cell.
    fn find_in_box(&self, row: usize, col: usize, digit: usize) -> (usize, usize) {
        box_cells(row, col)
            .into_iter()
            .find(|(row, col)| self.patterns[digit].has(*row, *col))
            .expect("no digit in box")
    }

    /// If the solution is unique, return it.
    pub fn unique(&self) -> Option<Solution> {
        let mut solution = Solution::default();

        for digit in 0..9 {
            let mut iter = Template::within(self.patterns[digit]);
            solution.0[digit] = iter.next()?;

            if iter.next().is_some() {
                // not unique
                return None;
            }
        }

        Some(solution).filter(|s| s.is_valid())
    }
}

/// Row-column pairs of all cells in box.  Contains the input cell.
const fn box_cells(row: usize, col: usize) -> [(usize, usize); 9] {
    let great_row = row / 3;
    let great_col = col / 3;
    [
        (3 * great_row + 0, 3 * great_col + 0),
        (3 * great_row + 0, 3 * great_col + 1),
        (3 * great_row + 0, 3 * great_col + 2),
        (3 * great_row + 1, 3 * great_col + 0),
        (3 * great_row + 1, 3 * great_col + 1),
        (3 * great_row + 1, 3 * great_col + 2),
        (3 * great_row + 2, 3 * great_col + 0),
        (3 * great_row + 2, 3 * great_col + 1),
        (3 * great_row + 2, 3 * great_col + 2),
    ]
}

impl std::fmt::Display for Possibilities {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Possibilities grid, each Sudoku cell separated with `|`
        for row in 0..9 {
            for col in 0..9 {
                for digit in 0..9 {
                    if self.patterns[digit].has(row, col) {
                        write!(f, "{}", digit + 1)?;
                    } else {
                        write!(f, " ")?;
                    }
                }
                if col < 8 {
                    write!(f, "|")?;
                }
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}
