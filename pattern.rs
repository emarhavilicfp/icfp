import io::reader_util;

import board::*;



macro_rules! sqpt {
    {$($p:ident),+} => {
        alt s { $($p)|+ {true} _ {false}}
    }
}


enum patsq {
    eat,    pass,    fall_r,    fall_l,    solid,    lam,    mt,    not_mt,
    any
}

fn mtc(ps: patsq, s: square) -> bool {
    alt ps {
      eat {sqpt!{earth, lambda, lift_o}}
      pass {sqpt!{earth, lambda, empty, lift_o}}
      fall_r {sqpt!{rock, lambda}}
      fall_l {s==rock}
      solid {sqpt!{rock, wall}} // maybe also earth?
      lam {s==lambda}
      mt {s==empty}
      not_mt {s!=empty}
      any {true}
    }
}

fn flip_patsq(ps: patsq, bad: @mut bool) -> patsq {
    alt ps {
      fall_l { fall_r }
      fall_r { fall_l }
      lam { *bad = true; lam } //cannot flip this pattern
      _ { ps }
    }
}

type pat = uint;

fn read_patterns(+in: io::reader) -> ~[pat] {
    fail;
}