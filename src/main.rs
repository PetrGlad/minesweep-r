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

const NEIGH: [(i8, i8); 8] = [
    (-1, -1), (-1, 0), (-1, 1),
    (0, -1), /*     */ (0, 1),
    (1, -1), (1, 0), (1, 1)
];

const MARGIN: usize = 1;

/// Get index ranges that may contain mines
fn active_ranges(shape: &[usize]) -> (Range<usize>, Range<usize>) {
    (MARGIN..(shape[0] - MARGIN),
     MARGIN..(shape[1] - MARGIN))
}

impl Field {
    fn random_new(rng: &mut ThreadRng,
                  n_rows: usize,
                  n_cols: usize,
                  fill_frac: f32) -> Field {
        assert!(0.0 <= fill_frac && fill_frac <= 1.0);
        let mut cells = Array::from_elem(Ix2(n_rows, n_cols), DANGER_NONE);
        let (rows, cols) = active_ranges(cells.shape());
        for i in rows {
            for j in cols.clone() {
                let idx = [i, j];
                cells[idx] = if rng.gen::<f32>() < fill_frac { DANGER_MINE } else { DANGER_NONE };
            }
        }
        return Field {
            cells
        };
    }

    fn fill_hints(&mut self) {
        let (rows, cols) = active_ranges(self.cells.shape());
        for i in rows.clone() {
            for j in cols.clone() {
                if is_mine(self.cells[(i, j)]) {
                    for k in &NEIGH {
                        let neigh_i = i as i32 + k.0 as i32;
                        let neigh_j = j as i32 + k.1 as i32;
                        self.cells[(neigh_i as usize, neigh_j as usize)] += 1
                    }
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
        Board {
            cells: Array::from_elem(Ix2(n_rows, n_cols), CellState::Unknown)
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
    let n_rows: usize = 20;
    let n_cols: usize = 80;

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::cmp;

    #[test]
    fn test_neigh_values() {
        assert!(NEIGH
            .iter()
            .map(|(r, c)| cmp::max(r, c))
            .max()
            .unwrap() <= &(MARGIN as i8));
        assert!(NEIGH.iter()
            .map(|(r, c)| cmp::min(r, c))
            .min()
            .unwrap() >= &-(MARGIN as i8));
    }
}
