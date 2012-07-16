use std;

import std::map;
import std::map::*;

import not_path = core::path;
import path;
import state;
import state::extensions;
import heuristics::*;

import path::target;

import dlist;
import dlist::extensions;

enum t {_dummy}
impl of path_find for t {
    fn get_paths(s: state::state) ->
            (fn @() -> option<(state::state, path::path)>) {

        let targets = dlist::create::<(uint, state::coord)>();
        for s.grid.lambdas().each |target| {
            pqins(targets, target, state::taxicab_distance(s.robotpos, target));
        }
        //let targets = dlist::from_vec(s.grid.lambdas().map(|x| (1, x));

        fn @() -> option<(state::state, path::path)> {
            loop {
                if (targets.is_empty()) {
                    ret none;
                }

                let (_, (x, y)) = option::unwrap(targets.pop());
                let result = navigar(s, (x, y));
                if (result.is_none()) {
                    again;
                }

                ret result;
            }
        }
    }
}

fn mk() -> path_find {
    let t = _dummy;
    t as path_find
}

// BUG: add this to vec.rs
pure fn map_mut<T, U>(v: &[T], f: fn(T) -> U) -> ~[mut U] {
    let mut result = ~[mut]; 
    unchecked{vec::reserve(result, vec::len(v));}
    for vec::each(v) |elem| { unsafe { vec::push(result, f(elem)); } }
    ret result;
}

type astarhelp = (state::coord, state::move, path::path, uint);
type pq<T> = dlist::dlist<(uint, T)>;

fn pqins<T: copy>(pq: pq<T>, troll: T, prio: uint) {
    let mut link = pq.peek_n();
    while link.is_some() {
        let neighbor = link.get();
        if (tuple::first(neighbor.data) >= prio) {
            pq.insert_after((prio,troll), neighbor);
            ret;
        }
        link = neighbor.next_link();
    }
    // worst lambda found so far. insert at tail
    pq.push((prio, troll));
}

fn navigar(s: state::state, dest: state::coord) -> option<(state::state, path::path)> {
    fn mk_hasher(s: state::state) -> (fn@(state::coord) -> uint) {
        let len = s.grid.grid.len();
        fn @(c:state::coord) -> uint {
            let (x, y) = c;
            (y * len) + x
        }
    };
    fn eqcoord(a: state::coord, b: state::coord) -> bool {
        a == b
    }
    fn mk_cost(s: state::state) -> (fn@(state::coord, state::coord, state::move) -> uint) {
        let h = s.grid.grid.len();
        fn @(a:state::coord, b:state::coord, m:state::move, copy s) -> uint {
            let mut c;
            /* TODO lift this somewhere and better heuristics */
            alt s.grid.at(b) {
                state::earth |
                state::empty { c = 5 }
                state::razor { c = 4 }
                state::lambda { c = 3 }
                _ { ret 0; }
            }

            let (ax, ay) = a;
            if (ay < h &&
                s.grid.at((ax, ay + 1)) == state::rock) {
                /* We are moving out from under a boulder */
                alt m {
                    state::D { /*Death*/ret 0; }
                    _ { c += 5; }
                }
            }

            ret c;
        }
    }

    fn distance(a: state::coord, b: state::coord)-> uint {
        let (ax, ay) = a;
        let (bx, by) = b;

        let (cx, cy) : (uint, uint);

        if (ax > bx) {
            cx = ax - bx;
        } else {
            cx = bx - ax;
        }

        if (ay > by) {
            cy = ay - by;
        } else {
            cy = by - ay;
        }

        cx + cy
    }

    fn state_apply(_s: state::state, _ml: path::path) -> option<state::state> {
        alt path::apply(_ml, _s, false) {
            state::stepped(state) { some(state::extract_step_result(state)) }
            state::endgame(*) | state::oops(_) { none } // XXX fix endgame
        }
    }

    let mut visited = map::hashmap(mk_hasher(s), eqcoord);
    let mut pq = dlist::create::<(uint, astarhelp)>();
    let cost = mk_cost(s);
    pqins(pq, (s.robotpos, state::W, ~[], 0), 1);

    while (!pq.is_empty()) {
        let mut (_, (spot, m, path, oldcost)) = option::unwrap(pq.pop());
        if (visited.contains_key(spot)) {
            again;
        }

        vec::push(path, m);
        if (spot == dest) {
            let mut newstate = state_apply(s, path);
            if (newstate.is_some()) {
                ret some((option::unwrap(newstate), path));
            } else {
                /* REALLY try */
                newstate = state_apply(s, path);
                if (newstate.is_some()) {
                    ret some((option::unwrap(newstate), path));
                }
            }
            again;
        }

        map::set_add(visited, spot);

        let spot_;
        alt s.grid.at(spot) {
            //TODO: Make trampoline access more sane
            state::trampoline(t) {
                let pad = s.tramp_map[t];
                let mut maybe = none;
                do s.grid.squares_i |sq, i| {
                   if (sq == state::target(pad)) {
                       maybe = some(i);
                   }
                }
                spot_ = option::unwrap(maybe);
            }
            _ { spot_ = spot }
        }

        for s.grid.neighbors_of(spot_) |n, m_| {
            if (visited.contains_key(n)) {
                again;
            }

            let tempcost = cost(spot_, n, m_);
            if (tempcost == 0) {
                again;
            }

            let newcost = oldcost + tempcost;
            pqins(pq, (n, m_, path, newcost), newcost + distance(dest, n));
        }
    }
    ret none;
}

#[test]
fn test_pull_my_thunk() {
    let state = state::read_board(io::str_reader(#include_str("../maps/contest1.map")));
    let count = state.lambdasleft;
    let astar = mk();
    let thunk = astar.get_paths(state);
    let mut i = 1;
    while(thunk().is_some()) {
    }
}
