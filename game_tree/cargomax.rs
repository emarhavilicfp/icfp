// Main robot driver.

import path;
import state;
import state::*;
import heuristics::*;
import dvec;
import dvec::extensions;
import dlist;
import dlist::extensions;
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
type search_result = (dvec::dvec<path::path>, state::state, int);

fn add_path_prefix(finishing_moves: dvec::dvec<path::path>, p: path::path) {
    finishing_moves.push(p);
}

type move_stack = dvec::dvec<path::path>;
type search_opts = {
    branch_factor: uint,
    verbose: bool,
    killable: bool,
    max_depth: uint,
    path_find: path_find,
    mut move_stack: @move_stack // Builds in forward order (push@tail)
};

fn stack_path(p: path::path, o: search_opts) {
    (*o.move_stack).push(p);
}
fn unstack_path(o: search_opts) {
    (*o.move_stack).pop();
}

fn default_opts() -> search_opts {
    { branch_factor: branch_factor() /* 1 */, verbose: false,
      killable: true,   max_depth: 10 /*uint::max_value*/,
      path_find: path_find::brushfire::mk(),
      mut move_stack: @dvec::dvec() }
}
fn default_opts_verbose(verbose: bool) -> search_opts {
    { verbose: verbose with default_opts() }
}
fn default_opts_bfac(bf: uint) -> search_opts {
    { branch_factor: bf with default_opts() }
}

// Moves from the horizon to the max point. Estimated score. REVERSED.
type greedy_result = @dvec::dvec<(path::path,int)>;
fn base_greedy_result() -> greedy_result { @dvec::dvec() }

fn done_searching(-s: state::state) -> search_result {
    let score = s.score;
    (dvec::dvec(), s, score)
}

// Repeatedly finds lambdas (hopefully).
// TODO: bblum: add a 'int how_hungry' param; -1 for play until end.
fn greedy_finish(-s: state::state, work_item: greedy_result, o: search_opts)
        -> search_result {
    // Test for time run out. TODO: Maybe check if it's save to finish greedy
    if signal::signal_received() && o.killable {
        ret done_searching(s);
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
        // stack_path(path, o); No need.
        let (finishing_moves,endstate,score) =
            greedy_finish(newstate, work_item, o);
        // unstack_path(o); No need.
        work_item.push((copy path, evaluate::evaluate(s)));
        // Do these moves "before". Must happen after copying path above.
        add_path_prefix(finishing_moves, path);
        (finishing_moves, endstate, score)
    } else {
        // All done.
        work_item.push((~[], evaluate::evaluate(s)));
        done_searching(s)
    }
}

fn greedy_all(-s: state::state, o: search_opts) -> search_result {
    greedy_finish(s, base_greedy_result(), o)
}

type work = {
    prefix: @~[path::path], // Forward order. Taken from the stack.
    suffix: @~[path::path], //Forward order. Taken from the greedy.
    eval: int // Estimated score
};
// global thing
type worklist = dlist::dlist<work>;
fn init_worklist() -> worklist {
    dlist::from_elt({prefix: @~[], suffix: @~[], eval: int::min_value})
}

// Premature things.
type work_item_list = dvec::dvec<(greedy_result,dvec::dvec<path::path>)>;
// TODO: bblum: compute max-max-pos |--->max here--> | no max allowed | end
fn add_work(-x: work_item_list, greed_depth_total: uint, o: search_opts) {
    while x.len() > 0 {
        let (eval_scores,moves) = x.pop();
        assert eval_scores.len() == moves.len();
    }
    let moves_to_horizon = @((*o.move_stack).get()); // prefix

    // remember to push not push_head
    fail
}

// Depth 1 search
fn search_horizon(-s: state::state, o: search_opts) -> search_result {
    let mut best = none;
    let mut best_score = none;
    let pathlist = o.path_find.get_paths(s);
    let work_items: work_item_list = dvec::dvec();
    let mut greed_depth_total = 0;
    // Test for time run out.
    if signal::signal_received() && o.killable {
        ret done_searching(s);
    }
    for iter::repeat(o.branch_factor) {
        let target_opt = pathlist();
        if target_opt.is_some() {
            let (newstate,path) = option::unwrap(target_opt);
            let work_item = base_greedy_result();
            // Recurse.
            stack_path(path, o);
            let (finishing_moves,endstate,this_score) =
                greedy_finish(newstate, work_item, o);
            unstack_path(o);
            // Collect statistics.
            greed_depth_total += finishing_moves.len();
            work_items.push((work_item,copy finishing_moves));
            // Update best.
            if best_score.is_none() || this_score > best_score.get() {
                // Prepend the moves we had (don't bother if not best)
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
        let result = option::unwrap(best);
        // Also add extra work to the queue.
        add_work(work_items, greed_depth_total, o);
        result
    } else {
        // Terminal state. Nothing to do.
        done_searching(s)
    }
}

fn process_work(-s_: state::state, w: work, o: search_opts) -> search_result {
    fn apply_known_path(-s: state::state, p: path::path) -> state::state {
        let res = brushfire::state_apply(s, p);
        assert res.is_some();
        option::unwrap(res)
    }
    let mut s <- some(s_);
    // Get past the old horizon.
    assert vec::len(o.move_stack.data) == 0;
    for w.prefix.each |path| {
        let mut shit = none;
        s <-> shit;
        s = some(apply_known_path(option::unwrap(shit), path));
        stack_path(path, o);
    }
    for w.suffix.each |path| {
        let mut shit = none;
        s <-> shit;
        s = some(apply_known_path(option::unwrap(shit), path));
        stack_path(path, o);
    }
    let result = search_horizon(option::unwrap(s), o);
    o.move_stack = @dvec::dvec();
    result
}

fn run_workqueue(-s: state::state, o: search_opts) -> search_result {
    let worklist = init_worklist();
    let mut best_result = greedy_all(copy s, { killable: false with o });
    while !worklist.is_empty() && !(signal::signal_received() && o.killable) {
        let work: work = worklist.pop().get();
        #error["SEARCH: Processing work at depth %u with estimated score %d",
               work.prefix.len() + work.suffix.len(), work.eval];
        let result = process_work(copy s, work, o);
        if (score_result(result) > score_result(best_result)) {
            #error["SEARCH: Found new best %d", score_result(result)];
            best_result = result;
        }
    }
    best_result
}

#[always_inline]
fn score_result(r: search_result) -> int { alt r { (_,_,s) { s } } }

impl of game_tree for search_opts {
    fn get_path(+s: state::state) -> ~[state::move] {
        let (moves_rev, _endstate, _score) = run_workqueue(s, self);
        let mut moves = ~[];
        let paths = dvec::unwrap(moves_rev);
        vec::reverse(paths);
        for paths.each |path| {
            vec::append(moves, path);
        }
        vec::push(moves, state::A);
        moves
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
        let (_, endstate, score) = greedy_all(copy s, default_opts());
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
        let (_, endstate, score) = greedy_all(copy s, default_opts());
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
        let (_, endstate, score) = greedy_all(copy s, default_opts());
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
