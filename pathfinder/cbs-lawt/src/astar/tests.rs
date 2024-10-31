use crate::prelude::{
    tests::{cell_info_strat, pair_strat},
    Pair, Path,
};
use proptest::{
    collection::vec,
    prelude::{Just, Strategy},
    prop_compose,
};
use rand::Rng;

use super::*;

pub fn grid_strat(max_cost: usize, prob: f64) -> impl Strategy<Value = Grid<CellInfo>> {
    let extent_s = pair_strat(Pair(100, 100));
    let extent_size_s = extent_s.prop_map(|e| (Just(e), (e.0 + 1) * (e.1 + 1)));
    let extent_data_s = extent_size_s
        .prop_flat_map(move |(extent, size)| (extent, vec(cell_info_strat(max_cost, prob), size)));
    extent_data_s.prop_map(|(extent, data)| Grid { extent, data })
}

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

pub fn formation_strat(
    max_size: Pair,
    min_spread: usize,
    max_offset: Pair,
) -> impl Strategy<Value = Vec<Pair>> {
    let size_s = (1..=max_size.0, 1..=max_size.1).prop_map(|(x, y)| Pair(x, y));
    let spread_s = min_spread..=(min_spread + 2);
    let offset_s = pair_strat(max_offset);
    (size_s, spread_s, offset_s).prop_map(|(size, spread, offset)| formation(size, spread, offset))
}

const GRID_CONST: usize = 15;

fn obstacles(extent: Pair, density: f64) -> Vec<Pair> {
    let mut out = Vec::new();
    let mut rng = rand::thread_rng();
    for i in 0..=extent.0 {
        for j in 0..=extent.1 {
            let roll: f64 = rng.gen();
            if roll < density {
                out.push(Pair(i, j));
            }
        }
    }
    out
}

fn make_grid(extent: Pair, density: f64, avoid: Vec<Pair>) -> Grid<Option<usize>> {
    let mut grid: Grid<Option<usize>> = Grid::init(extent, Some(1));
    for obstacle in obstacles(extent, density) {
        if !avoid.contains(&obstacle) {
            grid[obstacle] = None;
        }
    }
    grid
}

fn make_terrain() -> Terrain {
    let origins = formation(Pair(1, 5), 2, Pair(3, 2));
    let destinations = formation(Pair(1, 5), 2, Pair(15, 3));
    let unit_extent = Pair(0, 0);
    let mut clear = origins.clone();
    clear.append(&mut destinations.clone());
    let grid = make_grid(Pair(35, 35), 0.05, clear);
    Terrain::init(grid, unit_extent)
}

fn make_astar() -> AStar {
    let destinations = formation(Pair(1, 5), 2, Pair(15, 3));
    AStar::init(make_terrain(), destinations)
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

fn test_0() {
    let astar = baby_astar();
    let mut rng = rand::thread_rng();
    for _ in 0..10 {
        let start_cell = Pair(rng.gen_range(0..GRID_CONST), rng.gen_range(0..GRID_CONST));
        let end_cell = if rng.gen_bool(0.5) {
            Some(Pair(
                rng.gen_range(0..GRID_CONST),
                rng.gen_range(0..GRID_CONST),
            ))
        } else {
            None
        };
        let start_time = rng.gen_range(0..10);
        let end_time = None;
        let uid = start_cell;
        let constraints = Vec::new();
        let specs = Specification {
            uid,
            start_cell,
            start_time,
            end_cell,
            end_time,
            constraints,
        };
        astar.astar(specs);
    }
}

//fn test_1() {
//    let astar = test_astar();
//    let mut rng = rand::thread_rng();
//    for _ in 0..10_000 {
//        let start_cell = Pair(rng.gen_range(10..40), rng.gen_range(10..40));
//        let end_cell = if rng.gen_bool(0.5) {
//            Some(Pair(rng.gen_range(10..40), rng.gen_range(10..40)))
//        } else {
//            None
//        };
//        let start_time = rng.gen_range(0..10);
//        let end_time = if rng.gen_bool(0.5) {
//            Some(start_time + rng.gen_range(50..100))
//        } else {
//            None
//        };
//        let uid = start_cell;
//        let constraints = Vec::new();
//        let spec = Specification {
//            uid,
//            start_cell,
//            start_time,
//            end_cell,
//            end_time,
//            constraints,
//        };
//        //println!("Searching for path from {start_cell:?} to {end_cell:?}");
//        //println!("Path should span from t = {start_time} to {end_time:?}");
//        let path = astar.astar(spec);
//        //match path {
//        //    None => println!("Path not found"),
//        //    Some(p) => {
//        //        println!("Path found:");
//        //        draw_with_paths(&astar, vec![p]);
//        //    }
//        //}
//    }
//}
