import state;

fn main(args: ~[str]) {
    for args.each |s| {
        if s == "./hashes" {
            again;
        } else {
            let map = str::from_bytes(io::read_whole_file("./" + s).get());
            let state = state::read_board(io::str_reader(map));
            let hash = state.grid.hash;
            io::println(#fmt("%? {some(\"%s\")}", hash, s));
        }
    }
}