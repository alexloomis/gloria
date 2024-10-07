use crate::grid_ext::GridExt;
use crate::prelude::*;
use grid::Grid;
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::ops::Sub;
use std::rc::Rc;

fn filter_constraints(start: Pair, constraints: &[Constraint]) -> Vec<Constraint> {
    let mut out = Vec::with_capacity(constraints.len());
    for constraint in constraints {
        if constraint.uid == start {
            out.push(*constraint);
        }
    }
    out
}

// We assume the constraints have already been filtered
fn check_constraints(
    scored_cell: &ScoredCell,
    prev_departure: usize,
    constraints: &[Constraint],
) -> bool {
    for constraint in constraints {
        let relevant_cell = scored_cell.cell == constraint.cell;
        let relevant_time =
        // We havent left before the constraint begins
        constraint.duration.0 <= scored_cell.time
        &&
        // We didn't arrive after the constraint ended
        prev_departure < constraint.duration.1;
        if relevant_cell && relevant_time {
            return false;
        }
    }
    true
}

fn check_against(candidate: &ScoredCell, heap: &BinaryHeap<Reverse<ScoredCell>>) -> bool {
    for cell in heap {
        if cell.0.cost >= candidate.cost {
            break;
        } else if cell.0.cell == candidate.cell && cell.0.time == candidate.time {
            return false;
        }
    }
    true
}

fn check_against_rc(candidate: &ScoredCell, heap: &BinaryHeap<Reverse<Rc<ScoredCell>>>) -> bool {
    for cell in heap {
        if cell.0.cost >= candidate.cost {
            break;
        } else if cell.0.cell == candidate.cell && cell.0.time == candidate.time {
            return false;
        }
    }
    true
}

fn may_stop(candidate: &ScoredCell, constraints: &[Constraint]) -> bool {
    for constraint in constraints {
        if candidate.cell == constraint.cell && candidate.time <= constraint.duration.1 {
            return false;
        }
    }
    true
}

fn reconstruct_path(last: ScoredCell) -> Vec<ScoredCell> {
    let mut path = Vec::with_capacity(last.time + 1);
    path.push(last.clone());
    let mut prev = last;
    while let Some(scored_cell) = prev.prev {
        if prev.cell != scored_cell.cell {
            path.push(Rc::unwrap_or_clone(scored_cell.clone()));
        }
        prev = Rc::unwrap_or_clone(scored_cell.clone());
    }
    path.reverse();
    path
}

#[derive(PartialEq, Eq)]
pub struct MAPF {
    pub grid: GridExt,
    pub unit_size: Pair,
    pub origins: Vec<Pair>,
    pub destinations: Vec<Pair>,
    pub heuristic: Grid<usize>,
}

impl MAPF {
    fn verify_destination_count(&self) {
        if self.destinations.len() < self.origins.len() {
            panic!("More origins than destinations!")
        }
    }

    fn verify_cells(&mut self, cells: &[Pair]) {
        for cell in cells {
            if self.grid.in_bounds(*cell) && self.grid.is_clear(*cell) {
                self.grid.set_blocked(*cell, true)
            } else {
                panic!("Cell {:?} is not clear!", cell)
            }
        }
        for cell in cells {
            self.grid.set_blocked(*cell, false)
        }
    }

    fn generate_heuristic(&mut self) {
        for destination in &self.destinations {
            let distances = self.grid.djikstra(*destination);
            for ((x, y), cost) in distances.indexed_iter() {
                if *cost < self.heuristic[(x, y)] {
                    self.heuristic[(x, y)] = *cost
                }
            }
        }
    }

    fn verify_connectivity(&self) {
        let distances = self.grid.djikstra(self.origins[0]);
        for origin in &self.origins {
            if distances[*origin] == usize::MAX {
                panic!("Origin {:?} not reachable!", origin)
            }
        }
        for destination in &self.destinations {
            if distances[*destination] == usize::MAX {
                panic!("Destination {:?} not reachable!", destination)
            }
        }
    }

    pub fn init(
        origins: Vec<Pair>,
        destinations: Vec<Pair>,
        unit_size: Pair,
        grid: Grid<CellInfo>,
    ) -> MAPF {
        let mut out = MAPF {
            heuristic: Grid::init(
                grid.rows().sub(unit_size.1) + 1,
                grid.cols().sub(unit_size.0) + 1,
                usize::MAX,
            ),
            grid: GridExt::new(grid, unit_size),
            unit_size,
            origins: origins.clone(),
            destinations: destinations.clone(),
        };
        out.verify_destination_count();
        out.verify_cells(&origins);
        out.verify_cells(&destinations);
        out.verify_connectivity();
        out.generate_heuristic();
        out
    }

    fn successors(
        &self,
        scored_cell: Rc<ScoredCell>,
        constraints: &[Constraint],
    ) -> Vec<ScoredCell> {
        let neighbors = self.grid.neighbors(scored_cell.cell);
        let mut succ = Vec::with_capacity(neighbors.len() + 1);
        let wait = ScoredCell {
            cost: scored_cell.cost + 1,
            time: scored_cell.time + 1,
            cell: scored_cell.cell,
            prev: Some(Rc::clone(&scored_cell)),
        };
        if check_constraints(&wait, scored_cell.time, constraints) {
            succ.push(wait);
        }
        for neighbor in neighbors {
            let time = scored_cell.time + self.grid.cost(neighbor);
            let candidate = ScoredCell {
                cost: time + self.heuristic[neighbor],
                time,
                cell: neighbor,
                prev: Some(Rc::clone(&scored_cell)),
            };
            if check_constraints(&candidate, scored_cell.time, constraints) {
                succ.push(candidate);
            }
        }
        succ
    }

    pub fn astar(&self, start: Pair, constraints: &[Constraint]) -> Vec<ScoredCell> {
        let my_constraints = filter_constraints(start, constraints);
        let (x_extent, y_extent) = self.grid.extent();
        let mut closed = BinaryHeap::with_capacity(x_extent * y_extent);
        let mut open = BinaryHeap::with_capacity(x_extent * y_extent);
        open.push(Reverse(ScoredCell {
            cost: 0,
            time: 0,
            cell: start,
            prev: None,
        }));
        loop {
            let current = match open.pop() {
                None => {
                    return Vec::new();
                }
                Some(Reverse(sc)) => Rc::new(sc),
            };
            closed.push(Reverse(Rc::clone(&current)));
            for successor in self.successors(current, &my_constraints) {
                if check_against(&successor, &open) && check_against_rc(&successor, &closed) {
                    if self.destinations.contains(&successor.cell)
                        && may_stop(&successor, &my_constraints)
                    {
                        return reconstruct_path(successor);
                    }
                    open.push(Reverse(successor));
                }
            }
        }
    }
}
