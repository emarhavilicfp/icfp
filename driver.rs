fn main(args: ~[str]) {
    import result::*;

    if (args.len() < 2) {
        fail "Must specify a board name";
    }

    signal::init();

    let map_res = io::read_whole_file_str(args[1]);
    let fun_res = os::getenv("ICFP_HUMAN");

    let fun;
    let state;

    alt (fun_res) {
        some (_) { fun = human; }
        none { fun = robot; }
    }

    alt (map_res) {
        ok (map) { state = state::read_board(io::str_reader(map)); }
        err(msg) { fail msg; }
    }

    fun(state);
}

fn human(init: state::state) {
    import to_str::*;
    import state::*;
    import play::play_game;

    pattern::demo_pats(init.grid);

    let mut hist = ~[copy init];
    let input = io::stdin();

    io::println(hist[0].to_str());
    while (!input.eof()) {
        let res;
        let state = copy hist[0];
        alt (input.read_char()) {
            'q' { res = some(state.step(A, false)); }
            'w' { res = some(state.step(W, false)); }
            'h' { res = some(state.step(L, false)); }
            'j' { res = some(state.step(D, false)); }
            'k' { res = some(state.step(U, false)); }
            'l' { res = some(state.step(R, false)); }
            'p' {
                if hist.len() > 1 { vec::shift(hist); }
                again;
            }
            'n' {
                let (robotplan, _newstate) = play_game(copy state);
                let mv = vec::head(robotplan);
                res = some(state.step(mv, false));
            }
            '\n' { res = none; io::println(state.to_str()); }
            _ { again; }
        }

        alt (res) {
            some(res_) {
                alt (res_) {
                    stepped(newstate) { vec::unshift(hist, copy newstate); }
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
    let (_, end) = play::play_game(copy init);
    io::println(end.to_str());
}
