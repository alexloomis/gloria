use crate::prelude::*;
use core::panic;
use std::ops::{Index, IndexMut};

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

    // Defined to agree with Ord for Pair
    fn usize_to_pair_(extent: Pair, index: usize) -> Pair {
        Pair(index / (extent.0 + 1), index % (extent.0 + 1))
    }

    pub fn usize_to_pair(&self, index: usize) -> Pair {
        Grid::<T>::usize_to_pair_(self.extent, index)
    }

    // idx.j must be at most extent.j (unchecked)
    pub fn pair_to_usize(&self, index: Pair) -> usize {
        if index.0 > self.extent().0 || index.1 > self.extent().1 {
            panic!("Index {:?} exceeds extent {:?}!", index, self.extent())
        }
        index.1 + index.0 * self.size().1
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
