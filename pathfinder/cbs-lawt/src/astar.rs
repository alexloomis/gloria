use crate::grid_ext::GridExt;
use crate::prelude::*;
use grid::Grid;
use std::collections::BinaryHeap;
use std::ops::Sub;
use std::rc::Rc;

fn filter_constraints(uid: Pair, constraints: &[Constraint]) -> Vec<Constraint> {
    let mut out = Vec::with_capacity(constraints.len());
    for constraint in constraints {
        if constraint.uid == uid {
            out.push(*constraint);
        }
    }
    out
}

// We assume the constraints have already been filtered by unit
fn satisfies_constraints(scored_cell: &ScoredCell, constraints: &[Constraint]) -> bool {
    for constraint in constraints {
        let relevant_cell = constraint.location.intersects(scored_cell.location);
        let relevant_time =
        // We haven't left before the constraint begins
        constraint.duration.0 <= scored_cell.duration.1
        &&
        // We didn't arrive after the constraint ended
        scored_cell.duration.0 <= constraint.duration.1;
        if relevant_cell && relevant_time {
            return false;
        }
    }
    true
}

fn open_allows_candidate(candidate: &ScoredCell, open: &BinaryHeap<ScoredCell>) -> bool {
    for cell in open {
        // If candidate is cheaper than all remaining cells, we want to check this cell
        if candidate.cost < cell.cost {
            break;
        } else if cell.location == candidate.location && cell.duration == candidate.duration {
            return false;
        }
    }
    true
}

fn may_stop(candidate: &ScoredCell, constraints: &[Constraint]) -> bool {
    for constraint in constraints {
        if constraint.location.intersects(candidate.location)
            && candidate.duration.0 <= constraint.duration.1
        {
            return false;
        }
    }
    true
}

fn reconstruct_path(last: ScoredCell) -> Path {
    let mut path = Vec::with_capacity(last.duration.1 + 1);
    path.push(last.clone());
    let mut prev = last;
    while let Some(scored_cell) = prev.prev {
        if prev.location != scored_cell.location {
            path.push(Rc::unwrap_or_clone(scored_cell.clone()));
        }
        prev = Rc::unwrap_or_clone(scored_cell.clone());
    }
    path.reverse();
    path
}

#[derive(PartialEq, Eq)]
pub struct AStar {
    pub grid: GridExt,
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
            for ((x, y), cost) in distances.indexed_iter() {
                if *cost < self.heuristic[(x, y)] {
                    self.heuristic[(x, y)] = *cost
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

    pub fn init(
        origins: Vec<Pair>,
        destinations: Vec<Pair>,
        unit_extent: Pair,
        grid: Grid<CellInfo>,
    ) -> AStar {
        let mut out = AStar {
            heuristic: Grid::init(
                grid.rows().sub(unit_extent.1),
                grid.cols().sub(unit_extent.0),
                usize::MAX,
            ),
            unit_extent,
            grid: GridExt::new(grid),
            origins,
            destinations,
        };
        //out.verify_destination_count();
        //out.verify_cells(&origins);
        //out.verify_cells(&destinations);
        //out.verify_connectivity();
        out.generate_heuristic();
        out
    }

    fn successors(&self, scored_cell: ScoredCell, constraints: &[Constraint]) -> Vec<ScoredCell> {
        let neighbors = self.grid.neighbors(scored_cell.location);
        let mut succ = Vec::with_capacity(neighbors.len() + 1);
        let wait = ScoredCell {
            cost: scored_cell.cost + 1,
            duration: Pair(scored_cell.duration.0, scored_cell.duration.1 + 1),
            location: scored_cell.location,
            prev: scored_cell.prev.clone(),
        };
        if satisfies_constraints(&wait, constraints) {
            succ.push(wait);
        }
        let sc = Rc::new(scored_cell);
        for location in neighbors {
            let time = sc.duration.1 + self.grid.cost(location);
            let candidate = ScoredCell {
                cost: time + self.heuristic[location.origin.into()],
                duration: Pair(sc.duration.1 + 1, time),
                location,
                prev: Some(Rc::clone(&sc)),
            };
            if satisfies_constraints(&candidate, constraints) {
                succ.push(candidate);
            }
        }
        succ
    }

    pub fn astar(&self, start: Pair, constraints: &[Constraint]) -> Option<Path> {
        let my_constraints = filter_constraints(start, constraints);
        let Pair(x_extent, y_extent) = self.grid.effective_size(self.unit_extent);
        let mut open = BinaryHeap::with_capacity(x_extent * y_extent);
        open.push(ScoredCell {
            cost: 0,
            duration: Pair(0, 0),
            location: start.extend(self.unit_extent),
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
                    if self.destinations.contains(&successor.location.origin)
                        && may_stop(&successor, &my_constraints)
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
