use std::collections::HashMap;
use std::fmt::Debug;
use std::ops;

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

    pub fn intersects(self, other: Pair) -> bool {
        self.0 <= other.1 && other.0 <= self.1
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UnitState {
    pub uid: Pair,
    pub location: Rect,
    pub duration: Pair,
}

impl Ord for UnitState {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.duration
            .cmp(&other.duration)
            .then_with(|| self.uid.cmp(&other.uid))
            .then_with(|| self.location.cmp(&other.location))
    }
}

impl PartialOrd for UnitState {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

pub type Path = Vec<UnitState>;

// Assumes patch's first state has a duration of 1
pub fn patch_path(path: Path, mut patch: Path) -> Path {
    //println!("Patching:");
    //print_path(&path);
    //println!("with");
    //print_path(&patch);
    let mut new_path = Vec::with_capacity(path.len() + patch.len());
    let start_time = patch[0].duration.0;
    let end_time = patch[patch.len() - 1].duration.1;
    let overlaps = |state: UnitState| state.duration.0 > start_time && state.duration.1 <= end_time;
    let overlaps_end =
        |state: UnitState| state.duration.0 <= end_time && state.duration.1 > end_time;
    for state in path {
        if overlaps_end(state) {
            for time in (end_time + 1)..=state.duration.1 {
                let wait = UnitState {
                    uid: state.uid,
                    location: state.location,
                    duration: Pair(time, time),
                };
                new_path.push(wait);
            }
        } else if !overlaps(state) {
            new_path.push(state);
        }
    }
    patch.remove(0);
    new_path.append(&mut patch);
    new_path.sort();
    //println!("resulting in");
    //print_path(&new_path);
    //println!();
    new_path
}

pub fn print_path(path: &Path) {
    for state in path {
        println!(
            "({}, {}) × ({}, {})",
            state.location.origin.0, state.location.origin.1, state.duration.0, state.duration.1
        );
    }
}

pub fn check_path_times(path: &Path) {
    let mut message = None;
    if path[0].duration.0 > path[0].duration.1 {
        message = Some("decreasing time interval found at start of path!");
    }
    let mut now = path[0].duration.1;
    for state in path.iter().skip(1) {
        if state.duration.0 > state.duration.1 {
            message = Some("decreasing time interval found!");
        } else if state.duration.0 != now + 1 {
            message = Some("time discontinuity found!")
        }
        now = state.duration.1;
    }
    if let Some(msg) = message {
        println!("{msg}");
        print_path(path);
        panic!()
    }
}
