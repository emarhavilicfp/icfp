// Path generation.

import state;
import state::extensions;

export path;

trait path {
    // Return the number of moves in the path.
    fn len() -> uint;
    fn pop() -> option<state::move>;
}

/* XXX: This has a lot of copy.  It would be nice if we could have fewer copy. */
fn apply(p: path, st: state::state, strict: bool) -> state::step_result {
    let mut st_ = copy st;
    loop {
        alt p.pop() {
          none { ret state::stepped(st_) }
          some(move) {
            alt st_.step(move, strict) {
              state::stepped(st__) {
                st_ = copy st__;
              }
              res { ret copy res }
            }
          }
        }
    }
}

fn genpath(_b: state::grid, _dest: state::coord, _src: state::coord) -> path {
    // TODO: implement
    fail;
}
