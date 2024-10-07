use crate::prelude::*;
use grid::Grid;
use std::collections::HashMap;
use std::ops::Sub;

#[derive(PartialEq, Eq)]
pub struct GridExt {
    grid: Grid<CellInfo>,
    unit_size: Pair,
}

// Visible: new, cost, neighbors, djikstra

impl GridExt {
    pub fn new(grid: Grid<CellInfo>, unit_size: Pair) -> GridExt {
        GridExt { grid, unit_size }
    }

    // Effective number of (columns, rows)
    pub fn extent(&self) -> Pair {
        (
            self.grid.rows().sub(self.unit_size.0) + 1,
            self.grid.cols().sub(self.unit_size.1) + 1,
        )
    }

    // Accounts for unit size
    pub fn in_bounds(&self, cell: Pair) -> bool {
        cell.0 < self.extent().0 && cell.1 < self.extent().1
    }

    fn cell_is_clear(&self, cell: Pair) -> bool {
        !self.grid[cell].blocked
    }

    fn rect(&self, cell: Pair) -> Rect {
        Rect {
            origin: cell,
            extent: self.unit_size,
        }
    }

    pub fn is_clear(&self, cell: Pair) -> bool {
        let mut clear = true;
        for tile in self.rect(cell).cells() {
            clear = clear && self.cell_is_clear(tile)
        }
        clear
    }

    pub fn neighbors(&self, cell: Pair) -> Vec<Pair> {
        let mut out = Vec::with_capacity(4);
        let candidates = [
            (cell.0 + 1, cell.1),
            (cell.0, cell.1 + 1),
            (cell.0.wrapping_sub(1), cell.1),
            (cell.0, cell.1.wrapping_sub(1)),
        ];
        for candidate in candidates {
            if self.in_bounds(candidate) && self.is_clear(candidate) {
                out.push(candidate);
            }
        }
        out
    }

    pub fn set_blocked(&mut self, cell: Pair, blocked: bool) {
        for cell in self.rect(cell).cells() {
            self.grid[cell].blocked = blocked
        }
    }

    pub fn cost(&self, cell: Pair) -> usize {
        let mut total = 0;
        for tile in self.rect(cell).cells() {
            total += self.grid[tile].cost
        }
        total
    }

    // Costs to move *to* cell, not from
    pub fn djikstra(&self, cell: Pair) -> Grid<usize> {
        let mut reached = HashMap::with_capacity(self.extent().0 * self.extent().1);
        reached.insert(cell, 0);
        let mut checked = Grid::init(self.extent().0, self.extent().1, usize::MAX);
        while !reached.is_empty() {
            let min_cell;
            let current_cost = match reached.min_key() {
                Some((key, value)) => {
                    min_cell = key;
                    value
                }
                None => break,
            };
            checked[min_cell] = current_cost;
            reached.remove(&min_cell);
            for neighbor in self.neighbors(min_cell) {
                // If the neighbor has not been fully resolved yet
                if checked[neighbor] == usize::MAX {
                    // Cost of self, because the cost is to move *to* self
                    let new_cost = current_cost + self.cost(min_cell);
                    match reached.get(&neighbor) {
                        Some(value) => {
                            if new_cost < *value {
                                reached.insert(neighbor, new_cost);
                            }
                        }
                        None => {
                            reached.insert(neighbor, new_cost);
                        }
                    }
                }
            }
        }
        checked
    }
}
