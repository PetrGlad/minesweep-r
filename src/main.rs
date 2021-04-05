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
type Pos = (Coord, Coord);

type Danger = u8;

fn offset(x: usize, diff: i8) -> usize {
    (x as i32 + diff as i32) as usize
}

const DANGER_MINE: Danger = 10;

fn is_mine(x: Danger) -> bool {
    x >= DANGER_MINE
}

#[derive(Debug)]
struct Field {
    cells: Array<bool, Ix2>,
    n_mines: usize,
}

/// Neighbouring cells' offsets.
const NEIGH: [(i8, i8); 8] = [
    (-1, -1), (-1, 0), (-1, 1),
    (0, -1), /*     */ (0, 1),
    (1, -1), (1, 0), (1, 1)
];

/// NEIGH with center cell included.
const PATCH: [(i8, i8); 9] = [
    (-1, -1), (-1, 0), (-1, 1),
    (0, -1), (0, 0), (0, 1),
    (1, -1), (1, 0), (1, 1)
];


/// Adds padding at the field sides to avoid checking edge conditions every time.
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
        let mut cells = Array::from_elem(Ix2(n_rows, n_cols), false);
        let mut n_mines = 0;
        let (rows, cols) = active_ranges(cells.shape());
        for i in rows {
            for j in cols.clone() {
                if rng.gen::<f32>() < fill_frac {
                    cells[[i, j]] = true;
                    n_mines += 1;
                }
            }
        }
        return Field {
            cells,
            n_mines,
        };
    }

    fn probe(&self, pos: (usize, usize)) -> u8 {
        let mut count = 0;
        for p in &NEIGH {
            let neigh_i = offset(pos.0, p.0);
            let neigh_j = offset(pos.1, p.1);
            if self.cells[(neigh_i, neigh_j)] {
                count += 1
            }
        }
        count
    }
}

impl fmt::Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for row in self.cells.outer_iter() {
            for cell in row {
                if *cell {
                    write!(f, "{} ", Colour::Red.paint("*"))?
                } else {
                    write!(f, "  ")?
                }
            }
            write!(f, "\n")?
        }
        fmt::Result::Ok(())
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

// TODO (refactoring) Unify print code with Field.
impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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

#[derive(Debug)]
#[derive(Clone)]
enum CellDesc {
    Unknown,
    // TODO Actually we can have only 44 distinct ratios, no need in full float functionality.
    Estimate([f32; NEIGH.len()]),
    Free(u8),
    Mine,
}

fn max(xs: [f32; NEIGH.len()]) -> f32 {
    /* No Ord for f32 so default max won't work.
       Our p values are never Inf or NaN and array is not empty. */
    let mut result = xs[0];
    for x in &xs {
        if *x > result {
            result = *x;
        }
    }
    result
}

impl CellDesc {
    fn danger(self: &CellDesc) -> f32 {
        match self {
            CellDesc::Mine => 1f32,
            CellDesc::Estimate(ps) => max(*ps),
            CellDesc::Free(_) => 0f32,
            // Unknown can actually be estimated given total unknown count and remaining mines count.
            CellDesc::Unknown => panic!("Unreachable"),
        }
    }
}

fn main() {
    let mut rng = rand::thread_rng();
    let n_rows: usize = 20;
    let n_cols: usize = 80;
    assert!(n_rows >= MARGIN);
    assert!(n_cols >= MARGIN);

    let mines: Field = Field::random_new(&mut rng, n_rows, n_cols, 0.12);
    let mut board: Board = Board::new(n_rows, n_cols);

    let mut scratchpad: Array<CellDesc, Ix2> = Array::from_elem(Ix2(n_rows, n_cols), CellDesc::Unknown);
    let mut probe_here = (MARGIN, MARGIN); // TEMPORARY ALGORITHM STUB
    let mut mines_remaining = mines.n_mines;
    let mut edge: Vec<Pos> = Vec::with_capacity(200);

    loop {
        print!("{}{}{}", CursorHide, ClearScreen, CursorTo::TopLeft);

        println!("{}", &mines);
        // println!("Mines {:?}", &mines);
        println!("\n-------");
        println!("{}", &board);
        // println!("Board {:?}", &board);

        println!("{}", CursorShow);

        if mines_remaining == 0 {
            println!("Complete.");
            break;
        }

        let danger = mines.probe(probe_here);
        if is_mine(danger) {
            println!("Failed. Probe at {:?}", probe_here);
            break;
        }
        board.cells[probe_here] = CellState::Free;
        {
            scratchpad[probe_here] = CellDesc::Free(danger);

            /* In a GPU-like environment we could recalculate every estimate on the board every time.
               On a CPU one perhaps should be selective but it gets complicated.
               A compromise could be to update whole edge every time. */
            for neigh_d in &PATCH {
                let cell_pos = (offset(probe_here.0, neigh_d.0), offset(probe_here.1, neigh_d.1));
                match scratchpad[cell_pos] {
                    CellDesc::Free(danger) =>
                        edge.append(&mut update_estimates(&mut scratchpad, &cell_pos, danger)),
                    _ => ()
                }
            }

            /* TODO Need some deque+priority queue (or maybe 2 priority queues with opposite ordering).
               Consider https://lib.rs/crates/priority-queue
               Doing O(N) scan for now. */
            for pos in &edge {
                let cell_desc = &scratchpad[*pos];
                assert!(match cell_desc {
                    CellDesc::Estimate(_) => true,
                    _ => false // Can be lifted for Unknonws with a better implementation.
                }, "Only estimates should be on the edge.");
                let danger = cell_desc.danger();
                todo!();
                /* TODO Mark known mines (cell.danger() == 1f32 on the edge),
                   exclude the picks from edge, decrement mines_remaining */
                /* TODO Pick next probe position (prefer lowest cell.danger() on the edge),
                   exclude the pick from edge */
            }
        }

        // TEMPORARY ALGORITHM STUB
        if probe_here.1 == n_cols - MARGIN - 1 {
            if probe_here.0 == n_rows - MARGIN - 1 {
                panic!("[STUB] DONE PROBING (the scan should have been terminated already).");
            }
            probe_here = (probe_here.0 + 1, MARGIN)
        } else {
            probe_here = (probe_here.0, probe_here.1 + 1)
        }
    }
}

fn update_estimates(scratchpad: &mut Array<CellDesc, Ix2>, at: &Pos, danger: u8) -> Vec<Pos> {
    let mut n_mines = 0;
    let mut n_unknowns = 0;
    for neigh_d in &NEIGH {
        // TODO (refactoring) Extract this pattern.
        let neigh_pos = (offset(at.0, neigh_d.0), offset(at.1, neigh_d.1));
        match scratchpad[neigh_pos] {
            CellDesc::Unknown | CellDesc::Estimate(_) => n_unknowns += 1,
            CellDesc::Mine => n_mines += 1,
            CellDesc::Free(_) => ()
        }
    }
    if n_unknowns == 0 {
        return vec![];
    }
    let mut updated = Vec::with_capacity(NEIGH.len());
    /* Since known mines are excluded from danger score,
       estimate is set to danger evenly distributed over neigbour unknowns. */
    let p = (danger - n_mines) as f32 / n_unknowns as f32;
    for (i, neigh_d) in NEIGH.iter().enumerate() {
        let neigh_pos = (offset(at.0, neigh_d.0), offset(at.1, neigh_d.1));
        let c = &mut scratchpad[neigh_pos];
        match c {
            CellDesc::Unknown => {
                *c = CellDesc::Estimate([0f32; NEIGH.len()])
            },
            _ => ()
        }
        match c {
            CellDesc::Estimate(mut ps) => {
                ps[i] = p;
                updated.push(neigh_pos);
            },
            _ => ()
        }
    }
    return updated;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cmp;

    fn fit_margin(neigh: &[(i8, i8); 8]) -> usize {
        // Margin should be wide enough to fit all neighbours of a cell.
        neigh
            .iter()
            .map(|(r, c)| cmp::max(cmp::max(r, c), &cmp::min(*r, *c).abs()).to_owned())
            .max()
            .unwrap() as usize
    }

    #[test]
    fn test_neigh_values() {
        assert!(fit_margin(&NEIGH) <= MARGIN);
    }
}
