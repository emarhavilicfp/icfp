import path;
import state;
import state::*;
import heuristics::*;
import dvec;
import dvec::extensions;
import signal;
import path_find;
import path_find::*;
import evaluate::evaluate;

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
    
    while !signal::signal_received() && depth < 50 /* We're probably not getting much better from there. */ {
    	#debug["tobruos: here we go with depth %u", depth];
        /* Do a top-level search on an increasing depth. */
        let (path, score) = get_path_depth(opts, copy s, depth);
        if score > bestscore {
            bestpath = path;
            bestscore = score;
        }
        #debug["tobruos: depth %u done; best score %d (this: %d)", depth, bestscore, score];
        depth = depth+1;
        task::yield();
    }
    
    bestpath
}

fn get_path_depth(opts: settings, +s: state::state, depth: uint) -> (~[state::move], int) {
    /* Now just keep getting the best choice, and sticking it on the end. */
    let mut fullpath = ~[];
    let mut state = s;
    #debug["tobruos: trying with path depth %u", depth];
    while !signal::signal_received() {
        #debug["tobruos:    trying to make the next move..."];
        let result = get_best_top_option(opts, state, depth);
        let (path, newst) = alt result {
          some((p, ns)) { (p, ns) }
          none { break }
        };
        fullpath = vec::append(fullpath, path);
        #debug["tobruos:    came back with a best score of %d, path length of %u", state.score, fullpath.len()];
        state = newst;
        task::yield();
    }
    (fullpath, state.score)
}

/* Given a state, given the best top-level choice we can make, looking depth steps ahead. */
fn get_best_top_option(opts: settings, s: state::state, depth: uint) -> option<(~[state::move], state::state)> {
    let mut bestpath = none;
    let mut bestscore = 0;
    
    let mut paththunks = ~[];
    
    /* Prime it with a path thunk to pull on. */
    let rootthunk = opts.path_find.get_paths(s);
    loop {
        let curroot = rootthunk();
        let (st, rootpath) = alt curroot {
          some(r) { r }
          none { break }
        };
        #debug["tobruos:      root thunk returned a state of score %d", st.score];
        let mut scores : i64 = evaluate(st) as i64;
        let mut paths : i64 = 1;
        let mut bestlocal : i64 = 0;
        
        let localeval = evaluate(st);
        if localeval as i64 > bestlocal {
            bestlocal = localeval as i64;
        }
        
        /* We have a root -- start off with the traversal node for that root. */
        vec::push(paththunks, opts.path_find.get_paths(st));
        while paththunks.len() != 0 && !signal::signal_received() {
            /* Pull on the thunk. */
            alt paththunks[paththunks.len() - 1]() {
              none {
                /* I'm sorry, did I break your concentration? ... Oh, you were finished?  Well, allow me to retort. */
                vec::pop(paththunks);
              }
              some((news, path)) {
                if paththunks.len() < depth { /* Say what again. */
                    vec::push(paththunks, opts.path_find.get_paths(news)); 
                }
                let newseval = evaluate(news);
                scores = scores + newseval as i64;
                paths = paths + 1;
                
                if newseval as i64 > bestlocal { /* Then you know what I'm sayin'! */
                    bestlocal = newseval as i64;
                }
              }
            }
        }
        
        /* A better scoring mechanism here would be nice, too. */
        let rootscore = scores / paths + bestlocal * 3;
        if rootscore > bestscore {
            bestscore = rootscore;
            bestpath = some((rootpath, st));
        }
    }
    
    bestpath
}

fn mk(o: path_find) -> game_tree {
    {path_find: o} as game_tree
}

