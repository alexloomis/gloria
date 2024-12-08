// samply record ./path/to/bin to profile
#![allow(unused)]

use cbs_lawt::pibt::*;
use cbs_lawt::prelude::*;
use proptest::path;
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

fn make_grid(extent: Pair, density: f64, avoid: Vec<Pair>) -> Grid<CellCost> {
    let mut grid = Grid::init(extent, CellCost(Some(2)));
    let mut rng = rand::thread_rng();
    for i in 0..=extent.0 {
        for j in 0..=extent.1 {
            let roll: f64 = rng.gen();
            if roll < density && !avoid.contains(&Pair(i, j)) {
                grid[Pair(i, j)] = CellCost(None);
            }
        }
    }
    grid
}

fn odu() -> (Vec<Pair>, Vec<Pair>, Pair) {
    let origins = formation(Pair(4, 25), 2, Pair(3, 2));
    let destinations = formation(Pair(4, 25), 2, Pair(45, 30));
    let unit_extent = Pair(0, 0);
    (origins, destinations, unit_extent)
}

fn test_terrain() -> Terrain {
    let (origins, destinations, unit_extent) = odu();
    let mut clear = origins.clone();
    clear.append(&mut destinations.clone());
    let grid = make_grid(Pair(100, 100), 0.15, clear);
    Terrain::init(grid, unit_extent)
}

fn test_pibt() -> PIBT {
    let terrain = test_terrain();
    let (origins, destinations, unit_extent) = odu();
    PIBT::init(terrain, origins, destinations, unit_extent)
}

fn draw_with_paths(terrain: &Terrain, paths: Vec<Vec<Pair>>) {
    let mut path_cells = Vec::new();
    for path in paths {
        for cell in path {
            path_cells.push(cell);
        }
    }

    for j in 0..=terrain.extent().1 {
        for i in 0..=terrain.extent().0 {
            let coord = Pair(i, j);
            let mut char = " ";
            if terrain.is_cell_blocked(coord) {
                char = "×"
            } else if path_cells.contains(&coord) {
                char = "*"
            }
            print!("{:1}", char);
            if i == terrain.extent().0 {
                println!()
            }
        }
    }
}

fn main() {
    let mut pibt = test_pibt();
    println!("pibt initialized");
    pibt.run(Some(1_000));
    if !pibt.all_arrived() {
        for (unit, dest) in pibt.unit_dests() {
            println!("Unit at {unit:?} has dest {dest:?}");
        }
    }
    draw_with_paths(pibt.terrain(), pibt.paths());
    let makespan = pibt.paths().into_iter().map(|path| path.len()).max();
    println!("{makespan:?}");
}
