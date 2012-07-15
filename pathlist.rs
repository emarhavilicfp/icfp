// Path generator.

import path;
import state;
import state::extensions;
import heuristics;

// A lazy list of movelists.
trait pathlist {
    // Should always be the same state. Needs knowledge of region ptrs to fix.
    fn next_target_path(s: state::state)
        -> option<(state::state, path::path)>;
}

impl shit of pathlist for bblums_pathlist {
    fn next_target_path(s: state::state)
            -> option<(state::state, path::path)> {
        get_next_lambda(s, self)
    }
}

// TODO: Contain the game state by using a region pointer.
type bblums_pathlist = (@mut option<brushfire>, @mut option<brushfire>);
fn mk_bblums_pathlist() -> pathlist {
    let state: bblums_pathlist = (@mut none, @mut none);
    state as pathlist
}

/****************************************************************************
 * Glue
 ****************************************************************************/

// We use the Brushfire algorithm. Here is a webpage that mentions it:
// http://andrewferguson.net/2009/04/06/a-short-guide-to-robot-path-planning/
type brushfire = path::path_state;

fn path_len(p: path::path) -> uint { vec::len(p) }


fn state_apply(_s: state::state, _ml: path::path) -> option<state::state> {
    alt path::apply(_ml, _s, false) {
        state::stepped(state) { some(state::extract_step_result(state)) }
        state::endgame(*) | state::oops { none } // XXX fix endgame
    }
}

// This finds paths that don't require traversing any unsafe things.
//
// Unsafety is defined as moves that cause side effects, such as
// dislodging a boulder.
fn path_easy(s: state::state, fire: @mut option<brushfire>)
        -> option<path::path> {
    let lambdas = s.grid.lambdas();
    if fire.is_some() {
        let mut shit = none;
        *fire <-> shit;
        let (shit1, shit2) = option::unwrap(shit);
        // Get path and new state
        let (pathres, stateres) =
            path::genpath_restart(s.grid, s.robotpos, lambdas, shit1, shit2);
        *fire = some(stateres);
        pathres
    } else {
        let (pathres, stateres) = path::genpaths(s.grid, s.robotpos, lambdas);
        *fire = some(stateres);
        pathres
    }

}
fn path_aggressive(_s: state::state, _fire: @mut option<brushfire>)
        -> option<path::path> { none }

/****************************************************************************
 * Code
 ****************************************************************************/

// Finds a path to the lambda that makes us happiest.
fn get_next_lambda(s: state::state, ps: bblums_pathlist)
        -> option<(state::state,path::path)> {
    let (easy_state,aggr_state) = ps;
    let mut easy = path_easy(s, easy_state);
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
                   heuristics::path_aggr_weight(path_len(aggressive.expect("asdf")))
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
