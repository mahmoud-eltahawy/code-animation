// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{collections::HashMap, fs::File, io::Read};

use serde::{Deserialize, Serialize};

use syntect::{
    html::{ClassStyle, ClassedHTMLGenerator},
    parsing::SyntaxSet,
    util::LinesWithEndings,
};

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
struct Config {
    name: String,
    lessons: HashMap<String, String>,
}

#[tauri::command]
fn open_config(path: &str) -> Result<Config, String> {
    fn open(path: &str) -> Result<Config, Box<dyn std::error::Error>> {
        let mut file = File::open(path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        let config = serde_json::from_str::<Config>(&content)?;
        Ok(config)
    }

    open(path).map_err(|x| x.to_string())
}

fn generate_highlighted_code(code_rs: &str) -> String {
    let ss = SyntaxSet::load_defaults_newlines();
    let sr_rs = ss.find_syntax_by_extension("rs").unwrap();
    let mut rs_html_generator =
        ClassedHTMLGenerator::new_with_class_style(sr_rs, &ss, ClassStyle::Spaced);
    for line in LinesWithEndings::from(code_rs) {
        rs_html_generator
            .parse_html_for_line_which_includes_newline(line)
            .unwrap();
    }
    rs_html_generator.finalize()
}

#[tauri::command]
fn read_file(path: &str) -> Result<String, String> {
    fn open(path: &str) -> Result<String, Box<dyn std::error::Error>> {
        let mut file = File::open(path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        Ok(generate_highlighted_code(&content))
    }

    open(path).map_err(|x| x.to_string())
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![open_config, read_file])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
