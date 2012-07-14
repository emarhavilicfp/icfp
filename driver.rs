import to_str::*;
import state::*;
import play::play_game;

fn main(args: ~[str]) {
    let map;
    if (args.len() == 2) {
        alt (io::read_whole_file_str(args[1])) {
            result::ok (contents) { map = contents; }
            result::err (msg) { fail msg; }
        }

        pattern::demo_pats(state::read_board(io::str_reader(map)).grid);
    } else {
        map = #include_str("./maps/contest1.map");
    }
    let mut hist = ~[state::read_board(io::str_reader(map))];
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
                let robotplan = play_game(copy state);
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
