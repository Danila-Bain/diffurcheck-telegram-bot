use std::ops::AddAssign;
use std::ops::Div;

pub use nalgebra::{DMatrix, DVector, Matrix, OVector, Vector};
use num::One;
use num::Zero;
pub use polynomial_ring::Polynomial;
pub use polynomial_ring::polynomial;

use crate::linear_combination_typst_rev;
use crate::linear_equation::linear_combination_typst;
use crate::qpoly_as_typst;
use crate::rationalize_cf;
use crate::{Poly, QPoly, Root};

#[derive(Clone, PartialEq, Debug)]
pub struct LinSys {
    pub a_matrix: DMatrix<f64>,
    pub c_matrix: DMatrix<f64>,
    pub j_blocks: Vec<(Root, usize)>,
    pub roots: Vec<(Root, usize)>,
    pub char_coeffs: Vec<f64>,
    pub f: Vec<VQPoly>,
    pub y0_basis: Vec<VQPoly>,
    pub y_part: Vec<VQPoly>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct VQPoly {
    pub re: f64,
    pub im: f64,
    pub pcos: DVector<Poly>,
    pub psin: DVector<Poly>,
}

impl VQPoly {
    pub fn derivative(self) -> Self {
        let Self { re, im, pcos, psin } = self;

        let new_pcos = pcos.map(|p| p * re) + psin.map(|p| p * im) + pcos.map(|p| p.derivative());
        let new_psin = psin.map(|p| p * re) - pcos.map(|p| p * im) + psin.map(|p| p.derivative());

        Self {
            re,
            im,
            pcos: new_pcos,
            psin: new_psin,
        }
    }
}

#[must_use]
pub fn integral<R: Sized>(p: &Polynomial<R>) -> Polynomial<R>
where
    R: Sized + Zero + One + for<'x> AddAssign<&'x R> + for<'x> From<<&'x R as Div>::Output>,
    for<'x> &'x R: Div,
{
    let n = p.coeffs().len();
    let n = n + 1;
    let mut coeff = Vec::with_capacity(n);
    let mut i = R::one();
    coeff.push(R::zero());
    for c in p.coeffs().iter() {
        coeff.push(R::from(c / &i));
        i += &R::one();
    }
    Polynomial::new(coeff)
}

pub fn vqpoly_as_typst(f: &VQPoly) -> String {
    let mut res = String::new();

    let VQPoly { re, im, pcos, psin } = f;

    {
        let re = rationalize_cf(*re, 0.001);
        let exp_str = match re {
            (0, _) => "",
            (1, 1) => "e^(t)",
            (-1, 1) => "e^(-t)",
            (n, 1) => &format!("e^({n} t)"),
            (n, d) if n < 0 => &format!("e^(- {}/{} t)", n.abs(), d),
            (n, d) => &format!("e^({}/{} t)", n.abs(), d),
        };
        res.push_str(exp_str);
    }

    res.push_str("vec(");
    for (pcos, psin) in pcos.iter().zip(psin.iter()) {
        res.push_str(&qpoly_as_typst(
            &QPoly {
                re: 0.,
                im: *im,
                pcos: pcos.clone(),
                psin: psin.clone(),
            },
            "t",
        ));
        res.push_str(",");
    }
    res.push_str(")");

    match res.strip_prefix("+") {
        None => res,
        Some(res_stripped) => res_stripped.to_string(),
    }
}

impl LinSys {
    pub fn new(
        a_matrix: DMatrix<f64>,
        c_matrix: DMatrix<f64>,
        j_blocks: Vec<(Root, usize)>,
    ) -> Self {
        let mut y0_basis = vec![];

        let mut roots: Vec<(Root, usize)> = vec![];
        'outer: for (root, block_size) in j_blocks.iter() {
            for (r, m) in roots.iter_mut() {
                if r == root {
                    *m += block_size;
                    continue 'outer;
                }
            }
            roots.push((root.clone(), *block_size));
        }

        let mut char_polynomial = Poly::one();

        let mut i = 0;

        for (root, size) in j_blocks.iter() {
            match *root {
                Root::Real(r) => {
                    for _ in 0..*size {
                        char_polynomial = char_polynomial * polynomial![-r, 1.];
                    }

                    let v = c_matrix.column(i).clone().map(|c| polynomial![c]);

                    y0_basis.push(VQPoly {
                        re: r,
                        im: 0.,
                        pcos: v.clone(),
                        psin: v.map(|_| polynomial![]),
                    });

                    for j in 1..*size {
                        let h = y0_basis.last().unwrap().pcos.map(|p| integral(&p))
                            + c_matrix.column(i + j).clone().map(|c| polynomial![c]);

                        y0_basis.push(VQPoly {
                            re: r,
                            im: 0.,
                            pcos: h.clone(),
                            psin: h.map(|_| polynomial![]),
                        })
                    }

                    i += size;
                }
                Root::Complex { re, im } => {
                    for _ in 0..*size {
                        char_polynomial =
                            char_polynomial * polynomial![re * re + im * im, -2. * re, 1.];
                    }

                    let v1 = c_matrix.column(i).clone().map(|c| polynomial![c]);
                    let v2 = c_matrix.column(i + 1).clone().map(|c| polynomial![c]);

                    // true eighenvector V is - ?
                    // V e (cos + i sin) + ~V e (cos - i sin)
                    //
                    // goes to (2 Re V cos - 2 Im V sin)
                    // v1 = 2 Re V, v2 = -2 Im V

                    y0_basis.push(VQPoly {
                        re,
                        im,
                        pcos: v1.clone(),
                        psin: v2.clone(),
                    });
                    y0_basis.push(VQPoly {
                        re,
                        im,
                        pcos: v2.clone(),
                        psin: -v1.clone(),
                    });

                    for j in 1..*size {
                        let h1 = y0_basis[i + 2 * j - 2].pcos.map(|p| integral(&p))
                            + c_matrix
                                .column(i + 2 * j - 1)
                                .clone()
                                .map(|c| polynomial![c]);
                        let h2 = y0_basis[i + 2 * j - 2].psin.map(|p| integral(&p))
                            + c_matrix.column(i + 2 * j).clone().map(|c| polynomial![c]);

                        y0_basis.push(VQPoly {
                            re,
                            im,
                            pcos: h1.clone(),
                            psin: h2.clone(),
                        });
                        y0_basis.push(VQPoly {
                            re,
                            im,
                            pcos: h2.clone(),
                            psin: -h1.clone(),
                        });
                    }

                    i += size;
                    i += 2 * size;
                }
            }
        }

        Self {
            a_matrix,
            c_matrix,
            j_blocks,
            roots,
            f: vec![],
            y0_basis,
            y_part: vec![],
            char_coeffs: char_polynomial.coeffs().iter().cloned().collect(),
        }
    }

    pub fn with_y_part(self, y: VQPoly) -> Self {
        let mut y_part = self.y_part;
        y_part.push(y.clone());

        let y_der = y.clone().derivative();

        let y_mat = VQPoly {
            re: y.re,
            im: y.im,
            pcos: self.a_matrix.map(|c| polynomial![c]) * y.pcos,
            psin: self.a_matrix.map(|c| polynomial![c]) * y.psin,
        };

        let mut f = self.f;
        f.push(VQPoly {
            re: y.re,
            im: y.im,
            pcos: y_der.pcos - y_mat.pcos,
            psin: y_der.psin - y_mat.psin,
        });

        Self { f, y_part, ..self }
    }
}

impl LinSys {
    pub fn eq_homo_as_typst(&self) -> String {
        let mut res = String::new();
        res.push_str("cases(\n");

        let nrows = self.a_matrix.nrows();

        let vars = ["x", "y", "z", "w", "u", "v", "p", "q"]
            .into_iter()
            .take(nrows)
            .map(|v| v.to_string())
            .collect::<Vec<_>>();

        for (i, var) in vars.iter().enumerate() {
            res.push_str("\t");
            res.push_str(var);
            res.push_str("'=");

            res.push_str(&linear_combination_typst(
                &self.a_matrix.row(i).iter().copied().collect::<Vec<_>>(),
                &vars,
            ));
            res.push_str(",\n");
        }
        res.push_str(")");

        res
    }

    pub fn eq_as_typst(&self) -> String {
        let mut res = String::new();
        res.push_str("cases(\n");

        let nrows = self.a_matrix.nrows();

        let vars = ["x", "y", "z", "w", "u", "v", "p", "q"]
            .into_iter()
            .take(nrows)
            .map(|v| v.to_string())
            .collect::<Vec<_>>();

        for (i, var) in vars.iter().enumerate() {
            res.push_str(&format!("\t{var}'="));

            res.push_str(&linear_combination_typst(
                &self.a_matrix.row(i).iter().copied().collect::<Vec<_>>(),
                &vars,
            ));

            for (_, f) in self.f.iter().enumerate() {
                let f = QPoly {
                    re: f.re,
                    im: f.im,
                    pcos: f.pcos[i].clone(),
                    psin: f.psin[i].clone(),
                };
                let f_str = qpoly_as_typst(&f, "t");

                if !f_str.starts_with("-") {
                    res.push_str("+");
                }
                res.push_str(&f_str);
            }
            res.push_str(",\n");
        }
        res.push_str(")");

        res
    }

    pub fn char_eq_as_typst(&self) -> String {
        let mut res = String::new();

        if self.a_matrix.nrows() % 2 == 1 {
            res.push_str("-");
        }

        res.push_str("det mat(");
        for i in 0..self.a_matrix.nrows() {
            for j in 0..self.a_matrix.ncols() {
                let elem = self.a_matrix[(i, j)];
                match rationalize_cf(elem, 0.001) {
                    (0, _) => res.push_str("0"),
                    (num, 1) => res.push_str(&format!("{num}")),
                    (num, denom) => res.push_str(&format!("{num}/{denom}")),
                }
                if i == j {
                    res.push_str("-lambda")
                }
                // if j == self.a_matrix.ncols() - 1 {
                // } else {
                // }
                res.push_str(",")
            }
            res.push_str(";")
        }
        res.push_str(")=");
        res.push_str(&linear_combination_typst_rev(
            &self.char_coeffs,
            &((0..self.char_coeffs.len())
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

        let vars = ["x", "y", "z", "w", "u", "v", "p", "q"]
            .into_iter()
            .take(self.a_matrix.nrows())
            .map(|v| v.to_string())
            .collect::<Vec<_>>();
        let vars = vars.join(",");

        res.push_str(&format!("vec({vars})="));
        for (i, yy) in self.y0_basis.iter().enumerate() {
            let yy_str = vqpoly_as_typst(yy);

            if i != 0 {
                res.push_str("+");
            }

            res.push_str(&format!("c_({})", i + 1));

            res.push_str(&yy_str);
        }

        for (_i, yy) in self.y_part.iter().enumerate() {
            let yy_str = vqpoly_as_typst(yy);

            if !yy_str.starts_with("-") {
                res.push_str("+");
            }
            res.push_str(&yy_str);
        }
        res
    }
    pub fn solution_homo_as_typst(&self) -> String {
        let mut res = String::new();

        let vars = ["x_0", "y_0", "z_0", "w_0", "u_0", "v_0", "p_0", "q_0"]
            .into_iter()
            .take(self.a_matrix.nrows())
            .map(|v| v.to_string())
            .collect::<Vec<_>>();
        let vars = vars.join(",");

        res.push_str(&format!("vec({vars})="));
        for (i, yy) in self.y0_basis.iter().enumerate() {
            let yy_str = vqpoly_as_typst(yy);

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
