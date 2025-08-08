use std::collections::HashMap;

use proptest::bits::usize;

use crate::prelude::*;

#[derive(PartialEq, Eq, Clone, Copy)]
enum Status {
    Queued,
    // Wait of <= clock speed will be added to the queue when time passes
    Waiting(usize),
}

#[derive(PartialEq, Eq, Clone)]
pub struct UnitState {
    location: Pair,
    destination: Pair,
    status: Status,
    // Includes current location
    history: Vec<Pair>,
}

impl UnitState {
    pub fn init(origin: Pair) -> UnitState {
        UnitState {
            location: origin,
            destination: Pair(0, 0),
            status: Status::Queued,
            history: vec![origin],
        }
    }
}

pub struct UnitList {
    // Indexed by origin
    units: HashMap<Pair, UnitState>,
    queue: Vec<Pair>,
    waiting: Vec<Pair>,
}

// access
impl UnitList {
    pub fn units(&self) -> HashMap<Pair, UnitState> {
        self.units.clone()
    }

    pub fn queue(&self) -> Vec<Pair> {
        self.queue.clone()
    }

    pub fn done(&self) -> Vec<Pair> {
        self.waiting.clone()
    }
}

// logic
impl UnitList {
    fn yank(&mut self, uid: Pair) -> UnitState {
        self.queue.retain(|loc| *loc != uid);
        self.waiting.retain(|loc| *loc != uid);
        self.units.remove(&uid).unwrap()
    }

    pub fn enqueue(&mut self, uid: Pair) {
        let mut unit = self.yank(uid);
        unit.status = Status::Queued;
        self.units.insert(uid, unit);
        self.queue.push(uid);
    }

    pub fn move_unit(&mut self, uid: Pair, to: Pair, wait: usize) {
        let mut unit = self.yank(uid);
        unit.location = to;
        unit.history.push(to);
        unit.status = Status::Waiting(wait);
        self.units.insert(uid, unit);
        self.waiting.push(uid);
    }

    fn advance_time_unit(&mut self, uid: Pair, clock_speed: usize) {
        let mut unit = self.yank(uid);
        if let Status::Waiting(t) = unit.status {
            if t > clock_speed {
                unit.status = Status::Waiting(t - clock_speed);
                self.units.insert(uid, unit);
                self.waiting.push(uid);
            } else {
                unit.status = Status::Queued;
                self.units.insert(uid, unit);
                self.queue.push(uid);
            }
        }
    }

    pub fn advance_time(&mut self, clock_speed: usize) {
        let keys: Vec<Pair> = self.units.keys().cloned().collect();
        for uid in keys {
            self.advance_time_unit(uid, clock_speed);
        }
    }
}
