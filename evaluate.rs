// A lambda calculus evaluation heuristics.

import state;
import dlist;
import dvec::extensions;

// List of pairs (position, score)
type lambdalist = dvec::dvec<(state::coord, int)>;

fn add_lambda_sorted(ls: lambdalist, coord: state::coord, score: int) {
    let mut link = ls.peek_n();
    while link.is_some() {
        let neighbour_nobe = link.get();
        // Higher taxicab distance goes farther away from the head.
        if (tuple::second(neighbour_nobe.data) >= score) {
            ls.insert_after((coord,score), neighbour_nobe);
            ret;
        }
        link = neighbour_nobe.next_link();
    }
    // worst lambda found so far. insert at tail
    ls.push((coord, score));
}

// O(nl), n == size of board, l == number of lambdas
fn score_lambdas(g: state::grid, f: fn(state::coord) -> int) -> lambdalist {
    let lambdas = dlist::create::<(state::coord, int)>();
    do state::foldl(lambdas, g) |lambdas, square, coord| {
       if square == state::lambda {
           add_lambda_sorted(lambdas, coord, f(coord));
       }
       lambdas
    }
}

fn evaluate(state: state, expensive: bool) -> int {
    let const drown = 4;
    let const trap = 2;
    let const base_weight = 10;
    
    /* Speculate */
    do score_lambdas_fn(state.grid) |coord| {
        let (cx, cy) = coord;
        let mut weight = base_weight + (cy - state.water);
        let mut score = (weight * state::taxicab_distance(state.robotpos, coord);

        /* Will we drown? */
        let (x, y) = state.robotpos;
        let depth = state.water - y;
        if (depth > state.nextflood) {
            score += drownBad * state.lambdasleft;
        }

        /* Will we have unreachable lambdas? */
        if (expensive) {
            let ps = play::initial_path_state();
            let mut next =  play::get_next_lambda(copy s, ps);
            let mut final;
            while (next.is_some()) {
                (final, _) = next.expect("kemurphy bad 1");
                next = play::get_next_lambda(final, ps);
            }
            score += trapBad * final.lambdacount;
        }
    }
}
