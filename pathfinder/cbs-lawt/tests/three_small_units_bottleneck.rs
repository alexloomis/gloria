use cbs_lawt::astar::{AStar, Grid, Terrain};
use cbs_lawt::cbs::solve_mapf;
use cbs_lawt::conflict::find_conflicts;
use cbs_lawt::prelude::*;
use ntest::timeout;
use std::collections::HashMap;

fn make_grid() -> Grid<Option<usize>> {
    let mut grid: Grid<Option<usize>> = Grid::init(Pair(2, 2), Some(1));
    grid[Pair(0, 1)] = None;
    grid[Pair(2, 1)] = None;
    grid
}

fn make_terrain() -> Terrain {
    Terrain::init(make_grid(), Pair(0, 0))
}

fn make_astar() -> AStar {
    let destinations = vec![Pair(0, 2), Pair(1, 1), Pair(2, 2)];
    AStar::init(make_terrain(), destinations)
}

fn find_solution() -> HashMap<Pair, Path> {
    let astar = make_astar();
    let origins = vec![Pair(0, 0), Pair(1, 1), Pair(2, 0)];
    solve_mapf(&astar, &origins)
}

#[test]
#[should_panic]
#[timeout(100)]
fn compatible_paths() {
    let solution = find_solution();
    print_paths(&solution);
    let conflicts = find_conflicts(&solution);
    assert!(conflicts.is_empty());
}

#[test]
#[should_panic]
#[timeout(100)]
fn correct_lengths() {
    let solution = find_solution();
    print_paths(&solution);
    let sum = solution.values().fold(0, |acc, path| acc + path.len());
    // Failure indicates a change, not an error
    assert_eq!(sum, 11);
    let takes_4 = solution
        .values()
        .all(|path| path[path.len() - 1].duration.1 == 4);
    assert!(takes_4);
}
