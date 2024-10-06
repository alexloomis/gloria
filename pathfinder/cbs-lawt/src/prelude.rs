use std::collections::HashMap;
use std::ops::Sub;
use std::rc::Rc;

pub type Cell = (usize, usize);

#[derive(Clone, Copy)]
pub struct CellInfo {
    pub cost: usize,
    pub blocked: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rect {
    pub origin: Cell,
    pub extent: Cell,
}

impl Rect {
    pub fn cells(&self) -> Vec<Cell> {
        let mut out = Vec::with_capacity(self.extent.0 * self.extent.1);
        for dx in 0..self.extent.0 {
            for dy in 0..self.extent.1 {
                out.push((self.origin.0 + dx, self.origin.1 + dy))
            }
        }
        out
    }
}

pub trait HashMapExt<T> {
    fn min_key(&self) -> Option<(T, usize)>;
}

impl<T: Copy> HashMapExt<T> for HashMap<T, usize> {
    fn min_key(&self) -> Option<(T, usize)> {
        let mut best_key = None;
        let mut best_value = usize::MAX;
        for (key, value) in self.iter() {
            if *value < best_value {
                best_key = Some(*key);
                best_value = *value;
            }
        }
        best_key.map(|key| (key, best_value))
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub struct ScoredCell {
    // Cost including heuristic, what time do we think we will arrive?
    pub cost: usize,
    // Time of earliest departure, cost without heuristic
    pub time: usize,
    pub cell: Cell,
    pub prev: Option<Rc<ScoredCell>>,
}

pub fn unfold_path(path: Vec<ScoredCell>) -> Vec<Cell> {
    if path.is_empty() {
        return Vec::new();
    }
    let mut time = path[0].time;
    let mut out = Vec::with_capacity(path[path.len() - 1].time + 1);
    out.push(path[0].cell);
    for scored_cell in path {
        let dt = scored_cell.time.sub(time);
        for _ in 0..dt {
            out.push(scored_cell.cell);
        }
        time = scored_cell.time;
    }
    out
}

// Currently means the unit's origin is blocked from the tile, not the unit's entire body
#[derive(Clone, Copy, Debug)]
pub struct Constraint {
    // Unit ID is its starting coord
    pub uid: Cell,
    pub cell: Cell,
    pub time: usize,
}

// Maybe time shoule be an interval?
#[derive(Clone, Copy, Debug)]
pub struct Conflict {
    // Unit ID is its starting coord
    pub uid1: Cell,
    pub uid2: Cell,
    pub cell1: Cell,
    pub cell2: Cell,
    pub time: usize,
}
