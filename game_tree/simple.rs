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
        let mut result = self.path_find.get_paths(s)();
        let mut fullpath = ~[];
        while result != none {
            let (newstate, path) = option::unwrap(result);
            fullpath = vec::append(fullpath, path);
            result = self.path_find.get_paths(newstate)()
        }
        fullpath
    }
}

fn mk(o: path_find) -> game_tree {
    {path_find: o} as game_tree
}

