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

#[derive(Clone, Debug)]
pub struct Specification {
    pub uid: Pair,
    pub start_cell: Pair,
    pub start_time: usize,
    pub end_cell: Option<Pair>,
    pub end_time: Option<usize>,
    pub constraints: Vec<Constraint>,
}

impl Specification {
    pub fn new(uid: Pair) -> Specification {
        Specification {
            uid,
            start_cell: uid,
            start_time: 0,
            end_cell: None,
            end_time: None,
            constraints: Vec::new(),
        }
    }

    pub fn create(constraint: Constraint, constraints: &[Constraint]) -> Vec<Specification> {
        let mut spec = Specification::new(constraint.uid());
        spec.constraints = constraints.to_vec();
        let mut specs = spec.time_adaptations(constraint);
        for s in specs.iter_mut() {
            s.add_and_adapt(constraint);
        }
        specs
    }

    fn adapt_start(&mut self, unit_state: UnitState) {
        for constraint in &self.constraints {
            if let Constraint::Occupy(state) = constraint {
                if state.uid == self.uid
                    && state.duration.0 <= unit_state.duration.0
                    && self.start_time < state.duration.0
                {
                    self.start_time = state.duration.0;
                    self.start_cell = state.location.origin;
                }
            }
        }
    }

    fn adapt_end(&mut self, unit_state: UnitState) {
        for constraint in &self.constraints {
            if let Constraint::Occupy(state) = constraint {
                if state.uid == self.uid && unit_state.duration.1 <= state.duration.1 {
                    if let Some(time) = self.end_time {
                        if state.duration.1 < time {
                            self.end_time = Some(state.duration.1);
                            self.end_cell = Some(state.location.origin);
                        }
                    } else {
                        self.end_time = Some(state.duration.1);
                        self.end_cell = Some(state.location.origin);
                    }
                }
            }
        }
    }

    fn set_start(&mut self, unit_state: UnitState) {
        self.start_cell = unit_state.location.origin;
        self.start_time = unit_state.duration.1;
    }

    fn set_end(&mut self, unit_state: UnitState) {
        self.end_time = Some(unit_state.duration.0);
        self.end_cell = Some(unit_state.location.origin);
    }

    fn time_adaptations(self, constraint: Constraint) -> Vec<Specification> {
        match constraint {
            Constraint::Avoid(unit_state) => {
                let mut spec = self;
                spec.adapt_start(unit_state);
                spec.adapt_end(unit_state);
                vec![spec]
            }
            Constraint::Occupy(unit_state) => {
                let mut before = self.clone();
                before.adapt_start(unit_state);
                before.set_end(unit_state);
                let mut after = self;
                after.set_start(unit_state);
                after.adapt_end(unit_state);
                vec![before, after]
            }
        }
    }

    fn relevant_time(&self, constraint: Constraint) -> bool {
        if constraint.duration().1 >= self.start_time {
            if let Some(time) = self.end_time {
                if constraint.duration().0 <= time {
                    return true;
                }
            } else {
                return true;
            }
        }
        false
    }

    fn adapt_constraint(&self, constraint: Constraint) -> Option<Constraint> {
        if self.relevant_time(constraint) {
            if constraint.uid() == self.uid {
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

    fn adapt_constraints(&mut self) {
        self.constraints = self
            .constraints
            .iter()
            .filter_map(|constraint| self.adapt_constraint(*constraint))
            .collect();
    }

    fn add_and_adapt(&mut self, constraint: Constraint) {
        self.constraints.push(constraint);
        self.adapt_constraints();
    }
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

pub fn legal_path(path: Path, constraints: &[Constraint]) -> bool {
    path.iter()
        .all(|state| satisfies_constraints(*state, constraints))
}

fn prevents_stopping(unit: UnitState, constraint: Constraint) -> bool {
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
        .any(|constraint| prevents_stopping(unit, *constraint))
}
