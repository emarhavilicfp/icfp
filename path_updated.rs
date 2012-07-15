// Path generation.

import state;
import state::{extensions, coord};
import dvec;
import vec;
import vec::extensions;


type boundary_element = (state::coord, ~[state::move]);
type path_state = (~[~[mut (bool, ~[state::move])]], ~[boundary_element]);
type path = ~[state::move];

macro_rules! move {
    { $x:expr } => { unsafe { let y <- *ptr::addr_of($x); y } }
}

/// A thing that the pathfinder will try to route through.
trait target {
    /// Where is this target?
    pure fn coord() -> coord;

    /// How valuable is this target?
    ///
    /// Can be negative if it costs you something to go here. An
    /// example is in patterns.
    fn score() -> int;

    /// Call this when you get to this target.
    ///
    /// Targets like lambdas don't move you, but some do. Examples are
    /// traversing patterns or trampolines.
    pure fn traverse() -> (coord, path);
}

/// This is lambda-only implementation of target.
impl of target for state::coord {
    pure fn coord() -> coord { self }
    fn score() -> int { 25 }
    pure fn traverse() -> (coord, path) { (self, ~[]) }
}

fn apply(p: path, st: state::state, strict: bool) -> state::step_result {
    // TODO One copy left. Even Ben couldn't figure out how to get rid of it.
    let mut st_ = copy st;
    for p.each |the_move| {
            alt st_.step(the_move, strict) {
              state::stepped(st__) {
                st_ = state::extract_step_result(st__);
              }
              state::endgame(score) { ret state::endgame(score) }
              state::oops { ret state::oops }
          }
    }
    ret state::stepped(@mut some(st_));
}

fn genpaths<T: copy target>(b: state::grid, src: state::coord, 
             dests: ~[option<T>]) 
             -> (option<(path,T)>, path_state) {
    let (x, y) = src;
    let mut visited: ~[~[mut(bool, ~[state::move])]] = ~[];
    for iter::repeat(b.grid.len()) {
        vec::push(visited, vec::from_elem(b.grid[0].len(), (false, ~[])));
    }
    visited[y-1][x-1] = (true, ~[state::W]);
    //let mut condition: option<state::coord> = none;
    let mut boundary = ~[(src, ~[state::W])];
    genpath_restart(b, src, dests, visited, boundary)

}


fn genpath_restart<T: copy target>
    (b: state::grid, src: state::coord,
     dests: ~[option<T>],
     +v: ~[~[mut (bool, ~[state::move])]],
     bound: ~[boundary_element]) 
    -> (option<(path, T)>, path_state)
{
    let mut visited = v;
    let mut boundary = bound;
    let (x, y) = src;
    visited[y-1][x-1] = (true, ~[state::W]);
    let mut condition = none;
    while condition == none {
        boundary = propagate(b, boundary, visited);
        condition = winner(dests, visited);
        if (boundary.len() == 0) {
            //shit's fucked (no reachable)
            ret (none, (visited, boundary));
        }
    }
    alt copy condition {
      some(p) { ret (some((build_path(p, visited), p)), (visited, boundary)); }
      none {fail}
    }
}

fn build_path<T: target>(p: T,
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
      // This last one shouldn't happen.
      x        {x}
    }
}

fn build_path_backwards(p: state::coord,
              visited: ~[~[mut (bool, ~[state::move])]]) -> path {
    vec::map(build_path(p, visited), invert_move)
}

fn winner<T: target>(dests: ~[option<T>],
                     visited: ~[~[mut (bool, ~[state::move] )]])
    -> option<T>
{
    for dests.each() |o| {
        alt o {
          some(p) {
            let (x, y) = p.coord();
            let (cond, _moves) = visited[y-1][x-1];
            if cond {
                ret some(move!{p});
            }
          }
          none { again; }
        }
    }
    none
}

fn propagate(b: state::grid, boundary_list: ~[boundary_element],
             visited: ~[~[mut (bool, ~[state::move])]]) -> ~[boundary_element] {
    let mut ret_list: ~[boundary_element] = ~[];
    for boundary_list.each() |end| {
        let (p, _) = end;
        for get_passable_neighbors(p, b).each() |t| {
            let (neighbor, m) = t;
            let (x, y) = neighbor;
            let (cond, _moves) = visited[y-1][x-1];
            if !cond {
                ret_list += ~[(neighbor, ~[m])];
                visited[y-1][x-1] = (true, ~[m]);
            }
        }
    }
    ret_list
}

fn get_square(p: state::coord, b: state::grid) -> state::square {
    b.at(p)
}

fn get_passable_neighbors(p: state::coord,
                       b: state::grid) -> ~[(state::coord, state::move)] {
    vec::filter(get_neighbors(p), |t| {
        let (l, _) = t;
        alt get_square(l, b) {
          state::empty | state::earth |
          state::lambda { true }
          _ { false}
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

#[test]
fn test_genpath() {
    import state::*;
    import vec::*;
    let state = state::read_board(io::str_reader(#include_str("./maps/flood1.map")));
    let (p, _) = genpaths(state.grid,(6u,7u),~[some((6u,2u))]);
    assert p.is_some();
    let tuple = option::get(p);
    alt tuple {
      (list, _) {
        let len = list.len();
        assert len == 13;
      }
    }
}
