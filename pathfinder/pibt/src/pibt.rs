#![allow(dead_code)]
#![allow(unused_variables)]
use std::cmp::max;
use std::collections::HashMap;
use std::rc::Rc;

use crate::prelude::*;

#[derive(Eq, PartialEq, Debug, Clone, Copy)]
struct CellCost(Option<usize>);

impl Ord for CellCost {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self == other {
            std::cmp::Ordering::Equal
        } else if self.0.is_none() {
            std::cmp::Ordering::Greater
        } else if other.0.is_none() {
            std::cmp::Ordering::Less
        } else {
            self.cmp(other)
        }
    }
}

impl PartialOrd for CellCost {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(PartialEq, Eq)]
struct UnitState {
    uid: Pair,
    location: Rect,
    // wait == 1 means this turn set wait == 0; wait == 0 means may move
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
    grid: Grid<CellCost>,
    unit_extent: Pair,
    // from idx_0 to idx_1 the distance is value
    distances: HashMap<Pair, HashMap<Pair, CellCost>>,
    // maps from uid to target
    targets: HashMap<Pair, Pair>,
    // indexed by destination
    heuristics: HashMap<Pair, Grid<CellCost>>,
    state: PIBTState,
}

// init
impl PIBT {
    fn find_heuristics(&mut self) {
        self.heuristics = HashMap::with_capacity(self.targets.len());
        for destination in self.targets.values() {
            let heuristic = self.grid.djikstra(Rect {
                origin: *destination,
                extent: self.unit_extent,
            });
            self.heuristics.insert(*destination, heuristic);
        }
    }

    fn find_max_among(&self, origins: &[Pair], destinations: &[Pair]) -> (Pair, Pair) {
        let mut max_x = origins[0];
        let mut max_y = destinations[0];
        let mut max_val = self.distances[&max_x][&max_y];
        for x in origins {
            for y in destinations {
                let val = self.distances[x][y];
                if val > max_val {
                    max_x = *x;
                    max_y = *y;
                    max_val = val;
                }
            }
        }
        (max_x, max_y)
    }

    fn find_min_along(
        &self,
        one_of: (Pair, Pair),
        origins: &[Pair],
        destinations: &[Pair],
    ) -> (Pair, Pair) {
        let mut min_x = one_of.0;
        let mut min_y = one_of.1;
        let mut min_val = self.distances[&one_of.0][&one_of.1];
        for x in origins {
            let val = self.distances[x][&one_of.1];
            if val < min_val {
                min_val = val;
                min_x = *x;
                min_y = one_of.1;
            }
        }
        for y in destinations {
            let val = self.distances[&one_of.0][y];
            if val < min_val {
                min_val = val;
                min_x = one_of.0;
                min_y = *y
            }
        }
        (min_x, min_y)
    }

    // try to minimize makespan
    fn assign_targets(&mut self, origins: Vec<Pair>, destinations: Vec<Pair>) {
        self.targets.clear();
        let mut unassigned_origins = origins.clone();
        let mut unassigned_dests = destinations.clone();

        while !unassigned_origins.is_empty() {
            // Give either the worst origin or the worst destination its best choice
            let coord = self.find_max_among(&unassigned_origins, &unassigned_dests);
            let (origin, target) =
                self.find_min_along(coord, &unassigned_origins, &unassigned_dests);
            self.targets.insert(origin, target);
            unassigned_origins.retain(|origin_| *origin_ != origin);
            unassigned_dests.retain(|destination| *destination != target);
        }
    }

    fn find_swap(&self) -> Option<(Pair, Pair)> {
        for (origin_i, target_i) in &self.targets {
            for (origin_j, target_j) in &self.targets {
                let improve_i =
                    self.distances[origin_i][target_j] < self.distances[origin_i][target_i];
                let improve_j =
                    self.distances[origin_j][target_i] < self.distances[origin_j][target_j];
                let worsen_i =
                    self.distances[origin_i][target_j] > self.distances[origin_i][target_i];
                let worsen_j =
                    self.distances[origin_j][target_i] > self.distances[origin_j][target_j];
                if (improve_i && !worsen_j) || (improve_j && !worsen_i) {
                    return Some((*origin_i, *origin_j));
                }
            }
        }
        None
    }

    fn perform_swap(&mut self, swap: (Pair, Pair)) {
        let new_target_0 = self.targets[&swap.1];
        let new_target_1 = self.targets.insert(swap.0, new_target_0).unwrap();
        self.targets.insert(swap.1, new_target_1);
    }

    fn improve_assignments(&mut self) {
        while let Some(swap) = self.find_swap() {
            self.perform_swap(swap);
        }
    }

    fn new(grid: Grid<CellCost>, unit_extent: Pair) -> PIBT {
        PIBT {
            grid,
            unit_extent,
            distances: HashMap::new(),
            targets: HashMap::new(),
            heuristics: HashMap::new(),
            state: PIBTState {
                this_queue: Vec::new(),
                next_queue: Vec::new(),
            },
        }
    }

    pub fn init(
        grid: Grid<CellCost>,
        origins: Vec<Pair>,
        destinations: Vec<Pair>,
        unit_extent: Pair,
    ) -> PIBT {
        let mut pibt = PIBT::new(grid, unit_extent);
        pibt.find_heuristics();
        pibt.assign_targets(origins, destinations);
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
            BlockStatus::HighPrio
        } else if !low_prio.is_empty() {
            BlockStatus::LowPrio(low_prio)
        } else {
            BlockStatus::Clear
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
        let units = self.init_units();
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
