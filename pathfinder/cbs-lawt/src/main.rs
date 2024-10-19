// samply record ./path/to/bin to profile
use cbs_lawt::astar::AStar;
use cbs_lawt::cbs::solve_mapf;
use cbs_lawt::grid::Grid;
use cbs_lawt::pibt::PIBT;
use cbs_lawt::prelude::{CellInfo, Pair, Path};
use rand::seq::SliceRandom;
use rand::Rng;

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

fn make_grid(extent: Pair, density: f64, avoid: Vec<Pair>) -> Grid<CellInfo> {
    let mut grid: Grid<CellInfo> = Grid::init(
        extent,
        CellInfo {
            cost: 1,
            blocked: false,
        },
    );
    let mut rng = rand::thread_rng();
    for i in 0..=extent.0 {
        for j in 0..=extent.1 {
            let roll: f64 = rng.gen();
            if roll < density && !avoid.contains(&Pair(i, j)) {
                grid[Pair(i, j)].blocked = true
            }
        }
    }
    grid
}

fn test_case() -> AStar {
    let origins = formation(Pair(1, 5), 2, Pair(3, 2));
    let destinations = formation(Pair(1, 5), 2, Pair(15, 3));
    let unit_extent = Pair(0, 0);
    let mut clear = origins.clone();
    clear.append(&mut destinations.clone());
    let grid = make_grid(Pair(100, 75), 0.10, clear);
    let astar = AStar::init(origins, destinations, unit_extent, grid);
    draw_with_paths(&astar, Vec::new());
    astar
}

fn draw_with_paths(astar: &AStar, paths: Vec<Path>) {
    let mut path_cells = Vec::new();
    for path in paths {
        for sc in path {
            path_cells.push(sc.location.origin);
        }
    }

    for j in 0..=astar.grid.extent().1 {
        for i in 0..=astar.grid.extent().0 {
            let coord = Pair(i, j);
            let mut char = " ";
            if astar.grid[coord].blocked {
                char = "x"
            } else if astar.origins.contains(&coord) {
                char = "%"
            } else if astar.destinations.contains(&coord) {
                char = "$"
            } else if path_cells.contains(&coord) {
                char = "*"
            }
            print!("{:1}", char);
            if i == astar.grid.extent().0 {
                println!()
            }
        }
    }
}

fn main() {
    //let origins: [Pair; 2] = [Pair(0, 0), Pair(3, 0)];
    //let destinations: [Pair; 2] = [Pair(0, 3), Pair(2, 3)];
    //let mut grid: Grid<CellInfo> = Grid::init(
    //    5,
    //    6,
    //    CellInfo {
    //        cost: 1,
    //        blocked: false,
    //    },
    //);
    //grid[(0, 2)].blocked = true;
    //grid[(3, 2)].blocked = true;
    //grid[(1, 1)].cost = 2;
    //
    //let baby_example: AStar =
    //    AStar::init(origins.to_vec(), destinations.to_vec(), Pair(1, 1), grid);

    let test = test_case();
    let sln = solve_mapf(&test);
    for (i, path) in sln.iter().enumerate() {
        println!("Solution {}:", i);
        for j in path {
            println!("Stayed at {:?} for {:?}", j.location, j.duration);
        }
        println!()
    }
    draw_with_paths(&test, sln);

    //let mut origins = formation(Pair(4, 25), 2, Pair(3, 2));
    //let mut rng = rand::thread_rng();
    //origins.shuffle(&mut rng);
    //let destinations = formation(Pair(4, 25), 2, Pair(15, 3));
    //let unit_extent = Pair(0, 0);
    //let mut clear = origins.clone();
    //clear.append(&mut destinations.clone());
    //let pibt = PIBT::init(
    //    make_grid(Pair(100, 75), 0.10, clear),
    //    origins,
    //    destinations,
    //    unit_extent,
    //);
    //for (idx, origin) in pibt.origins.iter().enumerate() {
    //    println!(
    //        "Unit at {:?} targeting {:?}.",
    //        origin, pibt.destinations[idx]
    //    );
    //}
}
