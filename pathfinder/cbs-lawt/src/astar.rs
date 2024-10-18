use crate::grid::Grid;
use crate::prelude::*;
use std::cmp::min;
use std::collections::{BinaryHeap, HashMap};
use std::rc::Rc;
use std::usize;

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

    fn find_max_among(grid: &Grid<usize>, xs: &[usize], ys: &[usize]) -> Pair {
        let mut max_x = xs[0];
        let mut max_y = ys[0];
        let mut max_val = grid[Pair(max_x, max_y)];
        for x in xs {
            for y in ys {
                let val = grid[Pair(*x, *y)];
                if val > max_val {
                    max_x = *x;
                    max_y = *y;
                    max_val = val;
                }
            }
        }
        Pair(max_x, max_y)
    }

    fn find_min_along(grid: &Grid<usize>, target: Pair, xs: &[usize], ys: &[usize]) -> Pair {
        let mut min_x = target.0;
        let mut min_y = target.1;
        let mut min_val = grid[target];
        for x in xs {
            let val = grid[Pair(*x, target.1)];
            if val < min_val {
                min_val = val;
                min_x = *x;
                min_y = target.1;
            }
        }
        for y in ys {
            let val = grid[Pair(target.0, *y)];
            if val < min_val {
                min_val = val;
                min_x = target.0;
                min_y = *y
            }
        }
        Pair(min_x, min_y)
    }

    // Tries to minimize makespan
    pub fn assign_destinations(&mut self) {
        let mut dest_heuristics = HashMap::with_capacity(self.destinations.len());
        for destination in self.destinations.clone() {
            let heuristic = self.grid.djikstra(Rect {
                origin: destination,
                extent: self.unit_extent,
            });
            dest_heuristics.insert(destination, heuristic);
        }

        let mut distances = Grid::init(
            Pair(self.origins.len() - 1, self.destinations.len() - 1),
            usize::MAX,
        );
        for (Pair(origin_idx, dest_idx), value) in distances.indexed_iter_mut() {
            *value = dest_heuristics[&self.destinations[dest_idx]][self.origins[origin_idx]];
        }

        let mut unassigned_origins: Vec<usize> = (0..self.origins.len()).collect();
        let mut unassigned_dests: Vec<usize> = (0..self.destinations.len()).collect();
        let mut assignments = vec![usize::MAX; self.origins.len()];

        while !unassigned_origins.is_empty() {
            // Give either the worst origin or the worst destination its best choice
            let coord = AStar::find_max_among(&distances, &unassigned_origins, &unassigned_dests);
            let coord =
                AStar::find_min_along(&distances, coord, &unassigned_origins, &unassigned_dests);
            assignments[coord.0] = coord.1;
            unassigned_origins.retain(|origin| *origin != coord.0);
            unassigned_dests.retain(|destination| *destination != coord.0);
        }

        // TODO: see if can swap to better assignment, to find local optimum
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
                cost: time + self.heuristic[location.origin],
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
