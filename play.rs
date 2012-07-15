// Main robot driver.

import path;
import state;
import state::*;
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

// We use the Brushfire algorithm. Here is a webpage that mentions it:
// http://andrewferguson.net/2009/04/06/a-short-guide-to-robot-path-planning/
type brushfire = path::path_state;

fn path_len(p: path::path) -> uint { vec::len(p) }

fn path_for_each(p: path::path, blk: fn(state::move) -> bool) { reach(p,blk) }


fn state_apply(_s: state::state, _ml: path::path) -> option<state::state> {
    alt path::apply(_ml, _s, false) {
        state::stepped(state) { some(state::extract_step_result(state)) }
        state::endgame(*) | state::oops { none } // XXX fix endgame
    }
}

/// This finds paths that don't require traversing any unsafe things.
///
/// Unsafety is defined as moves that cause side effects, such as
/// dislodging a boulder.
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

/// Attempts to find paths, possibly using dangerous moves. We
/// traverse dangeroues moves by patterns.
fn path_aggressive(_s: state::state, _fire: @mut option<brushfire>)
        -> option<path::path> { none }

/****************************************************************************
 * Code
 ****************************************************************************/

type path_state = (@mut option<brushfire>, @mut option<brushfire>);
fn initial_path_state() -> path_state { (@mut none, @mut none) }

// Finds a path to the lambda that makes us happiest.
fn get_next_lambda(s: state::state, ps: path_state)
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
fn get_next_lambda_oneshot(s: state::state)
        -> option<(state::state,path::path)> {
    get_next_lambda(s, initial_path_state())
}

// Movelist in reverse order. End state. Best score.
type search_result = (dvec::dvec<state::move>, state::state, int);

fn add_path_prefix(finishing_moves: dvec::dvec<state::move>, p: path::path) {
    for path_for_each(p) |the_move| {
        finishing_moves.push(the_move);
    }
}

type search_opts = { branch_factor: uint, verbose: bool };
fn default_opts() -> search_opts {
    { branch_factor: 1, verbose: false }
}
fn default_opts_verbose(verbose: bool) -> search_opts {
    { branch_factor: 1, verbose: verbose }
}
fn default_opts_bfac(bf: uint) -> search_opts {
    { branch_factor: bf, verbose: false }
}

// Repeatedly finds lambdas (hopefully).
fn greedy_finish(-s: state::state, o: search_opts) -> search_result {
    // Attempt to do something next.
    let result = get_next_lambda_oneshot(s);
    if result.is_some() {
        let (newstate,path) = option::unwrap(result);
        if o.verbose {
            io::println("Pursuing path of " +
                        str::concat(vec::map(path, |i| { i.to_str() })));
        }
        // Find what to do next.
        let (finishing_moves,endstate,score) = greedy_finish(newstate, o);
        // Do this stuff before.
        add_path_prefix(finishing_moves, path);
        (finishing_moves, endstate, score)
    } else {
        // All done.
        let score = s.score;
        (dvec::from_elem(state::A), s, score)
    }
}

fn search(-s: state::state, depth: uint, o: search_opts) -> search_result {
    let mut best = none;
    let mut best_score = none; // Redundant. To avoid unwrapping 'best'.
    let path_state = initial_path_state();
    // Test for horizon node.
    if depth == 0 {
        ret greedy_finish(s, o);
    } else {
        // Iterate over possibilities.
        // TODO: maybe prune later?
        for iter::repeat(o.branch_factor) {
            let target_opt = get_next_lambda(s, path_state);
            if target_opt.is_some() {
                let (newstate,path) = option::unwrap(target_opt);
                // Recurse.
                let (finishing_moves,endstate,this_score) =
                    search(newstate, depth-1, o);
                // Update best.
                if best_score.is_none() || this_score > best_score.get() {
                    // Prepend the moves we had (don't bother if not best)
                    // TODO: this can probably be tweaked even more to only
                    // append once per search node.
                    add_path_prefix(finishing_moves, path);
                    best = some((finishing_moves, endstate, this_score));
                    best_score = some(this_score);
                } else {
                    again;
                }
            } else {
                // No targets found, huh?
                break;
            }
        }
        // Process best_moves and best_score.
        if best.is_some() {
            option::unwrap(best)
        } else {
            // Terminal state. Nothing to do.
            let score = s.score;
            (dvec::from_elem(state::A), s, score)
        }
    }
}

fn play_game(+s: state::state, verbose: bool)
        -> (~[mut state::move], state::state) {
    let (moves_rev, endstate, _score) =
        greedy_finish(s, default_opts_verbose(verbose));
    let moves = dvec::unwrap(moves_rev);
    vec::reverse(moves);
    (moves,endstate)
}

mod test {
    #[test]
    fn test_play_game_check_hash() {
        let s = #include_str("./maps/contest10.map");
        let mut s = state::read_board(io::str_reader(s));
        let mut result = get_next_lambda_oneshot(s);
        while result != none {
            let (newstate, _path) = option::unwrap(result);
            assert newstate.hash() == newstate.rehash();
            s = newstate;
            result = get_next_lambda_oneshot(s);
        }
    }
    #[test]
    fn test_zero_depth_equals_greedy() {
        let s = #include_str("./maps/contest10.map");
        let mut s = state::read_board(io::str_reader(s));
        let (_, endstate, score) = greedy_finish(copy s, default_opts());
        let (_, endstate2, score2) = search(copy s, 0, default_opts_bfac(0));
        let (_, endstate3, score3) = search(s, 0, default_opts_bfac(31337));
        assert endstate.grid.hash == endstate2.grid.hash;
        assert endstate.grid.hash == endstate3.grid.hash;
        assert score == score2;
        assert score == score3;
    }
    #[test]
    fn test_one_bf_equals_greedy() {
        let s = #include_str("./maps/contest10.map");
        let mut s = state::read_board(io::str_reader(s));
        let (_, endstate, score) = greedy_finish(copy s, default_opts());
        let (_, endstate2, score2) = search(copy s, 1, default_opts_bfac(1));
        let (_, endstate3, score3) = search(copy s, 10, default_opts_bfac(1));
        let (_, endstate4, score4) = search(s, 31337, default_opts_bfac(1));
        assert endstate.grid.hash == endstate2.grid.hash;
        assert endstate.grid.hash == endstate3.grid.hash;
        assert endstate.grid.hash == endstate4.grid.hash;
        assert score == score2;
        assert score == score3;
        assert score == score4;
    }
    #[cfg(test)]
    fn test_search_vs_greedy(mapstr: str, depth: uint, bf: uint) {
        let mut s = state::read_board(io::str_reader(mapstr));
        let (_, endstate, score) = greedy_finish(copy s, default_opts());
        let (_, endstate2, score2) = search(s, depth, default_opts_bfac(bf));
        #error["Search @ depth %u bfac %u beat greedy %d-%d",
               depth, bf, score2, score];
        if score2 == score {
            assert endstate.grid.hash == endstate2.grid.hash;
        }
        assert score2 > score;
    }
    #[test]
    #[ignore] // Passes as of this commit. Might break in the future.
    fn test_search_beats_greedy() {
        // 5 seems to be the min branch depth. Guess we find it on the 5th
        // closest lambda.
        test_search_vs_greedy(#include_str("./maps/contest5.map"), 1, 5);
        test_search_vs_greedy(#include_str("./maps/contest5.map"), 2, 5);
        test_search_vs_greedy(#include_str("./maps/contest5.map"), 3, 5);
    }
}
