// Path generation.

import board;

export path;

trait path {
    // Return the number of moves in the path.
    fn len() -> uint;
}

fn genpath(_b: board::grid, _dest: board::coord, _src: board::coord) -> path {
    // TODO: implement
    fail;
}
