use cbs_lawt::cbs::solve_mapf;
use cbs_lawt::mapf::MAPF;
use cbs_lawt::prelude::{CellInfo, Pair};
use grid::Grid;

fn main() {
    let origins: [Pair; 2] = [(0, 0), (3, 0)];
    let destinations: [Pair; 2] = [(0, 3), (2, 3)];
    let mut grid: Grid<CellInfo> = Grid::init(
        5,
        6,
        CellInfo {
            cost: 1,
            blocked: false,
        },
    );
    grid[(0, 2)].blocked = true;
    grid[(3, 2)].blocked = true;
    grid[(1, 1)].cost = 2;

    let mapf: MAPF = MAPF::init(origins.to_vec(), destinations.to_vec(), grid);
    //let mut constraints = Vec::new();
    //for i in 1..=10 {
    //    constraints.push(Constraint {
    //        uid: (3, 0),
    //        cell: (2, 0),
    //        time: i,
    //    })
    //}
    //let mut cbs = CBS::init(&mapf, ConstraintGenerator::NAIF);
    //cbs.apply_constraints(constraints);
    //for i in 0..cbs.solution.len() {
    //    println!("Solution {}:", i);
    //    for j in &cbs.solution[i] {
    //        println!("Departed {:?} at {}", j.cell, j.time);
    //    }
    //    println!()
    //}
    //println!("Cost: {:?}", cbs.cost);
    //println!();
    //println!("Conflicts: {:?}", cbs.conflicts);

    let sln = solve_mapf(&mapf);
    for i in 0..sln.len() {
        println!("Solution {}:", i);
        for j in &sln[i] {
            println!("Stayed at {:?} for {:?}", j.cell, j.stay);
        }
        println!()
    }
}
