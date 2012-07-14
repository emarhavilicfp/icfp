// Path generation.

import state;
import state::extensions;
import dvec;
import vec;
import vec::extensions;


type boundary_element = (state::coord, state::move);
type path_state = (~[~[mut (bool, option<state::move>)]], ~[boundary_element]);
type path = ~[state::move];




/* XXX: This has a lot of copy.  It would be nice if we could have fewer copy. */
fn apply(p: path, st: state::state, strict: bool) -> state::step_result {
    let mut st_ = copy st;
    for p.each |the_move| {
            alt st_.step(the_move, strict) {
              state::stepped(st__) {
                st_ = copy st__;
              }
              res { ret copy res }
          }
    }
    ret state::stepped(st_);
}

fn genpaths(b: state::grid, src: state::coord,
            dests: ~[state::coord]) -> (option<path>, path_state) {
    let (x, y) = src;
    let visited: ~[~[mut (bool, option<state::move>)]] = ~[~[mut]];
    visited[x][y] = (true, some(state::W));
    let mut condition: option<state::coord> = none;
    let mut boundary = ~[(src, state::W)];
    while condition == none {
        boundary = propagate(b, boundary, visited);
        condition = winner(dests, visited);
        if (boundary.len() == 0) {
            //shit's fucked (no reachable)
            ret (none, (visited, boundary));
        }
    }
    alt copy condition {
      some(p) { ret (some(build_path(p, visited)), (visited, boundary)); }
      none {fail}
    }
}

fn genpath_restart(b: state::grid, src: state::coord,
                   dests: ~[state::coord], v: ~[~[mut (bool, option<state::move>)]],
                   bound: ~[boundary_element]) -> (option<path>, path_state) {
    let mut visited = copy v;
    let mut boundary = bound;
    let (x, y) = src;
    visited[x][y] = (true, some(state::W));
    let mut condition: option<state::coord> = none;
    while condition == none {
        boundary = propagate(b, boundary, visited);
        condition = winner(dests, visited);
        if (boundary.len() == 0) {
            //shit's fucked (no reachable)
            ret (none, (visited, boundary));
        }
    }
    alt copy condition {
      some(p) { ret (some(build_path(p, visited)), (visited, boundary)); }
      none {fail}
    }
    
}

fn build_path(p: state::coord, visited: ~[~[mut (bool, option<state::move>)]]) -> path {
    let (x, y) = p;
    alt visited[x][y] {
      (false, _) {fail}
      (_, some(state::W)) {ret ~[];}
      (_, some(state::U)) {ret vec::append_one(build_path((x,y-1), visited), state::U);}
      (_, some(state::L)) {ret vec::append_one(build_path((x+1,y), visited), state::L);}
      (_, some(state::R)) {ret vec::append_one(build_path((x-1,y), visited), state::R);}
      (_, some(state::D)) {ret vec::append_one(build_path((x,y+1), visited), state::D);}
      (_, _) {fail}
    };
}

fn winner(dests: ~[state::coord],
          visited: ~[~[mut (bool, option<state::move>)]]) -> option<state::coord> {
    for dests.each() |p| {
        let (x, y) = p;
        let (cond, _move) = visited[x][y];
        if cond {
            ret some(p);
        }
    }
    none
}

fn propagate(b: state::grid, boundary_list: ~[boundary_element],
             visited: ~[~[mut (bool, option<state::move>)]]) -> ~[boundary_element] {
    let ret_list: ~[boundary_element] = ~[];
    for boundary_list.each() |end| {
        let (p, _) = end;
        for get_empty_neighbors(p, b).each() |t| {
            let (neighbor, m) = t;
            alt neighbor {
              (x, y) {
                let (cond, _move) = visited[x][y];
                if !cond {
                    vec::append_one(ret_list, (neighbor, m));
                    visited[x][y] = (true, some(m));
                }
              }
            }
        }
    }
    ret_list
}

fn get_square(p: state::coord, b: state::grid) -> state::square {
    alt p {
      (x, y) { b[x][y] }
    }
}

fn get_empty_neighbors(p: state::coord,
                       b: state::grid) -> ~[(state::coord, state::move)] {
    vec::filter(get_neighbors(p), |t|{let (l, _) = t; get_square(l, b) == state::empty})
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
    let state = grid::read_board(io::str_reader(#include_str("./maps/flood1.map")));
    let p = genpath(state.grid,(7,6),(2,6));
    assert (p.len() == 13);
}
