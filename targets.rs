import state::coord;
import path::path;

/// A thing that the pathfinder will try to route through.
trait target {
    /// Where is this target?
    pure fn coord() -> coord;

    /// How valuable is this target?
    ///
    /// Can be negative if it costs you something to go here. An
    /// example is in patterns.
    fn score() -> int;

    /// Call this when you get to this target.
    ///
    /// Targets like lambdas don't move you, but some do. Examples are
    /// traversing patterns or trampolines.
    pure fn traverse() -> (coord, path);
}

enum gotoThing {
    Lambda(coord),
    Razor(coord),
    OpenLift(coord),
    Pattern(coord, pattern::pat),
}

impl of target for gotoThing {
    pure fn coord() -> coord {
        alt self {
            Lambda(l) { l }
            Razor(l) { l }
            OpenLift(l) { l }
            Pattern(l,_) { l }
        }
    }

    fn score() -> int {
        alt self {
            Lambda(_) { 25 }
            Razor(_) /*{
                g.foldl(0, 
                    |i,s,fucksnotgiven| { 
                        alt s {
                            state::beard(a) { i + 1 }
                            _ { i }
                        }}) * 10
            }*/ { /* XXX */ 5 }
            OpenLift(_) { 9999 }
            Pattern(_,pat) { let c = pat.cost as int; -c }
        }
    }

    pure fn traverse() -> (coord, path) {
        alt self {
            Lambda(c) { (c, ~[]) }
            Razor(c) { (c, ~[]) }
            OpenLift(c) { (c, ~[]) }
            Pattern(c,p) { 
                let (x,y) = c;
                let finPos = (x + p.off_c_dest, y + p.off_r_dest);
                (finPos, p.cmd)
            }
        }
    }
}
