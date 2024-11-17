use crate::prelude::*;
use radix_heap::RadixHeapMap;

#[derive(Eq, PartialEq, Debug, Clone, Copy)]
pub struct CellCost(Option<usize>);

impl Ord for CellCost {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self == other {
            std::cmp::Ordering::Equal
        } else if self.0.is_none() {
            std::cmp::Ordering::Greater
        } else if other.0.is_none() {
            std::cmp::Ordering::Less
        } else {
            self.cmp(other)
        }
    }
}

impl PartialOrd for CellCost {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug)]
pub struct Terrain {
    base_costs: Grid<CellCost>,
    unit_extent: Pair,
    costs: Grid<Option<usize>>,
    distances: Grid<Grid<CellCost>>,
}

impl Terrain {
    fn new(costs: Grid<CellCost>, unit_extent: Pair) -> Terrain {
        Terrain {
            base_costs: costs,
            unit_extent,
            costs: Grid::init(Pair(0, 0), None),
            distances: Grid::init_clone(Pair(0, 0), Grid::init(Pair(0, 0), CellCost(None))),
        }
    }

    pub fn init(grid: Grid<CellCost>, unit_extent: Pair) -> Terrain {
        let mut terrain = Terrain::new(grid, unit_extent);
        terrain.find_costs();
        terrain.all_distances();
        terrain
    }

    // Max origin for a unit with extent `extent`
    pub fn extent(&self) -> Pair {
        Pair(
            self.base_costs.extent().0 - self.unit_extent.0,
            self.base_costs.extent().1 - self.unit_extent.1,
        )
    }

    pub fn unit_extent(&self) -> Pair {
        self.unit_extent
    }

    pub fn size(&self) -> Pair {
        self.extent() + Pair(1, 1)
    }

    // Accounts for unit size
    pub fn in_bounds(&self, cell: Pair) -> bool {
        cell.0 <= self.extent().0 && cell.1 <= self.extent().1
    }

    pub fn is_cell_blocked(&self, cell: Pair) -> bool {
        self.base_costs[cell].0.is_none()
    }

    fn compute_cost(&self, rect: Rect) -> Option<usize> {
        let mut total = Some(0);
        for cell in rect.cells() {
            if let Some(cost) = self.base_costs[cell].0 {
                total = total.map(|c| c + cost);
            } else {
                total = None;
                break;
            }
        }
        total
    }

    fn find_costs(&mut self) {
        let mut costs = Grid::init(self.extent(), None);
        for (cell, val) in costs.indexed_iter_mut() {
            *val = self.compute_cost(cell.extend(self.unit_extent));
        }
        self.costs = costs;
    }

    pub fn is_clear(&self, cell: Pair) -> bool {
        self.costs[cell].is_some()
    }

    pub fn is_blocked(&self, cell: Pair) -> bool {
        !self.is_clear(cell)
    }

    pub fn neighbors(&self, cell: Pair) -> Vec<Pair> {
        let candidates = [
            cell + Pair(1, 0),
            cell + Pair(0, 1),
            cell + Pair(usize::MAX, 0),
            cell + Pair(0, usize::MAX),
        ];
        let mut out = Vec::with_capacity(4);
        if cell.0 < self.extent().0 && self.is_clear(candidates[0]) {
            out.push(candidates[0]);
        }
        if cell.1 < self.extent().1 && self.is_clear(candidates[1]) {
            out.push(candidates[1]);
        }
        if cell.0 > 0 && self.is_clear(candidates[2]) {
            out.push(candidates[2]);
        }
        if cell.1 > 0 && self.is_clear(candidates[3]) {
            out.push(candidates[3]);
        }
        out
    }

    pub fn cost(&self, cell: Pair) -> usize {
        self.costs[cell].unwrap()
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
struct DjikstraCell {
    cost_so_far: usize,
    location: Pair,
}

impl Ord for DjikstraCell {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other
            .cost_so_far
            .cmp(&self.cost_so_far)
            .then_with(|| other.location.cmp(&self.location))
    }
}

impl PartialOrd for DjikstraCell {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Terrain {
    fn prefilled_djikstra(&self, to: Pair, data: &mut Grid<CellCost>) {
        // If cell is inaccessible, nothing to do
        if self.costs[to].is_none() {
            return;
        }
        data[to] = CellCost(Some(0));
        let border = self.costs.border(to);
        let mut open = RadixHeapMap::new_at(0);
        for cell in border.clone() {
            if let Some(cost) = data[cell].0 {
                let dc = DjikstraCell {
                    location: cell,
                    cost_so_far: cost,
                };
                // Cell is reopened to allow paths to travel trough it
                data[cell] = CellCost(None);
                open.push(-(cost as i64), dc);
            }
        }

        while !open.is_empty() {
            let cell = match open.pop() {
                Some((_, v)) => v,
                None => {
                    break;
                }
            };
            // If the cell has already been fully resolved, continue
            if data[cell.location].0.is_some() {
                continue;
            }
            data[cell.location] = CellCost(Some(cell.cost_so_far));

            for neighbor in self.neighbors(cell.location) {
                // If the neighbor has not been fully resolved yet
                if data[neighbor].0.is_none() {
                    // Cell is clear, and hence cost() does not panic
                    let new_cost = cell.cost_so_far + self.cost(cell.location);
                    let dc = DjikstraCell {
                        location: neighbor,
                        cost_so_far: new_cost,
                    };
                    open.push(-(new_cost as i64), dc);
                }
            }
        }
    }

    pub fn djikstra(&self, to: Pair) -> Grid<CellCost> {
        let mut data = Grid::init(self.extent(), CellCost(None));
        self.prefilled_djikstra(to, &mut data);
        data
    }

    // c_dist(from x, to y) = dist(to x, from y) + cost(y) - cost(x) = dist(to y, from x)
    fn c_distance(&self, loc_0: Pair, loc_1: Pair, distances: &Grid<Grid<CellCost>>) -> CellCost {
        if let Some(dist) = distances[loc_0][loc_1].0 {
            let c_dist = dist + self.cost(loc_1) - self.cost(loc_0);
            CellCost(Some(c_dist))
        } else {
            CellCost(None)
        }
    }

    // Outer grid is indexed by TO, inner by FROM, value is distance.
    fn all_distances(&mut self) {
        let effective_extent = self.extent();
        let inner = Grid::init(effective_extent, CellCost(None));
        let mut distances = Grid::init_clone(effective_extent, inner);

        for cell in distances.indicies() {
            // Fill in all distances from `cell`
            self.prefilled_djikstra(cell, &mut distances[cell]);
            // Use that information to fill out distances from `cell`
            let cell_idx = distances.pair_to_usize(cell);
            for to in distances.indicies().iter().skip(cell_idx + 1) {
                distances[*to][cell] = self.c_distance(cell, *to, &distances)
            }
        }
        self.distances = distances
    }

    pub fn distances(&self) -> &Grid<Grid<CellCost>> {
        &self.distances
    }
}
