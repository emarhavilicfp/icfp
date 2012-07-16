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

fn find_hard_coded(s: state::state) -> option<str> {
    alt s.grid.hash {
      1585882910 {some("contest10.map")}
      1652843932 {some("contest1.map")}
      2598939992 {some("contest2.map")}
      743435179 {some("contest3.map")}
      1452394536 {some("contest4.map")}
      2056414715 {some("contest5.map")}
      745578453 {some("contest6.map")}
      322669917 {some("contest7.map")}
      1711244539 {some("contest8.map")}
      2625681711 {some("contest9.map")}
      1185558828 {some("flood1.map")}
      1549412226 {some("flood2.map")}
      527250308 {some("flood3.map")}
      4056672759 {some("flood4.map")}
      4058792550 {some("flood5.map")}
      _ {none}
    }
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