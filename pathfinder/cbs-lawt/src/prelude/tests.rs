use super::*;
use proptest::bool::weighted;
use proptest::prelude::Strategy;

pub fn pair_strat(extent: Pair) -> impl Strategy<Value = Pair> {
    (0..=extent.0, 0..extent.1).prop_map(|(x, y)| Pair(x, y))
}

pub fn cell_info_strat(max_cost: usize, prob: f64) -> impl Strategy<Value = Option<usize>> {
    (1..=max_cost, weighted(prob))
        .prop_map(|(cost, blocked)| if blocked { None } else { Some(cost) })
}
