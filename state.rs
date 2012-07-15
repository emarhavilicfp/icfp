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
    trampoline(int),
    target(int),
    empty,
    // If you add a square type, remember to update the number in
    // hash_keys and gen_hashkeys.
}

type hash_val = u32;
type hash_keys = @~[~[[hash_val]/26]];

type grid = {
    grid: ~[mut ~[mut square]],
    mut hash: hash_val,
    keys: hash_keys,
};
type coord = (uint,uint); /* Always in *world* (1-based) coordinates -- (x,y)! */

impl extensions for coord {
    fn +(c: coord) -> coord {
        let (x, y) = self;
        let (xx, yy) = c;
        (x + xx, y + yy)
    }

    fn x() -> uint {
        let (x, _) = self;
        x
    }

    fn y() -> uint {
        let (_, y) = self;
        y
    }

    fn left() -> coord  { self + (-1,  0) }
    fn right() -> coord { self + ( 1,  0) }
    fn up() -> coord    { self + ( 0,  1) }
    fn down() -> coord  { self + ( 0, -1) }
}

type state = {
    /* Intrinsics */
    flooding: int,
    waterproof: int,
    target: ~[coord],
    trampoline: ~[coord],

    /* These changes periodically. */
    grid: grid, /* mut? */
    robotpos: coord,
    water: uint, /* not an option -- just 0 otherwise */
    nextflood: int, /* ticks until we flood next; ignored if not flooding */
    tramp_map: ~[int], /* trampoline[t] holds the target of t */
    underwater: int, /* how long we have been underwater */
    lambdas: int, /* how many lambdas we have collected */
    lambdasleft: int, /* how many lambdas we have left */
    destlambda: option<coord>, /* the lambda we were last known to be pursuing */
    score: int,
    /* We probably need a list of rocks here. */
};

enum move {
    U, D, L, R, W, A
}

impl extensions for square {
    fn to_uint() -> uint {
        alt self {
            bot { 1u }
            wall { 2u }
            rock { 3u }
            lambda { 4u }
            lift_c { 5u } 
            lift_o { 6u }
            earth { 7u }
            trampoline(i) { 7u + i as uint } /* i in range 1-9 */
            target(i) { 16u + i as uint } /* i in range 1-9 */
            empty { 25u }
        }
    }
}

impl extensions for hash_keys {
    fn get(c: coord, s: square) -> hash_val {
        let (y, x) = c;
        let (x, y) = (x - 1, y - 1);
        self[x][y][s.to_uint()]
    }
}

impl extensions for grid {
    fn squares(f: fn(square)) {
        let self = &self.grid;
        for self.each |row| {
            for row.each |s| { f(s) }
        }
    }

    /* Traverses in the order specified by section 2.3 (Map Update) -- left-to-right, then bottom-to-top. */
    fn squares_i(f: fn(square, coord)) {
        // FIXME (#13): this should follow the loop protocol.
        let self = &self.grid;
        for self.eachi |r, row| {
            for row.eachi |c, s| { f(s, (c+1, r+1)) }
        }
    }

    fn map_squares<T>(f: fn(square) -> T) -> ~[~[T]] {
        let mut res = ~[];
        let self = &self.grid;
        for self.each |row| {
            let mut r = ~[];
            for row.each |s| { vec::push(r, f(s)) }
            vec::push(res, r);
        }
        res
    }

    fn foldl<T: copy>(z: T, f: fn(T, square, coord) -> T) -> T {
        foldl(z, self, f)
    }

    fn at(c: coord) -> square {
        let (x, y) = c;
        self.grid[y-1][x-1]
    }

    fn in(c: coord) -> bool {
        let self = &self.grid;
        let (x, y) = c;
        ret x>0 && y>0 && y<=self.len() && x<=self[0].len();
    }

    fn set(c: coord, s: square) {
        let (x, y) = c;
        self.hash ^= self.keys.get(c, self.at(c));
        self.grid[y-1][x-1] = s;
        self.hash ^= self.keys.get(c, self.at(c));
    }
    
    fn lambdas() -> ~[coord] {
        self.foldl(
            ~[],
            fn @(l: ~[coord], sq: square, co: coord) -> ~[coord]
            {
                if sq == lambda || sq == lift_o{
                    vec::append_one(l, co)
                } else { l }
            })
    }

    fn rehash() -> hash_val {
        let mut hash = 0;
        do self.squares_i |s, c| {
            hash ^= self.keys.get(c, s);
        }
        hash
    }
}

impl extensions for ~[mut ~[mut square]] {
    fn gen_hashkeys() -> hash_keys {

        pure fn f() -> [hash_val]/26 unchecked {
            // FIXME: it'd be nice if the rng were created outside
            let r = rand::rng();
            [
                r.gen_u32(), /* bot    */
                r.gen_u32(), /* wall   */
                r.gen_u32(), /* rock   */
                r.gen_u32(), /* lambda */
                r.gen_u32(), /* lift_c */
                r.gen_u32(), /* lift_o */
                r.gen_u32(), /* earth  */
                r.gen_u32(), /* trampoline  */
                r.gen_u32(), /* trampoline  */
                r.gen_u32(), /* trampoline  */
                r.gen_u32(), /* trampoline  */
                r.gen_u32(), /* trampoline  */
                r.gen_u32(), /* trampoline  */
                r.gen_u32(), /* trampoline  */
                r.gen_u32(), /* trampoline  */
                r.gen_u32(), /* trampoline  */
                r.gen_u32(), /* target  */
                r.gen_u32(), /* target  */
                r.gen_u32(), /* target  */
                r.gen_u32(), /* target  */
                r.gen_u32(), /* target  */
                r.gen_u32(), /* target  */
                r.gen_u32(), /* target  */
                r.gen_u32(), /* target  */
                r.gen_u32(), /* target  */
                r.gen_u32(), /* empty  */
                ]/_
        }
        
        @self.map(|s| s.map(|_s| f()))
    }
}

fn foldl<T: copy>(z: T, g: grid, f: fn(T, square, coord) -> T) -> T {
    let mut accum = z;
    for g.grid.eachi |y,row| {
        for row.eachi |x,square| {
            accum = f(accum, square, (x+1,y+1));
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

impl of to_str::to_str for move {
    fn to_str() -> str {
        alt self {
            U { "U" }
            D { "D" }
            L { "L" }
            R { "R" }
            A { "A" }
            W { "W" }
        }
    }
}

fn tramp_to_str(i: int) -> str {
    str::from_char((i + 'A' as int - 1) as char)
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
          trampoline(i) { tramp_to_str(i) }
          target(i) { int::str(i) }
          empty { " " }
        }
    }
}

impl of to_str::to_str for grid {
    fn to_str() -> str {
        str::connect(vec::reversed(do self.grid.map |row| {
            pure fn sq_to_str (sq: square) -> str { unchecked { sq.to_str() } }
            str::concat(row.map(sq_to_str))
        }), "\n") + "\n"
    }
}

impl of to_str::to_str for state {
    fn to_str() -> str {
        self.grid.to_str()
         + "\n\nWater " + (uint::str(self.water))
         + "\nFlooding " + (int::str(self.flooding))
         + "\nWaterproof " + (int::str(self.waterproof))
         + str::concat(vec::mapi(self.tramp_map, |i, t| {
             if t == 0 || i == 0 { "" }
             else { "\nTrampoline " + tramp_to_str(i as int) + " targets " + int::str(t) }
         }))
         + "\n"
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
      'A' to 'I' { trampoline(c as int - 'A' as int + 1) }
      '1' to '9' { target(c as int - '1' as int + 1) }
      _ {
        #error("invalid square: %?", c);
        fail
      }
    }
}

// If there's a boulder on top of me, will it fall to the right?
fn right_fallable(g: grid, r: uint, c: uint) -> bool {
    let g = &g.grid;
    if g[r][c] == rock || g[r][c] == lambda {
        g[r][c+1] == empty && g[r-1][c+1] == empty
    } else {
        false
    }
}

// If there's a boulder on top of me, will it fall to the left?
fn left_fallable(g: grid, r: uint, c: uint) -> bool {
    let g = &g.grid;
    if g[r][c] == rock {
        !(g[r][c+1] == empty && g[r-1][c+1] == empty) &&
        g[r][c-1] == empty && g[r-1][c-1] == empty
    } else {
        false
    }
}

// If I'm a boulder at this position, will I fall in the update step?
fn fallable(g: grid, r: uint, c: uint) -> bool { 
    g.grid[r-1][c] == rock &&
    left_fallable(g, r-1, c) &&
    right_fallable(g, r-1, c)
}

// Is it safe to move into this tile next turn?
fn safe(g: grid, r: uint, c: uint) -> bool {
    if r == 0 || c == 0 || c == g.grid[0u].len() - 1u {
        true
    } else {
        // Die from above
        !((g.grid[r+2][c] == rock && g.grid[r+1][c] == empty)
        // Die from left boulder falling right
          || (g.grid[r+2][c-1] == rock && right_fallable(g, r+1, c-1))
        // Die from right boulder falling left
          || (g.grid[r+2][c+1] == rock && left_fallable(g, r+1, c+1)))
    }
}

fn safely_passable(g: grid, r: uint, c: uint) -> bool {
    alt g.grid[r][c] {
      rock | wall | lift_c { false }
      _ { safe(g,r,c) }
    }
}

fn read_board(+in: io::reader) -> state {
    let mut map_lines = "";
    let mut grid = ~[mut];
    let mut robot = none;
    let mut lambdasleft_ = 0;

    while (!in.eof()) {
        let line = in.read_line();
        if (line.len() == 0) {
            break;
        }
        map_lines += line + "\n";
    }

    let mut water = 0u;
    let mut flooding = 0;
    let mut waterproof = 10;
    let tramp_map = ~[mut 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    while (!in.eof()) {
        let line = in.read_line();
        let split = str::split_char_nonempty(line, ' ');
        alt (split[0]) {
            "Water" { water = option::get(uint::from_str(split[1])); }
            "Flooding" { flooding = option::get(int::from_str(split[1])); }
            "Waterproof" { waterproof = option::get(int::from_str(split[1])); }
            "Trampoline" {
                let tramp = str::char_at(split[1],0) as int - 'A' as int + 1;
                let targ = str::char_at(split[3],0) as int - '1' as int + 1;
                tramp_map[tramp] = targ;
            }
            _ { fail "Bad metadata in map file"; }
        }
    }

    let map_reader = io::str_reader(map_lines);
    let mut yinv = 0;
    let mut width = 0;
    for map_reader.each_line |line| {
        let mut x = 1;
        let mut row = ~[mut];
        for line.each_char |c| {
            let sq = square_from_char(c);
            if sq == bot {
                alt (robot) {
                    none { robot = some ((x, yinv)); }
                    some(_) { fail; }
                }
            }
            if sq == lambda {
                lambdasleft_ = lambdasleft_ + 1;
            }
            vec::push(row, sq);
            x += 1;
        }
        if width < row.len() { width = row.len(); }
        vec::push(grid, row);
        yinv += 1;
    }
    vec::reverse(grid);

    let mut i = 0u;
    while i < grid.len() {
        let len = grid[i].len();
        vec::grow(grid[i], width-len, empty);
        i += 1u;
    }

    for grid.each |row| { assert row.len() == width }

    let mut (x_, yinv_) = option::get(robot);
    let robotpos = (x_, grid.len() - yinv_);

    let hash_keys = grid.gen_hashkeys();

    let grid = {
        grid: grid,
        mut hash: 0,
        keys: hash_keys,
    };
    grid.hash = grid.rehash();

    let z = (0,0);
    let targets = ~[mut z, z, z, z, z, z, z, z, z, z];
    let trampolines = ~[mut z, z, z, z, z, z, z, z, z, z];
    do grid.squares_i |s, c| {
        alt s {
            target(i) { targets[i] = c; }
            trampoline(i) { trampolines[i] = c; }
            _ {}
        }
    }

    ret {
        flooding: flooding,
        waterproof: waterproof,
        target: vec::from_mut(targets),
        trampoline: vec::from_mut(trampolines),
        grid: grid,
        robotpos: robotpos,
        water: water,
        nextflood: flooding,
        tramp_map: vec::from_mut(tramp_map),
        underwater: 0,
        lambdas: 0,
        lambdasleft: lambdasleft_,
        destlambda: option::none,
        score: 0,
    }
}

enum step_result {
    stepped(@mut option<state>),
    endgame(int), /* points */
    oops /* accidental death or illegal move */
}

fn extract_step_result(x: @mut option<state>) -> state {
    let mut shit = none;
    *x <-> shit;
    if shit.is_none() { fail "Can't extract_step_result: already 'none'!"; }
    option::unwrap(shit)
}

impl extensions for state {
    fn step(move: move, strict: bool) -> step_result {
        let mut score_ = self.score - 1;
        let mut lambdas_ = self.lambdas;
        let mut lambdasleft_ = self.lambdasleft;
        let mut water_ = self.water;
        let mut nextflood_ = self.nextflood;
        let mut underwater_ = self.underwater;
        let mut tramp_map_ = self.tramp_map;
        let rocks_fall = @mut false; /* everybody dies -- delayed for later */
        let mut grid_ = copy self.grid;
        let grid = copy self.grid; /* XXX point to original self.grid later if able */
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
          empty | earth | bot { /* We're good. */ (xp, yp) }
          lambda {
            lambdas_ = lambdas_ + 1;
            lambdasleft_ = lambdasleft_ - 1;
            score_ = score_ + 25;
            (xp, yp)
          }
          lift_o { /* We've won -- ILHoist.hoist away! */
            ret endgame(score_ + self.lambdas * 50)
          }
          rock {
            if xp == x + 1 && yp == y &&
               grid_.at((xp, yp)) == rock && grid_.at((x+2, y)) == empty {
                grid_.set((x+2, yp), rock);
                grid.set((x+2, yp), rock);
                (xp, yp)
            } else
            if xp == x - 1 && yp == y &&
               grid_.at((xp, yp)) == rock && grid_.at((x-2, y)) == empty {
                grid_.set((x-2, yp), rock);
                grid.set((x-2, yp), rock);
                (xp, yp)
            } else {
                if strict { ret oops }
                (x, y)
            }
          }
          trampoline(id) {
            let targ = self.tramp_map[id];
            let (x_, y_) = self.target[targ];

            /* remove all trampolines for this target */
            tramp_map_ = vec::mapi(tramp_map_, |i, targ_| {
                if targ_ == targ {
                    grid_.set(self.trampoline[i], empty);
                    grid.set(self.trampoline[i], empty);
                    0
                }
                else { targ_ }
            });

            /* remove the target */
            grid_.set((x_, y_), empty);
            grid.set((x_, y_), empty);

            (x_, y_)
          }

          _ { if strict {ret oops}; (x, y) }
        };

        grid_.set((x, y), empty);
        grid_.set((x_, y_), bot);
        
        /* Set it in the old one too, to 'catch' the boulder. */
        grid.set((x, y), empty);
        grid.set((x_, y_), bot);
        
        /* Phase two -- update the map */

        /* If we're not underwater at the beginning of the map update phase, then we get reset. */
        if y_ > water_ {
            underwater_ = 0;
        }
        
        /* Only *after* that do we update the water.  (discussion of this was in #icfp-contest; I hope I got it right) */
        if nextflood_ != 0 {
            nextflood_ = nextflood_ - 1;
            if nextflood_ == 0 {
                water_ = water_ + 1;
                nextflood_ = self.flooding;
            }
        }

        /* Helper function, so we can determine if rocks fall. */
        let placerock = fn @( &grid_: grid, c: coord) {
            /* recall x_ and y_ at this point are where the robot has moved to */
            let (x, y) = c;
            if x == x_ && y == (y_ + 1) {
                *rocks_fall = true;
            }
            grid_.grid[y-1][x-1] = rock;
        };

        do grid.squares_i |sq, c| {
          let (sx, sy) = c;
          alt sq {
            rock {
              if grid.at((sx, sy-1)) == empty {
                  placerock(grid_, (sx, sy-1));
                  grid_.set((sx, sy), empty);
              } else if grid.at((sx, sy-1)) == rock &&
                        grid.at((sx+1, sy)) == empty &&
                        grid.at((sx+1, sy-1)) == empty {
                  placerock(grid_, (sx+1, sy-1));
                  grid_.set((sx, sy), empty);
              } else if grid.at((sx, sy-1)) == rock &&
                        (grid.at((sx+1, sy)) != empty ||
                         grid.at((sx+1, sy-1)) != empty) &&
                        grid.at((sx-1, sy)) == empty &&
                        grid.at((sx-1, sy-1)) == empty {
                  placerock(grid_, (sx-1, sy-1));
                  grid_.set((sx, sy), empty);
              } else if grid.at((sx, sy-1)) == lambda &&
                        grid.at((sx+1, sy)) == empty &&
                        grid.at((sx+1, sy-1)) == empty {
                  placerock(grid_, (sx+1, sy-1));
                  grid_.set((sx, sy), empty);
              }
            }
            lift_c {
              if lambdasleft_ == 0 {
                  grid_.set((sx, sy), lift_o);
              }
            }
            _ { }
          }
        }
        
        /* Blub blub blub... */
        if y_ <= water_ {
            underwater_ = underwater_ + 1;
        }
        
        if underwater_ > self.waterproof {
            if strict { ret oops; }
            ret endgame(score_);
        }

        /* Have we won? */
        if grid_.at((x_, y_)) == lift_o {
            ret endgame(score_ + lambdas_ * 50);
        }

        /* Check to see if rocks fall *after* we could have successfully taken the lambda lift. */
        if *rocks_fall {
            if strict { ret oops; }
            ret endgame(score_);
        }

        /* Here we go! */
        // FIXME: we could use the FRU syntax here.
        ret stepped(@mut some({
            flooding: self.flooding,
            waterproof: self.waterproof,
            target: self.target,
            trampoline: self.trampoline,

            grid: grid_,
            robotpos: (x_, y_),
            water: water_,
            nextflood: nextflood_,
            tramp_map: tramp_map_,
            underwater: underwater_,
            lambdas: lambdas_,
            lambdasleft: lambdasleft_,
            destlambda: self.destlambda,
            score: score_
        }));
    }

    fn hash() -> hash_val {
        self.grid.hash
    }

    fn rehash() -> hash_val {
        self.grid.rehash()
    }
}

// FIXME: we can't use state here, because it's not const.
type transposition_table<T: copy> = std::map::hashmap<hash_val, T>;

fn transposition_table<T: copy>() -> transposition_table<T> {
    std::map::hashmap(|s: hash_val| s as uint, |a, b| a == b)
}

mod test {
    #[test]
    fn trivial_to_str() {
        assert lambda.to_str() == "\\"
    }

    #[test]
    fn read_simple_board() {
        let s = #include_str("./maps/contest1.map");
        read_board(io::str_reader(s));
    }

    #[test]
    fn deparse() {
        let s = "####\nR*LO\n. ##\n";
        let s2 = read_board(io::str_reader(s)).grid.to_str();
        assert s == s2;
    }
    
    #[test]
    fn bouldering_problem() {
        let s = "#####\n# R #\n# * #\n#   #\n#####\n";
        let mut b = read_board(io::str_reader(s));
        b = alt b.step(W, false) {
            stepped(b) { extract_step_result(b) } _ { fail }
        };
        assert b.grid.to_str() == "#####\n# R #\n#   #\n# * #\n#####\n";
        b = alt b.step(W, false) {
            stepped(b) { extract_step_result(b) } _ { fail }
        };
        
assert b.grid.to_str() == "#####\n# R #\n#   #\n# * #\n#####\n";
    }

    #[test]
    #[ignore]
    fn bouldering_problem_hashes() {
        let s = "#####\n# R #\n# * #\n#   #\n#####\n";
        let mut b = read_board(io::str_reader(s));
        assert b.hash() == b.rehash();
        b = alt b.step(W, false) {
            stepped(b) { extract_step_result(b) } _ { fail }
        };
        assert b.hash() == b.rehash();
        assert b.grid.to_str() == "#####\n# R #\n#   #\n# * #\n#####\n";
        b = alt b.step(W, false) {
            stepped(b) { extract_step_result(b) } _ { fail }
        };
        assert b.hash() == b.rehash();
        assert b.grid.to_str() == "#####\n# R #\n#   #\n# * #\n#####\n";
    }    
}
