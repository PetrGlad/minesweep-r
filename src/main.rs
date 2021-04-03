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

fn offset(x: usize, diff: i8) -> usize {
    (x as i32 + diff as i32) as usize
}

const DANGER_NONE: Danger = 0;
const DANGER_MINE: Danger = 10;

fn is_mine(x: Danger) -> bool {
    x >= DANGER_MINE
}

#[derive(Debug)]
struct Field {
    cells: Array<bool, Ix2>,
    n_mines: usize,
}

const NEIGH: [(i8, i8); 8] = [
    (-1, -1), (-1, 0), (-1, 1),
    (0, -1), /*     */ (0, 1),
    (1, -1), (1, 0), (1, 1)
];

const PATCH: [(i8, i8); 9] = [
    (-1, -1), (-1, 0), (-1, 1),
    (0, -1), (0, 0), (0, 1),
    (1, -1), (1, 0), (1, 1)
];


/// Adds padding at the field sides to avoid checking edge conditions every time
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
        // let (rows, cols) = active_ranges(self.cells.shape());
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

// TODO Unify print code with Field.
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
    Estimate(f32),
    Free(u8),
    Mine,
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
    let mut edge = Vec::with_capacity(100);
    let mut probe_here = (MARGIN, MARGIN); // TEMPORARY ALGORITHM STUB
    let mut mines_remaining = mines.n_mines;

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
        if danger >= DANGER_MINE {
            println!("Failed. Probe at {:?}", probe_here);
            break;
        }
        board.cells[probe_here] = CellState::Free;
        {
            let mut neigh_mines = 0;
            for neigh_d in &NEIGH {
                let neigh_pos = (offset(probe_here.0, neigh_d.0), offset(probe_here.1, neigh_d.1));
                match scratchpad[neigh_pos] {
                    CellDesc::Mine => neigh_mines += 1,
                    _ => ()
                }
            }
            scratchpad[probe_here] = CellDesc::Free(danger - neigh_mines);

            /* Now that we cleared a place update estimates on all it's neighbours.
               Since known mines are already excluded from danger scores,
               estimate is set to danger score evenly distributed over neigbour unknowns. */
            /* In a GPU-like environment we could recalculate every estimate on the board every time.
               On a CPU one perhaps should be selective but it gets complicated.
               A compromise could be to update whole edge every time. */
            for cell_d in &PATCH {
                let cell_pos = (offset(probe_here.0, cell_d.0), offset(probe_here.1, cell_d.1));
                let mut unknowns: Vec<(usize, usize)> = Vec::with_capacity(NEIGH.len());
                for neigh_d in &NEIGH {
                    let neigh_pos = (offset(cell_pos.0, neigh_d.0), offset(cell_pos.1, neigh_d.1));
                    match scratchpad[neigh_pos] {
                        CellDesc::Unknown | CellDesc::Estimate(_) => unknowns.push(neigh_pos),
                        _ => ()
                    }
                }
                if !unknowns.is_empty() {
                    let p = danger as f32 / unknowns.len() as f32;
                    for neigh_pos in &unknowns {
                        let c = &mut scratchpad[*neigh_pos];
                        match *c {
                            CellDesc::Unknown => {
                                *c = CellDesc::Estimate(p);
                                edge.push(neigh_pos);
                            },
                            /* Note that we're updating after a new empty cell was encountered.
                               This means the estimated probability can only increase. */

                            /* TODO Simplification: Keep a list of all contributing probabilities?
                                So we can update it bot on clear and mine mark. */

                            CellDesc::Estimate(pre_p) if pre_p < p =>
                                *c = CellDesc::Estimate(p),
                            _ => ()
                        }
                    }
                }
            }
            /* First mark known mines to see which danger is still real in later probing. */
            // (Probably better start from tail of the edge instead)
            for c in edge {
                match scratchpad[c] {
                    CellDesc::Estimate(p) if p == 1f32 => {
                        board[c] = CellState::Marked;
                        scratchpad[c] = CellDesc::Mine;
                        for neigh_d in &NEIGH {
                            let neigh_pos = (offset(c.0, neigh_d.0), offset(c.1, neigh_d.1));
                            match scratchpad[neigh_pos] {
                                CellDesc::Free(n) =>
                                    /* TODO Now we need to update neighbour estimates but do
                                        not want to copy/paste the above monstrosity.
                                        Also this position should be excluded from the edge (would be O(N) :( ). */
                                    scratchpad[neig_pos] = CellDesc::Free(n - 1),
                                _ => ()
                            }
                        }
                    }
                    _ => ()
                }
            }
            /* Find something to probe next starting from safe cells.
               Pick ones on the edge with 0 probability first, if there are none 0s
               then pick one of the places with lowest estimated probability.
               */
            /* A further improvement may be to pick an unexplored position not on
               the edge if estimated probability there is lower than on the edge.
               Need to know (or have a good estimate of) total number of mines tor that. */
            // for c in edge {
            //     match scratchpad[c] {
            //         CellDesc::Estimate(p) if p == 1f32 => {
            //             probe_here = c;
            //             break;
            //         }
            //     }
            // }
            // TODO check if we have a new place to probe here / check finish conditions
        }

        // TEMPORARY ALGORITHM STUB
        if probe_here.1 == n_cols - MARGIN - 1 {
            if probe_here.0 == n_rows - MARGIN - 1 {
                println!("[STUB] DONE PROBING (should have terminated already).");
                break;
            }
            probe_here = (probe_here.0 + 1, MARGIN)
        } else {
            probe_here = (probe_here.0, probe_here.1 + 1)
        }
    }
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
