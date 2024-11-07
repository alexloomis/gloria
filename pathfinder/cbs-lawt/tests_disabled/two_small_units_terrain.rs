use cbs_lawt::astar::{AStar, Grid, Terrain};
use cbs_lawt::cbs::solve_mapf;
use cbs_lawt::conflict::find_conflicts;
use cbs_lawt::prelude::*;
use ntest::timeout;
use std::collections::HashMap;

fn make_grid() -> Grid<Option<usize>> {
    let mut grid: Grid<Option<usize>> = Grid::init(Pair(1, 2), Some(1));
    grid[Pair(0, 0)] = Some(2);
    grid
}

fn make_terrain() -> Terrain {
    Terrain::init(make_grid(), Pair(0, 0))
}

fn make_astar() -> AStar {
    let destinations = vec![Pair(0, 0), Pair(0, 2)];
    AStar::init(make_terrain(), destinations)
}

fn find_solution() -> HashMap<Pair, Path> {
    let astar = make_astar();
    let origins = vec![Pair(0, 1), Pair(1, 1)];
    solve_mapf(&astar, &origins)
}

#[test]
#[timeout(100)]
fn compatible_paths() {
    let solution = find_solution();
    print_paths(&solution);
    let conflicts = find_conflicts(&solution);
    println!("{conflicts:?}");
    assert!(conflicts.is_empty());
}

#[test]
#[timeout(100)]
fn correct_lengths() {
    let solution = find_solution();
    print_paths(&solution);
    let takes_2 = solution
        .values()
        .all(|path| path[path.len() - 1].duration.1 == 2);
    assert!(takes_2);
}
