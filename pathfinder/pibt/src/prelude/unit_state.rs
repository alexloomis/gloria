use crate::prelude::*;
use std::collections::HashMap;
use std::fmt::Debug;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UnitState {
    pub location: Rect,
    pub duration: Pair,
}

impl Ord for UnitState {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.duration
            .cmp(&other.duration)
            .then_with(|| self.location.cmp(&other.location))
    }
}

impl PartialOrd for UnitState {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

pub type Path = Vec<UnitState>;

pub fn print_path(path: &Path) {
    for state in path {
        println!(
            "({}, {}) × ({}, {})",
            state.location.origin.0, state.location.origin.1, state.duration.0, state.duration.1
        );
    }
}

pub fn print_paths(paths: &HashMap<Pair, Path>) {
    for (uid, path) in paths {
        println!("Path for unit {uid:?}");
        print_path(path);
    }
}
