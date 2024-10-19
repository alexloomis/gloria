use crate::prelude::*;
use core::panic;
use std::{
    collections::HashMap,
    ops::{Index, IndexMut, Sub},
    usize,
};

#[derive(PartialEq, Eq)]
pub struct Grid<T> {
    data: Vec<T>,
    extent: Pair,
}

impl<T> Grid<T> {
    // Max coordinate
    pub fn extent(&self) -> Pair {
        self.extent
    }

    pub fn size(&self) -> Pair {
        self.extent + Pair(1, 1)
    }

    fn usize_to_pair_(extent: Pair, index: usize) -> Pair {
        Pair(index % (extent.0 + 1), index / (extent.0 + 1))
    }

    fn usize_to_pair(&self, index: usize) -> Pair {
        Grid::<T>::usize_to_pair_(self.extent, index)
    }

    // idx.j must be at most extent.j (unchecked)
    fn pair_to_usize(&self, index: Pair) -> usize {
        if index.0 > self.extent().0 || index.1 > self.extent().1 {
            panic!("Index {:?} exceeds extent {:?}!", index, self.extent())
        }
        index.0 + index.1 * self.size().0
    }
}

impl<T> Index<Pair> for Grid<T> {
    type Output = T;
    fn index(&self, index: Pair) -> &Self::Output {
        &self.data[self.pair_to_usize(index)]
    }
}

impl<T> IndexMut<Pair> for Grid<T> {
    fn index_mut(&mut self, index: Pair) -> &mut Self::Output {
        let idx = self.pair_to_usize(index);
        &mut self.data[idx]
    }
}

impl<T: Copy> Grid<T> {
    pub fn init(extent: Pair, value: T) -> Grid<T> {
        let data = vec![value; (extent.0 + 1) * (extent.1 + 1)];
        Grid { data, extent }
    }

    pub fn indexed_iter(&self) -> impl Iterator<Item = (Pair, &T)> {
        self.data.iter().enumerate().map(move |(idx, i)| {
            let position = self.usize_to_pair(idx);
            (position, i)
        })
    }

    pub fn indexed_iter_mut(&mut self) -> impl Iterator<Item = (Pair, &mut T)> {
        let extent = self.extent;
        self.data.iter_mut().enumerate().map(move |(index, i)| {
            let position = Grid::<T>::usize_to_pair_(extent, index);
            (position, i)
        })
    }
}

impl Grid<CellInfo> {
    // Max origin for a unit with extent `extent`
    pub fn effective_extent(&self, extent: Pair) -> Pair {
        Pair(self.extent().0.sub(extent.0), self.extent().0.sub(extent.1))
    }

    pub fn effective_size(&self, extent: Pair) -> Pair {
        self.effective_extent(extent) + Pair(1, 1)
    }

    // Accounts for unit size
    pub fn in_bounds(&self, rect: Rect) -> bool {
        rect.max_coord().0 <= self.extent().0 && rect.max_coord().1 <= self.extent().1
    }

    fn cell_is_clear(&self, cell: Pair) -> bool {
        !self[cell].blocked
    }

    pub fn is_clear(&self, rect: Rect) -> bool {
        for tile in rect.cells() {
            if !self.cell_is_clear(tile) {
                return false;
            }
        }
        true
    }

    pub fn neighbors(&self, rect: Rect) -> Vec<Rect> {
        let candidates = [
            rect + Pair(1, 0),
            rect + Pair(0, 1),
            rect + Pair(usize::MAX, 0),
            rect + Pair(0, usize::MAX),
        ];
        let mut out = Vec::with_capacity(4);
        if rect.max_coord().0 < self.extent().0 && self.is_clear(candidates[0]) {
            out.push(candidates[0]);
        }
        if rect.max_coord().1 < self.extent().1 && self.is_clear(candidates[1]) {
            out.push(candidates[1]);
        }
        if rect.origin.0 > 0 && self.is_clear(candidates[2]) {
            out.push(candidates[2]);
        }
        if rect.origin.1 > 0 && self.is_clear(candidates[3]) {
            out.push(candidates[3]);
        }
        out
    }

    pub fn set_blocked(&mut self, rect: Rect, blocked: bool) {
        for cell in rect.cells() {
            self[cell].blocked = blocked
        }
    }

    pub fn cost(&self, rect: Rect) -> usize {
        let mut total = 0;
        for tile in rect.cells() {
            total += self[tile].cost
        }
        total
    }

    pub fn djikstra(&self, to: Rect) -> Grid<usize> {
        let size = self.effective_size(to.extent);
        let mut open = HashMap::with_capacity(size.0 * size.1);
        open.insert(to, 0);
        let mut closed = Grid::init(self.effective_extent(to.extent), usize::MAX);
        while !open.is_empty() {
            let min_cell;
            let current_cost = match open.min_value() {
                Some((key, value)) => {
                    min_cell = key;
                    value
                }
                None => break,
            };
            closed[min_cell.origin] = current_cost;
            open.remove(&min_cell);
            for neighbor in self.neighbors(min_cell) {
                // If the neighbor has not been fully resolved yet
                if closed[neighbor.origin] == usize::MAX {
                    // Cost of self, because the cost is to move *to* self
                    let new_cost = current_cost + self.cost(min_cell);
                    match open.get(&neighbor) {
                        Some(value) => {
                            if new_cost < *value {
                                open.insert(neighbor, new_cost);
                            }
                        }
                        None => {
                            open.insert(neighbor, new_cost);
                        }
                    }
                }
            }
        }
        closed
    }

    pub fn floyd_warshall(&self, extent: Pair) -> Grid<usize> {
        let max_idx = self.pair_to_usize(self.extent());
        let mut distances = Grid::init(Pair(max_idx, max_idx), usize::MAX);
        for (origin, _) in self.indexed_iter() {
            for neighbor in self.neighbors(Rect { origin, extent }) {
                let idx = Pair(
                    self.pair_to_usize(origin),
                    self.pair_to_usize(neighbor.origin),
                );
                distances[idx] = self.cost(neighbor);
            }
        }
        for idx in 0..=max_idx {
            distances[Pair(idx, idx)] = 0
        }
        for j in 0..=max_idx {
            for i in 0..=max_idx {
                for k in 0..=max_idx {
                    if distances[Pair(i, k)] > distances[Pair(i, j)] + distances[Pair(j, k)] {
                        distances[Pair(i, k)] = distances[Pair(i, j)] + distances[Pair(j, k)]
                    }
                }
            }
        }
        distances
    }
}
