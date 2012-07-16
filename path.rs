// Path generation.

import either::{either, left, right};

import state;
import state::{extensions, coord};
import dvec;
import vec;
import vec::extensions;
import targets::target;
import targets::viaThing;

type boundary_element = (state::coord, ~[state::move]);
type path_state = (~[~[mut (bool, ~[state::move])]], ~[boundary_element]);
type path = ~[state::move];

macro_rules! move {
    { $x:expr } => { unsafe { let y <- *ptr::addr_of($x); y } }
}


/// This is lambda-only implementation of target.
impl of target for state::coord {
    pure fn coord() -> coord { self }
    fn score() -> int { 25 }
    pure fn traverse() -> (coord, path) { (self, ~[]) }
}

/// How to combine targets
impl<L: target, R: target> of target for either<L, R> {
    pure fn coord() -> coord {
        alt self {
          left(x) { x.coord() }
          right(x) { x.coord() }
        }
    }

    fn score() -> int {
        alt self {
          left(x) { x.score() }
          right(x) { x.score() }
        }
    }

    pure fn traverse() -> (coord, path) {
        alt self {
          left(x) { x.traverse() }
          right(x) { x.traverse() }
        }
    }
}



fn apply(p: path, +st: state::state, strict: bool) -> state::step_result {
    let mut st_ <- st;
    for p.each |the_move| {
            alt st_.step(the_move, strict) {
              state::stepped(st__) {
                st_ = state::extract_step_result(st__);
              }
              state::endgame(score) { ret state::endgame(score) }
              state::oops(s_) { ret state::oops(s_) }
          }
    }
    ret state::stepped(@mut some(st_));
}

fn genpaths<T: copy target>(b: state::grid, src: state::coord,
             targets: &[const option<T>], vias:&[mut option<viaThing>],
             safe: bool)
             -> (option<(path, T)>, path_state) {
    let (x, y) = src;
    let mut visited: ~[~[mut(bool, ~[state::move])]] = ~[];
    for iter::repeat(b.grid.len()) {
        vec::push(visited, vec::from_elem(b.grid[0].len(), (false, ~[])));
    }
    visited[y-1][x-1] = (true, ~[state::W]);
    //let mut condition: option<state::coord> = none;
    let mut boundary = ~[(src, ~[state::W])];
    genpath_restart(b, src, targets, vias, visited, boundary, safe)

}

fn genpath_restart<T: copy target>
    (b: state::grid, src: state::coord,
     dests: &[const option<T>], vias: &[mut option<viaThing>],
     +v: ~[~[mut (bool, ~[state::move])]],
     bound: ~[boundary_element],
     safe: bool)
    -> (option<(path, T)>, path_state)
{
    let mut visited = v;
    let mut boundary = bound;
    let (x, y) = src;
    visited[y-1][x-1] = (true, ~[state::W]);
    let mut condition = none;
    while condition == none {
        boundary = propagate(b, boundary, visited, vias, safe);
        condition = winner(dests, visited);
        if (boundary.len() == 0) {
            //shit's fucked (no reachable)
            ret (none, (visited, boundary));
        }
    }
    alt copy condition {
      some(i) {
        let p = option::get(dests[i]);
        let (c,path) = p.traverse();
        let nubPath = build_path(p, visited);
        let finalPath = vec::append(nubPath, path);
        ret (some((finalPath, p)), (visited, boundary)); }
      none {fail}
    }
}

fn build_path<T: target>(+p: T,
                         visited: ~[~[mut (bool, ~[state::move])]]) -> path {
    //TODO(tony): handle trampolines.
    let (x, y) = p.coord();
    alt visited[y-1][x-1] {
      (false, _) {fail}
      (_, l) {
        if l == ~[] {
            fail
        }
        else if l == ~[state::W] {
            ret ~[];
        }
        else {
            let (dx, dy) = compute_delta(l);
            let lstack = copy l;
            ret vec::append(build_path((x-dx, y-dy), visited), lstack);
        }
      }
    };
}

pure fn compute_delta(l: ~[state::move]) -> (uint, uint) {
    let delta_x = vec::count(l, state::R) - vec::count(l, state::L);
    let delta_y = vec::count(l, state::U) - vec::count(l, state::D);
    (delta_x, delta_y)
}

#[inline(always)]
fn invert_move(m: state::move) -> state::move {
    alt m {
      state::U {state::D}
      state::D {state::U}
      state::L {state::R}
      state::R {state::L}
      state::Tramp(x,y) {state::Tramp(-x,-y)}
      // This last one shouldn't happen.
      x        {x}
    }
}

fn build_path_backwards(p: state::coord,
              visited: ~[~[mut (bool, ~[state::move])]]) -> path {
    vec::map(build_path(p, visited), invert_move)
}

fn winner<T: target>(dests: &[const option<T>],
          visited: ~[~[mut (bool, ~[state::move] )]])
    -> option<uint>
{
    for dests.eachi() |i, o| {
        alt o {
          some(p) {
            let (x, y) = p.coord();
            //#error("%? %? %?", (visited.len(), visited[0].len()),
            //       (y - 1, x - 1),
            //       (y, x));
            let (cond, _moves) = visited[y-1][x-1];
            if cond {
                ret some(i);
            }
          }
          none { again; }
        }
    }
    none
}

fn propagate(b: state::grid, boundary_list: ~[boundary_element],
             visited: ~[~[mut (bool, ~[state::move])]], 
             vias: &[mut option<viaThing>], safe: bool)
    -> ~[boundary_element]
{
    let mut ret_list: ~[boundary_element] = ~[];
    for boundary_list.each() |end| {
        let (p, _) = end;
        for get_passable_neighbors(p, b, safe).each() |t| {
            let (neighbor, m) = t;
            let (x, y) = neighbor;
            let (cond, _moves) = visited[y-1][x-1];
            if !cond {
                let mut got_one: bool = false;
                visited[y-1][x-1] = (true, ~[m]);
                if !safe {
                    for vias.eachi() |i, o| {
                        alt o{
                          some(via) {
                            if via.coord() == neighbor {
                                let (c, m) = via.traverse();
                                let (x2, y2) = c;
                                visited[y2 -1][x2 -1] = (true, m);
                                vias[i] = none;
                                ret_list += ~[(c, m)];
                                got_one = true;
                                break;
                            }
                          }
                          none { again; }
                        }
                    }
                }
                if !got_one {
                    ret_list += ~[(neighbor, ~[m])];
                }
            }
        }
    }
    ret_list
}

fn get_square(p: state::coord, b: state::grid) -> state::square {
    b.at(p)
}

fn get_passable_neighbors(p: state::coord,
                          b: state::grid,
                          safe: bool)
    -> ~[(state::coord, state::move)]
{
    vec::filter(get_neighbors(p), |t| {
        let (l, _) = t;
        alt get_square(l, b) {
          state::lambda | state::razor { true }
          state::target(fuck) { true }
          state::earth | state::empty {
            let upCoord = l.up();
            if b.in(upCoord) {
                alt get_square(upCoord, b) {
                    state::rock { !safe }
                    _ { true }
                }
            } else { true }
          }
          state::trampoline(fuck) { !safe }
          _ { false }
        }})
}

fn get_neighbors(p: state::coord) -> ~[(state::coord, state::move)] {
    alt p {
      (x, y) {
        ~[((x+1, y), state::R), ((x, y+1), state::U), ((x-1, y), state::L),
          ((x, y-1), state::D)]
      }
    }
}

#[cfg(test)]
fn test_a_path(state: state::state, src: state::coord,
               dests: ~[option<state::coord>], expected_len: uint) {
    import state::*;
    import vec::*;
    let state = state::read_board(io::str_reader(#include_str("./maps/flood1.map")));
    let (p, _) = genpaths(state.grid,(6u,7u),~[some((6u,2u))], true);
    assert p.is_some();
    let tuple = option::get(p);
    alt tuple {
      (list, _) {
        let len = list.len();
        assert len == expected_len;
      }
    }
}


#[test]
fn test_genpath() {
    import state::state;
    import state::read_board;
    let state = state::read_board(io::str_reader(#include_str("./maps/flood1.map")));
    test_a_path(state, (6u, 7u), ~[some((6u, 2u))], 13)
}

#[test]
#[ignore]
fn test_aggressive_pattern () {
    import state::state;
    import state::read_board;
    let state = state::read_board(io::str_reader(#include_str("./maps/pattern_test.map")));
    test_a_path(state , (4u, 3u), ~[some((3u, 3u))], 8u)
}

#[test]
#[ignore]
fn test_trampoline () {
    /* HEY YOU
    Yeah, you, removing the #[ignore, listen the fuck up.
    you need to figure out how in the devil the trampoline moves are being stored.
    in particular, what the length of the resultant path will be. Then change that
    999u to something sane.
    */
    import state::state;
    import state::read_board;
    let state = state::read_board(io::str_reader(#include_str("./maps/trampoline_test.map")));
    test_a_path(state, (2u, 4u), ~[some((4u, 4u))], 999u)
}
