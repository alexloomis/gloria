use crate::prelude::*;
use core::panic;
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

fn path_ok(path: &Path) -> bool {
    if path.is_empty() {
        return false;
    }
    let mut last_departure = 0;
    for state in path.iter().skip(1) {
        if state.duration.0 != last_departure + 1 {
            return false;
        }
        if state.duration.0 > state.duration.1 {
            return false;
        }
        last_departure = state.duration.1
    }
    true
}

// Assumes patch's first state has a duration of 1
pub fn patch_path(path: Path, mut patch: Path) -> Path {
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
    if !path_ok(&new_path) {
        println!("Invalid path created!");
        panic!();
    }
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

pub fn print_paths(paths: &HashMap<Pair, Path>) {
    for (uid, path) in paths {
        println!("Path for unit {uid:?}");
        print_path(path);
    }
}
