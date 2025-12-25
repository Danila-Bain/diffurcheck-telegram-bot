use base64::{Engine, prelude::BASE64_STANDARD};
use derive_typst_intoval::{IntoDict, IntoValue};
use image::{ImageBuffer, Rgba};
use rand::prelude::*;
use std::{
    fs,
    io::{self, Read},
};
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

    // like {"variant_number": 10, "generator": "test_assignment"}
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)?;
    let input: VariantGeneratorInput = serde_json::from_str(&buffer).unwrap_or_else(|_err| {
        log::error!("Failed to parse input, using dummy value");
        VariantGeneratorInput {
            variant_number: 666,
            generator: "test_assignment".to_string(),
        }
    });

    let problem_code = String::from(include_str!("problem.typ"));
    let solution_code = String::from(include_str!("solution.typ"));

    let problem_template = typst_as_lib::TypstEngine::builder()
        .main_file(problem_code.clone())
        .search_fonts_with(
            TypstKitFontOptions::default()
                .include_system_fonts(false)
                // This line is not necessary, because thats the default.
                .include_embedded_fonts(true),
        )
        .build();

    let task_funny_theme = ["котёнка", "мыш", "файлы эпшетйна", "что-то про математику"]
        .choose(&mut rng)
        .unwrap();
    let task_funny = Task {
        title: "Смешные мемы (10 баллов)".to_string(),
        body: format!(
            "Пришлите два или три смешных мема. Оценка выставляется за лучший мем. Бонус за мем, который содержит {task_funny_theme}."
            ).to_string(),
    };

    let task_random = [
        Task {
            title: "Грустные мемы (10 баллов)".to_string(),
            body: "Пришлите два или три грустных мема. Оценка выставляется за самый грустный мем."
                .to_string(),
        },
        Task {
            title: "Непонятные мемы (10 баллов)".to_string(),
            body:
                "Пришлите два или три непонятных мема. Оценка выставляется за самый непонятный мем."
                    .to_string(),
        },
    ]
    .choose(&mut rng)
    .unwrap()
    .clone();

    let doc: PagedDocument = problem_template
        .compile_with_input(Content {
            variant: input.variant_number.to_string(),
            tasks: vec![task_funny, task_random],
        })
        .output
        .expect("typst::compile() returned an error!");

    let options = Default::default();
    let pdf = typst_pdf::pdf(&doc, &options).expect("Could not generate pdf.");
    fs::write("./output.pdf", pdf).expect("Could not write pdf.");

    // let problem_output = compile_typst_images(problem_code.clone());
    let problem_output = doc
        .pages
        .iter()
        .map(|page| render_page_to_png(page, 450. / 72.))
        .map(|img| BASE64_STANDARD.encode(img))
        .collect::<Vec<_>>();

    // let solution_output = compile_typst_images(solution_code.clone());
    let solution_output = vec![];

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
    title: String,
    body: String,
}

fn render_page_to_png(page: &Page, scale: f32) -> Vec<u8> {
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

    log::debug!(
        "Generated image:\n\x1b_Ga=T,f=100,c=60;{}\x1b\\\n",
        BASE64_STANDARD.encode(&buf)
    );

    buf
}
