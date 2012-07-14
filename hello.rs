use std;
import std::list;

fn main() {
	let list = list::from_vec(["!", "world", " ", "ello", "H"]/_);
	let s = list::foldl("", list, |accum,elem| elem+accum);
	io::println(s);
}
