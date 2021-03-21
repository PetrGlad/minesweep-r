use ndarray::prelude::*;
use ndarray::{Array, Ix2};
use std::ops::Range;
use std::fmt;

/* TODO: ansi_escapes is unsupported, consider switching
   See https://github.com/LinusU/rust-ansi-escapes/pull/1 */
use ansi_escapes::{ClearScreen, CursorHide, CursorShow, CursorTo};
use ansi_term::Colour;
use rand::prelude::ThreadRng;
use rand::Rng;

type Coord = usize;

type Danger = u8;

const DANGER_NONE: Danger = 0;
const DANGER_MINE: Danger = 10;

fn is_mine(x: Danger) -> bool {
    x >= DANGER_MINE
}

#[derive(Debug)]
struct Field {
    cells: Array<Danger, Ix2>
}

impl Field {
    fn random_new(rng: &mut ThreadRng,
                  n_rows: usize,
                  n_cols: usize,
                  fill_frac: f32) -> Field {
        assert!(0.0 <= fill_frac && fill_frac <= 1.0);
        let mut cells = Array::from_elem(Ix2(n_rows, n_cols), DANGER_NONE);
        for i in 0..cells.shape()[0] {
            for j in 0..cells.shape()[1] {
                let idx = [i, j];
                cells[idx] = if rng.gen::<f32>() < fill_frac { DANGER_MINE } else { DANGER_NONE };
            }
        }
        return Field {
            cells
        };
    }

    fn fill_hints(&mut self) {
        for i in 0..self.cells.shape()[0] {
            for j in 0..self.cells.shape()[1] {
                let idx = [i, j];
                if is_mine(self.cells[idx]) {
                    // TODO Implement: +1 to neighbour cells
                }
            }
        }
    }
}

#[derive(Debug)]
#[derive(PartialEq, Eq, PartialOrd, Ord)]
#[derive(Clone)]
enum CellState {
    Unknown,
    // Marked as containing mine
    Marked,
    // Cleared
    Free,
}

#[derive(Debug)]
struct Board {
    cells: Array<CellState, Ix2>
}

impl Board {
    fn new(n_rows: Coord, n_cols: Coord) -> Board {
        let mut cells = Array::from_elem(Ix2(n_rows, n_cols), CellState::Unknown);
        Board {
            cells
        }
    }
}

impl fmt::Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for row in self.cells.outer_iter() {
            for cell in row {
                if is_mine(*cell) {
                    write!(f, "{} ", Colour::Red.paint("*"))?
                } else if *cell == 0u8 {
                    write!(f, "  ")?
                } else {
                    write!(f, "{:1} ", cell)?
                }
            }
            write!(f, "\n")?
        }
        fmt::Result::Ok(())
    }
}

// TODO Unify print code with Field.
impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for row in self.cells.outer_iter() {
            for cell in row {
                write!(f, "{} ",
                       match cell {
                           CellState::Free => Colour::White.paint(" "),
                           CellState::Marked => Colour::Yellow.paint("@"),
                           _ => Colour::Black.paint("#")
                       })?
            }
            write!(f, "\n")?
        }
        fmt::Result::Ok(())
    }
}

fn main() {
    let mut rng = rand::thread_rng();
    let n_rows = 15;
    let n_cols = 80;

    let mut mines: Field = Field::random_new(&mut rng, n_rows, n_cols, 0.12);
    Field::fill_hints(&mut mines);
    // TODO Make mines read-only now?

    let board: Board = Board::new(n_rows, n_cols);

    print!("{}{}{}", CursorHide, ClearScreen, CursorTo::TopLeft);

    println!("{}", &mines);
    // println!("Mines {:?}", &mines);
    println!("\n");
    println!("{}", &board);
    // println!("Board {:?}", &board);

    println!("{}", CursorShow);
}
