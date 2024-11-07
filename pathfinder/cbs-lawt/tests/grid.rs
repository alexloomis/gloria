use cbs_lawt::astar::Grid;
use cbs_lawt::prelude::*;

#[test]
fn correct_indicies() {
    let grid = Grid::init(Pair(0, 1), ());
    let expected = vec![Pair(0, 0), Pair(0, 1)];
    let indicies: Vec<Pair> = grid.indexed_iter().map(|x| x.0).collect();
    assert_eq!(indicies, expected);
    let mut grid_m = Grid::init(Pair(0, 1), ());
    let indicies_m: Vec<Pair> = grid_m.indexed_iter_mut().map(|x| x.0).collect();
    assert_eq!(indicies_m, expected);
}
