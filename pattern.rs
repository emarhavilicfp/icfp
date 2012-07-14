import io::reader_util;

import state::*;



macro_rules! sqpt {
    {$($p:ident),+} => {
        alt s { $($p)|+ {true} _ {false}}
    }
}


enum patsq {
    eat,    pass,    fall_r,    fall_l,    solid,    lam,    mt,    not_mt,
    boulder,
    any
}

fn mtc(ps: patsq, s: square) -> bool {
    alt ps {
      eat {sqpt!{earth, lambda, lift_o}}
      pass {sqpt!{earth, lambda, empty, lift_o}}
      fall_r {sqpt!{rock, lambda}}
      boulder | fall_l {s==rock}
      solid {sqpt!{rock, wall}} // maybe also earth?
      lam {s==lambda}
      mt {s==empty} // should also match accessible earth squares
      not_mt {s!=empty}
      any {true}
    }
}

fn ch_to_patsq(c: char) -> patsq {
    alt c {
      'R'|'E' { eat }
      'P' { pass }
      '>' { fall_r } '<' { fall_l }
      'S' { solid }
      '*' { boulder }
      '\\' { lam }
      '_' { mt }
      'X' { not_mt }
      '?' { any }
      _ { fail "failure YOU SUCK"; }
    }
}

fn flip_patsq(ps: patsq, ok: @mut bool) -> patsq {
    alt ps {
      fall_l { fall_r }
      fall_r { fall_l }
      lam { *ok = false; lam } //cannot flip this pattern
      _ { ps }
    }
}

type pat = {
    p: ~[~[patsq]],
    off_r: uint, off_c: uint,
    off_r_dest: uint, off_c_dest: uint,
    cost: uint,
    cmd: ~[const move]
};

fn match_offset(p: pat, g: grid, base: coord) -> bool {
    for p.p.eachi() |r, row| {
        for row.eachi() |c, psq| {
            let (x,y) = base; // idx into grid
            let ult_x = x+(c-p.off_c);
            let ult_y = y-(r-p.off_r);
            if !g.in((ult_x, ult_y)) { ret false; }
            if !mtc(psq, g.at((ult_x, ult_y))) { ret false; }
        }
    }
    ret true;
}

fn match_show(p: pat, g: grid, base: coord) {
    for p.p.eachi() |r, row| {
        for row.eachi() |c, _psq| {
            let (x,y) = base; // idx into grid
            let ult_x = x+(c-p.off_c);
            let ult_y = y-(r-p.off_r);
            io::print(g.at((ult_x, ult_y)).to_str());
        }
        io::println("");
    }
}
fn read_patterns(filename: str) -> ~[pat] {
    let in: io::reader = io::file_reader(filename).get();

    // this is so not Unicode-aware
    let mut rv = ~[];

    let mut p_pat: ~[~[patsq]] = ~[];
    let mut flip_p_pat: ~[~[patsq]] = ~[];
    let flip_ok = @mut true;
    let mut o_r: uint = 0;
    let mut o_c: uint = 0;
    for in.each_line() |raw_line| {
        let mut line = str::trim(str::splitn_char(raw_line, ';', 2)[0u]);
        if line.len() == 0 { again; }
        if str::char_at(line, 0) != '!' {
            let mut pp_line = ~[];
            let mut f_pp_line = ~[];
            for line.each_char() |c| { vec::push(pp_line, ch_to_patsq(c)); }
            for line.each_char() |c| { vec::unshift(f_pp_line,
                                                    flip_patsq(ch_to_patsq(c),
                                                               flip_ok)); }
            vec::push(p_pat, pp_line);
            vec::push(flip_p_pat, f_pp_line);
            alt str::find_char(line, 'R') {
              none {}
              some(idx) { o_r = p_pat.len(); o_c = idx; }
            }

        } else {
            str::shift_char(line);
            let meta = str::split_char(line, ' ');
            let mut cmd = ~[];
            for meta[0].each_char() |c| { vec::push(cmd, move_from_char(c)); }
            let delta_r = vec::count(cmd, D) - vec::count(cmd, U);
            let delta_c = vec::count(cmd, R) - vec::count(cmd, L);

            vec::push(rv,
                      {p: p_pat, off_r: o_r, off_c: o_c,
                       off_r_dest: o_r + delta_r, off_c_dest: o_c + delta_c,
                       cost: meta[1].len(), //worse is better
                       cmd: cmd});

            if *flip_ok {
                vec::push(rv, {p: flip_p_pat, off_r: o_r,
                               off_c: (p_pat[0u].len()-1u)-o_c,
                               off_r_dest: o_r + delta_r,
                               off_c_dest: o_c - delta_c,
                               cost: meta[1].len(), //worse: still better
                               cmd: do cmd.map |m| {m.flip()}} );
            }

            p_pat = ~[];
            flip_p_pat = ~[];
            *flip_ok = true;
        }
    }
    ret rv;
}

#[test]
fn test_some_stuff() {
    let pats = read_patterns("patterns/some_patterns");

    let s = #include_str("./maps/contest8.map");
    let g = copy read_board(io::str_reader(s)).grid;

    do pats.eachi() |_i, pat| {
        do g.squares_i() |_sq, coord| {
            if match_offset(pat, g, coord) {
                match_show(pat, g, coord);
                io::println("--------");
            }
        };
        true
    };

}
