import state::*;

macro_rules! test_step {
    { $s1:expr, $m:expr, $s2:expr } => { {
        let s1 = $s1;
        let s2 = $s2;
        let mut b = read_board(io::str_reader(s1));
        assert b.hash() == b.rehash();
        b = alt b.step($m, false) {
          stepped(b) { extract_step_result(b) } _ { fail }
        };
        assert b.hash() == b.rehash();
        if b.grid.to_str() != s2 {
            io::println("Starting board:");
            io::println(s1);
            io::println("\nStepped to:");
            io::println(b.grid.to_str());
            io::println("\nShould have stepped to:");
            io::println(s2);
            fail;
        }
    } }
}

#[test]
fn boulder1() {
    test_step!{
        "\
             #####\n\
             # R #\n\
             # * #\n\
             #   #\n\
             #####\n",
         W,
         "#####\n\
          # R #\n\
          #   #\n\
          # * #\n\
          #####\n"
    }
}    

#[test]
fn boulder2() {
    test_step!{
        "#####\n\
         # R #\n\
         # * #\n\
         # * #\n\
         #####\n",
        W,
        "#####\n\
         # R #\n\
         #   #\n\
         # **#\n\
         #####\n"
    }
}

#[test]
fn boulder3() {
    test_step!{
        "#####\n\
         # R #\n\
         # * #\n\
         # * #\n\
         # * #\n\
         #####\n",
        W,
        "#####\n\
         # R #\n\
         #   #\n\
         #  *#\n\
         # **#\n\
         #####\n"
    }
}

#[test]
fn boulder4() {
    test_step!{
        "#####\n\
         # R #\n\
         #   #\n\
         #* *#\n\
         #* *#\n\
         #####\n",
        W,
        "#####\n\
         # R #\n\
         #   #\n\
         #   #\n\
         #***#\n\
         #####\n"
    }
}

#[test]
fn beard1() {
    test_step!{
        "#####\n\
         # R #\n\
         #   #\n\
         # W #\n\
         #   #\n\
         #####\n\
              \n\
         Growth 1\n",
        W,
        "#####\n\
         # R #\n\
         #WWW#\n\
         #WWW#\n\
         #WWW#\n\
         #####\n"
    }
}

#[test]
fn tramp1() {
    test_step!{
        "#####\n\
         #AR #\n\
         #   #\n\
         #   #\n\
         #  1#\n\
         #####\n\
              \n\
         Trampoline A targets 1\n",
        L,
        "#####\n\
         #   #\n\
         #   #\n\
         #   #\n\
         #  R#\n\
         #####\n"
    }
}

#[test]
#[ignore]
// We die, as we should here, but the test harness isn't good enough
// ot handle that.
fn die1() {
    test_step!{
        "###\n\
         #*#\n\
         # #\n\
         # #\n\
         #R#\n\
         ###\n",
        U,
        "###\n\
         # #\n\
         #*#\n\
         #R#\n\
         # #\n\
         ###\n"
    }
}
