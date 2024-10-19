use crate::grid::Grid;
use crate::prelude::*;

#[derive(PartialEq, Eq)]
pub struct PIBT {
    pub grid: Grid<CellInfo>,
    pub origins: Vec<Pair>,
    pub destinations: Vec<Pair>,
    pub unit_extent: Pair,
    pub heuristics: Vec<Grid<usize>>,
}

impl PIBT {
    fn find_max_among(grid: &Grid<usize>, xs: &[usize], ys: &[usize]) -> Pair {
        let mut max_x = xs[0];
        let mut max_y = ys[0];
        let mut max_val = grid[Pair(max_x, max_y)];
        for x in xs {
            for y in ys {
                let val = grid[Pair(*x, *y)];
                if val > max_val {
                    max_x = *x;
                    max_y = *y;
                    max_val = val;
                }
            }
        }
        Pair(max_x, max_y)
    }

    fn find_min_along(grid: &Grid<usize>, target: Pair, xs: &[usize], ys: &[usize]) -> Pair {
        let mut min_x = target.0;
        let mut min_y = target.1;
        let mut min_val = grid[target];
        for x in xs {
            let val = grid[Pair(*x, target.1)];
            if val < min_val {
                min_val = val;
                min_x = *x;
                min_y = target.1;
            }
        }
        for y in ys {
            let val = grid[Pair(target.0, *y)];
            if val < min_val {
                min_val = val;
                min_x = target.0;
                min_y = *y
            }
        }
        Pair(min_x, min_y)
    }

    fn find_heuristics(&mut self) {
        self.heuristics = Vec::with_capacity(self.destinations.len());
        for destination in &self.destinations {
            let heuristic = self.grid.djikstra(Rect {
                origin: *destination,
                extent: self.unit_extent,
            });
            self.heuristics.push(heuristic);
        }
    }

    // assignment[i] = j meanse move origin i to index j
    fn update_origins(&mut self, assignments: &[usize]) {
        let mut origins = vec![Pair(usize::MAX, usize::MAX); self.origins.len()];
        for (old_idx, new_idx) in assignments.into_iter().enumerate() {
            origins[*new_idx] = self.origins[old_idx]
        }
        self.origins = origins;
    }

    // Tries to minimize makespan
    fn assign_destinations(&mut self) {
        let mut distances = Grid::init(
            Pair(self.origins.len() - 1, self.destinations.len() - 1),
            usize::MAX,
        );
        for (Pair(origin_idx, dest_idx), value) in distances.indexed_iter_mut() {
            *value = self.heuristics[dest_idx][self.origins[origin_idx]];
        }

        let mut unassigned_origins: Vec<usize> = (0..self.origins.len()).collect();
        let mut unassigned_dests: Vec<usize> = (0..self.destinations.len()).collect();
        let mut assignments = vec![usize::MAX; self.origins.len()];

        while !unassigned_origins.is_empty() {
            // Give either the worst origin or the worst destination its best choice
            let coord = PIBT::find_max_among(&distances, &unassigned_origins, &unassigned_dests);
            let coord =
                PIBT::find_min_along(&distances, coord, &unassigned_origins, &unassigned_dests);
            assignments[coord.0] = coord.1;
            unassigned_origins.retain(|origin| *origin != coord.0);
            unassigned_dests.retain(|destination| *destination != coord.1);
        }
        self.update_origins(&assignments);
    }

    fn perform_swap(distances: &Grid<usize>, assignments: &mut [usize]) -> Option<Pair> {
        for i in 0..assignments.len() {
            for j in 0..i {
                let improve_i =
                    distances[Pair(assignments[j], i)] < distances[Pair(assignments[i], i)];
                let improve_j =
                    distances[Pair(assignments[i], j)] < distances[Pair(assignments[j], j)];
                let worsen_i =
                    distances[Pair(assignments[j], i)] > distances[Pair(assignments[i], i)];
                let worsen_j =
                    distances[Pair(assignments[i], j)] > distances[Pair(assignments[j], j)];
                if (improve_i && !worsen_j) || (improve_j && !worsen_i) {
                    let new_i = assignments[j];
                    let new_j = assignments[i];
                    assignments[i] = new_i;
                    assignments[j] = new_j;
                    return Some(Pair(i, j));
                }
            }
        }
        None
    }

    fn improve_assignments(&mut self) {
        let mut assignments: Vec<usize> = (0..self.origins.len()).collect();
        let mut distances = Grid::init(
            Pair(self.origins.len() - 1, self.destinations.len() - 1),
            usize::MAX,
        );
        for (Pair(origin_idx, dest_idx), value) in distances.indexed_iter_mut() {
            *value = self.heuristics[dest_idx][self.origins[origin_idx]];
        }
        loop {
            if PIBT::perform_swap(&distances, &mut assignments).is_none() {
                break;
            }
        }
        self.update_origins(&assignments);
    }

    fn new(
        grid: Grid<CellInfo>,
        origins: Vec<Pair>,
        destinations: Vec<Pair>,
        unit_extent: Pair,
    ) -> PIBT {
        PIBT {
            grid,
            origins,
            destinations,
            unit_extent,
            heuristics: Vec::new(),
        }
    }

    pub fn init(
        grid: Grid<CellInfo>,
        origins: Vec<Pair>,
        destinations: Vec<Pair>,
        unit_extent: Pair,
    ) -> PIBT {
        let mut pibt = PIBT::new(grid, origins, destinations, unit_extent);
        pibt.find_heuristics();
        pibt.assign_destinations();
        pibt.improve_assignments();
        pibt
    }

    pub fn pibt(&self) {
        let mut units = Vec::with_capacity(self.origins.len());
        for (idx, origin) in self.origins.iter().enumerate() {
            let unit = PIBTState {
                idx,
                location: Rect {
                    origin: *origin,
                    extent: self.unit_extent,
                },
                wait: 0,
                history: Vec::new(),
            };
            units.push(unit);
        }
        // Units the farthest away should start with the highest priority (front of the list)
        units.sort_unstable_by_key(|unit| self.heuristics[unit.idx][unit.location.origin]);
        units.reverse();

        let mut done = false;
        while !done {
            done = true;
            for unit in &units {
                if unit.location.origin == self.destinations[unit.idx] {
                    continue;
                }
                done = false;
                // Move unit
            }
        }
    }
}

struct PIBTState {
    idx: usize,
    location: Rect,
    wait: usize,
    history: Vec<Pair>,
}
