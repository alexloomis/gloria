use crate::mapf::MAPF;
use crate::prelude::*;
use core::panic;
use std::cmp::max;
use std::collections::HashSet;

fn rect_conflict(cell1: Cell, cell2: Cell, unit_size: Cell) -> bool {
    let ((x1, y1), (x2, y2), (dx, dy)) = (cell1, cell2, unit_size);
    x1 < x2 + dx && x2 < x1 + dx && y1 < y2 + dy && y2 < y1 + dy
}

fn check_against(path1: &[ScoredCell], path2: &[ScoredCell], unit_size: Cell) -> Vec<Conflict> {
    let uid1 = path1[0].cell;
    let uid2 = path2[0].cell;
    let mut idx1 = 0;
    let mut idx2 = 0;
    let end_time = max(path1[path1.len() - 1].time, path2[path2.len() - 1].time);
    let mut conflicts = Vec::new();
    for time in 0..=end_time {
        let mut changed1 = false;
        let mut changed2 = false;
        if time > path1[idx1].time && idx1 < path1.len() - 1 {
            idx1 += 1;
            changed1 = true
        }
        if time > path2[idx2].time && idx2 < path2.len() - 1 {
            idx2 += 1;
            changed2 = true
        }
        if changed1 || changed2 {
            let cell1 = path1[idx1].cell;
            let cell2 = path2[idx2].cell;
            if rect_conflict(cell1, cell2, unit_size) {
                conflicts.push(Conflict {
                    uid1,
                    uid2,
                    cell1,
                    cell2,
                    time,
                });
            }
        }
    }
    conflicts
}

pub struct CBS<'a> {
    pub mapf: &'a MAPF,
    pub constraints: Vec<Constraint>,
    // *must* be kept in the same order as mapf.origins
    pub solution: Vec<Vec<ScoredCell>>,
    pub cost: usize,
    pub conflicts: Vec<Conflict>,
    pub children: Vec<CBS<'a>>,
}

impl CBS<'_> {
    fn new(mapf: &MAPF) -> CBS {
        CBS {
            mapf,
            constraints: Vec::new(),
            solution: vec![Vec::new(); mapf.origins.len()],
            cost: 0,
            conflicts: Vec::new(),
            children: Vec::new(),
        }
    }

    pub fn init(mapf: &MAPF) -> CBS {
        let mut cbs = CBS::new(mapf);
        let num_units = mapf.origins.len();
        let mut modified: HashSet<usize> = HashSet::with_capacity(num_units);
        for i in 0..num_units {
            modified.insert(i);
        }
        cbs.find_solution(&modified);
        cbs.find_cost(&modified);
        cbs.find_conflicts(&modified);
        cbs
    }

    fn unit_to_idx(&self, cell: Cell) -> usize {
        match self.mapf.origins.iter().position(|c| cell == *c) {
            Some(idx) => idx,
            None => panic!("No origin found at {:?}!", cell),
        }
    }

    fn idx_to_unit(&self, idx: usize) -> Cell {
        self.mapf.origins[idx]
    }

    fn find_solution(&mut self, modified: &HashSet<usize>) {
        for idx in modified {
            self.solution[*idx] = self.mapf.astar(self.idx_to_unit(*idx), &self.constraints);
        }
    }

    fn find_cost(&mut self, modified: &HashSet<usize>) {
        for idx in modified {
            let path = &self.solution[*idx];
            let candidate = path[path.len() - 1].time;
            if candidate > self.cost {
                self.cost = candidate
            }
        }
    }

    fn conflict_involves(&self, conflict: &Conflict, modified: &HashSet<usize>) -> bool {
        modified.contains(&self.unit_to_idx(conflict.uid1))
            || modified.contains(&self.unit_to_idx(conflict.uid2))
    }

    fn clear_conflicts(&mut self, modified: &HashSet<usize>) {
        let mut retained = Vec::with_capacity(self.conflicts.len());
        for conflict in &self.conflicts {
            if !self.conflict_involves(conflict, modified) {
                retained.push(*conflict);
            }
        }
        self.conflicts = retained;
    }

    fn find_conflicts(&mut self, modified: &HashSet<usize>) {
        self.clear_conflicts(modified);
        let checked = |x: usize, y: usize| x <= y && modified.contains(&x);
        for idx1 in modified {
            let path1 = &self.solution[*idx1];
            for (idx2, path2) in self.solution.iter().enumerate() {
                if !checked(*idx1, idx2) {
                    self.conflicts
                        .extend(check_against(path1, path2, self.mapf.unit_size));
                }
            }
        }
    }

    pub fn apply_constraints(&mut self, mut constraints: Vec<Constraint>) {
        let mut modified = HashSet::with_capacity(self.mapf.origins.len());
        for constraint in &constraints {
            modified.insert(self.unit_to_idx(constraint.uid));
        }
        self.constraints.append(&mut constraints);
        self.find_solution(&modified);
        self.find_cost(&modified);
        self.find_conflicts(&modified);
    }
}
