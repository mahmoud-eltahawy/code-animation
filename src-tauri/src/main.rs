// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![feature(iter_intersperse)]
use global_hotkey::{
    hotkey::{Code, HotKey, Modifiers},
    GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState,
};
use tauri::Manager;

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

static mut CODE_OLD_LINES: Vec<String> = Vec::new();

#[inline(always)]
fn get_old_code<'a>(new_code: &Vec<String>) -> Vec<(String, String)> {
    unsafe {
        let lines1 = CODE_OLD_LINES
            .clone()
            .into_iter()
            .intersperse("\n".to_string())
            .collect::<String>();
        let lines2 = new_code
            .to_owned()
            .into_iter()
            .intersperse("\n".to_string())
            .collect::<String>();
        let diffs = diff_lines(Algorithm::Myers, &lines1, &lines2)
            .into_iter()
            .flat_map(|(tag, str)| match tag {
                ChangeTag::Delete => {
                    Some(("-1".to_string(), str.replace(NEW_LINE, "\n").to_string()))
                }
                ChangeTag::Insert => {
                    Some(("1".to_string(), str.replace(NEW_LINE, "\n").to_string()))
                }
                ChangeTag::Equal => None,
            })
            .collect::<Vec<_>>();
        diffs
    }
}

const NEW_LINE: &str = "*#%NEW_LINE%#*";
fn set_old_code(new_lines: Vec<String>) {
    unsafe { CODE_OLD_LINES = new_lines }
}

#[tauri::command]
fn read_file(path: &str) -> Result<HashMap<i64, Option<String>>, String> {
    let Some(name_exten) = std::path::Path::new(path)
        .file_name()
        .and_then(|x| x.to_str().map(|x| x.split('.')))
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

    let r = if extension == "md" {
        let html = markdown::to_html(&new_lines);
        HashMap::from([(-1, Some(html))])
    } else {
        let html = generate_html_from_code(&new_lines, extension)?;
        HashMap::from([(0, Some(html))])
    };
    Ok(r)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let keys_manager = GlobalHotKeyManager::new()?;

    let open_lesson = HotKey::new(None, Code::KeyO);
    let quit_lesson = HotKey::new(None, Code::KeyQ);
    let font_increase = HotKey::new(None, Code::Equal);
    let font_decrease = HotKey::new(None, Code::Minus);
    let next_snippet = HotKey::new(None, Code::KeyL);
    let previous_snippet = HotKey::new(None, Code::KeyH);
    let next_snippet_stacked = HotKey::new(Some(Modifiers::SHIFT), Code::KeyL);

    keys_manager.register_all(&[
        open_lesson,
        quit_lesson,
        font_increase,
        font_decrease,
        next_snippet,
        previous_snippet,
        next_snippet_stacked,
    ])?;

    tauri::Builder::default()
        .setup(move |app| {
            let main_window1 = app.get_window("main").unwrap();
            let main_window2 = app.get_window("main").unwrap();
            let main_window3 = app.get_window("main").unwrap();
            main_window1.listen("set_code", move |ev| {
                let payload = ev.payload().unwrap_or_default();
                let payload: Vec<String> = serde_json::from_str(payload).unwrap();
                let payload = payload
                    .into_iter()
                    .map(|x| {
                        let x = x.replace("\\\"", "\"");
                        let x = x.replace("\\n", NEW_LINE);
                        x.replace("\n", NEW_LINE)
                    })
                    .collect::<Vec<_>>();
                main_window3
                    .emit("new_code", get_old_code(&payload))
                    .unwrap();
                set_old_code(payload);
            });
            std::thread::spawn(move || loop {
                if let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
                    if event.state == HotKeyState::Pressed {
                        if event.id == open_lesson.id() {
                            main_window2.emit(stringify!(open_lesson), ()).unwrap();
                        } else if event.id == quit_lesson.id() {
                            main_window2.emit(stringify!(quit_lesson), ()).unwrap();
                        } else if event.id == next_snippet.id() {
                            main_window2.emit(stringify!(next_snippet), ()).unwrap();
                        } else if event.id == previous_snippet.id() {
                            main_window2.emit(stringify!(previous_snippet), ()).unwrap();
                        } else if event.id == font_increase.id() {
                            main_window2.emit(stringify!(font_increase), ()).unwrap();
                        } else if event.id == font_decrease.id() {
                            main_window2.emit(stringify!(font_decrease), ()).unwrap();
                        }
                    }
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![open_config, read_file])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    Ok(())
}
