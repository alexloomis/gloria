use crate::prelude::*;
use radix_heap::RadixHeapMap;
use std::cmp::min;
use std::fmt::Debug;
use std::rc::Rc;

pub mod constraint;
pub mod grid;
pub mod terrain;

pub use constraint::*;
pub use grid::Grid;
pub use terrain::Terrain;

#[cfg(test)]
mod tests;

#[derive(Clone)]
pub struct ScoredCell {
    // Cost including heuristic, what time do we think we will arrive?
    pub unit: UnitState,
    pub cost: usize,
    pub prev: Option<Rc<ScoredCell>>,
}

impl ScoredCell {
    fn uid(&self) -> Pair {
        self.unit.uid
    }

    fn location(&self) -> Rect {
        self.unit.location
    }

    fn duration(&self) -> Pair {
        self.unit.duration
    }
}

// Cost is a function of location and duration, so if location and duration are equal,
// then so too *should* cost be.
impl PartialEq for ScoredCell {
    fn eq(&self, other: &Self) -> bool {
        self.location() == other.location() && self.duration() == other.duration()
    }
}

impl Eq for ScoredCell {}

// Lowest cost has highest priority, then earliest departure, then earliest arrival, then we don't
// really care, so we just do by cell.
impl Ord for ScoredCell {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other
            .cost
            .cmp(&self.cost)
            .then_with(|| other.duration().1.cmp(&self.duration().1))
            .then_with(|| other.duration().0.cmp(&self.duration().0))
            .then_with(|| other.location().cmp(&self.location()))
    }
}

impl PartialOrd for ScoredCell {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

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
            self.prev
        )
    }
}

fn open_allows_candidate(candidate: &ScoredCell, open: &RadixHeapMap<i64, ScoredCell>) -> bool {
    !open
        .iter()
        .any(|(_, cell)| candidate.cost <= cell.cost && cell.location() == candidate.location())
}

fn reconstruct_path(last: ScoredCell) -> Path {
    let mut path = Vec::with_capacity(last.duration().1 + 1);
    path.push(last.unit);
    let mut prev = Rc::new(last);
    while let Some(scored_cell) = &prev.prev {
        path.push(scored_cell.unit);
        prev = scored_cell.clone();
    }
    path.reverse();
    path
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
        let uid = sc.uid();
        let departure = sc.duration().1;
        let neighbors = self.terrain.neighbors(sc.location().origin);
        let prev = Some(sc.clone());
        let mut succ = Vec::with_capacity(neighbors.len() + 1);

        let unit = UnitState {
            uid,
            location: sc.location(),
            duration: Pair(departure + 1, departure + 1),
        };
        let wait = ScoredCell {
            cost: sc.cost + 1,
            unit,
            prev: prev.clone(),
        };
        if satisfies_constraints(wait.unit, constraints) {
            succ.push(wait);
        }

        for location in neighbors {
            if let Some(estimate) = heuristic[location] {
                let new_departure = departure + self.terrain.cost(location);
                let unit = UnitState {
                    uid,
                    location: location.extend(self.terrain.unit_extent()),
                    duration: Pair(departure + 1, new_departure),
                };
                let candidate = ScoredCell {
                    cost: new_departure + estimate,
                    unit,
                    prev: prev.clone(),
                };
                if satisfies_constraints(candidate.unit, constraints) {
                    succ.push(candidate);
                }
            }
        }
        //for s in &succ {
        //    if s.duration().0 != sc.duration().1 + 1 {
        //        println!("bad successor!");
        //        println!("from {sc:?}");
        //        println!("to {s:?}");
        //        panic!();
        //    }
        //}
        succ
    }

    fn arrived(
        &self,
        unit: UnitState,
        end_cell: Option<Pair>,
        end_time: Option<usize>,
        constraints: &[Constraint],
    ) -> bool {
        let good_cell = match end_cell {
            Some(cell) => unit.location.origin == cell,
            None => self.destinations.contains(&unit.location.origin),
        };
        let good_time = match end_time {
            Some(time) => unit.duration.1 >= time || may_stop(unit, constraints),
            None => may_stop(unit, constraints),
        };
        good_cell && good_time
    }

    fn satisfies_cutoff(scored_cell: &ScoredCell, end_time: Option<usize>) -> bool {
        match end_time {
            Some(time) => scored_cell.cost <= time,
            None => true,
        }
    }

    pub fn astar(&self, specs: Specification) -> Option<Path> {
        let initial = UnitState {
            uid: specs.uid,
            location: specs.start_cell.extend(self.terrain.unit_extent()),
            duration: Pair(specs.start_time, specs.start_time),
        };
        let heuristic = match specs.end_cell {
            Some(cell) => &self.terrain.distances()[cell],
            None => &self.heuristic,
        };

        // Are we too far away?
        match heuristic[specs.start_cell] {
            None => {
                return None;
            }
            Some(estimate) => {
                if let Some(time) = specs.end_time {
                    if specs.start_time + estimate > time {
                        return None;
                    }
                }
            }
        }

        let mut open = RadixHeapMap::new_at(0);
        let sc = ScoredCell {
            cost: 0,
            unit: initial,
            prev: None,
        };
        // May cause an issue if cost > 2^32
        open.push(-(sc.cost as i64), sc);

        loop {
            let (_, current) = match open.pop() {
                None => {
                    return None;
                }
                Some(sc) => sc,
            };
            if AStar::satisfies_cutoff(&current, specs.end_time) {
                for successor in self.successors(current.clone(), heuristic, &specs.constraints) {
                    if open_allows_candidate(&successor, &open) {
                        if self.arrived(
                            successor.unit,
                            specs.end_cell,
                            specs.end_time,
                            &specs.constraints,
                        ) {
                            //println!("{successor:?}");
                            let path = reconstruct_path(successor);
                            //check_path_times(&path);
                            return Some(path);
                        }
                        open.push(-(successor.cost as i64), successor);
                    }
                }
            }
        }
    }
}
