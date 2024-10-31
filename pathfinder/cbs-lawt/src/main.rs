// samply record ./path/to/bin to profile
#![allow(unused)]

use cbs_lawt::astar::{AStar, Grid, Specification, Terrain};
use cbs_lawt::cbs::{solve_mapf, CBS};
use cbs_lawt::prelude::{Pair, Path};
use rand::Rng;

const GRID_CONST: usize = 15;

fn formation(size: Pair, spread: usize, offset: Pair) -> Vec<Pair> {
    let mut out = Vec::with_capacity(size.0 * size.1);
    for x in 0..size.0 {
        for y in 0..size.1 {
            let cell = Pair(x * spread + offset.0, y * spread + offset.1);
            out.push(cell);
        }
    }
    out
}

fn make_grid(extent: Pair, density: f64, avoid: Vec<Pair>) -> Grid<Option<usize>> {
    let mut grid: Grid<Option<usize>> = Grid::init(extent, Some(1));
    let mut rng = rand::thread_rng();
    for i in 0..=extent.0 {
        for j in 0..=extent.1 {
            let roll: f64 = rng.gen();
            if roll < density && !avoid.contains(&Pair(i, j)) {
                grid[Pair(i, j)] = None;
            }
        }
    }
    grid
}

fn test_terrain() -> Terrain {
    let origins = formation(Pair(1, 5), 2, Pair(3, 2));
    let destinations = formation(Pair(1, 5), 2, Pair(15, 3));
    let unit_extent = Pair(0, 0);
    let mut clear = origins.clone();
    clear.append(&mut destinations.clone());
    let grid = make_grid(Pair(35, 35), 0.05, clear);
    Terrain::init(grid, unit_extent)
}

fn test_astar() -> AStar {
    let destinations = formation(Pair(1, 5), 2, Pair(15, 3));
    AStar::init(test_terrain(), destinations)
}

fn baby_astar() -> AStar {
    let grid = make_grid(Pair(GRID_CONST + 1, GRID_CONST + 1), 0.2, Vec::new());
    let terrain = Terrain::init(grid, Pair(0, 0));
    AStar::init(terrain, vec![Pair(GRID_CONST, 1)])
}

fn draw_with_paths(astar: &AStar, paths: Vec<Path>) {
    let mut path_cells = Vec::new();
    for path in paths {
        for sc in path {
            for cell in sc.location.cells() {
                path_cells.push(cell);
            }
        }
    }

    for j in 0..=astar.terrain.extent().1 {
        for i in 0..=astar.terrain.extent().0 {
            let coord = Pair(i, j);
            let mut char = " ";
            if astar.terrain.is_cell_blocked(coord) {
                char = "×"
            } else if astar.destinations.contains(&coord) {
                char = "$"
            } else if path_cells.contains(&coord) {
                char = "*"
            }
            print!("{:1}", char);
            if i == astar.terrain.extent().0 {
                println!()
            }
        }
    }
}

fn main() {
    for i in 0..1_000 {
        println!("test {i}");
        let astar = test_astar();
        let origins = formation(Pair(1, 5), 2, Pair(3, 2));
        let solution: Vec<Path> = solve_mapf(&astar, &origins).into_values().collect();
        for path in solution {
            draw_with_paths(&astar, vec![path]);
        }
    }
}
