use crate::astar::Constraint;
use crate::conflict::Conflict;
use crate::prelude::*;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Exploration {
    pub conflict: Conflict,
    pub constraints: [Constraint; 2],
    pub solutions: [Option<Path>; 2],
}

impl Exploration {
    fn score(&self) -> usize {
        self.solutions
            .iter()
            .map(|solution| {
                solution
                    .as_ref()
                    .map(|path| path.len())
                    .unwrap_or(usize::MAX)
            })
            .min()
            .unwrap()
    }

    fn secondary_score(&self) -> usize {
        self.solutions
            .iter()
            .map(|solution| {
                solution
                    .as_ref()
                    .map(|path| path.len())
                    .unwrap_or(usize::MAX)
            })
            .max()
            .unwrap()
    }

    pub fn uids(&self) -> [Pair; 2] {
        self.conflict.uids()
    }
}

// We want higher primary scores first, with lower secondary breaking ties
impl Ord for Exploration {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other
            .score()
            .cmp(&self.score())
            .then_with(|| self.secondary_score().cmp(&other.secondary_score()))
    }
}

impl PartialOrd for Exploration {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

fn prioritize(explorations: Vec<Exploration>) -> Vec<Exploration> {
    let mut out: Vec<Exploration> = Vec::with_capacity(explorations.len());
    for exploration in explorations {
        if exploration.solutions == [None, None] {
            continue;
        }
        let mut include = true;
        let mut replace_at = None;
        for (idx, chosen) in out.iter_mut().enumerate() {
            if exploration.uids() == chosen.conflict.uids() {
                if exploration.score() > chosen.score() {
                    replace_at = Some(idx);
                } else {
                    include = false
                }
                break;
            }
        }
        if include {
            match replace_at {
                Some(idx) => out[idx] = exploration,
                None => out.push(exploration),
            }
        }
    }
    out.sort();
    out
}

pub fn greedy_choices(explorations: Vec<Exploration>) -> Vec<Exploration> {
    let mut out = Vec::with_capacity(explorations.len());
    let mut seen = Vec::with_capacity(explorations.len() * 2);
    for exploration in prioritize(explorations) {
        let uids = exploration.uids();
        if !(seen.contains(&uids[0]) || seen.contains(&uids[1])) {
            out.push(exploration);
            seen.push(uids[0]);
            seen.push(uids[1]);
        }
    }
    out
}
