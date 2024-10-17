use std::collections::HashMap;
use std::fmt::Debug;
use std::ops;
use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Pair(pub usize, pub usize);

impl ops::Add<Pair> for Pair {
    type Output = Pair;
    fn add(self, rhs: Pair) -> Self::Output {
        Pair(self.0.wrapping_add(rhs.0), self.1.wrapping_add(rhs.1))
    }
}

impl From<(usize, usize)> for Pair {
    fn from(value: (usize, usize)) -> Self {
        Pair(value.0, value.1)
    }
}

impl From<Pair> for (usize, usize) {
    fn from(value: Pair) -> Self {
        (value.0, value.1)
    }
}

impl Pair {
    pub fn extend(self, extent: Pair) -> Rect {
        Rect {
            origin: self,
            extent,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct CellInfo {
    pub cost: usize,
    pub blocked: bool,
}

// A rect with origin (0,0) and extent (x,y) includes all points (a,b) with 0 <= a <= x and 0 <= b <= y.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct Rect {
    pub origin: Pair,
    pub extent: Pair,
}

impl ops::Add<Pair> for Rect {
    type Output = Rect;
    fn add(self, rhs: Pair) -> Self::Output {
        Rect {
            origin: self.origin + rhs,
            extent: self.extent,
        }
    }
}

impl Rect {
    pub fn size(self) -> Pair {
        self.extent + Pair(1, 1)
    }

    pub fn cells(self) -> Vec<Pair> {
        let mut out = Vec::with_capacity(self.size().0 * self.size().1);
        for dx in 0..self.size().0 {
            for dy in 0..self.size().1 {
                out.push(self.origin + Pair(dx, dy))
            }
        }
        out
    }

    pub fn max_coord(self) -> Pair {
        self.origin + self.extent
    }

    pub fn contains(self, cell: Pair) -> bool {
        let (x, dx, y, dy) = (self.origin.0, self.extent.0, self.origin.1, self.extent.1);
        x <= cell.0 && cell.0 <= x + dx && y <= cell.1 && cell.1 <= y + dy
    }

    pub fn intersects(self, rect_1: Rect) -> bool {
        self.origin.0 <= rect_1.max_coord().0
            && rect_1.origin.0 <= self.max_coord().0
            && self.origin.1 <= rect_1.max_coord().1
            && rect_1.origin.1 <= self.max_coord().1
    }
}

pub trait HashMapExt<T> {
    fn min_value(&self) -> Option<(T, usize)>;
}

impl<T: Copy> HashMapExt<T> for HashMap<T, usize> {
    fn min_value(&self) -> Option<(T, usize)> {
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

#[derive(Clone)]
pub struct ScoredCell {
    // Cost including heuristic, what time do we think we will arrive?
    pub location: Rect,
    pub duration: Pair,
    pub cost: usize,
    pub prev: Option<Rc<ScoredCell>>,
}

impl PartialEq for ScoredCell {
    fn eq(&self, other: &Self) -> bool {
        self.location == other.location && self.duration == other.duration
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
            .then_with(|| other.duration.1.cmp(&self.duration.1))
            .then_with(|| other.duration.0.cmp(&self.duration.0))
            .then_with(|| other.location.cmp(&self.location))
            .then_with(|| other.prev.cmp(&self.prev))
    }
}

impl PartialOrd for ScoredCell {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Debug for ScoredCell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "({}, {}) × ({}, {})",
            self.location.origin.0, self.location.origin.1, self.duration.0, self.duration.1
        )
    }
}

pub type Path = Vec<ScoredCell>;

pub fn unfold_path(path: Path) -> Vec<Rect> {
    if path.is_empty() {
        return Vec::new();
    }
    let mut out = Vec::with_capacity(path[path.len() - 1].duration.1 + 1);
    out.push(path[0].location);
    for scored_cell in path {
        for _ in scored_cell.duration.0..=scored_cell.duration.1 {
            out.push(scored_cell.location);
        }
    }
    out
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ConflictInfo {
    pub uid: Pair,
    pub location: Rect,
    pub duration: Pair,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Conflict(pub ConflictInfo, pub ConflictInfo);

impl Conflict {
    pub fn uids(self) -> (Pair, Pair) {
        (self.0.uid, self.1.uid)
    }
}

// Constraint means that the unit may not collide with the region
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Constraint {
    pub uid: Pair,
    pub location: Rect,
    pub duration: Pair,
}

impl Debug for Constraint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "({}, {}) : ({}, {}) × ({}, {})",
            self.uid.0,
            self.uid.1,
            self.location.origin.0,
            self.location.origin.1,
            self.duration.0,
            self.duration.1
        )
    }
}

impl Conflict {
    pub fn constraints(self) -> [Constraint; 2] {
        let constraint_0 = Constraint {
            uid: self.0.uid,
            location: self.1.location,
            duration: self.1.duration,
        };
        let constraint_1 = Constraint {
            uid: self.1.uid,
            location: self.0.location,
            duration: self.0.duration,
        };
        [constraint_0, constraint_1]
    }
}
