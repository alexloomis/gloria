#![allow(dead_code)]
#![allow(unused_variables)]
use std::cmp::max;
use std::collections::HashMap;
use std::rc::Rc;

use crate::prelude::*;

#[derive(PartialEq, Eq)]
struct UnitState {
    location: Rect,
    target: Pair,
    // wait == 1 means this turn set wait == 0; wait == 0 means may move
    wait: usize,
    has_moved: bool,
    _history: Vec<Pair>,
}

impl UnitState {
    fn init(origin: Pair, extent: Pair) -> UnitState {
        UnitState {
            location: Rect { origin, extent },
            target: Pair(0, 0),
            wait: 0,
            has_moved: false,
            _history: Vec::new(),
        }
    }
}

pub struct PIBT {
    terrain: Terrain,
    units: HashMap<Pair, UnitState>,
    // indexed by destination
    heuristics: HashMap<Pair, Grid<CellCost>>,
    queue: Vec<Pair>,
}

// init
impl PIBT {
    fn unit_extent(&self) -> Pair {
        self.terrain.unit_extent()
    }

    fn init_units(&mut self, origins: Vec<Pair>) {
        self.units = HashMap::with_capacity(origins.len());
        for origin in origins {
            let unit = UnitState::init(origin, self.unit_extent());
            self.units.insert(origin, unit);
        }
    }

    fn origins(&self) -> Vec<Pair> {
        let mut out = Vec::with_capacity(self.units.len());
        for unit in self.units.values() {
            out.push(unit.location.origin);
        }
        out
    }

    fn find_heuristics(&mut self, targets: Vec<Pair>) {
        self.heuristics = HashMap::with_capacity(self.units.len());
        for target in targets {
            let heuristic = self.terrain.djikstra(target);
            self.heuristics.insert(target, heuristic);
        }
    }

    fn targets(&self) -> Vec<Pair> {
        let mut out = Vec::with_capacity(self.units.len());
        for target in self.heuristics.keys() {
            out.push(*target);
        }
        out
    }

    fn heuristic(&self, origin: Pair, target: Pair) -> CellCost {
        self.heuristics[&target][origin]
    }

    fn find_max_among(&self, origins: &[Pair], targets: &[Pair]) -> (Pair, Pair) {
        let mut max_origin = origins[0];
        let mut max_target = targets[0];
        let mut max_val = self.heuristic(max_origin, max_target);
        for origin in origins {
            for target in targets {
                let val = self.heuristic(*origin, *target);
                if val > max_val {
                    max_origin = *origin;
                    max_target = *target;
                    max_val = val;
                }
            }
        }
        (max_origin, max_target)
    }

    fn find_min_along(
        &self,
        origin: Pair,
        target: Pair,
        origins: &[Pair],
        targets: &[Pair],
    ) -> (Pair, Pair) {
        let mut min_origin = origin;
        let mut min_target = target;
        let mut min_val = self.heuristic(origin, target);
        for x in origins {
            let val = self.heuristic(*x, target);
            if val < min_val {
                min_val = val;
                min_origin = *x;
                min_target = target;
            }
        }
        for y in targets {
            let val = self.heuristic(origin, *y);
            if val < min_val {
                min_val = val;
                min_origin = origin;
                min_target = *y;
            }
        }
        (min_origin, min_target)
    }

    // try to minimize makespan
    fn assign_targets(&mut self) {
        let mut origins = self.origins();
        let mut targets = self.targets();
        while !origins.is_empty() {
            // Give either the worst origin or the worst destination its best choice
            let coord = self.find_max_among(&origins, &targets);
            let (origin, target) = self.find_min_along(coord.0, coord.1, &origins, &targets);
            self.units
                .entry(origin)
                .and_modify(|unit| unit.target = target);
            origins.retain(|origin_| *origin_ != origin);
            targets.retain(|destination| *destination != target);
        }
    }

    fn target_map(&self) -> HashMap<Pair, Pair> {
        let mut out = HashMap::with_capacity(self.units.len());
        for (loc, unit) in &self.units {
            out.insert(*loc, unit.target);
        }
        out
    }

    fn find_swap(&self) -> Option<(Pair, Pair)> {
        for (origin_i, target_i) in self.target_map() {
            for (origin_j, target_j) in self.target_map() {
                let improve_i =
                    self.heuristic(origin_i, target_j) < self.heuristic(origin_i, target_i);
                let improve_j =
                    self.heuristic(origin_j, target_i) < self.heuristic(origin_j, target_j);
                let worsen_i =
                    self.heuristic(origin_i, target_j) > self.heuristic(origin_i, target_i);
                let worsen_j =
                    self.heuristic(origin_j, target_i) > self.heuristic(origin_j, target_j);
                if (improve_i && !worsen_j) || (improve_j && !worsen_i) {
                    return Some((origin_i, origin_j));
                }
            }
        }
        None
    }

    fn perform_swap(&mut self, swap: (Pair, Pair)) {
        let new_target_0 = self.units[&swap.1].target;
        let new_target_1 = self.units[&swap.0].target;
        self.units
            .entry(swap.0)
            .and_modify(|unit| unit.target = new_target_0);
        self.units
            .entry(swap.1)
            .and_modify(|unit| unit.target = new_target_1);
    }

    fn improve_assignments(&mut self) {
        while let Some(swap) = self.find_swap() {
            self.perform_swap(swap);
        }
    }

    fn dists_from_target(&self) -> HashMap<Pair, CellCost> {
        let origins = self.units.keys();
        let mut out = HashMap::with_capacity(origins.len());
        for origin in origins {
            let target = self.units[&origin].target;
            out.insert(*origin, self.heuristic(*origin, target));
        }
        out
    }

    // Units the farthest away should start with the highest priority (front of the list)
    fn init_queue(&mut self) {
        let dists = self.dists_from_target();
        self.queue = self.units.keys().into_iter().map(|k| *k).collect();
        self.queue.sort_unstable_by_key(|origin| dists[origin]);
        self.queue.reverse();
    }
    //}

    fn new(terrain: Terrain, unit_extent: Pair) -> PIBT {
        PIBT {
            terrain,
            heuristics: HashMap::new(),
            queue: Vec::new(),
            units: HashMap::new(),
        }
    }

    pub fn init(
        terrain: Terrain,
        origins: Vec<Pair>,
        targets: Vec<Pair>,
        unit_extent: Pair,
    ) -> PIBT {
        let mut pibt = PIBT::new(terrain, unit_extent);
        pibt.init_units(origins);
        pibt.find_heuristics(targets);
        pibt.assign_targets();
        pibt.improve_assignments();
        pibt.init_queue();
        pibt
    }
}

enum MoveStatus {
    Moved,
    Waiting(usize),
    Blocked,
}

impl PIBT {
    fn movement_targets(&self, uid: Pair) -> Vec<Rect> {
        let mut out = self.terrain.neighbors(cell);
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
    fn collisions(&self, location: Rect) -> [Vec<Rc<UnitState>>; 2] {
        let mut high_prio = Vec::with_capacity(self.units.moved.len());
        let mut low_prio = Vec::with_capacity(self.units.pending.len());
        for unit in &self.units.moved {
            if location.intersects(unit.location) {
                high_prio.push(unit.clone());
            }
        }
        for unit in &self.units.pending {
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
