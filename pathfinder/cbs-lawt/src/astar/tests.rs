use crate::prelude::{
    tests::{cell_info_strat, pair_strat},
    Pair,
};
use proptest::{
    collection::vec,
    option::of,
    prelude::{Just, Strategy},
    prop_compose, proptest,
};

use super::*;

const MAX_GRID_EXTENT: Pair = Pair(100, 100);
const MAX_CELL_COST: usize = 10;
const CELL_BLOCKED_CHANCE: f64 = 0.1;
const MAX_UNIT_EXTENT: Pair = Pair(0, 0);
//const MAX_FORMATION_SIZE: Pair = Pair(3,10);
//const MAX_FORMATION_SPREAD: usize = 2;

pub fn grid_strat(
    extent: Pair,
    max_cost: usize,
    prob: f64,
) -> impl Strategy<Value = Grid<Option<usize>>> {
    let num_cells = (extent.0 + 1) * (extent.1 + 1);
    let data_s = vec(cell_info_strat(max_cost, prob), num_cells);
    data_s.prop_map(move |data| Grid { extent, data })
}

prop_compose! {
    pub fn terrain_strat(extent: Pair, unit_extent: Pair)
(grid in grid_strat(extent, MAX_CELL_COST, CELL_BLOCKED_CHANCE)) -> Terrain {
        Terrain::init(grid, unit_extent)
    }
}

pub fn single_dest_astar_strat(extent: Pair, unit_extent: Pair) -> impl Strategy<Value = AStar> {
    let terrain_point_s = (terrain_strat(extent, unit_extent), pair_strat(extent));
    terrain_point_s.prop_map(|(terrain, pair)| AStar::init(terrain, vec![pair]))
}

prop_compose! {
pub fn spec_strat(extent: Pair)
    (start_cell in pair_strat(extent),
        start_time in 0usize..10,
        end_cell in of(pair_strat(extent)),
        end_time in Just(None)) -> Specification {
    Specification {
        uid: Pair(0, 0),
        start_cell,
        start_time,
        end_cell,
        end_time,
        constraints: Vec::new(),
    }
}
}

fn astar_spec_strat() -> impl Strategy<Value = (AStar, Specification)> {
    let extent_s = pair_strat(MAX_GRID_EXTENT);
    let unit_extent_s = pair_strat(MAX_UNIT_EXTENT);
    (extent_s, unit_extent_s).prop_flat_map(|(extent, unit_extent)| {
        (
            single_dest_astar_strat(extent + unit_extent, unit_extent),
            spec_strat(extent),
        )
    })
}

proptest! {
    #[test]
    fn astar_test(astar_spec in astar_spec_strat()) {
        let (astar, spec) = astar_spec;
        let _ = astar.astar(spec);
    }
}

//fn formation(size: Pair, spread: usize, offset: Pair) -> Vec<Pair> {
//    let mut out = Vec::with_capacity(size.0 * size.1);
//    for x in 0..size.0 {
//        for y in 0..size.1 {
//            let cell = Pair(x * spread + offset.0, y * spread + offset.1);
//            out.push(cell);
//        }
//    }
//    out
//}

//pub fn formation_strat(
//    max_size: Pair,
//    unit_extent: usize,
//    max_offset: Pair,
//) -> impl Strategy<Value = Vec<Pair>> {
//    let size_s = (1..=max_size.0, 1..=max_size.1).prop_map(|(x, y)| Pair(x, y));
//    let spread_s = unit_extent..=(unit_extent + MAX_FORMATION_SPREAD);
//    let offset_s = pair_strat(max_offset);
//    (size_s, spread_s, offset_s).prop_map(|(size, spread, offset)| formation(size, spread, offset))
//}

//fn clear(grid: &mut Grid<Option<usize>>, clear: Vec<Pair>) {
//    for cell in clear {
//        grid[cell] = Some(1);
//    }
//}

const GRID_CONST: usize = 15;

//fn test_0() {
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
