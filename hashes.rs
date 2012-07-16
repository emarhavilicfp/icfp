import state;

fn main(args: ~[str]) {
    for args.each |s| {
        if s == "./hashes" {
            again;
        } else {
            let state = state::read_board(io::file_reader("./" + s).get());
            let hash = state.grid.hash;
            io::println(#fmt("%s, %?", s, hash));
        }
    }
}