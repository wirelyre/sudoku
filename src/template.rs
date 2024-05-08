use std::sync::OnceLock;

use crate::pattern::Pattern;

/// A [`Pattern`] representing a legal layout for a single digit,
/// but stored in only two bytes instead of 12.
///
/// There are only 46656 unique layouts.  These are computed and cached.
///
/// This is convenient for brute-force search, and also reduces memory churn.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Template(u16);

/// Full solution to a Sudoku puzzle.  Just a [`Template`] for each digit.
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Solution(pub [Template; 9]);

fn leak_vec_as_array<T, const N: usize>(vec: Vec<T>) -> &'static [T; N] {
    // It's hard to allocate on the heap.
    // We'll use `std` to do it for us.
    assert!(vec.len() == N);
    let ptr = Box::into_raw(vec.into_boxed_slice());
    unsafe {
        // https://doc.rust-lang.org/reference/type-layout.html:
        // - "Slices have the same layout as the section of the array they slice."
        // - "Mutability of the pointer or reference does not change the layout."
        let ptr = ptr as *const [T; N];
        // https://doc.rust-lang.org/stable/std/primitive.pointer.html
        // - The pointer is valid and points to an initialized object.
        // - The only other reference to the object is via `ptr`, which is discarded.
        // - Ownership is leaked, so every synthesized lifetime is valid.
        ptr.as_ref().unwrap()
    }
}

impl Template {
    /// Cached list of all patterns.
    pub fn all() -> &'static [Pattern; 46656] {
        static ALL: OnceLock<&'static [Pattern; 46656]> = OnceLock::new();

        ALL.get_or_init(|| {
            // Go row by row, choosing a free column in a free box.
            fn fill(build: Pattern, cols: u16, boxes: u16, row: usize, into: &mut Vec<Pattern>) {
                if row == 9 {
                    into.push(build);
                    return;
                }
                for col in 0..9 {
                    let box_idx = row / 3 * 3 + col / 3;
                    if (1 << col) & cols == 0 && (1 << box_idx) & boxes == 0 {
                        fill(
                            build.with(row, col),
                            cols | (1 << col),
                            boxes | (1 << box_idx),
                            row + 1,
                            into,
                        );
                    }
                }
            }

            let mut vec = Vec::new();
            fill(Pattern::EMPTY, 0, 0, 0, &mut vec);
            leak_vec_as_array(vec)
        })
    }

    pub fn as_pattern(self) -> Pattern {
        Template::all()[self.0 as usize]
    }

    /// Templates that are subsets of `possible`.
    pub fn within(possible: Pattern) -> impl Iterator<Item = Template> {
        Template::all()
            .iter()
            .copied()
            .enumerate()
            .filter(move |(_i, pattern)| pattern.is_subset(possible))
            .map(|(i, _pattern)| Template(i as u16))
    }
}

impl Solution {
    /// Are the digit patterns nonoverlapping?
    pub fn is_valid(&self) -> bool {
        let mut filled = Pattern::EMPTY;
        for template in self.0 {
            if template.as_pattern().intersects(filled) {
                return false;
            }
            filled = filled | template.as_pattern();
        }
        true
    }

    fn cell(&self, row: usize, col: usize) -> u8 {
        for digit in 0..9 {
            if self.0[digit].as_pattern().has(row, col) {
                return digit as u8 + 1;
            }
        }
        panic!("empty cell");
    }

    pub fn to_grid(&self) -> Vec<u8> {
        (0..81)
            .into_iter()
            .map(|i| self.cell(i / 9, i % 9))
            .collect()
    }
}

impl std::fmt::Display for Solution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        assert!(self.is_valid());

        for row in 0..9 {
            for col in 0..9 {
                write!(f, "{}", self.cell(row, col))?;
            }
        }
        Ok(())
    }
}
