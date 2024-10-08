use crate::mapf::MAPF;
use crate::prelude::*;
use core::panic;
use std::collections::{BinaryHeap, HashSet};

fn rect_conflict(cell1: Pair, cell2: Pair, unit_size: Pair) -> bool {
    let ((x1, y1), (x2, y2), (dx, dy)) = (cell1, cell2, unit_size);
    x1 < x2 + dx && x2 < x1 + dx && y1 < y2 + dy && y2 < y1 + dy
}

struct ConfState {
    uid: Pair,
    idx: usize,
    cell: Pair,
    stay: Pair,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum ConstGen {
    Naive,
}

#[derive(Clone, PartialEq, Eq)]
pub struct CBS<'a> {
    pub mapf: &'a MAPF,
    pub constraints: Vec<Constraint>,
    pub solution: Vec<Vec<ScoredCell>>,
    pub cost: usize,
    pub conflicts: Vec<Conflict>,
    c_gen: ConstGen,
}

// Recall that higher ord means higher priority,
// so we want cost.x < cost.y => x > y.
impl Ord for CBS<'_> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.cost
            .cmp(&other.cost)
            .then_with(|| self.conflicts.len().cmp(&other.conflicts.len()))
            .then_with(|| self.constraints.cmp(&other.constraints))
            .reverse()
    }
}

impl PartialOrd for CBS<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl CBS<'_> {
    fn new(mapf: &MAPF) -> CBS {
        let mut cbs = CBS {
            mapf,
            constraints: Vec::new(),
            solution: Vec::with_capacity(mapf.origins.len()),
            cost: 0,
            conflicts: Vec::new(),
            c_gen: ConstGen::Naive,
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

    pub fn init(mapf: &MAPF, c_gen: ConstGen) -> CBS {
        let mut cbs = CBS::new(mapf);
        let num_units = cbs.solution.len();
        let mut modified: HashSet<usize> = HashSet::with_capacity(num_units);
        for i in 0..num_units {
            modified.insert(i);
        }
        cbs.find_solution(&modified);
        cbs.find_cost(&modified);
        cbs.find_conflicts(&modified);
        cbs.c_gen = c_gen;
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

    fn add_if_conflict(&mut self, state_i: &ConfState, state_j: &ConfState) {
        if rect_conflict(state_i.cell, state_j.cell, self.mapf.unit_size) {
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
            //println!("{:?}", Conflict(cii, cij));
            self.conflicts.push(Conflict(cii, cij));
        }
    }

    fn add_conflicts(&mut self, modified: &HashSet<usize>) {
        let mut state = Vec::with_capacity(self.solution.len());
        let end_time = self.cost;
        for path in &self.solution {
            state.push(ConfState {
                uid: path[0].cell,
                idx: 0,
                cell: path[0].cell,
                stay: path[0].stay,
            });
        }
        for time in 1..=end_time {
            // Update states
            let mut moved = vec![false; state.len()];
            for (i, path) in self.solution.iter().enumerate() {
                let idx = state[i].idx;
                if time > path[idx].stay.1 && idx < path.len() - 1 {
                    //println!("{i} moved from {:?} with a stay of {:?} to {:?} with a stay of {:?} at time {time}, generating the conflicts:",
                    //    state[i].cell,
                    //    state[i].stay,
                    //    path[idx + 1].cell,
                    //    (path[idx].time + 1, path[idx + 1].time),
                    //);
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

    pub fn generate_constraints(&self) -> Vec<Vec<Constraint>> {
        match self.c_gen {
            ConstGen::Naive => naive_generator(self),
        }
    }
}

fn apply_constraints(mut cbs: CBS, mut constraints: Vec<Constraint>) -> Option<CBS> {
    let mut modified = HashSet::with_capacity(cbs.solution.len());
    for constraint in &constraints {
        modified.insert(cbs.unit_to_idx(constraint.uid));
    }
    cbs.constraints.append(&mut constraints);
    //println!(
    //    "Attempting to find solutions with constraints {:?}",
    //    cbs.constraints
    //);
    cbs.find_solution(&modified);
    for idx in &modified {
        if cbs.solution[*idx].is_empty() {
            cbs.cost = usize::MAX;
            //println!(
            //    "Failed to find path for {idx} with constraints {:?}",
            //    cbs.constraints
            //);
            return None;
        }
    }
    //println!("Found!");
    cbs.find_cost(&modified);
    cbs.find_conflicts(&modified);
    Some(cbs)
}

// Forbids the first conflict
fn naive_generator(cbs: &CBS) -> Vec<Vec<Constraint>> {
    let conflict = cbs.conflicts[0];
    let constraint1 = Constraint {
        uid: conflict.0.uid,
        cell: conflict.0.cell,
        stay: conflict.1.stay,
    };
    let constraint2 = Constraint {
        uid: conflict.1.uid,
        cell: conflict.1.cell,
        stay: conflict.0.stay,
    };
    vec![vec![constraint1], vec![constraint2]]
}

fn children(cbs: CBS) -> Vec<CBS> {
    let new = cbs.generate_constraints();
    let mut children = Vec::with_capacity(new.len());
    for constraints in new {
        match apply_constraints(cbs.clone(), constraints) {
            None => {}
            Some(child) => children.push(child),
        }
    }
    children
}

pub fn solve_mapf(mapf: &MAPF) -> Vec<Vec<ScoredCell>> {
    let root = CBS::init(mapf, ConstGen::Naive);
    let mut heap = BinaryHeap::new();
    heap.push(root);
    loop {
        let best = heap.pop().unwrap();
        if best.conflicts.is_empty() {
            return best.solution;
        }
        heap.extend(children(best));
    }
}