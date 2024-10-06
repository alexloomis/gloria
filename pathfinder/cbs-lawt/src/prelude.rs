use std::collections::HashMap;

pub mod cbs;
pub mod grid_ext;
pub mod mapf;

pub use crate::prelude::grid_ext::*;
pub use crate::prelude::mapf::*;

pub type Cell = (usize, usize);

pub type Path = Vec<Cell>;

#[derive(Clone, Copy)]
pub struct CellInfo {
    pub cost: usize,
    pub blocked: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rect {
    pub origin: Cell,
    pub extent: Cell,
}

impl Rect {
    fn cells(&self) -> Vec<Cell> {
        let mut out = Vec::with_capacity(self.extent.0 * self.extent.1);
        for dx in 0..self.extent.0 {
            for dy in 0..self.extent.1 {
                out.push((self.origin.0 + dx, self.origin.1 + dy))
            }
        }
        out
    }
}

pub trait HashMapExt<T> {
    fn min_key(&self) -> Option<(T, usize)>;
}

impl<T: Copy> HashMapExt<T> for HashMap<T, usize> {
    fn min_key(&self) -> Option<(T, usize)> {
        let mut best_key = None;
        let mut best_value = usize::MAX;
        for (key, value) in self.iter() {
            if *value < best_value {
                best_key = Some(*key);
                best_value = *value;
            }
        }
        best_key.map(|key| (key, best_value))
    }
}
