use rand::Rng;
use rand::prelude::ThreadRng;
use std::ops::Range;

type Coord = u16;

#[derive(Debug)]
#[derive(PartialEq)]
#[derive(PartialOrd)]
struct Cell(Coord, Coord);

fn new_rnd_cell(rng: &mut ThreadRng, x_range: &Range<Coord>, y_range: &Range<Coord>) -> Cell {
    return Cell(rng.gen_range(x_range.start, x_range.end),
                rng.gen_range(y_range.start, y_range.end));
}

fn main() {
    let mut rng = rand::thread_rng();
    assert_eq!(Cell(1, 3), Cell(1, 3));
    assert!(Cell(1, 3) < Cell(2, 3));
    assert!(Cell(1, 2) < Cell(1, 3));

    let board_rows = 0..16;
    let board_cols = 0..16;
    let mines: Vec<Cell> = vec![new_rnd_cell(&mut rng, &board_cols, &board_rows)];
    println!("Mines {:?}", mines);
}
