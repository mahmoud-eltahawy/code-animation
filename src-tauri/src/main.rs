// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{collections::HashMap, fs::File, io::Read};

use serde::{Deserialize, Serialize};

use similar::{utils::diff_lines, Algorithm, ChangeTag};
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

fn generate_html_from_code(code_rs: &str, extension: &str) -> Result<String, String> {
    let ss = SyntaxSet::load_defaults_newlines();
    let Some(sr_rs) = ss.find_syntax_by_extension(extension) else {
        return Err("syntax does not exist".to_string());
    };
    let mut rs_html_generator =
        ClassedHTMLGenerator::new_with_class_style(sr_rs, &ss, ClassStyle::Spaced);
    for line in LinesWithEndings::from(code_rs) {
        rs_html_generator
            .parse_html_for_line_which_includes_newline(line)
            .unwrap_or_default();
    }
    Ok(rs_html_generator.finalize())
}

static mut OLD_LINES: String = String::new();

#[tauri::command]
fn read_file(path: &str) -> Result<HashMap<usize, Option<String>>, String> {
    let Some(name_exten) = std::path::Path::new(path)
        .file_name()
        .and_then(|x| x.to_str().map(|x| x.split(".")))
    else {
        return Err("file name or extension problem".to_string());
    };
    let [_, extension] = name_exten.into_iter().collect::<Vec<_>>()[..] else {
        return Err("extension problem".to_string());
    };
    fn open(path: &str) -> Result<String, Box<dyn std::error::Error>> {
        let mut file = File::open(path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        Ok(content)
    }

    let new_lines = open(path).unwrap_or_default();
    unsafe {
        let r = diff_lines(Algorithm::Myers, &OLD_LINES, &new_lines)
            .into_iter()
            .enumerate()
            .flat_map(|(index, (tag, text))| match tag {
                ChangeTag::Delete => Some((index, None)),
                ChangeTag::Insert => match generate_html_from_code(&text, extension) {
                    Ok(text) => Some((index, Some(text))),
                    Err(_) => None,
                },
                ChangeTag::Equal => None,
            })
            .collect::<HashMap<_, _>>();
        OLD_LINES = new_lines.clone();
        return Ok(r);
    }
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![open_config, read_file])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
