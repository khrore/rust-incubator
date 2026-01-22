use step_2_2::optimized::{Solver, Trinity};

fn main() {
    let mut s = Solver::new(
        Trinity { a: 1, b: 2, c: 3 },
        vec![
            Trinity { a: 1, b: 2, c: 3 },
            Trinity { a: 2, b: 1, c: 3 },
            Trinity { a: 2, b: 3, c: 1 },
            Trinity { a: 3, b: 1, c: 2 },
        ],
    );
    s.resolve();
    println!("{:?}", s)
}
