use crate::astar::{AStar, Constraint, Specification};
use crate::conflict::{find_conflicts, Conflict};
use crate::prelude::*;
use std::collections::{BinaryHeap, HashMap};
use std::io;
use std::ptr::eq as ptr_eq;

mod exploration;

use exploration::*;

#[derive(Clone)]
pub struct CBS<'a> {
    astar: &'a AStar,
    constraints: Vec<Constraint>,
    solution: HashMap<Pair, Path>,
    cost: usize,
    conflicts: Vec<Conflict>,
}

impl PartialEq for CBS<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.cost == other.cost
            && self.constraints == other.constraints
            && ptr_eq(self.astar, other.astar)
    }
}

impl Eq for CBS<'_> {}

// Min-heap, low cost first with ties broken by low numbers of conflicts, then constraints
impl Ord for CBS<'_> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other
            .cost
            .cmp(&self.cost)
            .then_with(|| other.conflicts.len().cmp(&self.conflicts.len()))
            .then_with(|| other.constraints.len().cmp(&self.constraints.len()))
            .then_with(|| other.constraints.cmp(&self.constraints))
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
            solution: HashMap::new(),
            cost: 0,
            conflicts: Vec::new(),
        }
    }

    pub fn init<'a>(astar: &'a AStar, origins: &[Pair]) -> CBS<'a> {
        let mut cbs = CBS::new(astar);
        cbs.init_paths(origins);
        cbs.extend_paths();
        cbs.find_conflicts();
        cbs
    }

    fn init_paths(&mut self, origins: &[Pair]) {
        for cell in origins {
            let path = self
                .astar
                .astar(Specification::new(*cell))
                .expect("Unable to find preliminary path!");
            self.solution.insert(*cell, path);
        }
    }

    // TODO: extend as waits, not as a block
    fn extend_paths(&mut self) {
        let mut end_time = 0;
        for path in self.solution.values() {
            if path.is_empty() {
                panic!("Empty solution!");
            }
            if path[path.len() - 1].duration.1 > end_time {
                end_time = path[path.len() - 1].duration.1
            }
        }
        for (_, path) in self.solution.iter_mut() {
            let idx = path.len() - 1;
            path[idx].duration.1 = end_time
        }
        self.cost = end_time;
    }

    fn find_conflicts(&mut self) {
        self.conflicts = find_conflicts(&self.solution);
    }

    /// Exploration functions

    fn explore_constraint(&self, constraint: Constraint) -> Option<Path> {
        let specs = Specification::create(constraint, &self.constraints);
        let mut new_path = self.solution[&constraint.uid()].clone();
        for spec in specs {
            if let Some(patch) = self.astar.astar(spec) {
                new_path = patch_path(new_path, patch)
            } else {
                return None;
            }
        }
        Some(new_path)
    }

    fn explore_conflict(&self, conflict: Conflict) -> Exploration {
        let constraints = Conflict::constraints(conflict);
        // TODO: fix, highly fragile!
        let uid_1_avoid = match constraints[1] {
            Constraint::Occupy(state) => Constraint::Avoid(UnitState {
                uid: conflict.uids()[1],
                location: state.location,
                duration: state.duration,
            }),
            Constraint::Avoid(_) => panic!("constraints outputting in a new order?"),
        };
        let path_0 = self.explore_constraint(constraints[0]);
        let path_1 = self.explore_constraint(uid_1_avoid);
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

    fn change_path(&mut self, uid: Pair, path: Path) {
        self.solution.insert(uid, path);
    }
}

fn update_cbs(mut cbs: CBS, constraint: Constraint, path: Path, path_uid: Pair) -> CBS {
    //if cbs.constraints.contains(&constraint) {
    //    println!("duplicate constraint!");
    //    println!("old constraints: {:?}", cbs.constraints);
    //    println!("new constraint: {constraint:?}");
    //    println!("old path:");
    //    print_path(&cbs.solution[&constraint.uid()]);
    //    println!("new path:");
    //    print_path(&path);
    //    panic!();
    //}
    cbs.constraints.push(constraint);
    cbs.change_path(path_uid, path);
    cbs.extend_paths();
    cbs
}

// TODO: Check me
fn expand_exploration(cbs: CBS, exploration: Exploration) -> Vec<CBS> {
    let mut out = Vec::with_capacity(exploration.constraints.len()); // with_capacity(2);
    let uids = exploration.uids();
    for (idx, solution) in exploration.solutions.into_iter().enumerate() {
        if let Some(path) = solution {
            let new = update_cbs(cbs.clone(), exploration.constraints[idx], path, uids[idx]);
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
    let explorations = cbs.explore();
    let greedy = greedy_choices(explorations);
    expand_explorations(cbs, greedy)
}

fn greedy_with_heuristic(cbs: CBS) -> HashMap<Pair, Vec<UnitState>> {
    let mut open = BinaryHeap::new();
    open.push(cbs);
    //let mut i = 0;
    loop {
        //println!("loop {i}");
        //i += 1;
        let node = match open.pop() {
            None => panic!("Exhausted states. Should be impossible."),
            Some(new_node) => new_node,
        };
        if node.conflicts.is_empty() {
            return node.solution;
        }
        //println!("from constraints {:?}", node.constraints);
        //println!("have solution");
        //print_paths(&node.solution);
        //println!("with conflicts {:?}", node.conflicts);
        //println!();
        let children = expand_node(node);
        for child in children {
            //println!("- with constraints {:?}", child.constraints);
            //println!("- made solution");
            //print_paths(&child.solution);
            //println!("- with conflicts {:?}", child.conflicts);
            //println!();
            open.push(child);
            //println!();
        }
        //let mut s = String::new();
        //let _ = io::stdin().read_line(&mut s);
    }
}

pub fn solve_mapf(astar: &AStar, origins: &[Pair]) -> HashMap<Pair, Vec<UnitState>> {
    let cbs = CBS::init(astar, origins);
    greedy_with_heuristic(cbs)
}
