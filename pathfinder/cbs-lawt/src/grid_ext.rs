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

    // Max coordinate
    pub fn extent(&self) -> Pair {
        Pair(self.grid.rows().sub(1), self.grid.cols().sub(1))
    }

    // Effective number of (columns, rows) for units with extent extent
    pub fn effective_size(&self, extent: Pair) -> Pair {
        Pair(
            self.grid.rows().sub(extent.0),
            self.grid.cols().sub(extent.1),
        )
    }

    // Accounts for unit size
    pub fn in_bounds(&self, rect: Rect) -> bool {
        rect.max_coord().0 <= self.extent().0 && rect.max_coord().1 <= self.extent().1
    }

    fn cell_is_clear(&self, cell: Pair) -> bool {
        !self.grid[cell.into()].blocked
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
        let mut out = Vec::with_capacity(4);
        let candidates = [
            Rect {
                origin: rect.origin + Pair(1, 0),
                extent: rect.extent,
            },
            Rect {
                origin: rect.origin + Pair(0, 1),
                extent: rect.extent,
            },
            Rect {
                origin: rect.origin + Pair(usize::MAX, 0),
                extent: rect.extent,
            },
            Rect {
                origin: rect.origin + Pair(0, usize::MAX),
                extent: rect.extent,
            },
        ];
        for candidate in candidates {
            if self.in_bounds(candidate) && self.is_clear(candidate) {
                out.push(candidate);
            }
        }
        out
    }

    pub fn set_blocked(&mut self, rect: Rect, blocked: bool) {
        for cell in rect.cells() {
            self.grid[cell.into()].blocked = blocked
        }
    }

    pub fn cost(&self, rect: Rect) -> usize {
        let mut total = 0;
        for tile in rect.cells() {
            total += self.grid[tile.into()].cost
        }
        total
    }

    pub fn djikstra(&self, to: Rect) -> Grid<usize> {
        let size = self.effective_size(to.extent);
        let mut open = HashMap::with_capacity(size.0 * size.1);
        open.insert(to, 0);
        let mut closed = Grid::init(size.0, size.1, usize::MAX);
        while !open.is_empty() {
            let min_cell;
            let current_cost = match open.min_value() {
                Some((key, value)) => {
                    min_cell = key;
                    value
                }
                None => break,
            };
            closed[min_cell.origin.into()] = current_cost;
            open.remove(&min_cell);
            for neighbor in self.neighbors(min_cell) {
                // If the neighbor has not been fully resolved yet
                if closed[neighbor.origin.into()] == usize::MAX {
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
