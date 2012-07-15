// A lambda calculus evaluation heuristics.

import state;
import state::extensions;
import dlist;
import dlist::extensions;
import dvec::extensions;
import heuristics;
import bblum = path_find::brushfire;
import path;
import path::target;

// List of pairs (position, score)
type lambdalist = dlist::dlist<(state::coord, int)>;

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

// day 3 evaluator

const lambda_in_bush_score: int =
    state::lambda_score / heuristics::bird_in_hand_multiplier;

fn evaluate(s: state::state) -> int {
    let mut score = s.score;
    // FIXME: open-coded special-case pathfinder. joshua mad.
    let targets = bblum::map_mut(s.grid.lambdas(), |x| some(x));
    let easy_state = @mut none;
    let mut easy_target = path_easy(s, easy_state, targets);
    // Don't attempt to validate.
    while easy_target.is_some() {
        let (path,sq) = easy_target.get();
        alt s.grid.at(sq) {
            // TODO: Horocks and razors go here!
            state::lambda {
                score += lambda_in_bush_score;
            }
            state::lift_o {
                // Won game, it seems. The rest of the search should find this
                // so no need to incorporate it in the eval.
            }
            _ {}
        }
        easy_target = path_easy(s, easy_state, targets);
    }
    score
}

/* // Kemp Urfy
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
*/

// Heinous glue.
// NB: Unlike the one in brushfire.rs, this one returns a coord, not a path.
fn path_easy(s: state::state, fire: @mut option<bblum::brushfire>,
             targets: &[mut option<state::coord>])
        -> option<(path::path,state::coord)> {
    // Sets a slot in the targets list to 'none'
    fn process_pathres(-x: option<(path::path,state::coord)>,
                       targets: &[mut option<state::coord>])
            -> option<(path::path,state::coord)> {
        if x.is_some() {
            let (path,target_found) = option::unwrap(x);
            // XXX XXX XXX: Asymptotic runtime loss: Fix genpaths to return
            // an *index*, not a coord.
            let mut i = 0;
            while i < targets.len() {
                if targets[i] == some(target_found) {
                    targets[i] = none;
                    break;
                }   
                i += 1;
            }   
            some((path,target_found))
        } else { none }
    }   
    // TODO: write this to produce a better datatype to begin with
    if fire.is_some() {
        let mut shit = none;
        *fire <-> shit;
        let (shit1, shit2) = option::unwrap(shit);
        // Get path and new state
        let (pathres, stateres) =
            path::genpath_restart(s.grid, s.robotpos, targets, shit1, shit2);
        *fire = some(stateres);
        process_pathres(pathres, targets)
    } else {
        let (pathres, stateres) = path::genpaths(s.grid, s.robotpos, targets);
        *fire = some(stateres);
        process_pathres(pathres, targets)
    }   

}

