use crate::mapf::AStar;
use crate::prelude::*;
use std::cmp::{max, min};
use std::collections::{BinaryHeap, HashSet};

struct UnitState {
    uid: Pair,
    idx: usize,
    cell: Pair,
    stay: Pair,
}

#[derive(Clone, PartialEq, Eq)]
pub struct CBS<'a> {
    pub mapf: &'a AStar,
    pub constraints: Vec<Constraint>,
    pub solution: Vec<Path>,
    pub cost: usize,
    pub conflicts: Vec<Conflict>,
}

// Min-heap, low cost first with ties broken by low numbers of conflicts, then constraints
impl Ord for CBS<'_> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other
            .cost
            .cmp(&self.cost)
            .then_with(|| other.conflicts.len().cmp(&self.conflicts.len()))
            .then_with(|| other.constraints.len().cmp(&self.constraints.len()))
            .then_with(|| other.solution.cmp(&self.solution))
    }
}

impl PartialOrd for CBS<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl CBS<'_> {
    /// init() functions

    fn new(mapf: &AStar) -> CBS {
        CBS {
            mapf,
            constraints: Vec::new(),
            solution: Vec::with_capacity(mapf.origins.len()),
            cost: 0,
            conflicts: Vec::new(),
        }
    }

    pub fn init(mapf: &AStar) -> CBS {
        let mut cbs = CBS::new(mapf);
        let num_units = cbs.solution.len();
        let mut modified: HashSet<usize> = HashSet::with_capacity(num_units);
        for i in 0..num_units {
            modified.insert(i);
        }
        cbs.find_paths();
        cbs.extend_paths();
        cbs.find_cost();
        cbs.find_conflicts();
        cbs
    }

    fn find_paths(&mut self) {
        for cell in &self.mapf.origins {
            let path = self
                .mapf
                .astar(*cell, &self.constraints)
                .expect("Unable to find preliminary path!");
            self.solution.push(path);
        }
    }

    fn extend_paths(&mut self) {
        let mut end_time = 0;
        for path in &self.solution {
            if path.is_empty() {
                return;
            }
            if path[path.len() - 1].duration.1 > end_time {
                end_time = path[path.len() - 1].duration.1
            }
        }
        for path in self.solution.iter_mut() {
            let idx = path.len() - 1;
            path[idx].duration.1 = end_time
        }
    }

    fn find_cost(&mut self) {
        let path = &self.solution[0];
        self.cost = path[path.len() - 1].duration.1;
    }

    fn add_if_conflict(&mut self, state_i: &UnitState, state_j: &UnitState) {
        if units_collide(state_i.cell, state_j.cell) {
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

    fn find_conflicts(&mut self) {
        let mut state = Vec::with_capacity(self.solution.len());
        let end_time = self.cost;
        for path in &self.solution {
            state.push(UnitState {
                uid: path[0].location,
                idx: 0,
                cell: path[0].location,
                stay: path[0].duration,
            });
        }
        for time in 1..=end_time {
            let mut moved = vec![false; state.len()];
            for (i, path) in self.solution.iter().enumerate() {
                let idx = state[i].idx;
                if time > path[idx].duration.1 && idx < path.len() - 1 {
                    state[i].cell = path[idx + 1].location;
                    state[i].stay = path[idx + 1].duration;
                    state[i].idx += 1;
                    moved[i] = true;
                }
            }
            // Check for conflicts
            for (i, i_moved) in moved.iter().enumerate() {
                for (j, j_moved) in moved.iter().enumerate().skip(i + 1) {
                    let includes_moved = *i_moved || *j_moved;
                    if includes_moved {
                        self.add_if_conflict(&state[i], &state[j])
                    }
                }
            }
        }
    }

    /// Exploration functions

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
        println!("{:?}", self.constraints);
        println!("{:?}", self.solution);
        for conflict in &self.conflicts {
            println!("{:?}", conflict);
        }
        println!();
        let mut explorations = Vec::with_capacity(self.conflicts.len());
        for conflict in &self.conflicts {
            let exploration = self.explore_conflict(*conflict);
            explorations.push(exploration);
        }
        explorations
    }

    fn change_path(&mut self, path: Path) {
        for (idx, old_path) in self.solution.iter().enumerate() {
            if path[0] == old_path[0] {
                self.solution[idx] = path;
                break;
            }
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Exploration {
    conflict: Conflict,
    constraints: (Constraint, Constraint),
    solutions: (Option<Path>, Option<Path>),
}

impl Exploration {
    fn score(&self) -> usize {
        let mut min_score = match &self.solutions.0 {
            Some(path) => path.len(),
            None => usize::MAX,
        };
        min_score = match &self.solutions.1 {
            Some(path) => min(min_score, path.len()),
            None => min_score,
        };
        min_score
    }

    fn secondary_score(&self) -> usize {
        let mut max_score = match &self.solutions.0 {
            Some(path) => path.len(),
            None => usize::MAX,
        };
        max_score = match &self.solutions.1 {
            Some(path) => max(max_score, path.len()),
            None => usize::MAX,
        };
        max_score
    }

    fn uids(&self) -> (Pair, Pair) {
        self.conflict.uids()
    }
}

// We want higher primary scores first, with lower secondary breaking ties
impl Ord for Exploration {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other
            .score()
            .cmp(&self.score())
            .then_with(|| self.secondary_score().cmp(&other.secondary_score()))
    }
}

impl PartialOrd for Exploration {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

fn prioritize(explorations: Vec<Exploration>) -> Vec<Exploration> {
    let mut out: Vec<Exploration> = Vec::with_capacity(explorations.len());
    for exploration in explorations {
        if exploration.solutions == (None, None) {
            continue;
        }
        let mut include = true;
        let mut replace_at = None;
        for (idx, chosen) in out.iter_mut().enumerate() {
            if exploration.uids() == chosen.conflict.uids() {
                if exploration.score() > chosen.score() {
                    replace_at = Some(idx);
                } else {
                    include = false
                }
                break;
            }
        }
        if include {
            match replace_at {
                Some(idx) => out[idx] = exploration,
                None => out.push(exploration),
            }
        }
    }
    out.sort();
    out
}

fn greedy_choices(explorations: Vec<Exploration>) -> Vec<Exploration> {
    let mut out = Vec::with_capacity(explorations.len());
    let mut seen = Vec::with_capacity(explorations.len() * 2);
    for exploration in explorations {
        let uids = exploration.uids();
        if !(seen.contains(&uids.0) || seen.contains(&uids.1)) {
            out.push(exploration);
            seen.push(uids.0);
            seen.push(uids.1);
        }
    }
    out
}

fn expand_exploration(cbs: CBS, exploration: Exploration) -> Vec<CBS> {
    let mut out = Vec::with_capacity(2);
    if let Some(path) = exploration.solutions.0 {
        let mut child = cbs.clone();
        child.cost = max(child.cost, path[path.len() - 1].duration.1);
        child.constraints.push(exploration.constraints.0);
        child.change_path(path);
        out.push(child);
    }
    if let Some(path) = exploration.solutions.1 {
        let mut child = cbs;
        child.cost = max(child.cost, path[path.len() - 1].duration.1);
        child.constraints.push(exploration.constraints.1);
        child.change_path(path);
        out.push(child);
    }
    out
}

fn expand_explorations(cbs: CBS, explorations: Vec<Exploration>) -> Vec<CBS> {
    let mut out = vec![cbs];
    for exploration in explorations {
        let mut new_out: Vec<CBS> = Vec::with_capacity(out.len() * 2);
        for state in out {
            new_out.append(&mut expand_exploration(state, exploration.clone()));
        }
        out = new_out;
    }
    for node in out.iter_mut() {
        node.extend_paths();
        node.conflicts = Vec::new();
        node.find_conflicts();
    }
    out
}

fn expand_node(cbs: CBS) -> Vec<CBS> {
    let mut explorations = cbs.explore();
    explorations = prioritize(explorations);
    explorations = greedy_choices(explorations);
    expand_explorations(cbs, explorations)
}

fn greedy_with_heuristic(cbs: CBS) -> Vec<Path> {
    let mut open = BinaryHeap::new();
    open.push(cbs);
    loop {
        let node = match open.pop() {
            None => panic!("Exhausted states. Should be impossible."),
            Some(new_node) => new_node,
        };
        let children = expand_node(node.clone());
        if children.len() == 1 {
            println!("{:?}", node.constraints);
            println!("{:?}", children[0].constraints);
            println!();
        }
        for child in children {
            if child.conflicts.is_empty() {
                return child.solution;
            } else {
                open.push(child);
            }
        }
    }
}

pub fn solve_mapf(mapf: &AStar) -> Vec<Path> {
    let cbs = CBS::init(mapf);
    greedy_with_heuristic(cbs)
}
