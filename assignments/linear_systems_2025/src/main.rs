mod format;
use format::{Coef, Cos, Exp, Sin};

use crate::format::Term;

trait Equation {
    fn problem(&self) -> String;
    fn solution(&self) -> String;
}

enum LinEq2 {
    Real(i32, i32),
    Complex { re: i32, im: i32 },
}

impl Equation for LinEq2 {
    fn problem(&self) -> String {
        match self {
            Self::Real(a, b) => String::from(format!(
                "y'' {:+} {:+} = 0",
                Term(Coef(-(a + b)), "y'"),
                Term(Coef(a * b), "y")
            )),
            Self::Complex { re, im } => String::from(format!(
                "y'' {:+} {:+} = 0",
                Term(Coef(-2 * re), "y'"),
                Term(Coef(re * re + im * im), "y")
            )),
        }
    }

    fn solution(&self) -> String {
        let char_equation = match self {
            Self::Real(a, b) => String::from(format!(
                "lambda^2 {:+} {:+} = 0",
                Term(Coef(-(a + b)), "lambda"),
                Term(Coef(a * b), "")
            )),
            Self::Complex { re, im } => String::from(format!(
                "lambda^2 {:+} {:+} = 0",
                Term(Coef(-2 * re), "lambda"),
                Term(Coef(re * re + im * im), "")
            )),
        };
        let lambdas = match self {
            Self::Real(a, b) => {
                if a == b {
                    String::from(format!("lambda_(1, 2) = {a}"))
                } else {
                    String::from(format!("lambda_1 = {a}, lambda_2 = {b}"))
                }
            },
            Self::Complex { re, im } => String::from(format!(
                "lambda_(1, 2) = {} plus.minus {im}", Term(Coef(*re), "")
            )),
        };
        let answer = match self {
            Self::Real(a, b) => {
                if a == b {
                    String::from(format!(
                        "y = c_1 {} + c_2 {} x",
                        Exp(Coef(*a), "x"),
                        Exp(Coef(*a), "x")
                    ))
                } else {
                    String::from(format!(
                        "y = c_1 {} + c_2 {}",
                        Exp(Coef(*a), "x"),
                        Exp(Coef(*b), "x")
                    ))
                }
            }
            Self::Complex { re, im } => String::from(format!(
                "y = c_1 {} {} + c_2 {} {}",
                Exp(Coef(*re), "x"),
                Cos(Coef(*im), "x"),
                Exp(Coef(*re), "x"),
                Sin(Coef(*im), "x"),
            )),
        };

        char_equation + "\n" + &lambdas + "\n" + &answer
    }
}

fn main() {

    let eq = LinEq2::Real(1, 1);
    println!("problem:\n{}\nsolution:\n{}\n\n", eq.problem(), eq.solution());

    let eq = LinEq2::Real(-2, 3);
    println!("problem:\n{}\nsolution:\n{}\n\n", eq.problem(), eq.solution());
    
    let eq = LinEq2::Real(-2, 2);
    println!("problem:\n{}\nsolution:\n{}\n\n", eq.problem(), eq.solution());
    
    let eq = LinEq2::Real(-99, 0);
    println!("problem:\n{}\nsolution:\n{}\n\n", eq.problem(), eq.solution());

    let eq = LinEq2::Real(3, 3);
    println!("problem:\n{}\nsolution:\n{}\n\n", eq.problem(), eq.solution());

    let eq = LinEq2::Complex{re: 2, im: 3};
    println!("problem:\n{}\nsolution:\n{}\n\n", eq.problem(), eq.solution());

    let eq = LinEq2::Complex{re: 1, im: 2};
    println!("problem:\n{}\nsolution:\n{}\n\n", eq.problem(), eq.solution());

    let eq = LinEq2::Complex{re: 0, im: 1};
    println!("problem:\n{}\nsolution:\n{}\n\n", eq.problem(), eq.solution());

    // Part one: linear equations
    // 1. Homogeneous with two distinct reals
    // 2. Homogeneous with one real root
    // 3. Homogeneous with complex roots
    //
    // Non-homoegeneity:
    // - Real no resonance,
    // -
}
