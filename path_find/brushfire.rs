// Path generator.

import not_path = core::path;
import path;
import state;
import state::extensions;
import heuristics::*;

import path::target;

enum t {_dummy}
impl of path_find for t {
    fn get_paths(s: state::state) ->
        (fn@() -> option<(state::state, path::path)>) {
        let it = mk_iter(s);
        |copy s| get_next_lambda(s, it)
    }
}

// TODO: Contain the game state by using a region pointer.
type bblums_pathlist = {
    pathstate: (@mut option<brushfire>, @mut option<brushfire>),
    targets: ~[mut option<state::coord>]
};

fn mk() -> path_find {
    let t = _dummy;
    t as path_find
}

fn mk_iter(s: state::state) -> bblums_pathlist {
    let t = map_mut(s.grid.lambdas(), |x| some(x));
    let state: bblums_pathlist =
        { pathstate: (@mut none, @mut none), targets: t };
    state
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
        state::endgame(state, _) { some(*state) }
        state::oops(_) { none } // XXX fix endgame
    }
}

// BUG: add this to vec.rs
pure fn map_mut<T, U>(v: &[T], f: fn(T) -> U) -> ~[mut U] {
    let mut result = ~[mut]; 
    unchecked{vec::reserve(result, vec::len(v));}
    for vec::each(v) |elem| { unsafe { vec::push(result, f(elem)); } }
    ret result;
}

// This finds paths that don't require traversing any unsafe things.
//
// Unsafety is defined as moves that cause side effects, such as
// dislodging a boulder.
fn path_easy(s: state::state, fire: @mut option<brushfire>,
             targets: &[mut option<state::coord>]) -> option<path::path> {
    // Sets a slot in the targets list to 'none'
    fn process_pathres(-x: option<(path::path,state::coord)>,
                       targets: &[mut option<state::coord>])
            -> option<path::path> {
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
            some(path)
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

fn path_aggressive(_s: state::state, _fire: @mut option<brushfire>)
    -> option<path::path>
{
    none
}

/****************************************************************************
 * Code
 ****************************************************************************/

// Finds a path to the lambda that makes us happiest.
fn get_next_lambda(s: state::state, bblum: bblums_pathlist)
        -> option<(state::state,path::path)> {
    let (easy_state,aggr_state) = bblum.pathstate;;
    let mut easy = path_easy(s, easy_state, bblum.targets);
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
                easy = path_easy(s, easy_state, bblum.targets);
                again;
            }
        }
    }
}
