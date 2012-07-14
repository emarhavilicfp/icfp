/// Map representation

use std;

import option::option;
import io::reader_util;

enum square {
    bot,
    wall,
    rock,
    lambda,
    lift_c,
    lift_o,
    earth,
    empty
}

type grid = ~[mut ~[mut square]];
type coord = (uint,uint); /* Always in *world* (1-based) coordinates -- (x,y)! */
type state = {
    /* Intrinsics */
    flooding: int,
    waterproof: int,
    
    /* These changes periodically. */
    grid: grid, /* mut? */
    robotpos: coord,
    water: int, /* not an option -- just 0 otherwise */
    nextflood: int, /* ticks until we flood next; ignored if not flooding */
    underwater: int, /* how long we have been underwater */
    lambdas: int, /* how many lambdas we have collected */
    score: int,
    /* We probably need a list of rocks here. */
};

enum move {
    U, D, L, R, W, A
}

impl extensions for grid {
    fn squares(f: fn(square)) {
        for self.each |row| {
            for row.each |s| { f(s) }
        }
    }
    
    fn squares_i(f: fn(square, coord)) {
        for self.eachi |r, row| {
            for row.eachi |c, s| { f(s, (r+1, c+1)) }
        }
    }
    
    fn foldl<T: copy>(z: T, f: fn(T, square, coord) -> T) -> T {
        foldl(z, self, f)
    }
}

fn foldl<T: copy>(z: T, g: grid, f: fn(T, square, coord) -> T) -> T {
    let mut accum = z;
    for g.eachi |y,row| {
        for row.eachi |x,square| {
            accum = f(accum, square, (x,y));
        }
    }
    accum
}

impl extensions for move {
    fn flip() -> move {
       alt self { L {R} R {L} _ {self} }
    }
}

fn taxicab_distance(dest: coord, src: coord) {
    let (x1,y1) = dest;
    let (x2,y2) = src;
    (if x1<x2 { x2-x1 } else { x1-x2 }) + (if y1<y2 { y2-y1 } else { y1-y2 });
}

fn move_from_char(c: char) -> move {
    alt c {
        'u' {U} 'd' {D} 'l' {L} 'r' {R} 'w' { W } 'a' { A }
        'U' {U} 'D' {D} 'L' {L} 'R' {R} 'W' { W } 'A' { A }
        _ { fail; /* XXX do something more reasonable here */ }
    }
}
impl of to_str::to_str for square {
    fn to_str() -> str {
        alt self {
          bot { "R" }
          wall { "#" }
          rock { "*" }
          lambda { "\\" }
          lift_c { "L" }
          lift_o { "O" }
          earth { "." }
          empty { " " }
        }
    }
}

impl of to_str::to_str for grid {
    fn to_str() -> str {
        str::connect(vec::reversed(do self.map |row| {
            pure fn sq_to_str (sq: square) -> str { unchecked { sq.to_str() } }
            str::concat(row.map(sq_to_str))
        }), "\n") + "\n"
    }
}

fn square_from_char(c: char) -> square {
    alt c  {
      'R'  { bot }
      '#'  { wall }
      '*'  { rock }
      '\\' { lambda }
      'L'  { lift_c }
      'O'  { lift_o }
      '.'  { earth }
      ' '  { empty }
      _ {
        #error("invalid square: %?", c);
        fail
      }
    }
}

fn safe(g: grid, r: uint, c: uint) -> bool {
    if r == 0 || c == 0 || c == g[0u].len() - 1u {
        true
    } else {
        !(g[r-1][c] == rock
          || (g[r-1][c-1] == rock
              && (g[r][c-1] == rock || g[r][c-1] == lambda))
          || (g[r-1][c+1] == rock
              && (g[r][c+1] == rock)))
    }
}

fn safely_passable(g: grid, r: uint, c: uint) -> bool {
    alt g[r][c] {
      rock | wall | lift_c { false }
      _ { safe(g,r,c) }
    }
}

fn read_board_grid(+in: io::reader) -> grid {
    let mut grid = ~[mut];
    for in.each_line |line| {
        let mut row = ~[mut];
        for line.each_char |c| {
            vec::push(row, square_from_char(c))
        }
        vec::push(grid, row)
    }
    vec::reverse(grid);
    let width = grid[0].len();
    for grid.each |row| { assert row.len() == width }
    grid
}

enum step_result {
    stepped(state),
    endgame(int) /* points */
}

impl extensions for state {
    fn step(_m: move) -> step_result {
        /* Phase one -- bust a move! */
        fail
        
        /* Phase two -- update the map */
        fail
        
        /* Phase three -- check for ending conditions */
        fail
    }
}

mod test {
    #[test]
    fn trivial_to_str() {
        assert lambda.to_str() == "\\"
    }

    #[test]
    fn read_simple_board() {
        let s = #include_str("./maps/contest1.map");
        read_board_grid(io::str_reader(s));
    }

    #[test]
    fn deparse() {
        let s = "####\nR*LO\n. ##\n";
        let gr = read_board_grid(io::str_reader(s));
        let s2 = gr.to_str();
        assert s == s2;
    }
}
