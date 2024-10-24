use crate::constraint::*;
use crate::grid::Grid;
use crate::prelude::*;
use std::collections::BinaryHeap;
use std::fmt::Debug;
use std::rc::Rc;
use std::usize;

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

impl PartialEq for ScoredCell {
    fn eq(&self, other: &Self) -> bool {
        self.location() == other.location() && self.duration() == other.duration()
    }
}

impl Eq for ScoredCell {}

// Lowest cost has highest priority, then earliest departure, then earliest arrival, then we don't
// really care, so we just do by cell then by prev.
impl Ord for ScoredCell {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other
            .cost
            .cmp(&self.cost)
            .then_with(|| other.duration().1.cmp(&self.duration().1))
            .then_with(|| other.duration().0.cmp(&self.duration().0))
            .then_with(|| other.location().cmp(&self.location()))
            .then_with(|| other.prev.cmp(&self.prev))
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
            "({}, {}) × ({}, {})",
            self.location().origin.0,
            self.location().origin.1,
            self.duration().0,
            self.duration().1
        )
    }
}

// TODO: Can this be made more restrictive?
fn open_allows_candidate(candidate: &ScoredCell, open: &BinaryHeap<ScoredCell>) -> bool {
    for cell in open {
        // If candidate is cheaper than all remaining cells, we want to check this cell
        if candidate.cost < cell.cost {
            break;
        } else if cell.location() == candidate.location() && cell.duration() == candidate.duration()
        {
            return false;
        }
    }
    true
}

fn reconstruct_path(last: ScoredCell) -> Path {
    let mut path = Vec::with_capacity(last.duration().1 + 1);
    path.push(last.unit);
    let mut prev = Rc::new(last);
    while let Some(scored_cell) = &prev.prev {
        if prev.location() != scored_cell.location() {
            path.push(scored_cell.unit);
        }
        prev = scored_cell.clone();
    }
    path.reverse();
    path
}

#[derive(PartialEq, Eq)]
pub struct AStar {
    pub grid: Grid<CellInfo>,
    pub origins: Vec<Pair>,
    pub destinations: Vec<Pair>,
    pub unit_extent: Pair,
    pub heuristic: Grid<usize>,
}

impl AStar {
    //fn verify_destination_count(&self) {
    //    if self.destinations.len() < self.origins.len() {
    //        panic!("More origins than destinations!")
    //    }
    //}
    //
    //// TODO: actual verification
    //
    //fn verify_cells(&mut self, cells: &[Pair]) {
    //    for cell in cells {
    //        if self.grid.in_bounds(*cell) && self.grid.is_clear(*cell) {
    //            self.grid.set_blocked(*cell, true)
    //        } else {
    //            panic!("Cell {:?} is not clear!", cell)
    //        }
    //    }
    //    for cell in cells {
    //        self.grid.set_blocked(*cell, false)
    //    }
    //}

    fn generate_heuristic(&mut self) {
        for destination in &self.destinations {
            let distances = self.grid.djikstra(destination.extend(self.unit_extent));
            for (pair, cost) in distances.indexed_iter() {
                if *cost < self.heuristic[pair] {
                    self.heuristic[pair] = *cost
                }
            }
        }
    }

    //fn verify_connectivity(&self) {
    //    let distances = self.grid.djikstra(self.origins[0]);
    //    for origin in &self.origins {
    //        if distances[*origin.into()] == usize::MAX {
    //            panic!("Origin {:?} not reachable!", origin)
    //        }
    //    }
    //    for destination in &self.destinations {
    //        if distances[*destination.into()] == usize::MAX {
    //            panic!("Destination {:?} not reachable!", destination)
    //        }
    //    }
    //}

    pub fn new(
        origins: Vec<Pair>,
        destinations: Vec<Pair>,
        unit_extent: Pair,
        grid: Grid<CellInfo>,
    ) -> AStar {
        AStar {
            heuristic: Grid::init(grid.effective_extent(unit_extent), usize::MAX),
            unit_extent,
            grid,
            origins,
            destinations,
        }
    }

    pub fn init(
        origins: Vec<Pair>,
        destinations: Vec<Pair>,
        unit_extent: Pair,
        grid: Grid<CellInfo>,
    ) -> AStar {
        let mut out = AStar::new(origins, destinations, unit_extent, grid);
        //out.verify_destination_count();
        //out.verify_cells(&origins);
        //out.verify_cells(&destinations);
        //out.verify_connectivity();
        out.generate_heuristic();
        out
    }

    fn successors(&self, scored_cell: ScoredCell, constraints: &[Constraint]) -> Vec<ScoredCell> {
        let neighbors = self.grid.neighbors(scored_cell.location());
        let mut succ = Vec::with_capacity(neighbors.len() + 1);
        let unit = UnitState {
            uid: scored_cell.uid(),
            location: scored_cell.location(),
            duration: Pair(scored_cell.duration().0, scored_cell.duration().1 + 1),
        };
        let wait = ScoredCell {
            cost: scored_cell.cost + 1,
            unit,
            prev: scored_cell.prev.clone(),
        };
        if satisfies_constraints(wait.unit, constraints) {
            succ.push(wait);
        }
        let sc = Rc::new(scored_cell);
        for location in neighbors {
            let time = sc.duration().1 + self.grid.cost(location);
            let unit = UnitState {
                uid: sc.uid(),
                location,
                duration: Pair(sc.duration().1 + 1, time),
            };
            let candidate = ScoredCell {
                cost: time + self.heuristic[location.origin],
                unit,
                prev: Some(Rc::clone(&sc)),
            };
            if satisfies_constraints(candidate.unit, constraints) {
                succ.push(candidate);
            }
        }
        succ
    }

    pub fn astar(&self, start: Pair, constraints: &[Constraint]) -> Option<Path> {
        let initial = UnitState {
            uid: start,
            location: start.extend(self.unit_extent),
            duration: Pair(0, 0),
        };
        let my_constraints = adapt_constraints(initial, constraints);
        let Pair(x_extent, y_extent) = self.grid.effective_size(self.unit_extent);
        let mut open = BinaryHeap::with_capacity(x_extent * y_extent);
        open.push(ScoredCell {
            cost: 0,
            unit: initial,
            prev: None,
        });

        loop {
            let current = match open.pop() {
                None => {
                    return None;
                }
                Some(sc) => sc,
            };
            for successor in self.successors(current, &my_constraints) {
                if open_allows_candidate(&successor, &open) {
                    if self.destinations.contains(&successor.location().origin)
                        && may_stop(successor.unit, &my_constraints)
                    {
                        let path = reconstruct_path(successor);
                        return Some(path);
                    }
                    open.push(successor);
                }
            }
        }
    }
}
