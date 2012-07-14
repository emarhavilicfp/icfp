// Main robot driver.

import path;
import state;
import heuristics::*;

// BUG: add this to libstd::util
// pure fn rational_mul(x: rational, y: rational) -> rational {
//    {num: x.num * y.num, den: x.den, y.den}
//}

type brushfire = ();

fn path_len(_p: path::path) -> uint { fail; }

fn state_apply(_s: state::state, _ml: path::path) -> option<state::state> { fail; }

fn path_easy(_s: state::state, _fire: @mut option<brushfire>) -> option<path::path> { fail; }
fn path_aggressive(_s: state::state, _fire: @mut option<brushfire>) -> option<path::path> { fail; }

// Finds a path to the lambda that makes us happiest.
fn get_next_lambda(-s: state::state) -> option<(state::state,path::path)> {
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
