use std::ops::Range;

use ansi_escapes::{ClearScreen, CursorHide, CursorShow, CursorTo};
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
    println!("Mines {:?}", board);
    print!("{}{}{}", CursorHide, ClearScreen, CursorTo::TopLeft);

    let mut ci = board.cells.iter();
    let mut cell = ci.next().unwrap();
    'rows: for row in board.ranges.0.clone() {
        for col in board.ranges.1.clone() {
            if cell <= &Cell(row, col) {
                print!("{} ", Colour::Red.paint("*"));
                // println!("\n>>> {:?} <> {:?}", cell, &Cell(row, col));
                let o_cell = ci.next();
                if o_cell.is_some() {
                    cell = o_cell.unwrap();
                } else {
                    break 'rows;
                }
            } else {
                print!("  ");
            }
        }
        println!();
    }
    println!("{}{}",
             CursorTo::AbsoluteXY(board.ranges.0.len() as u16, 0),
             CursorShow);
}

fn main() {
    let mut rng = rand::thread_rng();
    assert_eq!(Cell(1, 3), Cell(1, 3));
    assert!(Cell(1, 3) < Cell(2, 3));
    assert!(Cell(1, 2) < Cell(1, 3));

    let board_rows = 0..16;
    let board_cols = 0..16;
    let mines: Board = new_rnd_board(&mut rng, &board_rows, &board_cols, 12);
    print_board(&mines);
}
