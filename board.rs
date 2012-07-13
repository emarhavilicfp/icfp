/// Map representation

use std;

enum square {
    bot,
    wall,
    rock,
    lambda,
    closed_lift,
    open_lift,
    earth,
    empty
}

impl of to_str::to_str for square {
    fn to_str() -> str {
        alt self {
          bot { "R" }
          wall { "#" }
          rock { "*" }
          lambda { "\\" }
          closed_lift { "L" }
          open_lift { "O" }
          earth { "." }
          empty { " " } 
        }
    }
}

mod test {
    #[test]
    fn trivial_to_str() {
        assert lambda.to_str() == "\\"
    }
}
