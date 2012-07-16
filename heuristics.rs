fn path_aggr_weight(aggr_len: uint) -> uint { aggr_len * 3 / 2 }

fn branch_factor() -> uint { 5 }

// for the searcher granularity thingie. How much to trim the average path
// length? Just to be sure we don't get diqed.
fn chunking_avg_path_multiplier(avg_len: uint) -> uint { avg_len * 2 / 3 }

fn eval_severe_dropoff(old_score: int, new_score: int) -> bool {
    old_score * 6 > new_score * 10
}

const bird_in_hand_multiplier: int = 2;

// move ordering (trivial)
// water check in trans table
// dirt in trans table?
// eval
