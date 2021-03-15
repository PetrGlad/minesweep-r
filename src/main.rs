use std::ops::Range;
use std::collections::BTreeSet;

/* TODO: ansi_escapes is unsupported, consider switching
   See https://github.com/LinusU/rust-ansi-escapes/pull/1 */
use ansi_escapes::{ClearScreen, CursorHide, CursorShow, CursorTo, CursorMove};
use ansi_term::Colour;
use rand::prelude::ThreadRng;
use rand::Rng;

type Coord = u16;

#[derive(Debug)]
#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct Cell(Coord, Coord);

impl Cell {
    fn random_new(rng: &mut ThreadRng, x_range: &Range<Coord>, y_range: &Range<Coord>) -> Cell {
        return Cell(rng.gen_range(x_range.start, x_range.end),
                    rng.gen_range(y_range.start, y_range.end));
    }
}

#[derive(Debug)]
struct Field {
    ranges: (Range<Coord>, Range<Coord>),
    cells: Vec<Cell>,
}

impl Field {
    fn random_new(rng: &mut ThreadRng,
                  rows: &Range<Coord>, cols: &Range<Coord>,
                  count: usize) -> Field {
        assert!(count < (rows.len() * cols.len()));
        /* Note the following would be slow for dense fields due to duplicates.
           Need a different algo for that case. E.g. randomly selecting
           from list of all (x,y) pairs. */
        let mut cells = BTreeSet::new();
        while cells.len() < count {
            cells.insert(Cell::random_new(rng, &cols, &rows));
        }
        return Field {
            ranges: (rows.clone(), cols.clone()),
            cells: cells.into_iter().collect(),
        };
    }
}

#[derive(Debug)]
#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum CellState {
    // Unexplored yet
    Unknown,
    // Marked as containing mine
    Marked,
    // Cleared
    Free,
}

struct Board {
    ranges: (Range<Coord>, Range<Coord>),
    cells: Vec<CellState>,
}

// TODO Impl Paint for Board; unify print code with Field, maybe.

trait Paint {
    fn print(&self);
}

impl Paint for Field {
    fn print(&self) {
        print!("{}{}{}", CursorHide, ClearScreen, CursorTo::TopLeft);
        let mut ci = self.cells.iter();
        let mut cell = ci.next().unwrap();
        for row in self.ranges.0.clone() {
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
        println!("Mines {:?}", self); // DEBUG
    }
}

fn main() {
    let mut rng = rand::thread_rng();
    assert_eq!(Cell(1, 3), Cell(1, 3));
    assert!(Cell(1, 3) < Cell(2, 3));
    assert!(Cell(1, 2) < Cell(1, 3));

    let board_rows = 0..16;
    let board_cols = 0..16;
    let mines: Field = Field::random_new(&mut rng, &board_rows, &board_cols, 100);
    Field::print(&mines);
}
