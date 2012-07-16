import path;
import state;
import state::*;
import heuristics::*;
import dvec;
import dvec::extensions;
import signal;
import path_find;
import path_find::*;

type settings = {
    path_find: path_find
};

impl of game_tree for settings {
    fn get_path(+s: state::state) -> ~[state::move] {
        get_path(self, s)
    }
}

fn get_path(opts: settings, +s: state::state) -> ~[state::move] {
    let mut bestpath = ~[];
    let mut bestscore = 0;
    let mut depth = 0u;
    
    while !signal::signal_received() {
    	#error["tobruos: here we go with depth %u", depth];
        /* Do a top-level search on an increasing depth. */
        let (path, score) = get_path_depth(opts, copy s, depth);
        if score > bestscore {
            bestpath = path;
            bestscore = score;
        }
        depth = depth+1;
    }
    
    bestpath
}

fn get_path_depth(opts: settings, +s: state::state, depth: uint) -> (~[state::move], int) {
    /* Now just keep getting the best choice, and sticking it on the end. */
    let mut result = get_best_top_option(opts, s, depth);
    let mut fullpath = ~[];
    let mut state = s;
    while result != none && !signal::signal_received() {
        let path = option::unwrap(result);
        fullpath = vec::append(fullpath, path);
        
        /* Come up with the new state. */
        for path.each |mv| {
            alt state.step(mv, false) {
              state::stepped(s) { state = state::extract_step_result(s) }
              state::endgame(_, pts) { ret (fullpath, pts) }
              _ { fail }
            }
        }
        result = get_best_top_option(opts, state, depth)
    }
    (fullpath, state.score)
}

/* Given a state, given the best top-level choice we can make, looking depth steps ahead. */
fn get_best_top_option(opts: settings, s: state::state, depth: uint) -> option<~[state::move]> {
    let mut bestpath = ~[];
    let mut bestscore = 0;
    
    let mut paththunks = ~[];
    let mut fullpath = ~[];
    
    /* Prime it with a path thunk to pull on. */
    vec::push(paththunks, opts.path_find.get_paths(s));
    vec::push(fullpath, ~[]);
    
    while paththunks.len() != 0 && !signal::signal_received() {
        /* Pull on the thunk. */
        alt paththunks[paththunks.len() - 1]() {
          none {
            /* I'm sorry, did I break your concentration? ... Oh, you were finished?  Well, allow me to retort. */
            vec::pop(paththunks);
            vec::pop(fullpath);
          }
          some((news, path)) {
            if paththunks.len() < depth { /* Say what again. */
                vec::push(fullpath, path);
                vec::push(paththunks, opts.path_find.get_paths(news)); 
            }
            
            if news.score > bestscore { /* Then you know what I'm sayin'! */
                bestpath = fullpath;
                bestscore = news.score;
            }
          }
        }
    }
    
    if bestpath.len() <= 1 {
        none
    } else { 
        some(bestpath[1])
    }
}

fn mk(o: path_find) -> game_tree {
    {path_find: o} as game_tree
}

