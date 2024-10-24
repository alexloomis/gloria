use crate::prelude::*;
use core::fmt::Debug;

// Constraint means that the unit may not collide with the region
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Constraint {
    Occupy(UnitState),
    Avoid(UnitState),
}

impl Constraint {
    pub fn uid(self) -> Pair {
        match self {
            Self::Occupy(state) => state.uid,
            Self::Avoid(state) => state.uid,
        }
    }

    pub fn location(self) -> Rect {
        match self {
            Self::Occupy(state) => state.location,
            Self::Avoid(state) => state.location,
        }
    }

    pub fn duration(self) -> Pair {
        match self {
            Self::Occupy(state) => state.duration,
            Self::Avoid(state) => state.duration,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Conflict(pub UnitState, pub UnitState);

impl Conflict {
    pub fn uids(self) -> (Pair, Pair) {
        (self.0.uid, self.1.uid)
    }
}

impl Conflict {
    pub fn constraints(self) -> [Constraint; 2] {
        [Constraint::Occupy(self.0), Constraint::Avoid(self.0)]
    }
}

// Following functions assume the constraints have already been adapted, so do not verify UIDs
pub fn adapt_constraints(unit: UnitState, constraints: &[Constraint]) -> Vec<Constraint> {
    let mut out = Vec::with_capacity(constraints.len());
    for constraint in constraints {
        if unit.uid == constraint.uid() {
            out.push(*constraint);
        } else if let Constraint::Occupy(state) = constraint {
            out.push(Constraint::Avoid(*state));
        }
    }
    out
}

fn violates_constraint(unit: UnitState, constraint: Constraint) -> bool {
    match constraint {
        Constraint::Avoid(state) => {
            let relevant_cell = unit.location.intersects(state.location);
            let relevant_time = unit.duration.intersects(state.duration);
            relevant_cell && relevant_time
        }
        Constraint::Occupy(state) => {
            let different_location = unit.location != state.location;
            let relevant_time = unit.duration.intersects(state.duration);
            different_location && relevant_time
        }
    }
}

pub fn satisfies_constraints(unit: UnitState, constraints: &[Constraint]) -> bool {
    !constraints
        .iter()
        .any(|constraint| violates_constraint(unit, *constraint))
}

fn blocks_stop(unit: UnitState, constraint: Constraint) -> bool {
    match constraint {
        Constraint::Avoid(state) => {
            unit.location.intersects(state.location) && unit.duration.0 <= state.duration.1
        }
        Constraint::Occupy(state) => {
            unit.location != state.location && unit.duration.0 <= state.duration.1
        }
    }
}

pub fn may_stop(unit: UnitState, constraints: &[Constraint]) -> bool {
    !constraints
        .iter()
        .any(|constraint| blocks_stop(unit, *constraint))
}
