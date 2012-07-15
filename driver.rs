import io::reader_util;
import game_tree::game_tree;

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
    
    let path_find = path_find::brushfire::mk();
    let engine = game_tree::bblum::mk({
        path_find: path_find
        with game_tree::bblum::default_opts()});

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
    let input = io::stdin();

    let mut bot_n = 0;
    let mut robot_plan : option<~[state::move]> = none;

    io::println(hist[0].to_str());
    while (!input.eof()) {
        let res;
        let state = copy hist[hist.len()-1];
        alt (input.read_char()) {
            'p' {
                if hist.len() > 1 { vec::pop(hist); }
                robot_plan = none; 
                again;
            }
            'n' {
                alt robot_plan {
                    some(plan) {
                        let shit : state::move = plan[bot_n];
                        res = some(state.step(shit, false));
                    }
                    none {
                        bot_n = 0;
                        let plan = engine.get_path(copy init);
                        res = some(state.step(plan[bot_n], false));
                        robot_plan = some(plan);
                    }
                }
                bot_n += 1;
            }
            '\n' { res = none; io::println(state.to_str()); }
            c {
                res = some(state.step(state::move_from_char(c), false));
                robot_plan = none;
            }
        }

        alt (res) {
            some(res_) {
                alt (res_) {
                    stepped(newstate) {
                        vec::push(hist, extract_step_result(newstate));
                    }
                    endgame(score) { io::println(#fmt("Finished with %d points.", score)); break; }
                    oops { io::println("Oops.  Bye."); break; }
                }
            }
            none { }
        }
    }

}

fn robot(init: state::state, engine: game_tree) {
    import state::*;
    
    let moves = engine.get_path(copy init);
    for moves.each |m| {
        io::print(m.to_str());
    }
    io::println("");
}
