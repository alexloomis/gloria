use crate::prelude::*;
use core::panic;
use std::collections::HashMap;
use std::fmt::Debug;
use std::path::Path;

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
// TODO: this seems to be where the bug is coming from
pub fn patch_path_old(path: Path, mut patch: Path) -> Path {
    println!("Patching:");
    print_path(&path);
    println!("with");
    print_path(&patch);
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
    println!("resulting in");
    print_path(&new_path);
    if !path_ok(&new_path) {
        println!("Invalid path created!");
        panic!();
    }
    println!();
    new_path
}

// TODO: find out why patches are waiting (ie (1,1) (2,2) (3,3) for times)
// rather than reflecting the movement cost (ie (1,3) for times).
// Probably caused by giving A* the start of the duration rather than the end,
// probably also the cause of the bug.
// Clarifying how patch is constructed and what its invariants are
// will make it possible to rewrite this function clearly.
pub fn patch_path(path: Path, mut patch: Path) -> Path {
    // The zeroth element of the patch is the already-existing state it left from.
    patch.remove(0);
    let patch_start = patch[0].duration.0;
    // If we arrived at the same time at the same place as previously
    let patch_end = patch[patch.len() - 1].duration.1;
    path
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
