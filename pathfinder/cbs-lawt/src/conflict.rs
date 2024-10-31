use crate::astar::Constraint;
use crate::prelude::*;
use std::cmp::max;
use std::collections::HashMap;

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
