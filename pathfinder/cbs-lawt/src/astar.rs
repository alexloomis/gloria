use crate::prelude::*;
use radix_heap::RadixHeapMap;
use std::cmp::{min, Reverse};
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::rc::Rc;

pub mod constraint;
pub mod grid;
pub mod terrain;

pub use constraint::*;
pub use grid::Grid;
pub use terrain::Terrain;

#[derive(Clone)]
pub struct ScoredCell {
    // Cost including heuristic, what time do we think we will arrive?
    pub unit: UnitState,
    pub cost: usize,
    pub pred: Option<Rc<ScoredCell>>,
}

impl ScoredCell {
    fn location(&self) -> Rect {
        self.unit.location
    }

    fn duration(&self) -> Pair {
        self.unit.duration
    }
}

// Cost is a function of location and duration.1, so if location and duration are equal,
// then so too *should* cost be.
impl PartialEq for ScoredCell {
    fn eq(&self, other: &Self) -> bool {
        self.location() == other.location() && self.duration().1 == other.duration().1
    }
}

impl Eq for ScoredCell {}

// Lowest cost comes first.
// Finding all paths, so tie-breaking doesn't matter,
// but we define the ordering to agree with equality.
// Later durations come earlier, because they are closest to the target.
// Using a max heap, so we reverse the ordering
//impl Ord for ScoredCell {
//    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
//        self.cost
//            .cmp(&other.cost)
//            .then_with(|| other.duration().1.cmp(&self.duration().1))
//            .reverse()
//    }
//}

//impl PartialOrd for ScoredCell {
//    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
//        Some(self.cmp(other))
//    }
//}

impl Debug for ScoredCell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "({}, {}) × ({}, {}) $ {}\n  prev: {:?}",
            self.location().origin.0,
            self.location().origin.1,
            self.duration().0,
            self.duration().1,
            self.cost,
            self.pred
        )
    }
}

fn reconstruct_path(last: ScoredCell) -> Path {
    let mut path = Vec::with_capacity(last.duration().1 + 1);
    path.push(last.unit);
    let mut prev = Rc::new(last);
    while let Some(scored_cell) = &prev.pred {
        path.push(scored_cell.unit);
        prev = scored_cell.clone();
    }
    path.reverse();
    path
}

pub struct PathTree {
    nodes: HashSet<Rc<UnitState>>,
    origins: Vec<Rc<UnitState>>,
    destinations: Vec<Rc<UnitState>>,
    preds: HashMap<Rc<UnitState>, Vec<Rc<UnitState>>>,
    succs: HashMap<Rc<UnitState>, Vec<Rc<UnitState>>>,
}

impl PathTree {
    fn new() -> PathTree {
        PathTree {
            nodes: HashSet::new(),
            origins: Vec::new(),
            destinations: Vec::new(),
            preds: HashMap::new(),
            succs: HashMap::new(),
        }
    }

    fn backtrace_paths(mut ends: Vec<ScoredCell>) -> PathTree {
        let mut pt = PathTree::new();
        while let Some(node) = ends.pop() {
            let state = Rc::new(node.unit);
            // if the state is new, insert it and...
            if pt.nodes.insert(state) {
                if let Some(pred) = node.pred {
                    ends.push(pred);
                } else {
                    pt.origins.push(state);
                }
            }
        }
        pt
    }
}

#[derive(Debug)]
pub struct AStar {
    pub terrain: Terrain,
    pub destinations: Vec<Pair>,
    heuristic: Grid<Option<usize>>,
}

impl AStar {
    fn generate_heuristic(&mut self) {
        self.heuristic = self.terrain.distances()[self.destinations[0]].clone();
        for destination in self.destinations.iter().skip(1) {
            let distances = &self.terrain.distances()[*destination];
            for (cell, cost) in distances.indexed_iter() {
                if let Some(c_new) = cost {
                    self.heuristic[cell] = match self.heuristic[cell] {
                        None => Some(*c_new),
                        Some(c_old) => Some(min(*c_new, c_old)),
                    }
                }
            }
        }
    }

    fn new(terrain: Terrain, destinations: Vec<Pair>) -> AStar {
        AStar {
            terrain,
            heuristic: Grid::init(Pair(0, 0), None),
            destinations,
        }
    }

    pub fn init(terrain: Terrain, destinations: Vec<Pair>) -> AStar {
        let mut out = AStar::new(terrain, destinations);
        out.generate_heuristic();
        out
    }

    fn successors(
        &self,
        scored_cell: ScoredCell,
        heuristic: &Grid<Option<usize>>,
        constraints: &[Constraint],
    ) -> Vec<ScoredCell> {
        let sc = Rc::new(scored_cell);
        let departure = sc.duration().1;
        let neighbors = self.terrain.neighbors(sc.location().origin);
        let prev = Some(sc.clone());
        let mut succ = Vec::with_capacity(neighbors.len() + 1);

        let unit = UnitState {
            location: sc.location(),
            duration: Pair(departure + 1, departure + 1),
        };
        let wait = ScoredCell {
            cost: sc.cost + 1,
            unit,
            pred: prev.clone(),
        };
        if satisfies_constraints(wait.unit, constraints) {
            succ.push(wait);
        }

        for location in neighbors {
            if let Some(estimate) = heuristic[location] {
                let new_departure = departure + self.terrain.cost(location);
                let unit = UnitState {
                    location: location.extend(self.terrain.unit_extent()),
                    duration: Pair(departure + 1, new_departure),
                };
                let candidate = ScoredCell {
                    cost: new_departure + estimate,
                    unit,
                    pred: prev.clone(),
                };
                if satisfies_constraints(candidate.unit, constraints) {
                    succ.push(candidate);
                }
            }
        }
        succ
    }

    fn arrived(&self, unit: UnitState, end_time: usize) -> bool {
        let good_cell = self.destinations.contains(&unit.location.origin);
        let good_time = unit.duration.1 == end_time;
        good_cell && good_time
    }

    fn find_terminal(
        &self,
        origins: Vec<Pair>,
        end_time: usize,
        constraints: &[Constraint],
    ) -> Vec<ScoredCell> {
        // We either waited or we just arrived, giving potentially different dutations.
        let mut out = Vec::with_capacity(2 * self.destinations.len());

        let initial = origins.into_iter().map(|loc| ScoredCell {
            cost: 0,
            unit: {
                UnitState {
                    location: loc.extend(self.terrain.unit_extent()),
                    duration: Pair(0, 0),
                }
            },
            pred: None,
        });

        let mut open = RadixHeapMap::new_at(Reverse(0));
        for cell in initial {
            open.push(Reverse(cell.cost), cell);
        }

        loop {
            let (_, current) = match open.pop() {
                None => {
                    return out;
                }
                Some(sc) => sc,
            };
            for successor in self.successors(current.clone(), &self.heuristic, constraints) {
                if successor.cost <= end_time {
                    if self.arrived(successor.unit, end_time) {
                        out.push(successor);
                    } else {
                        open.push(Reverse(successor.cost), successor);
                    }
                }
            }
        }
    }
}
