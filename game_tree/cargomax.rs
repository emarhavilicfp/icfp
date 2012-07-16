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



// fixed version of the one from vec
fn vec_view<T,U>(v: &[T], start: uint, end: uint,
                      blk: fn([T]/&) -> U) -> U {
    assert (start <= end);
    assert (end <= vec::len(v));
    let mut slice = do vec::unpack_slice(v) |p, _len| {
        unsafe {
            ::unsafe::reinterpret_cast(
                (ptr::offset(p, start), (end - start) * sys::size_of::<T>()))
        }
    };
    blk(slice)
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
    mut move_stack: @move_stack, // Builds in forward order (push@tail)
    mut work_list: worklist,
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
      mut move_stack: @dvec::dvec(),
      mut work_list: empty_worklist() }
}
fn default_opts_verbose(verbose: bool) -> search_opts {
    { verbose: verbose with default_opts() }
}
fn default_opts_bfac(bf: uint) -> search_opts {
    { branch_factor: bf with default_opts() }
}

// Moves from the horizon to the max point. Estimated score. REVERSED.
type greedie = @dvec::dvec<(path::path,int)>;
fn empty_greedie() -> greedie { @dvec::dvec() }

fn done_searching(-s: state::state) -> search_result {
    let score = s.score;
    (dvec::dvec(), s, score)
}

// Repeatedly finds lambdas (hopefully).
// TODO: bblum: add a 'int how_hungry' param; -1 for play until end.
fn greedy_finish(-s: state::state, work_item: greedie, o: search_opts)
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
        let eval_score = evaluate::evaluate(newstate);
        if o.verbose {
            io::println("Pursuing path of " +
                        str::concat(vec::map(path, |i| { i.to_str() })));
        }
        // Find what to do next.
        // stack_path(path, o); No need.
        let (finishing_moves,endstate,score) =
            greedy_finish(newstate, work_item, o);
        // unstack_path(o); No need.
        work_item.push((copy path, eval_score));
        // Do these moves "before". Must happen after copying path above.
        add_path_prefix(finishing_moves, path);
        (finishing_moves, endstate, score)
    } else {
        // All done.
        // work_item.push((~[], evaluate::evaluate(s))); No.
        done_searching(s)
    }
}

fn greedy_all(-s: state::state, o: search_opts) -> search_result {
    greedy_finish(s, empty_greedie(), o)
}

type work = {
    prefix: @~[path::path], // Forward order. Taken from the stack.
    suffix: @mut ~[path::path], //Forward order. Taken from the greedy.
    eval: int // Estimated score
};
// global thing
type worklist = dlist::dlist<work>;
fn init_worklist() -> worklist {
    dlist::from_elt({prefix: @~[], suffix: @mut ~[], eval: int::min_value})
}
fn empty_worklist() -> worklist { dlist::create() }

// Premature thingies, to be turned into work. Proto-work.
// It is a list of greedies. A greedie is a list of the intermediate
// paths and the evaluation score at that path.
type work_item_list = dvec::dvec<greedie>;
// TODO: bblum: compute max-max-pos |--->max here--> | no max allowed | end

// This is the brain of cargomax.
fn add_work(-x: work_item_list, greed_depth_total: uint, o: search_opts) {
    let total_greedies = x.len();
    assert total_greedies <= o.branch_factor;
    let avg = greed_depth_total / total_greedies;
    // not_avg is how far ahead in the greedy branch we are allowed to branch.
    // note that in some cases a greedie might be still shorter than this.
    let not_avg = heuristics::cargomax_munge_avg_depth(avg);
    // this is where we are in the tree. common to all greedies.
    let moves_to_horizon = @((*o.move_stack).get()); // prefix
    // Iterate over each greedie, adding a work for it.
    while x.len() > 0 {
        // FIXME maybe this logiccan be optimised not to copy the path.
        let greedie = x.pop();
        // Remember the greedie is backwards.
        // We care about [.......not_avg <--- this bit --> ]
        let len = greedie.len();
        let deepest = if not_avg > len { len } else { not_avg };
        // Look at the section
        let (eval,pos,_) =
            //do vec_view(greedie.data, len-deepest, len) |g| {
            do vec::foldr(greedie.data, (int::min_value,len,len)) |greedo, accum| {
                let (best_eval, best_pos, last_pos) = accum;
                if last_pos >= len-deepest {
                    let (_,eval) = greedo;
                    if (eval > best_eval) { // In a tie, choose shallower.
                        (eval, last_pos-1, last_pos-1)
                    } else {
                        (best_eval, best_pos, last_pos-1)
                    }
                } else { accum }
            };
            //};
        assert pos < len; // Should have found something.
        assert pos >= 0;
        assert tuple::second(greedie[pos]) == eval; // Stereo 8-bit foundness.
        // Now pos is the index into the vector of the best eval.
        // Build suffix forwards from backwards greedie.
        let suffix = @mut ~[];
        vec::reserve(*suffix, len-pos);
        let mut i = len-1;
        //#error["%u", i];
        while (i >= pos) {
            assert i >= 0; assert i < len;
            vec::push(*suffix, tuple::first(greedie[i]));
            if i > 0 { i -= 1; } else { break }
        }
        // Insert work.
        let work: work =
            { prefix: moves_to_horizon, suffix: suffix, eval: eval };
        //#error["CARGOMAX: adding work with score %d", eval];
        if heuristics::worklist_sorted {
            fail; // TODO: implement
        } else {
            unsafe { o.work_list.push(work); } // Fix a borrow error.
        }
    }
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
            let eval_score = evaluate::evaluate(newstate);
            let work_item = empty_greedie();
            // Recurse.
            stack_path(path, o);
            let (finishing_moves,endstate,this_score) =
                greedy_finish(newstate, work_item, o);
            unstack_path(o);
            // Collect statistics.
            greed_depth_total += finishing_moves.len();
            // Minding the gap
            work_item.push((copy path, eval_score));
            work_items.push(work_item);
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

type cargomax = (search_result, (@~[path::path],@mut ~[path::path]));

//returns a prefix too
fn process_work(-s_: state::state, w: work, o: search_opts) -> cargomax {
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
    task::yield();
    (result,(w.prefix,w.suffix))
}

fn run_workqueue(-s: state::state, o: search_opts) -> cargomax {
    let mut worklist = init_worklist();
    let mut best_result =
        (greedy_all(copy s, { killable: false with o }),(@~[],@mut~[]));
    while !worklist.is_empty() && !(signal::signal_received() && o.killable) {
        let work: work = worklist.pop().get();
        //#error["CARGOMAX: Processing work @ depth %u with estimated score %d",
               //work.prefix.len() + work.suffix.len(), work.eval];
        let result = process_work(copy s, work, o);
        if (score_result(result) > score_result(best_result)) {
            //#error["CARGOMAX: Found new best %d", score_result(result)];
            best_result = result;
        }
        // The searcher builds the worklist for the next depth, stratified.
        // This could maybe be better if different. FIXME try it?
        if worklist.is_empty() {
            worklist <-> o.work_list;
            //#error["CARGOMAX: Advancing depth; best so far %d",
                   //score_result(best_result)];
        }
    }
    best_result
}

#[always_inline]
fn score_result(r: cargomax) -> int {
    alt r { ((_,_,s),_) { s } }
}

impl of game_tree for search_opts {
    fn get_path(+s: state::state) -> ~[state::move] {
        let ((moves_rev, _, _),(prefix,suffix)) = run_workqueue(s, self);
        let mut moves = vec::concat(*prefix);
        vec::push_all(moves, vec::concat(*suffix));
        let paths = dvec::unwrap(moves_rev);
        vec::reverse(paths);
        //#error["CARGOMAX: # of path chunks %u", paths.len()];
        for paths.each |path| {
            //#error["CARGOMAX: path chunk len %u", path.len()];
            vec::push_all(moves, path);
        }
        vec::push(moves, state::A);
        moves
    }
}

fn mk(o: search_opts) -> game_tree {
    (copy o) as game_tree
}
