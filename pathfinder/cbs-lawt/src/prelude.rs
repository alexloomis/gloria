use std::collections::HashMap;
use std::ops::Sub;
use std::rc::Rc;

pub type Pair = (usize, usize);

pub const UNIT_SIZE: Pair = (2, 2);

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct CellInfo {
    pub cost: usize,
    pub blocked: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
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

    pub fn contains(&self, cell: Pair) -> bool {
        //(self.origin.0..self.origin.0 + self.extent.0).contains(&cell.0)
        //    && (self.origin.1..self.origin.1 + self.extent.1).contains(&cell.1)
        let (x, dx, y, dy) = (self.origin.0, self.extent.0, self.origin.1, self.extent.1);
        x <= cell.0 && cell.0 < x + dx && y <= cell.1 && cell.1 < y + dy
    }
}

pub fn rect_conflict(cell1: Pair, cell2: Pair) -> bool {
    let ((x1, y1), (x2, y2), (dx, dy)) = (cell1, cell2, UNIT_SIZE);
    x1 < x2 + dx && x2 < x1 + dx && y1 < y2 + dy && y2 < y1 + dy
}

// A unit in this rect collides with a unit at cell
fn collisions(cell: Pair) -> Rect {
    let origin = (
        (cell.0 + 1).saturating_sub(UNIT_SIZE.0),
        (cell.1 + 1).saturating_sub(UNIT_SIZE.1),
    );
    let extent = ((2 * UNIT_SIZE.0).sub(1), (2 * UNIT_SIZE.1).sub(1));
    Rect { origin, extent }
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

#[derive(Clone, Debug)]
pub struct ScoredCell {
    // Cost including heuristic, what time do we think we will arrive?
    pub cost: usize,
    pub stay: Pair,
    pub cell: Pair,
    pub prev: Option<Rc<ScoredCell>>,
}

impl PartialEq for ScoredCell {
    fn eq(&self, other: &Self) -> bool {
        self.cell == other.cell && self.stay == other.stay
    }
}

impl Eq for ScoredCell {}

// Lowest cost has highest priority, then earliest departure, then earliest arrival, then we don't
// really care, so we just do by cell then by prev.
impl Ord for ScoredCell {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other
            .cost
            .cmp(&self.cost)
            .then_with(|| other.stay.1.cmp(&self.stay.1))
            .then_with(|| other.stay.0.cmp(&self.stay.0))
            .then_with(|| other.cell.cmp(&self.cell))
            .then_with(|| other.prev.cmp(&self.prev))
    }
}

impl PartialOrd for ScoredCell {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

pub type Path = Vec<ScoredCell>;

pub fn unfold_path(path: Path) -> Vec<Pair> {
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ConflictInfo {
    pub uid: Pair,
    pub cell: Pair,
    pub stay: Pair,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Conflict(pub ConflictInfo, pub ConflictInfo);

impl Conflict {
    pub fn uids(&self) -> (Pair, Pair) {
        (self.0.uid, self.1.uid)
    }
}

// Constraint means that the unit's origin is blocked from the tiles, not the unit's entire body
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Constraint {
    // Unit ID is its starting coord
    pub uid: Pair,
    pub rect: Rect,
    pub stay: Pair,
}

pub fn generate_constraints(conflict: Conflict) -> (Constraint, Constraint) {
    let constraint_0 = Constraint {
        uid: conflict.0.uid,
        rect: collisions(conflict.1.cell),
        stay: conflict.1.stay,
    };
    let constraint_1 = Constraint {
        uid: conflict.1.uid,
        rect: collisions(conflict.0.cell),
        stay: conflict.0.stay,
    };
    (constraint_0, constraint_1)
}
