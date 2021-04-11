use std::collections::HashSet;
use std::fmt;
// use std::io::{self, BufRead};
use std::ops::Range;

use ansi_escapes::{EraseLine, ClearScreen, CursorHide, CursorShow, CursorTo};
use ansi_term::Colour;
use ndarray::{Array, Ix2};
use rand::prelude::ThreadRng;
use rand::Rng;

/* TODO: ansi_escapes is unsupported, consider switching
   See https://github.com/LinusU/rust-ansi-escapes/pull/1 */
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
    mines: Array<bool, Ix2>,
    active_ranges: (Range<usize>, Range<usize>),
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
    fn new(n_rows: usize, n_cols: usize) -> Field {
        let cells = Array::from_elem(Ix2(n_rows, n_cols), false);
        let ranges = active_ranges(&cells.shape());
        return Field {
            mines: cells,
            active_ranges: ranges,
            n_mines: 0,
        };
    }

    fn random_fill(&mut self, rng: &mut ThreadRng, fill_frac: f32) {
        assert!(0.0 <= fill_frac && fill_frac <= 1.0);
        let (rows, cols) = &self.active_ranges;
        for i in rows.to_owned() {
            for j in cols.clone() {
                if rng.gen::<f32>() < fill_frac {
                    self.mines[[i, j]] = true;
                    self.n_mines += 1;
                }
            }
        }
    }

    fn probe(&self, pos: (usize, usize)) -> u8 {
        if self.mines[pos] {
            return DANGER_MINE
        }
        let mut count = 0;
        for p in &NEIGH {
            let neigh_i = offset(pos.0, p.0);
            let neigh_j = offset(pos.1, p.1);
            if self.mines[(neigh_i, neigh_j)] {
                count += 1
            }
        }
        count
    }

    fn is_active(&self, pos: &Pos) -> bool {
        self.active_ranges.0.contains(&pos.0)
            && self.active_ranges.1.contains(&pos.1)
    }
}

impl fmt::Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for row in self.mines.outer_iter() {
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

impl fmt::Display for CellState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}",
               match self {
                   CellState::Free => Colour::White.paint(" "),
                   CellState::Marked => Colour::Yellow.paint("@"),
                   _ => Colour::Black.paint("#")
               })
    }
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

fn display_grid<C: fmt::Display>(cells: &Array<C, Ix2>, f: &mut fmt::Formatter) -> fmt::Result {
    for row in cells.outer_iter() {
        for cell in row {
            write!(f, "{} ", cell)?
        }
        write!(f, "\n")?
    }
    fmt::Result::Ok(())
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        display_grid(&self.cells, f)
    }
}

#[derive(Debug)]
#[derive(Clone)]
enum CellDesc {
    Unknown,
    // TODO (improvement) Actually we can have only 44 distinct ratios, no need in full float functionality.
    Estimate([f32; NEIGH.len()]),
    ShouldFree,
    // Estimated to be free (should become Free(0) after probe)
    Free(u8),
    Mine,
}

fn max(xs: &[f32; NEIGH.len()]) -> f32 {
    /* No Ord for f32 so default max won't work.
       Our p values are never Inf or NaN and array is not empty. */
    let mut result = xs[0];
    for x in xs {
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
            CellDesc::Estimate(ps) => max(ps),
            CellDesc::ShouldFree | CellDesc::Free(_) => 0f32,
            // Unknown can actually be estimated given total unknown count and remaining mines count.
            CellDesc::Unknown => panic!("Unreachable"),
        }
    }
}

impl fmt::Display for CellDesc {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}",
               match self {
                   CellDesc::Mine => Colour::Green.paint("@"),
                   CellDesc::Estimate(xs) => {
                       let m = max(xs);
                       if m == 1f32 {
                           Colour::Yellow.paint("%")
                       } else {
                           let c = (255 as f32 * m) as u8;
                           Colour::RGB(c, c, c).paint("%")
                       }
                   },
                   CellDesc::ShouldFree => Colour::RGB(50, 50, 50).paint("0"),
                   CellDesc::Free(0) => Colour::Black.paint(" "),
                   CellDesc::Free(n) => Colour::Cyan.paint(format!("{}", n)),
                   _ => Colour::Black.paint("#")
               })
    }
}

#[derive(Debug)]
struct ScratchPad {
    cells: Array<CellDesc, Ix2>
}

impl ScratchPad {
    fn new(n_rows: usize, n_cols: usize) -> Self {
        ScratchPad {
            cells: Array::from_elem(Ix2(n_rows, n_cols), CellDesc::Unknown)
        }
    }
}

#[derive(Debug)]
enum Action {
    Mark(Pos),
    Probe(Pos),
}

type Actions = Vec<Action>;

impl fmt::Display for ScratchPad {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        display_grid(&self.cells, f)
    }
}

fn main() {
    let n_rows: usize = 50;
    let n_cols: usize = 100;
    assert!(n_rows >= MARGIN);
    assert!(n_cols >= MARGIN);

    let mines: Field = {
        let mut m = Field::new(n_rows, n_cols);
        let mut rng = rand::thread_rng();
        m.random_fill(&mut rng, 0.1);
        // m.mines[(3, 3)] = true;
        // m.n_mines = 1;
        m
    };
    let mut board: Board = Board::new(n_rows, n_cols);
    let mut uncleared = mines.active_ranges.0.len() * mines.active_ranges.1.len();
    let mut step = 0;

    let mut scratchpad = ScratchPad::new(n_rows, n_cols);
    let mut edge: HashSet<Pos> = HashSet::with_capacity(200);
    let mut actions = Actions::with_capacity(100); // TODO (refactoring) Have a batch probe API with the algorithm.
    actions.push(Action::Probe((MARGIN, MARGIN)));

    // let stdin = io::stdin();
    // let mut user_input = stdin.lock().lines();

    print!("{}", ClearScreen);
    'game: loop {
        step += 1;
        print!("{}{}", CursorHide, CursorTo::TopLeft);

        println!("Step {}, uncleared {}", step, uncleared);
        // println!("{}", &mines);
        // println!("Mines {:?}", &mines); // DEBUG
        // println!("\n-------");
        // println!("{}", &board);
        // println!("Board {:?}", &board); // DEBUG

        // println!("Edge {:?}", &edge); // DEBUG
        // stdout().write(format!("{}"));
        println!("Scratch\n{}", &scratchpad); // DEBUG
        // println!("{}Actions {:?}", EraseLine, &actions); // DEBUG

        for action in &actions {
            match action {
                Action::Mark(pos) => {
                    assert_eq!(board.cells[*pos], CellState::Unknown);
                    board.cells[*pos] = CellState::Marked
                },
                Action::Probe(pos) => {
                    assert_eq!(board.cells[*pos], CellState::Unknown);
                    if mines.mines[*pos] {
                        println!("Failed, uncleared {}. Probe at {:?}", uncleared, pos);
                        break 'game;
                    }
                    board.cells[*pos] = CellState::Free;
                }
            }
            uncleared -= 1;
        }

        if uncleared == 0 {
            println!("{}", &board);
            println!("Complete.");
            break;
        }
        println!("{}", CursorShow);
        {
            for action in &actions {
                match action {
                    Action::Mark(pos) => {
                        scratchpad.cells[*pos] = CellDesc::Mine;
                        edge.remove(pos);
                    },
                    Action::Probe(pos) => {
                        let danger = mines.probe(*pos);
                        assert!(!is_mine(danger));
                        scratchpad.cells[*pos] = CellDesc::Free(danger);
                        edge.remove(pos);
                    }
                }
            }

            /* In a GPU-like environment we could recalculate every estimate on the board every time.
               On a CPU one perhaps should be selective but it gets complicated.
               A compromise could be to update whole edge every time. */
            for action in actions.drain(..) {
                let action_pos = match action {
                    Action::Mark(pos) | Action::Probe(pos) => pos
                };
                for neigh_d in &PATCH {
                    let cell_pos = (offset(action_pos.0, neigh_d.0), offset(action_pos.1, neigh_d.1));
                    match scratchpad.cells[cell_pos] {
                        CellDesc::Free(danger) =>
                            update_estimates(&mines, &mut scratchpad, &cell_pos, danger, &mut edge),
                        _ => ()
                    }
                }
            }

            /* TODO Need some deque+priority queue (or maybe 2 priority queues with opposite ordering).
               Consider https://lib.rs/crates/priority-queue
               Doing O(N) scan for now. */
            let mut risky_pick = None;
            for pos in &edge {
                let cell_desc = &scratchpad.cells[*pos];
                assert!(match cell_desc {
                    CellDesc::ShouldFree | CellDesc::Estimate(_) => true,
                    _ => false // Can be lifted for Unknonws with a better implementation (use "mines remaining / uncleared").
                }, "Only estimates should be on the edge.");

                let danger = if mines.is_active(pos) { cell_desc.danger() } else { 0. };
                if danger == 1f32 {
                    actions.push(Action::Mark(*pos));
                } else if danger == 0f32 {
                    actions.push(Action::Probe(*pos));
                } else {
                    match risky_pick {
                        Some((_, pick_danger)) =>
                            if danger < pick_danger {
                                risky_pick = Some((pos, danger))
                            },
                        None =>
                            risky_pick = Some((pos, danger))
                    }
                }
            }
            if actions.is_empty() {
                match risky_pick {
                    Some((pos, _)) => {
                        assert!(mines.is_active(&pos));
                        actions.push(Action::Probe(*pos));
                    },
                    None => {
                        println!("Scratch\n{}", &scratchpad); // DEBUG
                        println!("Edge {:?}", &edge); // DEBUG
                        if uncleared == 0 {
                            println!("{}", &board);
                            println!("Complete.");
                            break 'game;
                        }
                        panic!("No position selected.")
                    }
                }
            }
        }
        // user_input.next().unwrap().unwrap(); // DEBUG
    }
    println!("{}", CursorShow);
}

fn update_estimates(
    field: &Field,
    scratchpad: &mut ScratchPad,
    at: &Pos,
    danger: u8,
    edge: &mut HashSet<Pos>,
) {
    if !field.is_active(at) {
        return
    }
    let mut n_mines = 0;
    let mut n_unknowns = 0;
    for neigh_d in &NEIGH {
        // TODO (refactoring) Extract this pattern.
        let neigh_pos = (offset(at.0, neigh_d.0), offset(at.1, neigh_d.1));
        match scratchpad.cells[neigh_pos] {
            CellDesc::Unknown | CellDesc::Estimate(_) => n_unknowns += 1,
            CellDesc::Mine => n_mines += 1,
            CellDesc::Free(_) | CellDesc::ShouldFree => ()
        }
    }
    /* Since known mines are excluded from danger score,
       estimate is set to danger evenly distributed over neighbour unknowns. */
    let p = if danger == 0u8 || n_unknowns == 0 { 0f32 } else {
        assert!(danger >= n_mines);
        (danger - n_mines) as f32 / n_unknowns as f32
    };
    // println!("at {:?} p={}", &at, &p); // DEBUG
    for (i, neigh_d) in NEIGH.iter().enumerate() {
        let neigh_pos = (offset(at.0, neigh_d.0), offset(at.1, neigh_d.1));
        if !field.is_active(&neigh_pos) {
            continue // XXX Added margin to avoid checks, and still needing them?
        }
        let c = &mut scratchpad.cells[neigh_pos];
        if p == 0f32 {
            match *c {
                CellDesc::Unknown | CellDesc::Estimate(_) => {
                    *c = CellDesc::ShouldFree;
                    if field.is_active(&neigh_pos) {
                        edge.insert(neigh_pos);
                    }
                },
                _ => ()
            }
        } else {
            match c { // Setting default
                CellDesc::Unknown => {
                    *c = CellDesc::Estimate([0f32; NEIGH.len()])
                },
                _ => ()
            }
            match c {
                CellDesc::Estimate(ps) => {
                    ps[i] = p;
                    if field.is_active(&neigh_pos) {
                        edge.insert(neigh_pos);
                    }
                },
                _ => ()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cmp;

    use super::*;

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
