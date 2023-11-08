// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use global_hotkey::{GlobalHotKeyEvent, HotKeyState};
use tauri::{Manager, State};

use std::{collections::HashMap, fs::File, io::Read};

use serde::{Deserialize, Serialize};

use syntect::{
    html::{ClassStyle, ClassedHTMLGenerator},
    parsing::SyntaxSet,
    util::LinesWithEndings,
};

use html_editor::Element;
use html_editor::{parse, Node};

mod html_element;
mod keys;
mod state;

use html_element::*;
use keys::*;
use state::*;

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
struct Config {
    name: String,
    lessons: HashMap<String, String>,
}

fn wrap_with_id_spans(
    nodes: Vec<Node>,
    genration: usize,
    family: String,
    syntax_set: &SyntaxSet,
) -> Vec<Node> {
    let mapping = |(index, node)| match node {
        Node::Element(ele) => {
            let mut element = ele;
            element.attrs = element
                .attrs
                .into_iter()
                .filter(|x| x.0 != "id")
                .chain(vec![(
                    "id".to_string(),
                    format!("{genration}:{index}@{family}"),
                )])
                .collect::<Vec<_>>();
            element.code_children(syntax_set);
            element.children = wrap_with_id_spans(
                element.children,
                genration + 1,
                format!("{}:{}", family, index),
                syntax_set,
            );
            element.into_node()
        }
        _ => Node::Element(Element::new(
            "span",
            vec![("id", format!("{genration}:{index}@{family}").as_str())],
            vec![node],
        )),
    };
    nodes
        .into_iter()
        .enumerate()
        .map(mapping)
        .collect::<Vec<_>>()
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

fn generate_html_from_code(
    code: &str,
    name: &str,
    syntaxt_set: &SyntaxSet,
) -> Result<String, String> {
    let (first, else_letters) = name.split_at(1);
    let first = first.to_uppercase();
    let language_name = first + else_letters;
    let syntax_ref = syntaxt_set.find_syntax_by_name(&language_name);
    let syntax_ref = match syntax_ref {
        Some(s) => s,
        None => match syntaxt_set.find_syntax_by_extension(name) {
            Some(s) => s,
            None => return Err("syntax does not exist".to_string()),
        },
    };
    let mut rs_html_generator =
        ClassedHTMLGenerator::new_with_class_style(syntax_ref, syntaxt_set, ClassStyle::Spaced);
    for line in LinesWithEndings::from(code) {
        rs_html_generator
            .parse_html_for_line_which_includes_newline(line)
            .unwrap_or_default();
    }
    Ok(rs_html_generator.finalize())
}

#[tauri::command]
fn read_file(
    syntax_set: State<'_, SyntaxSet>,
    state: State<'_, AppState>,
    path: &str,
) -> Result<Vec<(String, String)>, String> {
    let Some(name_exten) = std::path::Path::new(path)
        .file_name()
        .and_then(|x| x.to_str().map(|x| x.split('.')))
    else {
        return Err("file name or extension problem".to_string());
    };
    let [_, extension] = name_exten.into_iter().collect::<Vec<_>>()[..] else {
        return Err("extension problem".to_string());
    };

    let is_markdown = extension == "md";

    fn open(path: &str) -> Result<String, Box<dyn std::error::Error>> {
        let mut file = File::open(path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        Ok(content)
    }

    fn replace_new_lines(html: String) -> String {
        html.replace("\\n", NEW_LINE).replace('\n', NEW_LINE)
    }

    let new_lines = open(path).unwrap_or_default();

    let result = if is_markdown {
        let html = markdown::to_html(&new_lines);
        let html = replace_new_lines(html);
        let html = html.replace("&quot;", "\"");

        let dom = parse(&html).unwrap_or_default();
        let dom = wrap_with_id_spans(dom, 0, "-1:-1".to_string(), syntax_set.inner());
        let mut dom = Element::container(dom);
        let spans = dom.seperate_html_elements().sort_html_elements().to_html();

        state.compare_markdown(spans)
    } else {
        let html = generate_html_from_code(&new_lines, extension, &syntax_set)?;
        let html = replace_new_lines(html);

        let dom = parse(&html).unwrap_or_default();
        let dom = wrap_with_id_spans(dom, 0, "-2:-1".to_string(), &syntax_set);
        let mut dom = Element::container(dom);
        let spans = dom.seperate_html_elements().sort_html_elements().to_html();

        state.compare_code(spans)
    };

    Ok(result)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let keys = Keys::prepare()?;
    tauri::Builder::default()
        .setup(move |app| {
            let main_window = app.get_window("main").unwrap();
            std::thread::spawn(move || loop {
                let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() else {
                    continue;
                };
                if !matches!(event.state, HotKeyState::Pressed) {
                    continue;
                }
                for (k, v) in &keys.keys_map {
                    if event.id == v.id() {
                        main_window.emit(k, ()).unwrap();
                    }
                }
            });
            Ok(())
        })
        .manage(SyntaxSet::load_defaults_newlines())
        .manage(AppState::init())
        .invoke_handler(tauri::generate_handler![open_config, read_file])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    Ok(())
}
