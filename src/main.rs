use std::ops::Range;

use ansi_escapes::{ClearScreen, CursorHide, CursorShow, CursorTo, CursorMove};
use ansi_term::Colour;
use rand::prelude::ThreadRng;
use rand::Rng;

type Coord = u16;

#[derive(Debug)]
#[derive(PartialOrd)]
#[derive(Ord)]
#[derive(PartialEq)]
#[derive(Eq)]
struct Cell(Coord, Coord);

#[derive(Debug)]
struct Board {
    ranges: (Range<Coord>, Range<Coord>),
    cells: Vec<Cell>,
}

fn new_rnd_cell(rng: &mut ThreadRng, x_range: &Range<Coord>, y_range: &Range<Coord>) -> Cell {
    return Cell(rng.gen_range(x_range.start, x_range.end),
                rng.gen_range(y_range.start, y_range.end));
}

fn new_rnd_board(rng: &mut ThreadRng,
                 rows: &Range<Coord>, cols: &Range<Coord>,
                 count: u16) -> Board {
    let mut cells = vec![];
    for _n in 0..count {
        cells.push(new_rnd_cell(rng, &cols, &rows));
    }
    cells.sort();
    return Board {
        ranges: (rows.clone(), cols.clone()),
        cells,
    };
}

fn print_board(board: &Board) {
    print!("{}{}{}", CursorHide, ClearScreen, CursorTo::TopLeft);
    let mut ci = board.cells.iter();
    let mut cell = ci.next().unwrap();
    for row in board.ranges.0.clone() {
        let mut col = 0;
        while cell.0 <= row {
             print!("{}{}",
                   CursorMove::X((cell.1 - col) as i16 * 2),
                   Colour::Red.paint("*"));
            let o_cell = ci.next();
            if o_cell.is_none() {
                break;
            }
            col = cell.1;
            cell = o_cell.unwrap();
        }
        println!();
    }
    println!("{}", CursorShow);
    // println!("Mines {:?}", board); // DEBUG
}

fn main() {
    let mut rng = rand::thread_rng();
    assert_eq!(Cell(1, 3), Cell(1, 3));
    assert!(Cell(1, 3) < Cell(2, 3));
    assert!(Cell(1, 2) < Cell(1, 3));

    let board_rows = 0..16;
    let board_cols = 0..16;
    let mines: Board = new_rnd_board(&mut rng, &board_rows, &board_cols, 30);
    print_board(&mines);
}
