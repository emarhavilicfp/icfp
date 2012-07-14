// Path generation.

import state;

export path;

trait path {
    // Return the number of moves in the path.
    fn len() -> uint;
}

fn genpath(_b: state::grid, _dest: state::coord, _src: state::coord) -> path {
    // TODO: implement
    fail;
}
