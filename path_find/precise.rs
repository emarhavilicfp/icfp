import not_path = core::path;

import path;
import state::extensions;

enum u {_dummy(path_find)}
impl of path_find for u {
    #[warn(no_implicit_copys)]
    fn get_paths(s: state::state) ->
            (fn @() -> option<(state::state, path::path)>) {

        let get_paths_imp = (*self).get_paths(s);
        ret || {
            alt get_paths_imp() {
                some((_, path)) { self.handle_imp_path(s, path) }
                none { none }
            }
        }
    }

    fn handle_imp_path(s : state::state, imp_path : path::path)
        -> option<(state::state, path::path)>
    {
        let (applied_path, step_result) = path_apply(imp_path, copy s, true);
        alt step_result {
            state::stepped(s_) { some((state::extract_step_result(copy s_), imp_path)) }
            state::endgame(*) { none } // XXX maybe wrong?
            state::oops(s_) { 
                alt self.get_paths(*s_)() {
                    some((s__, p__)) { some((copy s__, applied_path + p__)) }
                    none { none }
                }
            }
        }
    }
}

fn path_apply(p: path::path, +st: state::state, strict: bool)
    -> (path::path, state::step_result)
{
    let mut st_ <- st;
    let mut moves = ~[];
    for p.each |the_move| {
            alt st_.step(the_move, strict) {
              state::stepped(st__) {
                st_ = state::extract_step_result(st__);
                vec::push(moves, the_move);
              }
              state::endgame(score) {
                  vec::push(moves, the_move);
                  ret (moves, state::endgame(score))
              }
              state::oops(s_) { ret (moves, state::oops(s_))  }
          }
    }
    ret (moves, state::stepped(@mut some(st_)));
}

fn mk(imprecise: path_find) -> path_find {
    let u = _dummy(imprecise);
    u as path_find
}
