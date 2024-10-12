use crate::mapf::MAPF;
use crate::prelude::*;
use core::panic;
use std::{collections::HashSet, ops::Sub};

fn rect_conflict(cell1: Pair, cell2: Pair) -> bool {
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

struct UnitState {
    uid: Pair,
    idx: usize,
    cell: Pair,
    stay: Pair,
}

pub struct Exploration {
    conflict: Conflict,
    constraints: (Constraint, Constraint),
    solutions: (Vec<ScoredCell>, Vec<ScoredCell>),
}

#[derive(Clone, PartialEq, Eq)]
pub struct CBS<'a> {
    pub mapf: &'a MAPF,
    pub constraints: Vec<Constraint>,
    pub solution: Vec<Vec<ScoredCell>>,
    pub cost: usize,
    pub conflicts: Vec<Conflict>,
}

impl CBS<'_> {
    fn new(mapf: &MAPF) -> CBS {
        let mut cbs = CBS {
            mapf,
            constraints: Vec::new(),
            solution: Vec::with_capacity(mapf.origins.len()),
            cost: 0,
            conflicts: Vec::new(),
        };
        for origin in &cbs.mapf.origins {
            cbs.solution.push(vec![ScoredCell {
                cell: *origin,
                cost: 0,
                stay: (0, 0),
                prev: None,
            }]);
        }
        cbs
    }

    pub fn init(mapf: &MAPF) -> CBS {
        let mut cbs = CBS::new(mapf);
        let num_units = cbs.solution.len();
        let mut modified: HashSet<usize> = HashSet::with_capacity(num_units);
        for i in 0..num_units {
            modified.insert(i);
        }
        cbs.find_solution(&modified);
        cbs.find_cost(&modified);
        cbs.find_conflicts(&modified);
        cbs
    }

    fn unit_to_idx(&self, cell: Pair) -> usize {
        match self.solution.iter().position(|c| cell == c[0].cell) {
            Some(idx) => idx,
            None => panic!("No solution starting at {:?}!", cell),
        }
    }

    fn idx_to_unit(&self, idx: usize) -> Pair {
        self.solution[idx][0].cell
    }

    fn find_solution(&mut self, modified: &HashSet<usize>) {
        for idx in modified {
            self.solution[*idx] = self.mapf.astar(self.idx_to_unit(*idx), &self.constraints);
        }
        self.extend_paths();
    }

    fn extend_paths(&mut self) {
        let mut end_time = 0;
        for path in &self.solution {
            if path.is_empty() {
                return;
            }
            if path[path.len() - 1].stay.1 > end_time {
                end_time = path[path.len() - 1].stay.1
            }
        }
        for path in self.solution.iter_mut() {
            let idx = path.len() - 1;
            path[idx].stay.1 = end_time
        }
    }

    fn find_cost(&mut self, modified: &HashSet<usize>) {
        for idx in modified {
            let path = &self.solution[*idx];
            let candidate = path[path.len() - 1].stay.1;
            if candidate > self.cost {
                self.cost = candidate
            }
        }
    }

    fn conflict_involves(&self, conflict: &Conflict, modified: &HashSet<usize>) -> bool {
        modified.contains(&self.unit_to_idx(conflict.0.uid))
            || modified.contains(&self.unit_to_idx(conflict.1.uid))
    }

    fn clear_conflicts(&mut self, modified: &HashSet<usize>) {
        let mut retained = Vec::with_capacity(self.conflicts.len());
        for conflict in &self.conflicts {
            if !self.conflict_involves(conflict, modified) {
                retained.push(*conflict);
            }
        }
        self.conflicts = retained;
    }

    fn add_if_conflict(&mut self, state_i: &UnitState, state_j: &UnitState) {
        if rect_conflict(state_i.cell, state_j.cell) {
            let cii = ConflictInfo {
                uid: state_i.uid,
                cell: state_i.cell,
                stay: state_i.stay,
            };
            let cij = ConflictInfo {
                uid: state_j.uid,
                cell: state_j.cell,
                stay: state_j.stay,
            };
            self.conflicts.push(Conflict(cii, cij));
        }
    }

    fn add_conflicts(&mut self, modified: &HashSet<usize>) {
        let mut state = Vec::with_capacity(self.solution.len());
        let end_time = self.cost;
        for path in &self.solution {
            state.push(UnitState {
                uid: path[0].cell,
                idx: 0,
                cell: path[0].cell,
                stay: path[0].stay,
            });
        }
        for time in 1..=end_time {
            let mut moved = vec![false; state.len()];
            for (i, path) in self.solution.iter().enumerate() {
                let idx = state[i].idx;
                if time > path[idx].stay.1 && idx < path.len() - 1 {
                    state[i].cell = path[idx + 1].cell;
                    state[i].stay = path[idx + 1].stay;
                    state[i].idx += 1;
                    moved[i] = true;
                }
            }
            // Check for conflicts
            for (i, i_moved) in moved.iter().enumerate() {
                for (j, j_moved) in moved.iter().enumerate().skip(i + 1) {
                    let includes_moved = *i_moved || *j_moved;
                    let includes_modified = modified.contains(&i) || modified.contains(&j);
                    if includes_moved && includes_modified {
                        self.add_if_conflict(&state[i], &state[j])
                    }
                }
            }
        }
    }

    fn find_conflicts(&mut self, modified: &HashSet<usize>) {
        self.clear_conflicts(modified);
        self.add_conflicts(modified);
    }

    fn explore_conflict(&self, conflict: Conflict) -> Exploration {
        let constraints = generate_constraints(conflict);
        let mut constraints_0 = self.constraints.clone();
        constraints_0.push(constraints.0);
        let path_0 = self.mapf.astar(constraints.0.uid, &constraints_0);
        let mut constraints_1 = self.constraints.clone();
        constraints_1.push(constraints.1);
        let path_1 = self.mapf.astar(constraints.1.uid, &constraints_1);
        Exploration {
            conflict,
            constraints,
            solutions: (path_0, path_1),
        }
    }

    fn explore(&self) -> Vec<Exploration> {
        let mut explorations = Vec::with_capacity(self.conflicts.len());
        for conflict in &self.conflicts {
            let exploration = self.explore_conflict(*conflict);
            explorations.push(exploration);
        }
        explorations
    }
}

fn generate_constraints(conflict: Conflict) -> (Constraint, Constraint) {
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
