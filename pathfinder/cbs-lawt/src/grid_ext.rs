use crate::prelude::*;
use grid::Grid;
use std::collections::HashMap;
use std::ops::Sub;

#[derive(PartialEq, Eq)]
pub struct GridExt {
    grid: Grid<CellInfo>,
}

// Visible: new, cost, neighbors, djikstra

impl GridExt {
    pub fn new(grid: Grid<CellInfo>) -> GridExt {
        GridExt { grid }
    }

    // Effective number of (columns, rows)
    pub fn extent(&self) -> Pair {
        Pair(
            self.grid.rows().sub(UNIT_SIZE.0) + 1,
            self.grid.cols().sub(UNIT_SIZE.1) + 1,
        )
    }

    // Accounts for unit size
    pub fn in_bounds(&self, cell: Pair) -> bool {
        cell.0 < self.extent().0 && cell.1 < self.extent().1
    }

    fn cell_is_clear(&self, cell: Pair) -> bool {
        !self.grid[cell.into()].blocked
    }

    fn rect(&self, cell: Pair) -> Rect {
        Rect {
            origin: cell,
            extent: UNIT_SIZE,
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
            Pair(cell.0 + 1, cell.1),
            Pair(cell.0, cell.1 + 1),
            Pair(cell.0.wrapping_sub(1), cell.1),
            Pair(cell.0, cell.1.wrapping_sub(1)),
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
            self.grid[cell.into()].blocked = blocked
        }
    }

    pub fn cost(&self, cell: Pair) -> usize {
        let mut total = 0;
        for tile in self.rect(cell).cells() {
            total += self.grid[tile.into()].cost
        }
        total
    }

    pub fn djikstra(&self, to: Pair) -> Grid<usize> {
        let mut open = HashMap::with_capacity(self.extent().0 * self.extent().1);
        open.insert(to, 0);
        let mut closed = Grid::init(self.extent().0, self.extent().1, usize::MAX);
        while !open.is_empty() {
            let min_cell;
            let current_cost = match open.min_value() {
                Some((key, value)) => {
                    min_cell = key;
                    value
                }
                None => break,
            };
            closed[min_cell.into()] = current_cost;
            open.remove(&min_cell);
            for neighbor in self.neighbors(min_cell) {
                // If the neighbor has not been fully resolved yet
                if closed[neighbor.into()] == usize::MAX {
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
}
