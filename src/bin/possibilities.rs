use sudoku::Possibilities;

fn main() {
    let mut puzzle = Possibilities::new();

    puzzle.set(0, 7, 1).unwrap();

    puzzle.set(1, 5, 2).unwrap();
    puzzle.set(1, 8, 3).unwrap();

    puzzle.set(2, 3, 4).unwrap();

    puzzle.set(3, 6, 5).unwrap();

    puzzle.set(4, 0, 4).unwrap();
    puzzle.set(4, 2, 1).unwrap();
    puzzle.set(4, 3, 6).unwrap();

    puzzle.set(5, 2, 7).unwrap();
    puzzle.set(5, 3, 1).unwrap();

    puzzle.set(6, 1, 5).unwrap();
    puzzle.set(6, 6, 2).unwrap();

    puzzle.set(7, 4, 8).unwrap();
    puzzle.set(7, 7, 4).unwrap();

    puzzle.set(8, 1, 3).unwrap();
    puzzle.set(8, 3, 9).unwrap();
    puzzle.set(8, 4, 1).unwrap();

    println!("{}", puzzle);
}
