import io::reader_util;
import state::*;

fn main(args: ~[str]) {
    import result::*;
    
    let map;
    let moveseq;

    if (args.len() < 3) {
        fail #fmt("%s: usage: %s board moveseq", args[0], args[0]);
    }
    
    alt (io::read_whole_file_str(args[1])) {
        ok (mm) { map = mm; }
        err(msg) { fail msg; }
    }
    
    moveseq = args[2];

    let mut state = state::read_board(io::str_reader(map));

    signal::init();
    
    do str::chars_iter(moveseq) |c| {
        let move = alt c {
          'R' { state::R }
          'L' { state::L }
          'U' { state::U }
          'D' { state::D }
          'A' { state::A }
          'W' { state::W }
          _ { fail }
        };
        
        alt state.step(move, false) {
          state::stepped(s) { state = state::extract_step_result(s); }
          state::endgame(points) {
            io::print(#fmt("%d", points));
          }
          _ { fail }
        }
    }
}
