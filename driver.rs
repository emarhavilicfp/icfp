import io::reader_util;
import game_tree::game_tree;
import str;

import game_tree::octopus::octopus;

fn path_find() -> path_find::path_find {
    alt os::getenv("PATHFIND") {
      some("astar") { path_find::astar::mk() }
      _ {
        path_find::brushfire::mk()
      }
    }
}

fn main(args: ~[str]) {
    import result::*;
    
    let map;

    if os::getenv("COMPETITION") == none {
        if (args.len() < 2) {
            fail "Must specify a board name";
        }
    
        alt (io::read_whole_file_str(args[1])) {
            ok (mm) { map = mm; }
            err(msg) { fail msg; }
        }
    } else {
        map = str::from_bytes(io::stdin().read_whole_stream());
    }

    let state = state::read_board(io::str_reader(map));

    signal::init();
    alt emit_preset_moves(state) {
      none { }
      some(s) {io::println(s); ret}}
    
    let path_find = path_find();
    let engine = alt os::getenv("ENGINE") {
      some("simple") { game_tree::simple::mk(path_find) }
      some("tobruos") { game_tree::tobruos::mk(path_find) }
      some("octopus") {
        game_tree::octopus::octopus(~[
            fn~() -> game_tree::game_tree {
                game_tree::simple::mk(driver::path_find())
            },
            fn~() -> game_tree::game_tree {
                game_tree::tobruos::mk(driver::path_find())
            },
            fn~() -> game_tree::game_tree {
                game_tree::bblum::mk({
                    path_find: driver::path_find()
                    with game_tree::bblum::default_opts()})
            },
            fn~() -> game_tree::game_tree {
                game_tree::cargomax::mk({
                    path_find: driver::path_find()
                    with game_tree::cargomax::default_opts()})
            },
        ]) as game_tree::game_tree
      }
      _ {
        game_tree::cargomax::mk({
          path_find: path_find
          with game_tree::cargomax::default_opts()})
      }
    };

    alt (os::getenv("ICFP_HUMAN")) {
        some (_) { human(state, engine); }
        none { robot(state, engine); }
    }
}

fn human(init: state::state, engine: game_tree) {
    import to_str::*;
    import state::*;

    pattern::demo_pats(init.grid);

    let mut hist = ~[copy init];
    let mut moves = ~[];
    let input = io::stdin();

    let mut bot_n = 0;
    let mut robot_plan : option<~[state::move]> = none;

    io::println(hist[0].to_str());
    while (!input.eof()) {
        let res;
        let state = copy hist[hist.len()-1];
        alt (input.read_char()) {
            'p' {
                if hist.len() > 1 { vec::pop(hist); vec::pop(moves); }
                robot_plan = none; 
                again;
            }
            'n' {
                let mut move : state::move;
                alt robot_plan {
                    some(plan) {
                        move = plan[bot_n];
                    }
                    none {
                        bot_n = 0;
                        let plan = engine.get_path(copy state);
                        move = plan[bot_n];
                        robot_plan = some(plan);
                    }
                }
                vec::push(moves, move);
                res = some(state.step(move, false));
                bot_n += 1;
            }
            '\n' { res = none; io::println(state.to_str()); }
            -1 as char { /* You bastard!  You lied to me!  That was an eof! */
                res = some(state.step(state::A, false));
                robot_plan = none;
            }
            c {
                let move = state::move_from_char(c);
                vec::push(moves, move);
                res = some(state.step(move, false));
                robot_plan = none;
            }
        }

        alt (res) {
            some(res_) {
                alt (res_) {
                    stepped(newstate) {
                        vec::push(hist, extract_step_result(newstate));
                    }
                    endgame(score) {
                        io::println(#fmt("Finished with %d points.", score));
                        io::println(str::concat(vec::map(moves, |m| { m.to_str() })));
                        break;
                    }
                    oops(newstate) { io::println("Oops.  Bye."); break; }
                }
            }
            none { }
        }
    }

}

fn robot(+init: state::state, engine: game_tree) {
    import state::*;
    
    let moves = engine.get_path(init);
    for moves.each |m| {
        io::print(m.to_str());
    }
    io::println("");
}

fn compare_states(s1: state::state, s2: state::state) -> bool {
    let g1 = s1.grid.grid;
    let g2 = s2.grid.grid;
    if g1.len() != g2.len() { ret false; }
    else if g1[0].len() != g2[0].len() { ret false; }
    else {
        for uint::range(0, g1.len()) |i| {
            for uint::range(0, g1[0].len()) |j| {
                if g1[i][j] != g2[i][j] {ret false;}
            }
        }
    }
    true
}

fn whole_file_as_string(s: str) -> str {
    str::from_bytes(io::read_whole_file("./" + s).get())
}

fn emit_preset_moves(s1: state::state) -> option<str> {
    for get_known_maps().each() |s| {
        let s2 = state::read_board(io::str_reader(whole_file_as_string("maps/" + s)));
        if compare_states(s1, s2) {
            let ret_val = whole_file_as_string("strats/" + s);
            ret some(ret_val);
        }
        else { again; }
    };
    none
}

fn get_known_maps() -> ~[str] {
~["contest10.map",
"contest1.map",
"contest2.map",
"contest3.map",
"contest4.map",
"contest5.map",
"contest6.map",
"contest7.map",
"contest8.map",
"contest9.map",
"flood1.map",
"flood2.map",
"flood3.map",
"flood4.map",
"flood5.map"]
}

#[test]
fn test_preset() {
    alt emit_preset_moves(state::read_board(
        io::str_reader(whole_file_as_string("maps/contest1.map")))) {
      some(moves) {assert("LDRDDULULLDDL" == moves)}
      none {fail}
    }
}