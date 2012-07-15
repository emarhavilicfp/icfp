import io::reader_util;

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

    alt (os::getenv("ICFP_HUMAN")) {
        some (_) { human(state); }
        none { robot(state); }
    }
}

fn human(init: state::state) {
    import to_str::*;
    import state::*;
    import play::play_game;

    pattern::demo_pats(init.grid);

    let mut hist = ~[copy init];
    let input = io::stdin();

    let mut bot_n = 0;
    let mut robot_plan : option<~[mut state::move]> = none;

    io::println(hist[0].to_str());
    while (!input.eof()) {
        let res;
        let state = copy hist[hist.len()-1];
        alt (input.read_char()) {
            'q' { res = some(state.step(A, false)); robot_plan = none; }
            ' ' { res = some(state.step(W, false)); robot_plan = none; }
            'h' | 'a' { res = some(state.step(L, false)); robot_plan = none; }
            'j' | 's' { res = some(state.step(D, false)); robot_plan = none; }
            'k' | 'w' { res = some(state.step(U, false)); robot_plan = none; }
            'l' | 'd' { res = some(state.step(R, false)); robot_plan = none; }
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
                        let (plan, _n) = play_game(copy state, true);
                        res = some(state.step(plan[bot_n], false));
                        robot_plan = some(plan);
                    }
                }
                bot_n += 1;
            }
            '\n' { res = none; io::println(state.to_str()); }
            _ { again; }
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

fn robot(init: state::state) {
    import state::*;
    let (moves, _) = play::play_game(copy init, false);
    for moves.each |m| {
        io::print(m.to_str());
    }
    io::println("");
}
