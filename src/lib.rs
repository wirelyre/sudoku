//! Sudoku solver.
//!
//! See [`Pattern`], [`Possibilities`], and [`Template`].

use wasm_bindgen::prelude::wasm_bindgen;

mod pattern;
mod setup;
mod template;

pub use pattern::Pattern;
pub use setup::{ImpossiblePuzzle, Possibilities};
pub use template::{Solution, Template};

/// Prepare a puzzle from user input.
pub fn prepare(input: &[[u8; 9]; 9]) -> Result<Possibilities, ImpossiblePuzzle> {
    let mut puzzle = Possibilities::new();

    for row in 0..9 {
        for col in 0..9 {
            if input[row][col] > 0 {
                puzzle.set(row as u8, col as u8, input[row][col])?;
            }
        }
    }

    Ok(puzzle)
}

/// Solve a puzzle, stopping after a maximum number of solutions.
#[wasm_bindgen]
pub fn solve(puzzle: Vec<u8>, max_solutions: usize) -> Vec<String> {
    // Two-phase solving.
    //   1.  Typical logic; see [`Possibilities`].
    //   2.  Exhaustive search by digit; see [`Template`].
    // This seems to be a perfect balance between logic and brute force.
    // The logic pares down the search space very effectively.

    let mut possibilities = Possibilities::new();
    for cell in 0..81 {
        if puzzle[cell as usize] == 0 {
            continue;
        }

        if possibilities
            .set(cell / 9, cell % 9, puzzle[cell as usize])
            .is_err()
        {
            return Vec::new(); // no solutions
        }
    }

    // Search digits from most- to least-restricted.
    //   - If the puzzle has a unique solution then this order doesn't do much.
    //   - If there are only a few clues, this makes it way faster.  :-)
    //   - Downside: adding clues makes solution ordering unstable.  :-(

    let mut templates: [(usize, Vec<Template>); 9] = Default::default();
    for digit in 0..9 {
        templates[digit] = (
            digit,
            Template::within(possibilities.patterns[digit]).collect(),
        );
    }
    templates.sort_by_key(|(_digit, possible)| possible.len());

    let mut solutions = Vec::new();
    let mut solution = Solution::default();

    fn search(
        out: &mut Vec<Solution>,
        solution: &mut Solution,
        filled: Pattern,
        templates: &[(usize, Vec<Template>)],
        max_solutions: usize,
    ) {
        match templates.split_first() {
            None => out.push(solution.clone()),

            Some(((digit, possible), rest)) => {
                for &template in possible {
                    if template.as_pattern().intersects(filled) {
                        continue;
                    }

                    solution.0[*digit] = template;

                    let filled = filled | template.as_pattern();
                    search(out, solution, filled, rest, max_solutions);

                    if out.len() >= max_solutions {
                        return;
                    }
                }
            }
        }
    }

    // web_sys::console::time_with_label("solution search");
    search(
        &mut solutions,
        &mut solution,
        Pattern::EMPTY,
        &templates,
        max_solutions,
    );
    // web_sys::console::time_end_with_label("solution search");

    solutions.into_iter().map(|s| format!("{}", s)).collect()
}
