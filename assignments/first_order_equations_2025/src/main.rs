// mod format;
// mod linear_equation_old;
//
//
use std::io::{self, Read};

use base64::prelude::*;
use derive_typst_intoval::{IntoDict, IntoValue};
use image::{ImageBuffer, Rgba};

use rand::prelude::*;
use typst::{
    foundations::{Dict, IntoValue},
    layout::{Page, PagedDocument},
};
use typst_as_lib::typst_kit_options::TypstKitFontOptions;
use typst_render::render;
use variant_generation::{VariantGeneratorInput, VariantGeneratorOutput};

use indoc::indoc;

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
    solution: String,
}

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

    let task1_variants = vec![
        Task {
            problem: indoc!(
                "
                 = Решите следующее дифференциальное уравнение:
                 $
                 (x^2 - 1) y' - 1 = y^2
                 $
                 "
            )
            .to_string(),
            solution: indoc!(
                "
                = Решите $(x^2 - 1) y' - 1 = y^2$

                Разделим переменные: $ (dif y) / (1 + y^2) = (dif x) / (x^2 - 1) $.

                Проинтегрировав, получим $arctan y = 1/2 ln abs((x-1)/(x+1)) + c$.

                Можно выразить $y = tan (1/2 ln abs((x-1)/(x+1)) + c)$.
            "
            )
            .to_string(),
        },
        Task {
            problem: indoc!(
                "
                         = Решите следующее дифференциальное уравнение:
                         $
                         6 x y' = ln(x)/y^2
                         $
                         "
            )
            .to_string(),
            solution: indoc!(
                "
                = Решите $3 x y' = 2 ln(x)/y^2$

                Разделим переменные: $3 y^2 dif y = 2 (ln x)/x dif x$

                Проинтегрировав, получим $y^3 = ln^2 x + c$.

                Можно выразить $y = root(3, ln^2 x + c)$.
                "
            )
            .to_string(),
        },
        Task {
            problem: indoc!(
                "
                         = Решите следующее дифференциальное уравнение:
                         $
                         y'/sin(x) = x/cos(y)
                         $
                         "
            )
            .to_string(),
            solution: indoc!(
                "
                    = Решите $y'/sin(x) = x/cos(y)$

                    Разделим переменные: $cos(y) dif y = sin(x) dif x$

                    Проинтегрировав, получим $sin(y) = cos(x) + c$.
                    "
            )
            .to_string(),
        },
        Task {
            problem: indoc!(
                "
                         = Решите следующее дифференциальное уравнение:
                         $
                         y' = x y^(-1) e^(x - y)
                         $
                         "
            )
            .to_string(),
            solution: indoc!(
                "
                    = Решите $y' = x y^(-1) e^(x - y)$

                    Разделим переменные: $y e^y dif y = x e^x dif x$

                    Проинтегрировав по частям, получим $(y - 1)e^y = (x - 1)e^x + c$.
                    "
            )
            .to_string(),
        },
    ];

    let task2_variants = vec![
        Task {
            problem: indoc!(
                "
                = #[
                  Среди следующих дифференциальных уравнений укажите уравнения в полных 
                  дифференциалах и решите их. 
                ]

                == $(y e^x + 2 x ln y) dif x - (x^2/y + e^x + cos(y)) dif y = 0$, 

                == $(y e^x - 2 x ln y) dif x = (x^2/y - e^x - cos(y)) dif y$,  // y e^x - x^2 ln (y) + sin(y)

                == $(y e^x - 2 x ln y) dif x + (x^2/y - e^x + cos(y)) dif y = 0$,
                         "
            )
            .to_string(),
            solution: indoc!(
                "
                = Уравнением в полных дифференциалах является только уравнение (b)

                Его решение: $y e^x - x^2 ln (y) + sin(y) = c$
                "
            )
            .to_string(),
        },
        Task {
            problem: indoc!(
                "
                = #[
                  Среди следующих дифференциальных уравнений укажите уравнения в полных 
                  дифференциалах и решите их. 
                ]


                == $(e^(x - y) - sin(x)/y + 1) dif x + (cos(x)/y^2 - e^(x - y) - 1) dif y = 0$, 

                == $(e^(x - y) - sin(x)/y - 1) dif x + (cos(x)/y^2 + e^(x - y) - 1) dif y = 0$, 

                == $(e^(x - y) - sin(x)/y + 1) dif x = (cos(x)/y^2 + e^(x - y) + 1) dif y$, // cos(x)/y + e^(x - y) + x - y
                         "
            )
            .to_string(),
            solution: indoc!(
                "
                = Уравнением в полных дифференциалах является только уравнение (c)

                Его решение: $cos(x)/y + e^(x - y) + x - y = c$
                "
            )
            .to_string(),
        },
        Task {
            problem: indoc!(
                "
                = #[
                  Среди следующих дифференциальных уравнений укажите уравнения в полных 
                  дифференциалах и решите их. 
                ]


                == $(2 x y^2 - (2 x + 1) e^(2 x) + 2 y e^(2 x) ) dif x = ( e^(2 x) + 2 x^2 y) dif y$,

                // $x^2 y^2 + x e^(2 x) + y e^(2 x)$
                == $(2 x y^2 + (2 x + 1) e^(2 x) + 2 y e^(2 x) ) dif x + (e^(2 x) + 2 x^2 y) dif y = 0$, 

                == $(2 x y^2 - (2 x + 1) e^(2 x) + 2 y e^(2 x) ) dif x + (e^(2 x) - 2 x^2 y) dif y = 0$.
                         "
            )
            .to_string(),
            solution: indoc!(
                "
                = Уравнением в полных дифференциалах является только уравнение (b)

                Его решение: $x^2 y^2 + x e^(2 x) + y e^(2 x) = c$
                "
            )
            .to_string(),
        },
        Task {
            problem: indoc!(
                "
                = #[
                  Среди следующих дифференциальных уравнений укажите уравнения в полных 
                  дифференциалах и решите их. 
                ]


                  == $(1/(x e^y) + x^2 cos(y)) dif x + (ln(x)/e^y + 2 y^2 + x^3/3 sin(y)) dif y = 0$,

                  == $(1/(x e^y) + x^2 cos(y)) dif x = (ln(x)/e^y + 2 y + x^3 sin(y)) dif y$,

                  // ln(x) e^(-y) + 1/3 x^3 cos(y) - (y+1)^2
                  == $(1/(x e^y) + x^2 cos(y)) dif x = (ln(x)/e^y + 2 y + x^3/3 sin(y) + 2) dif y$,
                         "
            )
            .to_string(),
            solution: indoc!(
                "
                = Уравнением в полных дифференциалах является только уравнение (c)

                Его решение: $ln(x) e^(-y) + 1/3 x^3 cos(y) - (y+1)^2 = c$
                "
            )
            .to_string(),
        },
    ];

    let task3_variants = vec![
        Task {
            problem: indoc!(
                "
                = #[
                Можно ли заменой привести следующее уравнение к однородному? 
                Если да, то укажите эту замену и вид уравнения после замены. (Решать его не нужно). // 
                ]

                $
                y' = (3 x + y - 4)/(x - 2 y + 1)
                $
                "
            )
            .to_string(),
            solution: indoc!(
                "
                = Привести можно. Вид после замены: $v' = (3 u + v)/(u - 2 v)$. Замена: $x = u + 1$, $y = v + 1$.
                "
            )
            .to_string(),
        },
        Task {
            problem: indoc!(
                "
                = #[
                Можно ли заменой привести следующее уравнение к однородному? 
                Если да, то укажите эту замену и вид уравнения после замены. (Решать его не нужно). // 
                ]

                $
                y' = (-2 x + y + 5)/(x - 2 y + 3)
                $
                "
            )
            .to_string(),
            solution: indoc!(
                "
                = Привести можно. Вид после замены: $v' = (-2 u + v)/(u - 2 v)$. Замена: $x = u + 1$, $y = v - 1$.
                "
            )
            .to_string(),
        },
        Task {
            problem: indoc!(
                "
                = #[
                Можно ли заменой привести следующее уравнение к однородному? 
                Если да, то укажите эту замену и вид уравнения после замены. (Решать его не нужно). // 
                ]

                $
                (x - 3 y + 5) dif x = (2 y - 4 x - 10) dif y
                $
                "
            )
            .to_string(),
            solution: indoc!(
                "
                = Привести можно. Вид после замены: $(u - 3 v) dif u = (2 v - 4 u) dif v$. Замена: $x = u - 2$, $y = v + 1$.
                "
            )
            .to_string(),
        },
        Task {
            problem: indoc!(
                "
                = #[
                Можно ли заменой привести следующее уравнение к однородному? 
                Если да, то укажите эту замену и вид уравнения после замены. (Решать его не нужно). // 
                ]

                $
                (2 x - 3 y - 1) dif y = (y - 3 x - 2) dif x 
                $
                "
            )
            .to_string(),
            solution: indoc!(
                "
                = Привести можно. Вид после замены: $(2 u - 3 v) dif v = (v - 3 u) dif u$. Замена: $x = u - 1$, $y = v - 1$.
                "
            )
            .to_string(),
        },
    ];
    let task4_variants = vec![
        Task {
            problem: indoc!(
                "
                = Какие из следующих уравнений являются однородными?  Найдите решения тех, которые являются однородными. 

                #set math.lr(size: 100%)
                == $y' - (cos^2(y))/(cos^2(x)) = y/x$; #h(1fr)

                == $y' - cos^2(y/x) = y/x$; #h(1fr) //tg(y/x) = ln(x) + c

                == $y'/x - cos^2(y/x) = y/x$; #h(1fr) 

                "
            )
            .to_string(),
            solution: indoc!(
                "
                = Однородное уравнение только (b). Его решение: $tg(y/x) = ln(x) + c$
                "
            )
            .to_string(),
        },
        Task {
            problem: indoc!(
                "
                = Какие из следующих уравнений являются однородными?  Найдите решения тех, которые являются однородными. 

                #set math.lr(size: 100%)
                == $y'/x = 2 e^(-y/(2x)) + y/x$; #h(1fr)
                == $y' = 2 e^(-y/(2x)) + y/x$; #h(1fr) // e^(y/(2 x)) = ln(x) + c
                == $y' = (2 e^(-y))/e^(2x) + y/x$; #h(1fr) 

                "
            )
            .to_string(),
            solution: indoc!(
                "
                = Однородное уравнение только (b). Его решение: $e^(y/(2 x)) = ln(x) + c$
                "
            )
            .to_string(),
        },
        Task {
            problem: indoc!(
                "
                = Какие из следующих уравнений являются однородными?  Найдите решения тех, которые являются однородными. 

                #set math.lr(size: 100%)
                == $(y' - y)cos(y/x) = x$; #h(1fr) 
                == $(y' - y)/x^2 cos(y/x) = 1/y$; #h(1fr) 

                == $y' = ( y cos(y/x) + x)/(x cos(y/x))$. #h(1fr)// sin(y / x) = ln(x) + c

                "
            )
            .to_string(),
            solution: indoc!(
                "
                = Однородное уравнение только (c). Его решение: $sin(y / x) = ln(x) + c$
                "
            )
            .to_string(),
        },

        Task {
            problem: indoc!(
                "
                = Какие из следующих уравнений являются однородными?  Найдите решения тех, которые являются однородными. 

                == $(y' - y)sin(y/x) = x$; #h(1fr) 
                == $y' = ( y sin(y/x) + x)/(x sin(y/x))$. #h(1fr)// cos(y / x) = ln(x) + c

                == $(y' - y)/x^2 sin(y/x) = 1/y$; #h(1fr) 

                "
            )
            .to_string(),
            solution: indoc!(
                "
                = Однородное уравнение только (b). Его решение: $cos(y / x) = ln(x) + c$
                "
            )
            .to_string(),
        },

    ];
    let task5_variants = vec![
        Task {
            problem: indoc!("
                 = Решите следующие дифференциальные уравнения:
                 == $x y' =  y$.
                 == $2 x y' + sqrt(x) = 2 y$; 
                 == $6 sqrt(x) y y' + 1/y = (2 y^2)/sqrt(x)$; 
             ").to_string(),

            solution: indoc!(
                "
                = Решения:
                == Пункт (a) является уравнением с разделяющимися переменными, а также линейным однородным уравнением. Его решение: $y = c x$.
                == Пункт (b) является неоднородным линейным уравнением. Приводится к виду $y' = y/x + 1/(2 sqrt(x))$. Его решение: $y = c x - sqrt(x)$.
                == Пункт (c) является уравнением линейным Бернулли, приводящееся к предыдущему уравнению заменой $z = y^3$. Тогда $z = c x - sqrt(x)$ и $y = root(3, c x - sqrt(x))$.
                "
            )
            .to_string(),
        }

    ];

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
        task1_variants.choose(&mut rng).unwrap().clone(),
        task2_variants.choose(&mut rng).unwrap().clone(),
        task3_variants.choose(&mut rng).unwrap().clone(),
        task4_variants.choose(&mut rng).unwrap().clone(),
        task5_variants.choose(&mut rng).unwrap().clone(),
    ];

    let content = Content {
        variant: input.variant_number.to_string(),
        tasks: tasks.clone(),
    };

    log::debug!("{content:?}");

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
