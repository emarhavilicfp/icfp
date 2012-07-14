import to_str::*;
import state::*;

fn main(_args: ~[str]) {
    let map = #include_str("./maps/contest1.map");
    let mut hist = ~[state::read_board(io::str_reader(map))];
    let input = io::stdin();

    io::println(hist[0].grid.to_str());
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
            '\n' { res = none; io::println(state.grid.to_str()); }
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
