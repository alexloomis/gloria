use crate::astar::AStar;
use crate::prelude::{Path, *};
use std::collections::BinaryHeap;

struct UnitState {
    uid: Pair,
    path_idx: usize,
    location: Rect,
    duration: Pair,
}

#[derive(Clone, PartialEq, Eq)]
pub struct CBS<'a> {
    pub astar: &'a AStar,
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

    fn new(astar: &AStar) -> CBS {
        CBS {
            astar,
            constraints: Vec::new(),
            solution: Vec::with_capacity(astar.origins.len()),
            cost: 0,
            conflicts: Vec::new(),
        }
    }

    pub fn init(astar: &AStar) -> CBS {
        let mut cbs = CBS::new(astar);
        cbs.find_paths();
        cbs.extend_paths();
        cbs.find_cost();
        cbs.find_conflicts();
        cbs
    }

    fn find_paths(&mut self) {
        for cell in &self.astar.origins {
            let path = self
                .astar
                .astar(*cell, &self.constraints)
                .expect("Unable to find preliminary path!");
            self.solution.push(path);
        }
    }

    fn extend_paths(&mut self) {
        let mut end_time = 0;
        for path in &self.solution {
            if path.is_empty() {
                panic!("Empty solution!");
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

    fn to_conflict(state_i: &UnitState, state_j: &UnitState) -> Conflict {
        let cii = ConflictInfo {
            uid: state_i.uid,
            location: state_i.location,
            duration: state_i.duration,
        };
        let cij = ConflictInfo {
            uid: state_j.uid,
            location: state_j.location,
            duration: state_j.duration,
        };
        Conflict(cii, cij)
    }

    fn find_conflicts(&mut self) {
        let mut state = Vec::with_capacity(self.solution.len());
        let end_time = self.cost;
        for path in &self.solution {
            state.push(UnitState {
                uid: path[0].location.origin,
                path_idx: 0,
                location: path[0].location,
                duration: path[0].duration,
            });
        }
        for time in 1..=end_time {
            let mut moved = vec![false; state.len()];
            for (i, path) in self.solution.iter().enumerate() {
                let idx = state[i].path_idx;
                if time > path[idx].duration.1 && idx < path.len() - 1 {
                    state[i].location = path[idx + 1].location;
                    state[i].duration = path[idx + 1].duration;
                    state[i].path_idx += 1;
                    moved[i] = true;
                }
            }
            // Check for conflicts
            for (i, i_moved) in moved.iter().enumerate() {
                for (j, j_moved) in moved.iter().enumerate().skip(i + 1) {
                    let intersects = state[i].location.intersects(state[j].location);
                    let includes_moved = *i_moved || *j_moved;
                    if intersects && includes_moved {
                        self.conflicts.push(CBS::to_conflict(&state[i], &state[j]));
                    }
                }
            }
        }
    }

    /// Exploration functions

    fn explore_constraint(&self, constraint: Constraint) -> Option<Path> {
        let mut constraints = self.constraints.clone();
        constraints.push(constraint);
        self.astar.astar(constraint.uid, &constraints)
    }

    fn explore_conflict(&self, conflict: Conflict) -> Exploration {
        let constraints = Conflict::constraints(conflict);
        let path_0 = self.explore_constraint(constraints[0]);
        let path_1 = self.explore_constraint(constraints[1]);
        Exploration {
            conflict,
            constraints,
            solutions: [path_0, path_1],
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

    fn change_path(&mut self, path: Path) {
        for (idx, old_path) in self.solution.iter().enumerate() {
            if path[0].location == old_path[0].location {
                self.solution[idx] = path;
                break;
            }
        }
    }
}

// TODO: 70% sure the bug is somewhere between here and EOF.

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Exploration {
    conflict: Conflict,
    constraints: [Constraint; 2],
    solutions: [Option<Path>; 2],
}

impl Exploration {
    fn score(&self) -> usize {
        self.solutions
            .iter()
            .map(|solution| {
                solution
                    .as_ref()
                    .map(|path| path.len())
                    .unwrap_or(usize::MAX)
            })
            .min()
            .unwrap()
    }

    fn secondary_score(&self) -> usize {
        self.solutions
            .iter()
            .map(|solution| {
                solution
                    .as_ref()
                    .map(|path| path.len())
                    .unwrap_or(usize::MAX)
            })
            .max()
            .unwrap()
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
        if exploration.solutions == [None, None] {
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
    for exploration in prioritize(explorations) {
        let uids = exploration.uids();
        if !(seen.contains(&uids.0) || seen.contains(&uids.1)) {
            out.push(exploration);
            seen.push(uids.0);
            seen.push(uids.1);
        }
    }
    out
}

// TODO: sus

fn update_cbs(mut cbs: CBS, constrait: Constraint, path: Path) -> CBS {
    cbs.constraints.push(constrait);
    cbs.change_path(path);
    cbs.extend_paths();
    cbs.find_cost();
    cbs
}

fn expand_exploration(cbs: CBS, exploration: Exploration) -> Vec<CBS> {
    let mut out = Vec::with_capacity(exploration.constraints.len());
    for (idx, solution) in exploration.solutions.into_iter().enumerate() {
        if let Some(path) = solution {
            let new = update_cbs(cbs.clone(), exploration.constraints[idx], path);
            out.push(new);
        }
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
