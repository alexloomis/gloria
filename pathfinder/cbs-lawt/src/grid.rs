use crate::prelude::*;
use core::panic;
use std::{
    cmp::min,
    collections::BinaryHeap,
    ops::{Index, IndexMut, Sub},
    usize,
};

#[derive(PartialEq, Eq, Clone)]
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

    pub unsafe fn get_unchecked(&self, index: Pair) -> &T {
        self.data.get_unchecked(self.pair_to_usize(index))
    }

    pub fn indexed_iter(&self) -> impl Iterator<Item = (Pair, &T)> {
        self.data.iter().enumerate().map(move |(idx, i)| {
            let position = self.usize_to_pair(idx);
            (position, i)
        })
    }

    pub fn indicies(&self) -> Vec<Pair> {
        (0..self.data.len())
            .map(|index| self.usize_to_pair(index))
            .collect()
    }

    pub fn indexed_iter_mut(&mut self) -> impl Iterator<Item = (Pair, &mut T)> {
        let extent = self.extent;
        self.data.iter_mut().enumerate().map(move |(index, i)| {
            let position = Grid::<T>::usize_to_pair_(extent, index);
            (position, i)
        })
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

impl<T: Clone> Grid<T> {
    pub fn init_clone(extent: Pair, value: T) -> Grid<T> {
        let length = (extent.0 + 1) * (extent.1 + 1);
        let mut data = Vec::with_capacity(length);
        for _ in 0..length {
            data.push(value.clone());
        }
        Grid { data, extent }
    }
}

impl<T: Copy> Grid<T> {
    pub fn init(extent: Pair, value: T) -> Grid<T> {
        let data = vec![value; (extent.0 + 1) * (extent.1 + 1)];
        Grid { data, extent }
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
        let mut open = BinaryHeap::with_capacity(size.0 * size.1);
        open.push(DjikstraCell {
            location: to,
            cost: 0,
        });
        let mut closed = Grid::init(self.effective_extent(to.extent), usize::MAX);

        while !open.is_empty() {
            let cell = match open.pop() {
                Some(c) => c,
                None => break,
            };
            // If the cell has already been fully resolved
            if closed[cell.location.origin] < usize::MAX {
                continue;
            }
            closed[cell.location.origin] = cell.cost;
            for neighbor in self.neighbors(cell.location) {
                // If the neighbor has not been fully resolved yet
                if closed[neighbor.origin] == usize::MAX {
                    let new_cost = cell.cost + self.cost(cell.location);
                    open.push(DjikstraCell {
                        location: neighbor,
                        cost: new_cost,
                    });
                }
            }
        }
        closed
    }

    fn prefilled_djikstra(&self, from: Rect, data: &mut Grid<Option<usize>>) {
        let size = self.effective_size(from.extent);
        let mut open = BinaryHeap::with_capacity(size.0 * size.1);
        open.push(DjikstraCell {
            location: from,
            cost: 0,
        });
        let skip_before = self.pair_to_usize(from.origin);

        while !open.is_empty() {
            let cell = match open.pop() {
                Some(c) => c,
                None => break,
            };
            // If the cell has already been fully resolved
            if data[cell.location.origin].is_some() {
                continue;
            }
            data[cell.location.origin] = Some(cell.cost);

            for neighbor in self.neighbors(cell.location) {
                if self.pair_to_usize(neighbor.origin) < skip_before {
                    continue;
                }
                // If the neighbor has not been fully resolved yet
                if data[neighbor.origin].is_none() {
                    let new_cost = cell.cost + self.cost(neighbor);
                    open.push(DjikstraCell {
                        location: neighbor,
                        cost: new_cost,
                    });
                }
            }
        }
    }

    // c_dist(x,y) = dist(x,y) + cost(x) - cost(y) = dist(y,x)
    fn c_distance(
        &self,
        loc_0: Rect,
        loc_1: Rect,
        distances: &Grid<Grid<Option<usize>>>,
    ) -> Option<usize> {
        if let Some(dist) = distances[loc_0.origin][loc_1.origin] {
            let c_dist = dist + self.cost(loc_0) - self.cost(loc_1);
            Some(c_dist)
        } else {
            None
        }
    }

    // Outer grid is indexed by FROM, inner by TO, value is distance.
    pub fn all_distances(&self, unit_extent: Pair) -> Grid<Grid<Option<usize>>> {
        let effective_extent = self.effective_extent(unit_extent);
        let inner = Grid::init(effective_extent, None);
        let mut distances = Grid::init_clone(effective_extent, inner);

        for cell in distances.indicies() {
            // Fill in all distances from `cell`
            self.prefilled_djikstra(cell.extend(unit_extent), &mut distances[cell]);
            // Use that information to fill out distances to `cell`
            let cell_idx = distances.pair_to_usize(cell);
            for from in distances.indicies().iter().skip(cell_idx + 1) {
                distances[*from][cell] = self.c_distance(
                    from.extend(unit_extent),
                    cell.extend(unit_extent),
                    &distances,
                )
            }
        }
        distances
    }

    pub fn floyd_warshall(&self, unit_extent: Pair) -> Grid<usize> {
        let max_idx = self.pair_to_usize(self.extent());
        let mut distances = Grid::init(Pair(max_idx, max_idx), usize::MAX);
        for (origin, _) in self.indexed_iter() {
            for neighbor in self.neighbors(Rect {
                origin,
                extent: unit_extent,
            }) {
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
                //if i == j {
                //    continue;
                //}
                let d_ij = unsafe { *distances.get_unchecked(Pair(i, j)) };
                for k in 0..i {
                    //if j == k {
                    //    continue;
                    //}
                    let d_ik = unsafe { *distances.get_unchecked(Pair(i, k)) };
                    let d_jk = unsafe { *distances.get_unchecked(Pair(j, k)) };
                    distances[Pair(i, k)] = min(d_ik, d_ij + d_jk);
                }
            }
        }
        distances
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
struct DjikstraCell {
    cost: usize,
    location: Rect,
}

impl Ord for DjikstraCell {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other
            .cost
            .cmp(&self.cost)
            .then_with(|| other.location.cmp(&self.location))
    }
}

impl PartialOrd for DjikstraCell {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
