use crate::prelude::*;

pub struct CBS<'a> {
    pub mapf: &'a MAPF,
    pub constraints: Vec<Constraint>,
    pub solution: Vec<Path>,
    pub cost: usize,
    pub conflicts: Vec<Conflict>,
    pub children: Vec<CBS<'a>>,
}

impl CBS<'_> {
    fn new(mapf: &MAPF) -> CBS {
        CBS {
            mapf,
            constraints: Vec::new(),
            solution: Vec::new(),
            cost: usize::MAX,
            conflicts: Vec::new(),
            children: Vec::new(),
        }
    }

    fn find_solution() {}

    fn make_child(&mut self, constraints: Vec<Constraint>) {
        let mut child = CBS::new(self.mapf);
        child.constraints = [self.constraints.clone(), constraints].concat();
        self.children.push(child);
    }
}
