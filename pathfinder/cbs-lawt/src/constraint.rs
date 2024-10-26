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

pub struct Specification {
    pub uid: Pair,
    pub start_cell: Pair,
    pub start_time: usize,
    pub end_cell: Option<Pair>,
    pub end_time: Option<usize>,
    pub constraints: Vec<Constraint>,
}

impl Specification {
    fn new(uid: Pair) -> Specification {
        Specification {
            uid,
            start_cell: uid,
            start_time: 0,
            end_cell: None,
            end_time: None,
            constraints: Vec::new(),
        }
    }

    pub fn init(new_constraint: Constraint, constraints: &[Constraint]) -> Specification {
        let spec = spec_bounds(new_constraint, constraints);
        adapt_const(spec, constraints)
    }
}

fn spec_bounds(new_constraint: Constraint, constraints: &[Constraint]) -> Specification {
    let uid = new_constraint.uid();
    let mut specs = Specification::new(uid);
    // First we find the bounds
    for constraint in constraints {
        if let Constraint::Occupy(state) = constraint {
            if state.uid == uid {
                if state.duration.0 <= new_constraint.duration().0
                    && specs.start_time < state.duration.0
                {
                    specs.start_time = state.duration.0;
                    specs.start_cell = state.location.origin;
                }
                if new_constraint.duration().1 <= state.duration.1 {
                    if let Some(time) = specs.end_time {
                        if state.duration.1 < time {
                            specs.end_time = Some(state.duration.1);
                            specs.end_cell = Some(state.location.origin);
                        }
                    } else {
                        specs.end_time = Some(state.duration.1);
                        specs.end_cell = Some(state.location.origin);
                    }
                }
            }
        }
    }
    specs
}

fn relevant_time(constraint: Constraint, spec: &Specification) -> bool {
    if constraint.duration().1 >= spec.start_time {
        if let Some(time) = spec.end_time {
            if constraint.duration().0 <= time {
                return true;
            }
        } else {
            return true;
        }
    }
    false
}

fn adapt_constraint(constraint: Constraint, spec: &Specification) -> Option<Constraint> {
    if relevant_time(constraint, spec) {
        if constraint.uid() == spec.uid {
            Some(constraint)
        } else {
            match constraint {
                Constraint::Occupy(state) => Some(Constraint::Avoid(state)),
                Constraint::Avoid(_) => None,
            }
        }
    } else {
        None
    }
}

fn adapt_const(mut spec: Specification, constraints: &[Constraint]) -> Specification {
    let adapted = constraints
        .iter()
        .filter_map(|constraint| adapt_constraint(*constraint, &spec))
        .collect();
    spec.constraints = adapted;
    spec
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
