import game_tree::game_tree;
import future::extensions;
import state::to_str;

/// Run several engines in parallel, and pick the best one.
///
/// In the future, we might make them periodically redistribute states
/// so we join on the best so far.
class octopus : game_tree {
    let engines: box<~[fn~() -> game_tree]>;

    new(+engines: ~[fn~() -> game_tree]) {
        self.engines = box(engines);
    }

    fn get_path(+g: state::state) -> ~[state::move] {
        let mut futures = ~[];
        do vec::consume(self.engines.unwrap()) |_i, engine| {

            let keys = *g.grid.keys;
            let grid = copy g.grid.grid;
            let hash = g.grid.hash;

            let fl = g.flooding;
            let wa = g.waterproof;
            let ta = g.target;
            let tr = g.trampoline;
            let gr = g.growth;

            let ro = g.robotpos;
            let wat = g.water;
            let ne = g.nextflood;
            let tra = g.tramp_map;
            let un = g.underwater;
            let ra = g.razors;
            let la = g.lambdas;
            let lam = g.lambdasleft;
            let sc = g.score;

            let g = fn~() -> state::state {
                let grid = {
                    grid: copy grid,
                    mut hash: hash,
                    keys: @keys
                };

                {
                    flooding: fl,
                    waterproof: wa,
                    target: ta,
                    trampoline: tr,
                    growth: gr,
                    grid: grid,
                    robotpos: ro,
                    water: wat,
                    nextflood: ne,
                    tramp_map: tra,
                    underwater: un,
                    razors: ra,
                    lambdas: la,
                    lambdasleft: lam,
                    score: sc
                }
            };

            vec::push(futures, future::spawn(|move engine, copy g| {
                extern mod rustrt {
                    fn unsupervise();
                }

                rustrt::unsupervise();

                let g = g();
                let engine = engine();
                let path = engine.get_path(copy g);
                // ???: should strict be true instead of false here?
                let score = alt path::apply(path, g, false) {
                  state::endgame(score) { score }
                  state::stepped(@some(s)) { s.score }
                  state::oops(@s) { s.score }
                  _ {
                    #error("I don't know what to do with this state");
                    int::min_value
                  }
                };
                
                (score, path)
            }));
        }

        let mut best = int::min_value;
        let mut path = ~[];

        for futures.each |f| {
            let (score, pth) = f.get();
            if score > best {
                #error("found new best path with score %?: %?",
                       score, path.to_str());
                best = score;
                path = pth;
            }
        }

        path
    }
}

class box<T> {
    let mut contents: option<T>;
    new(+x: T) { self.contents = some(x); }

    fn swap(f: fn(+T) -> T) {
        let mut tmp = none;
        self.contents <-> tmp;
        self.contents = some(f(option::unwrap(tmp)));
    }

    fn unwrap() -> T {
        let mut tmp = none;
        self.contents <-> tmp;
        option::unwrap(tmp)
    }
}
