use std::{
    io::{self, Read, Write},
    process::Command,
};

use base64::{Engine, prelude::BASE64_STANDARD};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct VariantGeneratorInput {
    pub variant_number: i32,
    pub generator: String,
}

#[derive(Serialize, Deserialize)]
pub struct VariantGeneratorOutput {
    pub variant_number: i32,
    pub generator: String,
    pub problem_code: String,
    pub problem_images: Vec<String>, // Base64-encoded PNGs
    pub solution_code: String,
    pub solution_images: Vec<String>, // Base64-encoded PNGs
}

impl VariantGeneratorInput {
    pub fn from_stdin() -> Result<Self, Box<dyn std::error::Error>> {
        let mut buffer = String::new();
        let _len = io::stdin().read_to_string(&mut buffer)?;
        Ok(serde_json::from_str(&buffer)?)
    }

    pub fn to_stdout(&self) -> Result<(), Box<dyn std::error::Error>> {
        let json_output = serde_json::to_string_pretty(self)?;
        println!("{json_output}");
        Ok(())
    }
}

impl VariantGeneratorOutput {
    pub fn from_stdin() -> Result<Self, Box<dyn std::error::Error>> {
        let mut buffer = String::new();
        let _len = io::stdin().read_to_string(&mut buffer)?;
        Ok(serde_json::from_str(&buffer)?)
    }

    pub fn to_stdout(&self) -> Result<(), Box<dyn std::error::Error>> {
        let json_output = serde_json::to_string_pretty(self)?;
        println!("{json_output}");
        Ok(())
    }
}

fn double_braced(key: &str) -> String {
    format!("{{{{{}}}}}", key)
}

pub fn double_braced_substitute(mut text: String, substitutions: &Vec<(String, String)>) -> String {
    for (key, value) in substitutions.iter() {
        text = text.replace(&double_braced(key), value)
    }
    text
}

#[deprecated]
pub fn compile_typst_images(source: String) -> Vec<String> {
    let mut images = vec![];
    for page in 1.. {
        let Ok(output) = Command::new("typst")
            .args(&[
                "compile",
                "--format=png",
                "--ppi=450",
                format!("--pages={page}").as_str(),
                "-", // stdin
                "-", // stdout
            ])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .and_then(|mut child| {
                child.stdin.as_mut().unwrap().write_all(source.as_bytes())?;
                child.wait_with_output()
            })
        else {
            break;
        };

        if output.status.success() {
            if output.stdout.len() > 0 {
                log::debug!("Generated PNG: {} bytes", output.stdout.len());
                images.push(output.stdout);
            } else {
                break;
            }
        } else {
            let err_msg = String::from_utf8_lossy(&output.stderr);
            log::error!("Typst error: {}", err_msg);
            break;
        }
    }

    images
        .into_iter()
        .map(|img| BASE64_STANDARD.encode(img))
        .collect()
}
