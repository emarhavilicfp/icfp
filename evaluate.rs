// A lambda calculus evaluation heuristics.

import state;
import dlist;
import dlist::extensions;

// List of pairs (position, score)
// Sorted with the, ahem, "highest order" lambda first.
type lambdalist = dlist::dlist<(state::coord, uint)>;

fn add_lambda_sorted(ls: lambdalist, coord: state::coord, score: uint) {
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
fn score_lambdas(g: state::grid, player_pos: state::coord) -> lambdalist {
    let lambdas = dlist::create::<(state::coord, uint)>();
    do state::foldl(lambdas, g) |lambdas, square, coord| {
       if square == state::lambda {
           add_lambda_sorted(lambdas, coord,
                             state::taxicab_distance(player_pos, coord));
       }
       lambdas
    }
}
