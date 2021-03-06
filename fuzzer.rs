import state;
import state::extensions;

fn rand_map_char(r: rand::rng) -> char {
    r.gen_char_from("#*\\\\....    ")
}

fn rand_flooding(r: rand::rng) -> int {
    r.gen_uint_range(0u, 51u) as int // MAGIC
}

fn rand_waterproof(r: rand::rng) -> int {
    r.gen_uint_range(0u, 51u) as int // MAGIC
}

fn rand_water(r: rand::rng, m: uint) -> uint {
    r.gen_uint_range(0u, m + 1u)
}

fn rand_grid(r: rand::rng, m: uint, n: uint)
    -> (state::grid, state::coord)
{
    let mut s = "";
    for n.times {
        s += "#";
    }
    for (m - 2).times {
        s += "\n#";
        for (n - 2).times {
            s += str::from_char(rand_map_char(r));
        }
        s += "#";
    }
    s += "\n";
    for n.times {
        s += "#";
    }
    let grid = copy state::read_board(io::str_reader(s)).grid;

    let bot_x = r.gen_uint_range(1u, m - 1u);
    let bot_y = r.gen_uint_range(1u, n - 1u);

    let mut lift_x = bot_x;
    let mut lift_y = bot_y;
    while lift_x == bot_x && lift_y == bot_y {
        lift_x = r.gen_uint_range(1u, m - 1u);
        lift_y = r.gen_uint_range(1u, n - 1u);
    }

    grid.grid[bot_x][bot_y] = state::bot;
    grid.grid[lift_x][lift_y] = state::lift_c;

    (grid, (bot_x + 1, bot_y + 1)) // NIH MAGIC
}

fn gen_state(m: uint, n: uint) -> state::state {
    let r = rand::rng();

    let (grid, robotpos) = rand_grid(r, m, n);

    let z = (0,0);

    // TODO: implement trampoline fuzzing
    // TODO: implement beard fuzzing

    {flooding: rand_flooding(r),
     waterproof: rand_waterproof(r),
     target: ~[z, z, z, z, z, z, z, z, z, z],
     trampoline: ~[z, z, z, z, z, z, z, z, z, z],
     growth: 0,
     grid: grid,
     robotpos: robotpos,
     water: rand_water(r, m),
     nextflood: 0,
     tramp_map: ~[0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
     underwater: 0,
     razors: 0,
     lambdas: 0,
     lambdasleft: 0,
     mut score: 0}
}
