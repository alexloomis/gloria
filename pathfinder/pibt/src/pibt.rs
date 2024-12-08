#![allow(dead_code)]
#![allow(unused_variables)]
use core::panic;
use std::cmp::{max, min};
use std::collections::HashMap;
use std::vec;

use crate::prelude::*;

#[derive(PartialEq, Eq, Clone)]
struct UnitState {
    destination: Pair,
    // wait == 1 means this turn set wait == 0; wait == 0 means may move
    wait: usize,
    history: Vec<Pair>,
}

impl UnitState {
    fn init(origin: Pair) -> UnitState {
        UnitState {
            destination: Pair(0, 0),
            wait: 0,
            history: vec![origin],
        }
    }
}

pub struct PIBT {
    terrain: Terrain,
    units: HashMap<Pair, UnitState>,
    // indexed by destination
    heuristics: HashMap<Pair, Grid<CellCost>>,
    queue: Vec<Pair>,
    moved: Vec<Pair>,
}

// init
impl PIBT {
    fn unit_extent(&self) -> Pair {
        self.terrain.unit_extent()
    }

    fn init_units(&mut self, origins: Vec<Pair>) {
        self.units = HashMap::with_capacity(origins.len());
        for origin in origins {
            let unit = UnitState::init(origin);
            self.units.insert(origin, unit);
        }
    }

    fn origins(&self) -> Vec<Pair> {
        self.units.keys().copied().collect()
    }

    fn find_heuristics(&mut self, destinations: Vec<Pair>) {
        self.heuristics = HashMap::with_capacity(self.units.len());
        for destination in destinations {
            let heuristic = self.terrain.distances()[destination].clone();
            self.heuristics.insert(destination, heuristic);
        }
    }

    fn destinations(&self) -> Vec<Pair> {
        let mut out = Vec::with_capacity(self.units.len());
        for destination in self.heuristics.keys() {
            out.push(*destination);
        }
        out
    }

    fn heuristic(&self, origin: Pair, destination: Pair) -> CellCost {
        self.heuristics[&destination][origin]
    }

    fn is_connected(&self) -> bool {
        for origin in self.units.keys() {
            for destination in self.destinations() {
                if self.heuristic(*origin, destination).0.is_none() {
                    return false;
                }
            }
        }
        true
    }

    fn find_max_among(&self, origins: &[Pair], destinations: &[Pair]) -> (Pair, Pair) {
        let mut max_origin = origins[0];
        let mut max_destination = destinations[0];
        let mut max_val = self.heuristic(max_origin, max_destination);
        for origin in origins {
            for destination in destinations {
                let val = self.heuristic(*origin, *destination);
                if val > max_val {
                    max_origin = *origin;
                    max_destination = *destination;
                    max_val = val;
                }
            }
        }
        (max_origin, max_destination)
    }

    fn find_min_along(
        &self,
        origin: Pair,
        destination: Pair,
        origins: &[Pair],
        destinations: &[Pair],
    ) -> (Pair, Pair) {
        let mut min_origin = origin;
        let mut min_destination = destination;
        let mut min_val = self.heuristic(origin, destination);
        for x in origins {
            let val = self.heuristic(*x, destination);
            if val < min_val {
                min_val = val;
                min_origin = *x;
                min_destination = destination;
            }
        }
        for y in destinations {
            let val = self.heuristic(origin, *y);
            if val < min_val {
                min_val = val;
                min_origin = origin;
                min_destination = *y;
            }
        }
        (min_origin, min_destination)
    }

    // try to minimize makespan
    fn assign_destinations(&mut self) {
        let mut origins = self.origins();
        let mut destinations = self.destinations();
        while !origins.is_empty() {
            // Give either the worst origin or the worst destination its best choice
            let coord = self.find_max_among(&origins, &destinations);
            let (origin, destination) =
                self.find_min_along(coord.0, coord.1, &origins, &destinations);
            self.units
                .entry(origin)
                .and_modify(|unit| unit.destination = destination);
            origins.retain(|origin_| *origin_ != origin);
            destinations.retain(|destination_| *destination_ != destination);
        }
    }

    fn destination_map(&self) -> HashMap<Pair, Pair> {
        let mut out = HashMap::with_capacity(self.units.len());
        for (loc, unit) in &self.units {
            out.insert(*loc, unit.destination);
        }
        out
    }

    fn find_swap(&self) -> Option<(Pair, Pair)> {
        for (origin_i, destination_i) in self.destination_map() {
            for (origin_j, destination_j) in self.destination_map() {
                let d_ii = self.heuristic(origin_i, destination_i);
                let d_ij = self.heuristic(origin_i, destination_j);
                let d_ji = self.heuristic(origin_j, destination_i);
                let d_jj = self.heuristic(origin_j, destination_j);
                let better_max = max(d_ij, d_ji) < max(d_ii, d_jj);
                let same_max = max(d_ij, d_ji) == max(d_ii, d_jj);
                let better_min = min(d_ij, d_ji) < min(d_ii, d_jj);
                if better_max || (same_max && better_min) {
                    return Some((origin_i, origin_j));
                }
            }
        }
        None
    }

    fn perform_swap(&mut self, swap: (Pair, Pair)) {
        let new_destination_0 = self.units[&swap.1].destination;
        let new_destination_1 = self.units[&swap.0].destination;
        self.units
            .entry(swap.0)
            .and_modify(|unit| unit.destination = new_destination_0);
        self.units
            .entry(swap.1)
            .and_modify(|unit| unit.destination = new_destination_1);
    }

    // Running this every step allows us to rule out several types of collisions.
    // Much slower than managing the collisions, but simpler code.
    fn improve_assignments(&mut self) {
        while let Some(swap) = self.find_swap() {
            self.perform_swap(swap);
        }
    }

    fn dists_from_destination(&self) -> HashMap<Pair, CellCost> {
        let origins = self.units.keys();
        let mut out = HashMap::with_capacity(origins.len());
        for origin in origins {
            let destination = self.units[origin].destination;
            out.insert(*origin, self.heuristic(*origin, destination));
        }
        out
    }

    // Units the farthest away should start with the highest priority (front of the list)
    fn init_queue(&mut self) {
        let dists = self.dists_from_destination();
        self.queue = self.units.keys().copied().collect();
        self.queue.sort_unstable_by_key(|origin| dists[origin]);
        self.queue.reverse();
    }

    fn init_starts_empty(&mut self) {
        self.moved = Vec::with_capacity(self.units.len());
    }

    fn new(terrain: Terrain, unit_extent: Pair) -> PIBT {
        PIBT {
            terrain,
            units: HashMap::new(),
            heuristics: HashMap::new(),
            queue: Vec::new(),
            moved: Vec::new(),
        }
    }

    pub fn init(
        terrain: Terrain,
        origins: Vec<Pair>,
        destinations: Vec<Pair>,
        unit_extent: Pair,
    ) -> PIBT {
        println!("a");
        let mut pibt = PIBT::new(terrain, unit_extent);
        println!("b");
        pibt.init_units(origins);
        println!("c");
        pibt.find_heuristics(destinations);
        if !pibt.is_connected() {
            panic!("not conntected!");
        }
        println!("d");
        pibt.assign_destinations();
        println!("e");
        pibt.improve_assignments();
        println!("f");
        pibt.init_queue();
        println!("g");
        pibt.init_starts_empty();
        println!("h");
        pibt
    }
}

impl PIBT {
    fn intersects(&self, origin_0: Pair, origin_1: Pair) -> bool {
        origin_0
            .extend(self.unit_extent())
            .intersects(origin_1.extend(self.unit_extent()))
    }

    fn intersects_moved(&self, to: Pair) -> bool {
        self.moved.iter().any(|origin| self.intersects(to, *origin))
    }

    fn movement_targets(&self, from: Pair) -> Vec<Pair> {
        // Four directions of movement,plus stand still
        let mut neighbors = Vec::with_capacity(5);
        neighbors.push(from);
        neighbors.append(&mut self.terrain.neighbors(from));
        let mut out: Vec<Pair> = neighbors
            .into_iter()
            .filter(|to| !self.intersects_moved(*to))
            .collect();
        let destination = self.units[&from].destination;
        // TODO: Add max wait time of intersecting units to heuristic
        out.sort_unstable_by_key(|cell| self.heuristic(*cell, destination));
        out
    }

    fn queued_blocker(&self, to: Pair) -> Option<Pair> {
        self.queue
            .iter()
            .copied()
            .find(|origin| self.intersects(to, *origin))
    }

    fn move_unit(&mut self, from: Pair, to: Pair) {
        if let Some(mut unit) = self.units.remove(&from) {
            unit.history.push(to);
            self.units.insert(to, unit);
        }
    }

    fn try_to_move(&mut self) -> Option<Pair> {
        let from = self.queue.pop().unwrap();
        let to = self.movement_targets(from)[0];
        if let Some(blocker) = self.queued_blocker(to) {
            self.queue.push(from);
            self.queue.retain(|origin| *origin != blocker);
            self.queue.push(blocker);
            None
        } else {
            self.move_unit(from, to);
            self.moved.push(to);
            Some(from)
        }
    }

    fn move_all(&mut self) {
        let mut consecutive_failures = 0;
        while !self.queue.is_empty() {
            match self.try_to_move() {
                Some(_) => consecutive_failures = 0,
                None => {
                    consecutive_failures += 1;
                    if consecutive_failures >= self.queue.len() {
                        panic!("looping!");
                    }
                }
            }
        }
    }

    pub fn step(&mut self) {
        self.move_all();
        // empties self.moved
        self.queue.append(&mut self.moved);
        self.improve_assignments();
    }

    pub fn all_arrived(&self) -> bool {
        self.units
            .iter()
            .all(|(loc, unit)| unit.destination == *loc)
    }

    pub fn terrain(&self) -> &Terrain {
        &self.terrain
    }

    pub fn paths(&self) -> Vec<Vec<Pair>> {
        let mut out = Vec::with_capacity(self.units.len());
        for unit in self.units.values() {
            out.push(unit.history.clone());
        }
        out
    }

    pub fn unit_dests(&self) -> Vec<(Pair, Pair)> {
        self.units
            .iter()
            .map(|(loc, unit)| (*loc, unit.destination))
            .collect()
    }

    pub fn run(&mut self, max_steps: Option<usize>) {
        let mut num_steps = 0;
        while !self.all_arrived() {
            if max_steps.is_some_and(|m| m == num_steps) {
                break;
            }
            self.step();
            num_steps += 1;
        }
    }
}
