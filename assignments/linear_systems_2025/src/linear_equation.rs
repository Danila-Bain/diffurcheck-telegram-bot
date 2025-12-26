pub use num_traits::Zero;
pub use polynomial_ring::Polynomial;
pub type Poly = polynomial_ring::Polynomial<f64>;

use num_traits::One;
use polynomial_ring::polynomial;

#[derive(Clone, PartialEq, Debug)]
pub struct LinEq {
    pub coeffs: Vec<f64>,
    pub roots: Vec<(Root, usize)>,
    pub f: Vec<QPoly>,
    pub y0_basis: Vec<QPoly>,
    pub y_part: Vec<QPoly>,
}

#[derive(Clone, PartialEq, Debug)]
pub enum Root {
    Real(f64),
    Complex { re: f64, im: f64 },
}

#[derive(Clone, PartialEq, Debug)]
pub struct QPoly {
    pub re: f64,
    pub im: f64,
    pub pcos: Poly,
    pub psin: Poly,
}

impl QPoly {
    pub fn derivative(self) -> Self {
        let Self { re, im, pcos, psin } = self;

        Self {
            re,
            im,
            pcos: &pcos * re + &psin * im + pcos.clone().derivative(),
            psin: &psin * re - &pcos * im + psin.clone().derivative(),
        }
    }
}

impl LinEq {
    pub fn from_roots(roots: Vec<Root>) -> Self {
        let mut char_polynomial = Poly::one();
        let mut y0_basis = vec![];

        for root in roots.iter().cloned() {
            match root {
                Root::Real(lambda) => char_polynomial = char_polynomial * polynomial![-lambda, 1.],
                Root::Complex { re, im } => {
                    char_polynomial = char_polynomial * polynomial![re * re + im * im, -2. * re, 1.]
                }
            }
        }

        let mut root_multiplicities: Vec<(Root, usize)> = vec![];
        'outer: for root in roots.iter() {
            for (r, m) in root_multiplicities.iter_mut() {
                if r == root {
                    *m += 1;
                    continue 'outer;
                }
            }
            root_multiplicities.push((root.clone(), 1));
        }

        for (root, m) in root_multiplicities.iter() {
            for i in 0..*m {
                let mut coefs = vec![0.; i as usize + 1];
                coefs[i as usize] = 1.;
                let p = Poly::new(coefs);

                match *root {
                    Root::Real(lambda) => {
                        y0_basis.push(QPoly {
                            re: lambda,
                            im: 0.,
                            pcos: p,
                            psin: Poly::zero(),
                        });
                    }
                    Root::Complex { re, im } => {
                        y0_basis.push(QPoly {
                            re,
                            im,
                            pcos: p.clone(),
                            psin: Poly::zero(),
                        });
                        y0_basis.push(QPoly {
                            re,
                            im,
                            pcos: Poly::zero(),
                            psin: p,
                        });
                    }
                }
            }
        }

        Self {
            coeffs: char_polynomial.coeffs().iter().cloned().collect(),
            roots: root_multiplicities,
            f: vec![],
            y0_basis,
            y_part: vec![],
        }
    }

    pub fn with_y_part(mut self, y: QPoly) -> Self {
        let mut y_diff = y.clone();
        let mut f = QPoly {
            re: y.re,
            im: y.im,
            pcos: Poly::zero(),
            psin: Poly::zero(),
        };
        for c in self.coeffs.iter().cloned() {
            f.pcos = f.pcos + y_diff.pcos.clone() * c;
            f.psin = f.psin + y_diff.psin.clone() * c;

            y_diff = y_diff.derivative();
        }

        if f.pcos.deg().is_some() || f.psin.deg().is_some() {
            self.y_part.push(y);
            self.f.push(f);
        }
        self
    }
}

pub fn linear_combination_typst_rev(coeffs: &[f64], vars: &[String]) -> String {
    let mut result_str = String::new();
    let mut count_nonzero = 0;

    for (var, coeff) in (vars.iter()).zip(coeffs.iter().cloned()).rev() {
        let (num, denom) = rationalize_cf(coeff, 0.001);
        if num != 0 {
            count_nonzero += 1;
            match (num, denom) {
                (1, 1) if count_nonzero == 1 => {}
                (1, 1) => result_str.push_str("+"),
                (-1, 1) => result_str.push_str("-"),
                (num, 1) if count_nonzero == 1 => result_str.push_str(&format!("{num}")),
                (num, 1) => result_str.push_str(&format!("{num:+}")),
                (num, denom) if num < 0 => {
                    result_str.push_str(&format!("-({})/({})", num.abs(), denom))
                }
                (num, denom) if count_nonzero == 1 => {
                    result_str.push_str(&format!("({})/({})", num, denom))
                }
                (num, denom) => result_str.push_str(&format!("+({})/({})", num, denom)),
            }
            match var.as_str() {
                "" if (num.abs(), denom) == (1, 1) => result_str.push_str("1"),
                v => result_str.push_str(v),
            }
        }
    }
    result_str
}

pub fn linear_combination_typst(coeffs: &[f64], vars: &[String]) -> String {
    let mut result_str = String::new();
    let mut count_nonzero = 0;

    for (var, coeff) in (vars.iter()).zip(coeffs.iter().cloned()) {
        let (num, denom) = rationalize_cf(coeff, 0.001);
        if num != 0 {
            count_nonzero += 1;
            match (num, denom) {
                (1, 1) if count_nonzero == 1 => {}
                (1, 1) => result_str.push_str("+"),
                (-1, 1) => result_str.push_str("-"),
                (num, 1) if count_nonzero == 1 => result_str.push_str(&format!("{num}")),
                (num, 1) => result_str.push_str(&format!("{num:+}")),
                (num, denom) if num < 0 => {
                    result_str.push_str(&format!("-({})/({})", num.abs(), denom))
                }
                (num, denom) if count_nonzero == 1 => {
                    result_str.push_str(&format!("({})/({})", num, denom))
                }
                (num, denom) => result_str.push_str(&format!("+({})/({})", num, denom)),
            }
            match var.as_str() {
                "" if (num.abs(), denom) == (1, 1) => result_str.push_str("1"),
                v => result_str.push_str(v),
            }
        }
    }
    result_str
}

pub fn qpoly_as_typst(f: &QPoly, var: &str) -> String {
    let mut res = String::new();

    let QPoly { re, im, pcos, psin } = f;
    let re = rationalize_cf(*re, 0.001);
    let im = rationalize_cf(*im, 0.001);

    let exp_str = match re {
        (0, _) => "",
        (1, 1) => &format!("e^({var})"),
        (-1, 1) => &format!("e^(-{var})"),
        (n, 1) => &format!("e^({n} {var})"),
        (n, d) if n < 0 => &format!("e^(- {}/{} {var})", n.abs(), d),
        (n, d) => &format!("e^({}/{} {var})", n.abs(), d),
    };

    let polynomial_str = |coeffs: &[f64]| {
        let mut result_str = String::new();

        result_str.push_str(&linear_combination_typst_rev(
            coeffs,
            &((0..coeffs.len())
                .map(|power| match power {
                    0 => ("").to_string(),
                    1 => (var).to_string(),
                    p => format!("{var}^{p}"),
                })
                .collect::<Vec<_>>()),
        ));

        let mut count_nonzero = 0;
        for c in coeffs.iter().cloned() {
            if rationalize_cf(c, 0.001).0 != 0 {
                count_nonzero += 1;
            }
        }

        result_str = if count_nonzero == 1 || exp_str == "" && im.0 == 0 {
            if result_str.starts_with("-") {
                result_str
            } else {
                format!("+{result_str}")
            }
        } else if count_nonzero > 1 {
            format!("+({result_str})")
        } else {
            result_str
        };

        if !(exp_str == "" && im.0 == 0) {
            if &result_str == "+1" {
                return "+".to_string();
            } else if &result_str == "-1" {
                return "-".to_string();
            }
        }
        return result_str;
    };
    let pcos_str = polynomial_str(pcos.coeffs());
    let psin_str = polynomial_str(psin.coeffs());

    match im {
        (0, _) if exp_str == "" => res.push_str(&format!("{pcos_str}")),
        (0, _) => res.push_str(&format!("{pcos_str} {exp_str}")),
        (1, 1) => {
            if pcos_str != "" {
                res.push_str(&format!("{pcos_str} {exp_str}cos({var})"))
            }
            if psin_str != "" {
                res.push_str(&format!("{psin_str} {exp_str}sin({var})"))
            }
        }
        (n, 1) => {
            if pcos_str != "" {
                res.push_str(&format!("{pcos_str} {exp_str}cos({n} {var})"))
            }
            if psin_str != "" {
                res.push_str(&format!("{psin_str} {exp_str}sin({n} {var})"))
            }
        }
        (n, d) => {
            if pcos_str != "" {
                res.push_str(&format!("{pcos_str} {exp_str}cos({n}/{d} {var})"))
            }
            if psin_str != "" {
                res.push_str(&format!("{psin_str} {exp_str}sin({n}/{d} {var})"))
            }
        }
    }
    res = match res.strip_prefix("+") {
        None => res,
        Some(res_stripped) => res_stripped.to_string(),
    };

    if res.is_empty()  {
        res.push_str("0");
    }

    res
}

impl LinEq {
    pub fn eq_homo_as_typst(&self) -> String {
        let mut lhs = String::new();

        lhs.push_str(&linear_combination_typst_rev(
            &self.coeffs,
            &((0..self.coeffs.len())
                .map(|power| match power {
                    0 => ("y").to_string(),
                    1 => ("y'").to_string(),
                    2 => ("y''").to_string(),
                    3 => ("y'''").to_string(),
                    4 => ("y''''").to_string(),
                    p => format!("y^(({p}))"),
                })
                .collect::<Vec<_>>()),
        ));

        lhs.push_str("=0");
        lhs
    }
    pub fn eq_as_typst(&self) -> String {
        // let deg = self.coeffs.len() - 1;
        // let powers = (0..self.coeffs.len()).rev();
        // let coeffs = self.coeffs.iter().rev().cloned();

        let mut lhs = String::new();

        lhs.push_str(&linear_combination_typst_rev(
            &self.coeffs,
            &((0..self.coeffs.len())
                .map(|power| match power {
                    0 => ("y").to_string(),
                    1 => ("y'").to_string(),
                    2 => ("y''").to_string(),
                    3 => ("y'''").to_string(),
                    4 => ("y''''").to_string(),
                    p => format!("y^(({p}))"),
                })
                .collect::<Vec<_>>()),
        ));

        lhs.push_str("=");

        let mut rhs = String::new();

        if self.f.is_empty() {
            rhs.push_str("0");
        } else {
            for (i, f) in self.f.iter().enumerate() {
                let f_str = qpoly_as_typst(f, "x");

                if i != 0 && !f_str.starts_with("-") {
                    rhs.push_str("+");
                }
                rhs.push_str(&f_str);
            }
        }

        match rhs.strip_prefix("+") {
            None => lhs.push_str(&rhs),
            Some(rhs_stripped) => lhs.push_str(&rhs_stripped),
        }

        lhs
    }

    pub fn char_eq_as_typst(&self) -> String {
        let mut res = String::new();
        res.push_str(&linear_combination_typst_rev(
            &self.coeffs,
            &((0..self.coeffs.len())
                .map(|power| match power {
                    0 => ("").to_string(),
                    1 => ("lambda").to_string(),
                    p => format!("lambda^{p}"),
                })
                .collect::<Vec<_>>()),
        ));
        res.push_str("=0");
        res
    }

    pub fn solution_as_typst(&self) -> String {
        let mut res = String::new();

        res.push_str("y=");
        for (i, yy) in self.y0_basis.iter().enumerate() {
            let yy_str = qpoly_as_typst(yy, "x");

            if i != 0 {
                res.push_str("+");
            }

            res.push_str(&format!("c_({})", i + 1));

            res.push_str(&yy_str);
        }

        for (_i, yy) in self.y_part.iter().enumerate() {
            let yy_str = qpoly_as_typst(yy, "x");

            if !yy_str.starts_with("-") {
                res.push_str("+");
            }
            res.push_str(&yy_str);
        }
        res
    }
    pub fn solution_homo_as_typst(&self) -> String {
        let mut res = String::new();

        res.push_str("y_0=");
        for (i, yy) in self.y0_basis.iter().enumerate() {
            let yy_str = qpoly_as_typst(yy, "x");

            if i != 0 {
                res.push_str("+");
            }

            res.push_str(&format!("c_({})", i + 1));

            res.push_str(&yy_str);
        }

        res
    }

    pub fn char_roots_as_typst(&self) -> String {
        let mut res = String::new();
        let mut i = 1;
        for (root, m) in self.roots.iter() {
            if i != 1 {
                res.push_str(",");
            }

            match *root {
                Root::Real(root) => {
                    res.push_str(&format!("lambda_({i}"));
                    i += 1;
                    for _ in 1..*m {
                        res.push_str(&format!(",{i}"));
                        i += 1;
                    }
                    res.push_str(&format!(")={root}"));
                }
                Root::Complex { re, im } => {
                    res.push_str(&format!("lambda_({i}"));
                    i += 1;
                    for _ in 1..(2 * m) {
                        res.push_str(&format!(",{i}"));
                        i += 1;
                    }
                    match re {
                        0. => res.push_str(&format!(")= plus.minus {im} i")),
                        re => res.push_str(&format!(")={re} plus.minus {im} i")),
                    }
                }
            }
        }
        res.push_str(".");

        res
    }
}

/// Rational approximation of x using continued fractions.
/// Returns (numerator, denominator) such that |x - n/d| <= delta.
pub fn rationalize_cf(x: f64, delta: f64) -> (i64, i64) {
    let mut a = x.floor() as i64;
    let mut h1 = 1i64;
    let mut k1 = 0i64;
    let mut h = a;
    let mut k = 1i64;
    let mut x_rem = x - a as f64;

    while x_rem.abs() > delta {
        if x_rem == 0.0 {
            break;
        }
        let r = 1.0 / x_rem;
        a = r.floor() as i64;
        let h_next = a * h + h1;
        let k_next = a * k + k1;

        h1 = h;
        k1 = k;
        h = h_next;
        k = k_next;

        x_rem = r - a as f64;
    }

    assert!(((h as f64 / k as f64) - x).abs() < delta);

    let gcd = gcd::euclid_u64(h.abs() as u64, k.abs() as u64) as i64;
    (h / gcd, k / gcd)
}
