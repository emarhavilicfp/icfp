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

// BUG: add this
fn reach<T>(v: &[const T], blk: fn(T) -> bool) {
    do vec::unpack_slice(v) |p, n| {
        let mut i = 1;
        while i <= n {
            unsafe {
                if !blk(*ptr::offset(p, n-i)) { break; }
            }
            i += 1;
        }
    }
}

/****************************************************************************
 * Glue
 ****************************************************************************/

type brushfire = path::path_state;

fn path_len(p: path::path) -> uint { vec::len(p) }

fn path_for_each(p: path::path, blk: fn(state::move) -> bool) { reach(p,blk) }


fn state_apply(_s: state::state, _ml: path::path) -> option<state::state> {
    alt path::apply(_ml, _s, false) {
        state::stepped(state) { some(state::extract_step_result(state)) }
        state::endgame(*) | state::oops { none } // XXX fix endgame
    }
}

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

// Repeatedly finds lambdas (hopefully). Generates the movelist backwards.
fn greedy_finish(-s: state::state, verbose: bool)
        -> (dvec::dvec<state::move>, state::state) {
    // Attempt to do something next.
    let result = get_next_lambda(s);
    if verbose {
        io::println("Pursuing");
    }

    if result.is_some() {
        let (newstate,path) = option::unwrap(result);
        // Find what to do next.
        let (finishing_moves,endstate) = greedy_finish(newstate, verbose);
        // Do this stuff before.
        for path_for_each(path) |the_move| {
            finishing_moves.push(the_move);
        }
        (finishing_moves, endstate)
    } else {
        // All done.
        (dvec::from_elem(state::A), s)
    }
}

// type search_opts = 

//fn search(s: state::state, -moves_so_far: dvec::dvec<state::move>,
//          o: search_opts) -> (

fn play_game(+s: state::state, verbose: bool)
    -> (~[mut state::move], state::state) {
    let (moves_rev, endstate) = greedy_finish(s, verbose);
    let moves = dvec::unwrap(moves_rev);
    vec::reverse(moves);
    (moves,endstate)
}

mod test {
    #[test]
    fn play_game_check_hash() {
        let s = #include_str("./maps/contest10.map");
        let mut s = state::read_board(io::str_reader(s));
        let mut result = get_next_lambda(s);
        while result != none {
            let (newstate, _path) = option::unwrap(result);
            assert newstate.hash() == newstate.rehash();
            s = newstate;
            result = get_next_lambda(s);
        }
    }
}
