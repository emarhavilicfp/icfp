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
    
    fn at(c: coord) -> square {
        let (x, y) = c;
        self[y-1][x-1]
    }
    
    fn set(c: coord, s: square) {
        let (x, y) = c;
        self[y-1][x-1] = s;
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

fn taxicab_distance(dest: coord, src: coord) -> uint {
    let (x1,y1) = dest;
    let (x2,y2) = src;
    (if x1<x2 { x2-x1 } else { x1-x2 }) + (if y1<y2 { y2-y1 } else { y1-y2 })
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

// If there's a boulder on top of me, will it fall to the right?
fn right_fallable(g: grid, r: uint, c: uint) -> bool {
    if g[r][c] == rock || g[r][c] == lambda {
        g[r][c+1] == empty && g[r-1][c+1] == empty
    } else {
        false
    }
}

// If there's a boulder on top of me, will it fall to the left?
fn left_fallable(g: grid, r: uint, c: uint) -> bool {
    if g[r][c] == rock {
        !(g[r][c+1] == empty && g[r-1][c+1] == empty) &&
        g[r][c-1] == empty && g[r-1][c-1] == empty
    } else {
        false
    }
}

// If I'm a boulder at this position, will I fall in the update step?
fn fallable(g: grid, r: uint, c: uint) -> bool {
    g[r-1][c] == rock &&
    left_fallable(g, r-1, c) &&
    right_fallable(g, r-1, c)
}

// Is it safe to move into this tile next turn?
fn safe(g: grid, r: uint, c: uint) -> bool {
    if r == 0 || c == 0 || c == g[0u].len() - 1u {
        true
    } else {
        // Die from above
        !((g[r+2][c] == rock && g[r+1][c] == empty)
        // Die from left boulder falling right
          || (g[r+2][c-1] == rock && right_fallable(g, r+1, c-1))
        // Die from right boulder falling left
          || (g[r+2][c+1] == rock && left_fallable(g, r+1, c+1)))
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
    fn step(move: move) -> step_result {
        let mut score_ = self.score - 1;
        let mut lambdas_ = self.lambdas;
        let mut grid_ = copy self.grid;
        let (x, y) = self.robotpos;
        
        /* Phase one -- bust a move! */
        let mut (xp, yp) = alt move {
          L { (x-1, y) }
          R { (x+1, y) }
          U { (x, y+1) }
          D { (x, y-1) }
          W { (x, y) }
          A { /* Abort!  Abort! */
            ret endgame(score_ + self.lambdas * 25)
          }
        };
        
        /* Is the move valid? */
        let (x_, y_) = alt grid_.at((xp, yp)) {
          empty | earth { /* We're good. */ (xp, yp) }
          lambda {
            lambdas_ = lambdas_ + 1;
            (xp, yp)
          }
          lift_o { /* We've won. */
            ret endgame(score_ + self.lambdas * 50)
          }
          rock {
            if xp == x + 1 && yp == y && 
               grid_.at((xp, yp)) == rock && grid_.at((x+2, y)) == empty {
                grid_.set((x+2, yp), rock);
                (xp, yp)
            } else
            if xp == x - 1 && yp == y && 
               grid_.at((xp, yp)) == rock && grid_.at((x-2, y)) == empty {
                grid_.set((x-2, yp), rock);
                (xp, yp)
            } else {
                (x, y)
            }
          }
          _ { (x, y) }
        };
        
        grid_.set((x, y), empty);
        grid_.set((x_, y_), bot);
        
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
