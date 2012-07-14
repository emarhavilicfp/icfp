// Main robot driver.

import path;
import state;
import state::extensions;
import heuristics::*;
import dvec;
import dvec::extensions;

// BUG: add this to libstd::util
// pure fn rational_mul(x: rational, y: rational) -> rational {
//    {num: x.num * y.num, den: x.den * y.den}
//}

type brushfire = path::path_state;

fn path_len(p: path::path) -> uint { vec::len(p) }

fn path_for_each(p: path::path, blk: fn(state::move) -> bool) { p.each(blk) }


fn state_apply(_s: state::state, _ml: path::path) -> option<state::state> {
    alt path::apply(_ml, _s, false) {
        state::stepped(state) { some(copy state) } // XXX remove copy
        state::endgame(*) | state::oops { none } // XXX fix endgame
    }
}

fn path_easy(s: state::state, fire: @mut option<brushfire>) -> option<path::path> {
    let lambdas = s.grid.lambdas();
    if fire.is_some() {
        let mut shit = none;
        *fire <-> shit;
        let (shit1, shit2) = option::unwrap(shit);
        // Get path and new state
        let (pathres, stateres) = path::genpath_restart(s.grid, s.robotpos, lambdas, shit1, shit2);
        *fire = some(stateres);
        pathres
    } else {
        let (pathres, stateres) = path::genpaths(s.grid, s.robotpos, lambdas);
        *fire = some(stateres);
        pathres
    }

}
fn path_aggressive(_s: state::state, _fire: @mut option<brushfire>) -> option<path::path> { none }

// Finds a path to the lambda that makes us happiest.
fn get_next_lambda(s: state::state) -> option<(state::state,path::path)> {
    let easy_state = @mut none;
    let mut easy = path_easy(s, easy_state);
    let aggr_state = @mut none;
    let mut aggressive = path_aggressive(s, aggr_state);

    // Diamonds are forever.
    loop {
        // Did we find *nothing*? God damn.
        if aggressive.is_none() && easy.is_none() {
            ret none;
        // See if we want to use the aggressive path. Maybe there is no easy
        // path, or maybe they all suck.
        } else if easy.is_none() ||
                  (aggressive.is_some() &&
                   path_aggr_weight(path_len(aggressive.expect("asdf")))
                   < path_len(easy.expect("ghjkl"))) {
            let try_path = aggressive.expect("Ben's boolean logic bad (1)");
            // Bouldered path was 2/3 the length of easy path.
            let newstate = state_apply(s, try_path);
            if newstate.is_some() {
                ret some((option::unwrap(newstate), try_path)); // Satisfied!
            } else {
                aggressive = path_aggressive(s, aggr_state);
                again;
            }
        } else {
            let try_path = easy.expect("Ben's boolean logic bad (2)");
            // Easy path seemed shorter, or "not too much" longer.
            let newstate = state_apply(s, try_path);
            if newstate.is_some() {
                ret some((option::unwrap(newstate), try_path)); // Satisfied!
            } else {
                easy = path_easy(s, easy_state);
                again;
            }
        }
    }
}

// Repeatedly finds lambdas (hopefully).
fn play_game_prime(-s: state::state, -moves_so_far: dvec::dvec<state::move>)
        -> (~[const state::move], state::state) {
    // Attempt to do something next.
    let result = get_next_lambda(s);
    if result.is_some() {
        let (newstate,path) = option::unwrap(result);
        // Append path to moves_so_far.
        for path_for_each(path) |the_move| {
            moves_so_far.push(the_move);
        }
        // Keep going!
        play_game_prime(newstate, moves_so_far)
    } else {
        // All done.
        (dvec::unwrap(moves_so_far) + [state::A]/_, s)
    }
}

fn play_game(+s: state::state) -> (~[const state::move], state::state) {
    play_game_prime(s, dvec::dvec())
}
