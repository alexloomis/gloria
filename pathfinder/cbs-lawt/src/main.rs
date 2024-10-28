#![allow(unused)]
use cbs_lawt::astar::{self, AStar};
use cbs_lawt::cbs::{solve_mapf, CBS};
use cbs_lawt::constraint::Specification;
// samply record ./path/to/bin to profile
//use cbs_lawt::astar::AStar;
//use cbs_lawt::cbs::solve_mapf;
use cbs_lawt::grid::Grid;
use cbs_lawt::prelude::{Pair, Path};
use cbs_lawt::terrain::Terrain;
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
    let astar = test_astar();
    let origins = formation(Pair(1, 5), 2, Pair(3, 2));
    let solution = solve_mapf(&astar, &origins);
    for path in solution {
        draw_with_paths(&astar, vec![path]);
    }
}

//fn main() {
//    let astar = baby_astar();
//    let mut rng = rand::thread_rng();
//    for _ in 0..10 {
//        let start_cell = Pair(rng.gen_range(0..GRID_CONST), rng.gen_range(0..GRID_CONST));
//        let end_cell = if rng.gen_bool(0.5) {
//            Some(Pair(
//                rng.gen_range(0..GRID_CONST),
//                rng.gen_range(0..GRID_CONST),
//            ))
//        } else {
//            None
//        };
//        let start_time = rng.gen_range(0..10);
//        let end_time = None;
//        let uid = start_cell;
//        let constraints = Vec::new();
//        let specs = Specification {
//            uid,
//            start_cell,
//            start_time,
//            end_cell,
//            end_time,
//            constraints,
//        };
//        astar.astar(specs);
//    }
//}

//fn main() {
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
