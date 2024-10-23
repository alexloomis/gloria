use std::cmp::max;
use std::rc::Rc;
use std::usize;

use crate::grid::Grid;
use crate::prelude::*;

#[derive(PartialEq, Eq)]
struct UnitState {
    idx: usize,
    location: Rect,
    // wait == 1 means this turn set wait == 0, next turn move
    wait: usize,
    _history: Vec<Pair>,
}

#[derive(PartialEq, Eq)]
struct PIBTState {
    this_queue: Vec<Rc<UnitState>>,
    next_queue: Vec<Rc<UnitState>>,
}

#[derive(PartialEq, Eq)]
pub struct PIBT {
    grid: Grid<CellInfo>,
    origins: Vec<Pair>,
    destinations: Vec<Pair>,
    unit_extent: Pair,
    heuristics: Vec<Grid<usize>>,
    state: PIBTState,
}

// init
impl PIBT {
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

    fn find_heuristics(&mut self) {
        self.heuristics = Vec::with_capacity(self.destinations.len());
        for destination in &self.destinations {
            let heuristic = self.grid.djikstra(Rect {
                origin: *destination,
                extent: self.unit_extent,
            });
            self.heuristics.push(heuristic);
        }
    }

    // assignment[i] = j meanse move origin i to index j
    fn update_origins(&mut self, assignments: &[usize]) {
        let mut origins = vec![Pair(usize::MAX, usize::MAX); self.origins.len()];
        for (old_idx, new_idx) in assignments.into_iter().enumerate() {
            origins[*new_idx] = self.origins[old_idx]
        }
        self.origins = origins;
    }

    // Tries to minimize makespan
    fn assign_destinations(&mut self) {
        let mut distances = Grid::init(
            Pair(self.origins.len() - 1, self.destinations.len() - 1),
            usize::MAX,
        );
        for (Pair(origin_idx, dest_idx), value) in distances.indexed_iter_mut() {
            *value = self.heuristics[dest_idx][self.origins[origin_idx]];
        }

        let mut unassigned_origins: Vec<usize> = (0..self.origins.len()).collect();
        let mut unassigned_dests: Vec<usize> = (0..self.destinations.len()).collect();
        let mut assignments = vec![usize::MAX; self.origins.len()];

        while !unassigned_origins.is_empty() {
            // Give either the worst origin or the worst destination its best choice
            let coord = PIBT::find_max_among(&distances, &unassigned_origins, &unassigned_dests);
            let coord =
                PIBT::find_min_along(&distances, coord, &unassigned_origins, &unassigned_dests);
            assignments[coord.0] = coord.1;
            unassigned_origins.retain(|origin| *origin != coord.0);
            unassigned_dests.retain(|destination| *destination != coord.1);
        }
        self.update_origins(&assignments);
    }

    fn perform_swap(distances: &Grid<usize>, assignments: &mut [usize]) -> Option<Pair> {
        for i in 0..assignments.len() {
            for j in 0..i {
                let improve_i =
                    distances[Pair(assignments[j], i)] < distances[Pair(assignments[i], i)];
                let improve_j =
                    distances[Pair(assignments[i], j)] < distances[Pair(assignments[j], j)];
                let worsen_i =
                    distances[Pair(assignments[j], i)] > distances[Pair(assignments[i], i)];
                let worsen_j =
                    distances[Pair(assignments[i], j)] > distances[Pair(assignments[j], j)];
                if (improve_i && !worsen_j) || (improve_j && !worsen_i) {
                    let new_i = assignments[j];
                    let new_j = assignments[i];
                    assignments[i] = new_i;
                    assignments[j] = new_j;
                    return Some(Pair(i, j));
                }
            }
        }
        None
    }

    fn improve_assignments(&mut self) {
        let mut assignments: Vec<usize> = (0..self.origins.len()).collect();
        let mut distances = Grid::init(
            Pair(self.origins.len() - 1, self.destinations.len() - 1),
            usize::MAX,
        );
        for (Pair(origin_idx, dest_idx), value) in distances.indexed_iter_mut() {
            *value = self.heuristics[dest_idx][self.origins[origin_idx]];
        }
        loop {
            if PIBT::perform_swap(&distances, &mut assignments).is_none() {
                break;
            }
        }
        self.update_origins(&assignments);
    }

    fn new(
        grid: Grid<CellInfo>,
        origins: Vec<Pair>,
        destinations: Vec<Pair>,
        unit_extent: Pair,
    ) -> PIBT {
        PIBT {
            grid,
            origins,
            destinations,
            unit_extent,
            heuristics: Vec::new(),
            state: PIBTState {
                this_queue: Vec::new(),
                next_queue: Vec::new(),
            },
        }
    }

    pub fn init(
        grid: Grid<CellInfo>,
        origins: Vec<Pair>,
        destinations: Vec<Pair>,
        unit_extent: Pair,
    ) -> PIBT {
        let mut pibt = PIBT::new(grid, origins, destinations, unit_extent);
        pibt.find_heuristics();
        pibt.assign_destinations();
        pibt.improve_assignments();
        pibt
    }
}

enum BlockStatus {
    Clear,
    Wait(usize),
    HighPrio,
    LowPrio(Vec<Rc<UnitState>>),
    Stuck,
}

enum PushStatus {
    Clear,
    Wait(usize),
    HighPrio,
    Stuck,
}

fn wait_time(states: &[Rc<UnitState>]) -> usize {
    states.iter().map(|state| state.wait).max().unwrap_or(0)
}

// Unit movement
impl PIBT {
    fn movement_targets(&self, location: Rect, allow_stationary: bool) -> Vec<Rect> {
        todo!()
    }

    fn collisions(&self, location: Rect) -> [Vec<Rc<UnitState>>; 2] {
        let mut high_prio = Vec::with_capacity(self.state.next_queue.len());
        let mut low_prio = Vec::with_capacity(self.state.this_queue.len());
        for unit in &self.state.next_queue {
            if location.intersects(unit.location) {
                high_prio.push(unit.clone());
            }
        }
        for unit in &self.state.this_queue {
            if location.intersects(unit.location) {
                low_prio.push(unit.clone());
            }
        }
        [high_prio, low_prio]
    }

    fn block_status(&self, location: Rect) -> BlockStatus {
        let collisions = self.collisions(location);
        let max_wait = max(wait_time(&collisions[0]), wait_time(&collisions[1]));
        if max_wait > 0 {
            return BlockStatus::Wait(max_wait);
        }
        let [high_prio, low_prio] = collisions;
        if !high_prio.is_empty() {
            return BlockStatus::HighPrio;
        } else if !low_prio.is_empty() {
            return BlockStatus::LowPrio(low_prio);
        } else {
            return BlockStatus::Clear;
        }
    }

    fn move_unit(&mut self, unit: UnitState) {}

    fn take_action(&mut self, unit: UnitState, target: Rect) {
        match self.block_status(target) {
            BlockStatus::Clear => self.move_unit(unit),
            BlockStatus::Wait(time) => todo!(),
            BlockStatus::HighPrio => todo!(),
            BlockStatus::LowPrio(list) => todo!(),
            BlockStatus::Stuck => todo!(),
        }
    }

    fn init_units(&self) -> Vec<UnitState> {
        let mut units = Vec::with_capacity(self.origins.len());
        for (idx, origin) in self.origins.iter().enumerate() {
            let unit = UnitState {
                idx,
                location: Rect {
                    origin: *origin,
                    extent: self.unit_extent,
                },
                wait: 0,
                _history: Vec::new(),
            };
            units.push(unit);
        }
        // Units the farthest away should start with the highest priority (front of the list)
        units.sort_unstable_by_key(|unit| self.heuristics[unit.idx][unit.location.origin]);
        units.reverse();
        units
    }

    pub fn pibt(&self) {
        let mut units = self.init_units();
        let mut done = false;
        while !done {
            done = true;
            for unit in &units {
                if unit.location.origin == self.destinations[unit.idx] {
                    continue;
                }
                done = false;
                // Move unit
            }
        }
    }
}
