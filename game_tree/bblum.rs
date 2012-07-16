// Main robot driver.

import path;
import state;
import state::*;
import heuristics::*;
import dvec;
import dvec::extensions;
import signal;
import path_find;
import path_find::*;

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

fn path_for_each(p: path::path, blk: fn(state::move) -> bool) { reach(p,blk) }

// Movelist in reverse order. End state. Best score.
type search_result = (dvec::dvec<state::move>, state::state, int);

fn add_path_prefix(finishing_moves: dvec::dvec<state::move>, p: path::path) {
    for path_for_each(p) |the_move| {
        finishing_moves.push(the_move);
    }
}

type search_opts = {
    branch_factor: uint,
    verbose: bool,
    killable: bool,
    max_depth: uint,
    path_find: path_find
};

fn default_opts() -> search_opts {
    { branch_factor: branch_factor() /* 1 */, verbose: false,
      killable: true,   max_depth: 10 /*uint::max_value*/,
      path_find: path_find::brushfire::mk() }
}
fn default_opts_verbose(verbose: bool) -> search_opts {
    { verbose: verbose with default_opts() }
}
fn default_opts_bfac(bf: uint) -> search_opts {
    { branch_factor: bf with default_opts() }
}

// Repeatedly finds lambdas (hopefully).
// TODO: bblum: add a 'int how_hungry' param; -1 for play until end.
fn greedy_finish(-s: state::state, o: search_opts) -> search_result {
    // Test for time run out. TODO: Maybe check if it's save to finish greedy
    if signal::signal_received() && o.killable {
        let score = s.score;
        ret (dvec::from_elem(state::A), s, score);
    }
    // Attempt to do something next.
    let thunk = path_find::brushfire::mk().get_paths(s);
    let result = thunk();
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
    let pathlist = o.path_find.get_paths(s);
    // Test for time run out.
    if signal::signal_received() && o.killable {
        let score = s.score;
        ret (dvec::from_elem(state::A), s, score);
    }
    // Test for horizon node.
    if depth == 0 {
        ret greedy_finish(s, o);
    } else {
        // Iterate over possibilities.
        // TODO: maybe prune later?
        for iter::repeat(o.branch_factor) {
            let target_opt = pathlist();
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
            task::yield();
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

#[always_inline]
fn score_result(r: search_result) -> int { alt r { (_,_,s) { s } } }

fn iterative_search(-s: state::state, o: search_opts) -> search_result {
    let mut depth = 1;
    let mut best_result = greedy_finish(copy s, { killable: false with o });
    // Loop until (A) Reach maximum specified depth, (B) signalled
    while depth <= o.max_depth && !(signal::signal_received() && o.killable) {
        #error["SEARCH: Searching depth %u; best so far %d",
             depth, score_result(best_result)];
        // Search.
        let result = search(copy s, depth, o);
        // Interpret findings.
        if (score_result(result) > score_result(best_result)) {
            #error["SEARCH: Found new best %d", score_result(result)];
            best_result = result;
        } else if (score_result(result) == score_result(best_result)) {
            #error["SEARCH: Nothing new"];
        } else {
            #error["SEARCH: Worse..?"];
        }
        depth += 1;
        task::yield();
    }
    best_result
}

impl of game_tree for search_opts {
    fn get_path(+s: state::state) -> ~[state::move] {
        let (moves_rev, _endstate, _score) = iterative_search(s, self);
        let moves = dvec::unwrap(moves_rev);
        vec::reverse(moves);
        vec::from_mut(moves)
    }
}

fn mk(o: search_opts) -> game_tree {
    o as game_tree
}

mod test {
    #[test]
    fn test_play_game_check_hash() {
        let s = #include_str("../maps/contest10.map");
        let mut s = state::read_board(io::str_reader(s));
        let mut thunk = path_find::brushfire::mk().get_paths(s);
        let mut result = thunk();
        while result != none {
            let (newstate, _path) = option::unwrap(result);
            assert newstate.hash() == newstate.rehash();
            s = newstate;
            thunk = path_find::brushfire::mk().get_paths(s);
            result = thunk();
        }
    }
    #[test]
    fn test_zero_depth_equals_greedy() {
        let s = #include_str("../maps/contest10.map");
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
        let s = #include_str("../maps/contest10.map");
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
    fn test_search_beats_greedy() {
        // 5 seems to be the min branch depth. Guess we find it on the 5th
        // closest lambda.
        test_search_vs_greedy(#include_str("../maps/contest5.map"), 1, 5);
        test_search_vs_greedy(#include_str("../maps/contest5.map"), 2, 5);
        test_search_vs_greedy(#include_str("../maps/contest5.map"), 3, 5);
    }
}
