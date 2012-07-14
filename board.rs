/// Map representation

use std;

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

type grid = ~[~[square]];

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
        str::connect(do self.map |row| {
            str::concat(do row.map |sq| { sq.to_str() })
        }, "\n") + "\n"
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
    let mut grid = ~[];
    for in.each_line |line| {
        let mut row = ~[];
        for line.each_char |c| {
            vec::push(row, square_from_char(c))
        }
        vec::push(grid, row)
    }
    let width = grid[0].len();
    for grid.each |row| { assert row.len() == width }
    grid
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
