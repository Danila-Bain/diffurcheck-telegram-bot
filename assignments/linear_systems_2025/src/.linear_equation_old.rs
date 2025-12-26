use crate::format::{Coef, Cos, Exp, Sin};

use crate::format::Term;

pub trait Equation {
    fn problem(&self) -> String;
    fn solution(&self) -> String;
}

// pub enum LinEq2 {
//     Real(i32, i32),
//     Complex { re: i32, im: i32 },
// }
//
pub struct LinEq2Real(pub i32, pub i32);
pub struct LinEq2Complex {
    pub re: i32,
    pub im: i32,
}

impl Equation for LinEq2Real {
    fn problem(&self) -> String {
        let Self(a, b) = self;
        String::from(format!(
            "y'' {:+} {:+} = 0",
            Term(Coef(-(a + b)), "y'"),
            Term(Coef(a * b), "y")
        ))
    }

    fn solution(&self) -> String {
        let Self(a, b) = self;

        let char_equation = String::from(format!(
            "lambda^2 {:+} {:+} = 0",
            Term(Coef(-(a + b)), "lambda"),
            Term(Coef(a * b), "")
        ));
        let lambdas = if a == b {
            String::from(format!("lambda_(1, 2) = {a}"))
        } else {
            String::from(format!("lambda_1 = {a}, lambda_2 = {b}"))
        };
        let answer = if a == b {
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
        };

        char_equation + "\n" + &lambdas + "\n" + &answer
    }
}

impl Equation for LinEq2Complex {
    fn problem(&self) -> String {
        let Self { re, im } = self;
        String::from(format!(
            "y'' {:+} {:+} = 0",
            Term(Coef(-2 * re), "y'"),
            Term(Coef(re * re + im * im), "y")
        ))
    }

    fn solution(&self) -> String {
        let Self { re, im } = self;
        let char_equation = String::from(format!(
            "lambda^2 {:+} {:+} = 0",
            Term(Coef(-2 * re), "lambda"),
            Term(Coef(re * re + im * im), "")
        ));
        let lambdas = String::from(format!(
            "lambda_(1, 2) = {} plus.minus {im}",
            Term(Coef(*re), "")
        ));
        let answer = String::from(format!(
            "y = c_1 {} {} + c_2 {} {}",
            Exp(Coef(*re), "x"),
            Cos(Coef(*im), "x"),
            Exp(Coef(*re), "x"),
            Sin(Coef(*im), "x"),
        ));
        char_equation + "\n" + &lambdas + "\n" + &answer
    }
}
