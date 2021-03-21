use std::ops::Range;

/* TODO: ansi_escapes is unsupported, consider switching
   See https://github.com/LinusU/rust-ansi-escapes/pull/1 */
use ansi_escapes::{ClearScreen, CursorHide, CursorShow, CursorTo};
use ansi_term::Colour;
use rand::prelude::ThreadRng;
use rand::Rng;

type Coord = u8;

#[derive(Debug)]
#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct Cell(Coord, Coord);

type Danger = u8;

const DANGER_NONE: Danger = 0;
const DANGER_MINE: Danger = 10;

fn is_mine(x: Danger) -> bool {
    x >= DANGER_MINE
}

#[derive(Debug)]
struct Field {
    cells: Vec<Vec<Danger>>
}

impl Field {
    fn random_new(rng: &mut ThreadRng,
                  rows: &Range<Coord>,
                  cols: &Range<Coord>,
                  fill_frac: f32) -> Field {
        assert!(0.0 <= fill_frac && fill_frac <= 1.0);
        let mut cells = Vec::with_capacity(rows.len());
        for _r in rows.start..rows.end {
            let mut row = Vec::with_capacity(cols.len());
            for _c in cols.start..cols.end {
                row.push(if rng.gen::<f32>() < fill_frac { DANGER_MINE } else { DANGER_NONE });
            }
            cells.push(row);
        }
        return Field {
            cells
        };
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
    cells: Vec<Vec<CellState>>
}

impl Board {
    fn new(rows: &Range<Coord>, cols: &Range<Coord>) -> Board {
        let mut cells = Vec::with_capacity(rows.len());
        for _r in rows.start..rows.end {
            cells.push(vec![CellState::Unknown; cols.len()]);
        }
        Board {
            cells
        }
    }
}

trait Paint {
    fn print(&self);
}

impl Paint for Field {
    fn print(&self) {
        for row in &self.cells {
            for cell in row {
                if is_mine(*cell) {
                    print!("{} ", Colour::Red.paint("*"))
                } else if *cell == 0u8 {
                    print!("  ");
                } else {
                    print!("{:1} ", cell);
                }
            }
            println!();
        }
    }
}

// TODO Unify print code with Field.
impl Paint for Board {
    fn print(&self) {
        for row in &self.cells {
            for cell in row {
                print!("{} ",
                       match cell {
                           CellState::Free => Colour::White.paint(" "),
                           CellState::Marked => Colour::Yellow.paint("@"),
                           _ => Colour::Black.paint("#")
                       });
            }
            println!();
        }
    }
}

fn main() {
    let mut rng = rand::thread_rng();
    assert_eq!(Cell(1, 3), Cell(1, 3));
    assert!(Cell(1, 3) < Cell(2, 3));
    assert!(Cell(1, 2) < Cell(1, 3));

    let board_rows = 0..15;
    let board_cols = 0..15;

    let mines: Field = Field::random_new(&mut rng, &board_rows, &board_cols, 0.12);
    let board: Board = Board::new(&board_rows, &board_cols);

    print!("{}{}{}", CursorHide, ClearScreen, CursorTo::TopLeft);

    Field::print(&mines);
    println!("Mines {:?}", &mines);
    println!("\n");
    Board::print(&board);
    println!("Board {:?}", &board);

    println!("{}", CursorShow);
}
