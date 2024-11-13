use crate::astar::Constraint;
use crate::prelude::*;
use std::cmp::max;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Conflict(pub UnitState, pub UnitState);

impl Conflict {
    pub fn uids(self) -> [Pair; 2] {
        [self.0.uid, self.1.uid]
    }
}

impl Conflict {
    pub fn constraints(self) -> [Constraint; 2] {
        [Constraint::Avoid(self.0), Constraint::Occupy(self.0)]
    }
}

pub fn find_conflicts(paths: &HashMap<Pair, Path>) -> Vec<Conflict> {
    let mut out = Vec::new();
    let mut state = Vec::with_capacity(paths.len());
    let mut end_time = 0;
    for path in paths.values() {
        state.push((0, path[0]));
        let this_end = path[path.len() - 1].duration.1;
        end_time = max(end_time, this_end);
    }
    for time in 1..=end_time {
        let mut moved = vec![false; state.len()];
        for (i, (_, path)) in paths.iter().enumerate() {
            let idx = state[i].0;
            if time > path[idx].duration.1 && idx < path.len() - 1 {
                state[i].0 += 1;
                state[i].1 = path[idx + 1];
                moved[i] = true;
            }
        }
        // Check for conflicts
        for (i, (_, state_i)) in state.iter().enumerate() {
            for (j, (_, state_j)) in state.iter().enumerate().skip(i + 1) {
                let intersects = state_i.location.intersects(state_j.location);
                let includes_moved = moved[i] || moved[j];
                if intersects && includes_moved {
                    out.push(Conflict(*state_i, *state_j));
                }
            }
        }
    }
    out
}

//pub fn find_conflicts(paths: &HashMap<Pair, Path>) -> Vec<Conflict> {
//    print_paths(paths);
//    let keys: Vec<&Pair> = paths.keys().collect();
//    let mut out = Vec::new();
//    let mut state = HashMap::with_capacity(keys.len());
//    for (uid, _) in paths {
//        state.insert(uid, 0);
//    }
//    let mut time = 0;
//    while !state.is_empty() {
//        time += 1;
//        println!("at t = {time}");
//        println!("state is {state:?}");
//        println!();
//        let mut moved = HashSet::with_capacity(keys.len());
//        for uid in keys.iter() {
//            let idx = state[uid];
//            let s = paths[uid][idx];
//            if time > s.duration.1 {
//                if paths[uid].len() > idx + 1 {
//                    state.insert(*uid, idx + 1);
//                    moved.insert(uid);
//                } else {
//                    state.remove(uid);
//                }
//            }
//        }
//        // Check for conflicts
//        for (uid_0, idx_0) in state.iter() {
//            for (uid_1, idx_1) in state.iter().filter(|(uid, _)| *uid > uid_0) {
//                let state_0 = paths[uid_0][*idx_0];
//                let state_1 = paths[uid_1][*idx_1];
//                let intersects = state_0.location.intersects(state_1.location);
//                let includes_moved = moved.contains(&uid_0) || moved.contains(&uid_1);
//                if intersects && includes_moved {
//                    println!("found conflict between");
//                    println!("{state_0:?}");
//                    println!("{state_1:?}");
//                    println!();
//                    out.push(Conflict(state_0, state_1));
//                }
//            }
//        }
//    }
//    out
//}
