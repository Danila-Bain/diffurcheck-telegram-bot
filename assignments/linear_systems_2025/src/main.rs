// mod format;
// mod linear_equation_old;
//
//
pub mod linear_equation;
pub mod linear_system;

use std::io::{self, Read};

use base64::prelude::*;
use derive_typst_intoval::{IntoDict, IntoValue};
use image::{ImageBuffer, Rgba};
use linear_equation::*;
use linear_system::*;

use nalgebra::{Matrix2, Matrix3, vector};
use polynomial_ring::polynomial;
use rand::prelude::*;
use typst::{
    foundations::{Dict, IntoValue},
    layout::{Page, PagedDocument},
};
use typst_as_lib::typst_kit_options::TypstKitFontOptions;
use typst_render::render;
use variant_generation::{VariantGeneratorInput, VariantGeneratorOutput};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();
    let mut rng = rand::rng();

    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)?;
    let input: VariantGeneratorInput = serde_json::from_str(&buffer).unwrap_or_else(|_err| {
        log::error!("Failed to parse input, using dummy value");
        VariantGeneratorInput {
            variant_number: 666,
            generator: "test_assignment".to_string(),
        }
    });

    // Task 1:
    // - distinct real roots
    // - e^x + x^2 or x e^x
    let eq1 = {
        let root_options: [f64; _] = [-3., -2., -1., 1., 2., 3.];
        let roots = loop {
            let (a, b) = (
                root_options.choose(&mut rng).unwrap(),
                root_options.choose(&mut rng).unwrap(),
            );
            if a.abs() != b.abs() {
                break (*a, *b);
            }
        };
        let mut eq = LinEq::from_roots(vec![Root::Real(roots.0), Root::Real(roots.1)]);

        let k = loop {
            let k = *root_options.choose(&mut rng).unwrap();
            if k != roots.0 && k != roots.1 {
                break k;
            }
        };
        let coeff_options = [-5., -3., -2., 2., 3., 5., 7.];
        let coeff1 = *coeff_options.choose(&mut rng).unwrap();
        let coeff2 = *coeff_options.choose(&mut rng).unwrap();
        let coeff3 = *coeff_options.choose(&mut rng).unwrap();

        if rand::random_bool(0.5) {
            let y1 = QPoly {
                re: k,
                im: 0.,
                pcos: polynomial![coeff1],
                psin: Poly::zero(),
            };

            let y2 = QPoly {
                re: 0.,
                im: 0.,
                pcos: polynomial![coeff2, 0., coeff3],
                psin: Poly::zero(),
            };
            eq = eq.with_y_part(y1).with_y_part(y2);
        } else {
            let y1 = QPoly {
                re: k,
                im: 0.,
                pcos: polynomial![0., 0., coeff3],
                psin: Poly::zero(),
            };

            eq = eq.with_y_part(y1);
        }
        eq
    };

    let eq2 = {
        if rand::random_bool(0.5) {
            // let root_options: [f64; _] = [-3., -2., -1., 1., 2., 3.];
            let root = *[-3., -2., -1., 1., 2., 3.].choose(&mut rng).unwrap();
            let mut eq = LinEq::from_roots(vec![Root::Real(root), Root::Real(root)]);

            let coeff1 = *[-5., -3., -2., 2., 3., 5., 7.].choose(&mut rng).unwrap();
            let coeff2 = *[-5., -3., -2., 2., 3., 5., 7.].choose(&mut rng).unwrap();

            let y1 = QPoly {
                re: root,
                im: 0.,
                pcos: polynomial![0., 0., coeff1],
                psin: Poly::zero(),
            };

            let y2 = QPoly {
                re: -root,
                im: 0.,
                pcos: polynomial![coeff2],
                psin: Poly::zero(),
            };
            eq = eq.with_y_part(y1).with_y_part(y2);
            eq
        } else {
            let im = *[2., 3., 4.].choose(&mut rng).unwrap();
            let mut eq = LinEq::from_roots(vec![Root::Complex { re: 0., im }]);

            let re = *[-3., -2., -1., 1., 2., 3.].choose(&mut rng).unwrap();
            let coeff1 = *[-5., -3., -2., 2., 3., 5., 7.].choose(&mut rng).unwrap();
            let coeff2 = *[-5., -3., -2., 2., 3., 5., 7.].choose(&mut rng).unwrap();

            if rand::random_bool(0.5) {
                let y1 = QPoly {
                    re: 0.,
                    im: im,
                    pcos: polynomial![coeff1],
                    psin: Poly::zero(),
                };
                let y2 = QPoly {
                    re: re,
                    im: im,
                    pcos: Poly::zero(),
                    psin: polynomial![coeff2],
                };
                eq = eq.with_y_part(y1).with_y_part(y2);
                eq.y_part.shuffle(&mut rng);
            } else {
                let y1 = QPoly {
                    re: 0.,
                    im: im,
                    pcos: Poly::zero(),
                    psin: polynomial![coeff1],
                };
                let y2 = QPoly {
                    re: re,
                    im: im,
                    pcos: polynomial![coeff2],
                    psin: Poly::zero(),
                };
                eq = eq.with_y_part(y1).with_y_part(y2);
                eq.y_part.shuffle(&mut rng);
            }
            eq
        }
    };

    // Bi quadratic complex
    let eq3 = {
        // let root_options: [f64; _] = [-3., -2., -1., 1., 2., 3.];
        let im = *[2., 3., 4.].choose(&mut rng).unwrap();

        let mut eq = LinEq::from_roots(vec![
            Root::Complex { re: 0., im },
            Root::Complex { re: 0., im },
        ]);

        let coeff1 = *[-5., -3., -2., 2., 3., 5., 7.].choose(&mut rng).unwrap();

        let y1 = QPoly {
            re: 0.,
            im: im,
            pcos: polynomial![0., 0., coeff1],
            psin: Poly::zero(),
        };

        eq = eq.with_y_part(y1);
        eq
    };

    let sys1 = {
        let root_options: [f64; _] = [-3., -2., -1., 1., 2., 3.];
        let roots = loop {
            let (a, b) = (
                root_options.choose(&mut rng).unwrap(),
                root_options.choose(&mut rng).unwrap(),
            );
            if a.abs() != b.abs() {
                break (*a, *b);
            }
        };

        let j_mat = Matrix2::from_diagonal(&vector![roots.0, roots.1]);
        let j_blocks = vec![(Root::Real(roots.0), 1), (Root::Real(roots.1), 1)];

        let c_mat = 'c: loop {
            let data: [f64; 4] =
                std::array::from_fn(|_| *[-2., -1., 1., 2.].choose(&mut rng).unwrap());

            let c_mat = Matrix2::from_row_slice(&data);

            if let Some(inv) = c_mat.try_inverse() {
                for &el in inv.as_slice().iter() {
                    if el.floor() != el {
                        continue 'c;
                    }
                }

                break c_mat;
            }
        };

        let a_mat = c_mat * j_mat * c_mat.try_inverse().unwrap();

        let mut sys = LinSys::new(
            a_mat.resize(a_mat.nrows(), a_mat.ncols(), 0.),
            c_mat.resize(c_mat.nrows(), c_mat.ncols(), 0.),
            j_blocks,
        );

        let k = loop {
            let k = *root_options.choose(&mut rng).unwrap();
            if k != roots.0 && k != roots.1 {
                break k;
            }
        };
        let coeff_options = [-5., -3., -2., 2., 3., 5., 7.];
        let coeff1 = *coeff_options.choose(&mut rng).unwrap();
        let coeff2 = *coeff_options.choose(&mut rng).unwrap();
        let coeff3 = *coeff_options.choose(&mut rng).unwrap();

        if rand::random_bool(0.5) {
            let mut p1 = vec![polynomial![coeff1], Poly::zero()];
            p1.shuffle(&mut rng);

            let y1 = VQPoly {
                re: k,
                im: 0.,
                pcos: DVector::from_vec(p1),
                psin: DVector::from_vec(vec![Poly::zero(), Poly::zero()]),
            };

            let mut p2 = vec![polynomial![coeff2], polynomial![0., 0., coeff3]];
            p2.shuffle(&mut rng);

            let y2 = VQPoly {
                re: 0.,
                im: 0.,
                pcos: DVector::from_vec(p2),
                psin: DVector::from_vec(vec![Poly::zero(), Poly::zero()]),
            };
            sys = sys.with_y_part(y1).with_y_part(y2);
        } else {
            let mut p1 = vec![polynomial![coeff1, 0., coeff3], polynomial![0., coeff2, 0.]];
            p1.shuffle(&mut rng);
            let y1 = VQPoly {
                re: k,
                im: 0.,
                pcos: DVector::from_vec(p1),
                psin: DVector::from_vec(vec![polynomial![], polynomial![]]),
            };

            sys = sys.with_y_part(y1);
        }
        sys
    };

    let sys2 = {
        if rand::random_bool(0.5) {
            let root_options: [f64; _] = [-3., -2., -1., 1., 2., 3.];
            let root = *root_options.choose(&mut rng).unwrap();

            let j_mat = Matrix2::from_row_slice(&[root, 1., 0., root]);
            let j_blocks = vec![(Root::Real(root), 2)];

            let c_mat = 'c: loop {
                let data: [f64; 4] =
                    std::array::from_fn(|_| *[-2., -1., 1., 2.].choose(&mut rng).unwrap());

                let c_mat = Matrix2::from_row_slice(&data);

                if let Some(inv) = c_mat.try_inverse() {
                    for &el in inv.as_slice().iter() {
                        if el.floor() != el {
                            continue 'c;
                        }
                    }

                    break c_mat;
                }
            };

            let a_mat = c_mat * j_mat * c_mat.try_inverse().unwrap();

            let sys = LinSys::new(
                a_mat.resize(a_mat.nrows(), a_mat.ncols(), 0.),
                c_mat.resize(c_mat.nrows(), c_mat.ncols(), 0.),
                j_blocks,
            );

            let coeff_options = [-5., -3., -2., 2., 3., 5., 7.];
            let coeff1 = *coeff_options.choose(&mut rng).unwrap();
            let coeff2 = *coeff_options.choose(&mut rng).unwrap();

            let mut p1 = vec![polynomial![coeff1], Poly::zero()];
            let mut p2 = vec![Poly::zero(), polynomial![coeff2]];
            if rand::random_bool(0.5) {
                std::mem::swap(&mut p1, &mut p2);
            }

            let y1 = VQPoly {
                re: root,
                im: 0.,
                pcos: DVector::from_vec(p1),
                psin: DVector::from_vec(vec![Poly::zero(), Poly::zero()]),
            };

            let y2 = VQPoly {
                re: -root,
                im: 0.,
                pcos: DVector::from_vec(p2),
                psin: DVector::from_vec(vec![Poly::zero(), Poly::zero()]),
            };

            sys.with_y_part(y1).with_y_part(y2)
        } else {
            let root_options: [f64; _] = [-3., -2., -1., 1., 2., 3.];
            let re = *root_options.choose(&mut rng).unwrap();
            let im = *[1., 2., 3.].choose(&mut rng).unwrap();

            let j_mat = Matrix2::from_row_slice(&[re, im, -im, re]);
            let j_blocks = vec![(Root::Complex { re, im }, 1)];

            let c_mat = 'c: loop {
                let data: [f64; 4] =
                    std::array::from_fn(|_| *[-2., -1., 1., 2.].choose(&mut rng).unwrap());

                let c_mat = Matrix2::from_row_slice(&data);

                if let Some(inv) = c_mat.try_inverse() {
                    for &el in inv.as_slice().iter() {
                        if el.floor() != el {
                            continue 'c;
                        }
                    }

                    break c_mat;
                }
            };

            let a_mat = c_mat * j_mat * c_mat.try_inverse().unwrap();

            let sys = LinSys::new(
                a_mat.resize(a_mat.nrows(), a_mat.ncols(), 0.),
                c_mat.resize(c_mat.nrows(), c_mat.ncols(), 0.),
                j_blocks,
            );

            let coeff_options = [-5., -3., -2., 2., 3., 5., 7.];
            let coeff1 = *coeff_options.choose(&mut rng).unwrap();
            let coeff2 = *coeff_options.choose(&mut rng).unwrap();

            let mut p1 = vec![polynomial![coeff1], Poly::zero()];
            let mut p2 = vec![Poly::zero(), polynomial![coeff2]];
            if rand::random_bool(0.5) {
                std::mem::swap(&mut p1, &mut p2);
            }

            let y1 = VQPoly {
                re: re,
                im: 0.,
                pcos: DVector::from_vec(p1),
                psin: DVector::from_vec(vec![Poly::zero(), Poly::zero()]),
            };

            let y2 = VQPoly {
                re: re,
                im: im,
                pcos: DVector::from_vec(p2),
                psin: DVector::from_vec(vec![Poly::zero(), Poly::zero()]),
            };

            sys.with_y_part(y1).with_y_part(y2)
        }
    };

    let sys3 = {
        let root_options: [f64; _] = [-3., -2., -1., 1., 2., 3.];
        let root = *root_options.choose(&mut rng).unwrap();

        let j_mat = Matrix3::from_row_slice(&[
            root, 1., 0., //
            0., root, 0., //
            0., 0., root, //
        ]);
        let j_blocks = vec![(Root::Real(root), 2), (Root::Real(root), 1)];

        let c_mat = 'try_c: loop {
            let data: [f64; 9] =
                std::array::from_fn(|_| *[-2., -1., 1., 2.].choose(&mut rng).unwrap());

            let c_mat = Matrix3::from_row_slice(&data);

            if c_mat.abs().sum() >= 7.
                && let Some(inv) = c_mat.try_inverse()
            {
                for &el in inv.as_slice().iter() {
                    if el.floor() != el {
                        continue 'try_c;
                    }
                }

                break c_mat;
            }
        };

        let a_mat = c_mat * j_mat * c_mat.try_inverse().unwrap();

        let sys = LinSys::new(
            a_mat.resize(a_mat.nrows(), a_mat.ncols(), 0.),
            c_mat.resize(c_mat.nrows(), c_mat.ncols(), 0.),
            j_blocks,
        );

        let coeff_options = [2., 3., 5., 7.];
        let coeff1 = *coeff_options.choose(&mut rng).unwrap();

        let mut p1 = vec![polynomial![coeff1], Poly::zero(), Poly::zero()];
        p1.shuffle(&mut rng);

        let y1 = VQPoly {
            re: root,
            im: 0.,
            pcos: DVector::from_vec(p1),
            psin: DVector::from_vec(vec![Poly::zero(), Poly::zero(), Poly::zero()]),
        };

        sys.with_y_part(y1)
    };

    let problem_code = String::from(include_str!("problem.typ"));
    let solution_code = String::from(include_str!("solution.typ"));

    let problem_engine = typst_as_lib::TypstEngine::builder()
        .main_file(problem_code.clone())
        .search_fonts_with(
            TypstKitFontOptions::default()
                .include_system_fonts(false)
                // This line is not necessary, because thats the default.
                .include_embedded_fonts(true),
        )
        .build();

    let solution_engine = typst_as_lib::TypstEngine::builder()
        .main_file(solution_code.clone())
        .search_fonts_with(
            TypstKitFontOptions::default()
                .include_system_fonts(false)
                // This line is not necessary, because thats the default.
                .include_embedded_fonts(true),
        )
        .build();

    let tasks = vec![
        Task {
            problem: "Для следующих однородного и неоднородного уравнений, найдите общее решение"
                .to_string(),
            equation_homo: eq1.eq_homo_as_typst(),
            equation: eq1.eq_as_typst(),
            char_equation: eq1.char_eq_as_typst(),
            solution_homo: eq1.solution_homo_as_typst(),
            solution: eq1.solution_as_typst(),
            char_roots: eq1.char_roots_as_typst()
        },
        Task {
            problem: "Для следующих однородного и неоднородного уравнений, найдите общее решение"
                .to_string(),
            equation_homo: eq2.eq_homo_as_typst(),
            char_equation: eq2.char_eq_as_typst(),
            solution_homo: eq2.solution_homo_as_typst(),
            equation: eq2.eq_as_typst(),
            solution: eq2.solution_as_typst(),
            char_roots: eq2.char_roots_as_typst()
        },
        Task {
            problem: "Для следующих однородного и неоднородного уравнений, найдите общее решение"
                .to_string(),
            equation_homo: eq3.eq_homo_as_typst(),
            char_equation: eq3.char_eq_as_typst(),
            solution_homo: eq3.solution_homo_as_typst(),
            equation: eq3.eq_as_typst(),
            solution: eq3.solution_as_typst(),
            char_roots: eq3.char_roots_as_typst()
        },
        Task {
            problem: "Для следующих однородной и неоднородной системы уравнений уравнений, найдите общее решение"
                .to_string(),
            equation_homo: sys1.eq_homo_as_typst(),
            equation: sys1.eq_as_typst(),
            char_equation: sys1.char_eq_as_typst(),
            solution_homo: sys1.solution_homo_as_typst(),
            solution: sys1.solution_as_typst(),
            char_roots: sys1.char_roots_as_typst()
        },
        Task {
            problem: "Для следующих однородной и неоднородной системы уравнений уравнений, найдите общее решение"
                .to_string(),
            equation_homo: sys2.eq_homo_as_typst(),
            char_equation: sys2.char_eq_as_typst(),
            solution_homo: sys2.solution_homo_as_typst(),
            equation: sys2.eq_as_typst(),
            solution: sys2.solution_as_typst(),
            char_roots: sys2.char_roots_as_typst()
        },
        Task {
            problem: "Для следующих однородной и неоднородной системы уравнений уравнений, найдите общее решение"
                .to_string(),
            equation_homo: sys3.eq_homo_as_typst(),
            char_equation: sys3.char_eq_as_typst(),
            solution_homo: sys3.solution_homo_as_typst(),
            equation: sys3.eq_as_typst(),
            solution: sys3.solution_as_typst(),
            char_roots: sys3.char_roots_as_typst()
        },
    ];

    let content = Content {
            variant: input.variant_number.to_string(),
            tasks: tasks.clone(),
        };

    // log::debug!("{content:?}");

    let problem_doc: PagedDocument = problem_engine
        .compile_with_input(content)
        .output
        .expect("typst::compile() returned an error!");

    let solution_doc: PagedDocument = solution_engine
        .compile_with_input(Content {
            variant: input.variant_number.to_string(),
            tasks: tasks.clone(),
        })
        .output
        .expect("typst::compile() returned an error!");

    let problem_output = problem_doc
        .pages
        .iter()
        .map(|page| render_page_to_png(page, 300. / 72.))
        .map(|img| BASE64_STANDARD.encode(img))
        .collect::<Vec<_>>();
    let solution_output = solution_doc
        .pages
        .iter()
        .map(|page| render_page_to_png(page, 300. / 72.))
        .map(|img| BASE64_STANDARD.encode(img))
        .collect::<Vec<_>>();

    let output = VariantGeneratorOutput {
        variant_number: input.variant_number,
        generator: input.generator,
        problem_code,
        problem_images: problem_output,
        solution_code,
        solution_images: solution_output,
    };

    println!("{}", serde_json::to_string(&output)?);

    Ok(())
}

#[derive(Debug, Clone, IntoValue, IntoDict)]
struct Content {
    variant: String,
    tasks: Vec<Task>,
}

// Implement Into<Dict> manually, so we can just pass the struct
// to the compile function.
impl From<Content> for Dict {
    fn from(value: Content) -> Self {
        value.into_dict()
    }
}

#[derive(Debug, Clone, IntoValue, IntoDict)]
struct Task {
    problem: String,
    equation_homo: String,
    equation: String,
    char_equation: String,
    char_roots: String,
    solution_homo: String,
    solution: String,
}

fn render_page_to_png(page: &Page, scale: f32) -> Vec<u8> {
    let make_buf = |page, scale| {
        let pixmap = render(page, scale);

        log::debug!(
            "Render page with {} width and {} height",
            pixmap.width(),
            pixmap.height()
        );

        let img = ImageBuffer::<Rgba<u8>, _>::from_raw(
            pixmap.width(),
            pixmap.height(),
            pixmap.data().to_vec(),
        )
        .expect("invalid pixmap");

        let mut buf = Vec::new();
        img.write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)
            .unwrap();
        buf
    };

    log::debug!(
        "Generated image:\n\x1b_Ga=T,f=100;{}\x1b\\\n",
        BASE64_STANDARD.encode(&make_buf(page, 1.))
    );

    make_buf(page, scale)
}
