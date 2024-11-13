use cbs_lawt::astar::{AStar, Grid, Terrain};
use cbs_lawt::cbs::solve_mapf;
use cbs_lawt::conflict::find_conflicts;
use cbs_lawt::prelude::*;
use ntest::timeout;
use std::collections::HashMap;

fn make_grid() -> Grid<Option<usize>> {
    let mut grid: Grid<Option<usize>> = Grid::init(Pair(4, 4), Some(1));
    grid[Pair(0, 2)] = None;
    grid[Pair(3, 2)] = None;
    grid
}

fn make_terrain() -> Terrain {
    Terrain::init(make_grid(), Pair(1, 1))
}

fn make_astar() -> AStar {
    let destinations = vec![Pair(0, 3), Pair(2, 3)];
    AStar::init(make_terrain(), destinations)
}

fn find_solution() -> HashMap<Pair, Path> {
    let astar = make_astar();
    let origins = vec![Pair(0, 0), Pair(3, 0)];
    solve_mapf(&astar, &origins)
}

#[test]
#[timeout(1000)]
fn compatible_paths() {
    let solution = find_solution();
    print_paths(&solution);
    let conflicts = find_conflicts(&solution);
    println!("Conflicts: {conflicts:?}");
    assert!(conflicts.is_empty());
}

#[test]
#[timeout(1000)]
fn correct_lengths() {
    let solution = find_solution();
    print_paths(&solution);
    let takes_32 = solution
        .values()
        .all(|path| path[path.len() - 1].duration.1 == 8 * 4);
    assert!(takes_32);
}
