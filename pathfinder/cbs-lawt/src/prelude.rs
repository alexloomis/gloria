use std::collections::HashMap;
use std::rc::Rc;

pub type Pair = (usize, usize);

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct CellInfo {
    pub cost: usize,
    pub blocked: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rect {
    pub origin: Pair,
    pub extent: Pair,
}

impl Rect {
    pub fn cells(&self) -> Vec<Pair> {
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
    pub stay: Pair,
    pub cell: Pair,
    pub prev: Option<Rc<ScoredCell>>,
}

pub fn unfold_path(path: Vec<ScoredCell>) -> Vec<Pair> {
    if path.is_empty() {
        return Vec::new();
    }
    let mut out = Vec::with_capacity(path[path.len() - 1].stay.1 + 1);
    out.push(path[0].cell);
    for scored_cell in path {
        for _ in scored_cell.stay.0..=scored_cell.stay.1 {
            out.push(scored_cell.cell);
        }
    }
    out
}

// Currently means the unit's origin is blocked from the tile, not the unit's entire body
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Constraint {
    // Unit ID is its starting coord
    pub uid: Pair,
    pub cell: Pair,
    pub stay: Pair,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ConflictInfo {
    pub uid: Pair,
    pub cell: Pair,
    pub stay: Pair,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Conflict(pub ConflictInfo, pub ConflictInfo);
