// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use global_hotkey::{
    hotkey::{Code, HotKey, Modifiers},
    GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState,
};
use tauri::{Manager, State};

use std::{cmp::Ordering, collections::HashMap, fs::File, io::Read, sync::Mutex};

use serde::{Deserialize, Serialize};

use similar::{utils::diff_lines, Algorithm, ChangeTag};
use syntect::{
    html::{ClassStyle, ClassedHTMLGenerator},
    parsing::SyntaxSet,
    util::LinesWithEndings,
};

use itertools::Itertools;

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
struct Config {
    name: String,
    lessons: HashMap<String, String>,
}

use html_editor::{operation::*, Element};
use html_editor::{parse, Node};

trait ElementExtra {
    fn container(nodes: Vec<Node>) -> Self;
    fn get_attirbute(&self, attr: &str) -> Option<&String>;
    fn get_id(&self) -> Option<&String>;
    fn split_id(&self) -> Option<(i32, i32, Vec<&str>)>;
    fn are_childern_code(&self) -> Option<(&str, &String)>;
    fn code_children(&mut self, syntaxt_set: &SyntaxSet);
    fn seperate_html_elements(&mut self) -> &mut Self;
    fn sort_html_elements(&mut self) -> Self;
    fn to_html(self) -> Vec<String>;
}

impl ElementExtra for Element {
    fn get_attirbute(&self, attr: &str) -> Option<&String> {
        self.attrs
            .iter()
            .filter(|(head, _)| head == attr)
            .map(|(_, value)| value)
            .nth(0)
    }

    #[inline(always)]
    fn get_id(&self) -> Option<&String> {
        self.get_attirbute("id")
    }

    fn split_id(&self) -> Option<(i32, i32, Vec<&str>)> {
        let Some(id) = self.get_id() else {
            return None;
        };
        let [ps, family, ..] = id.split('@').collect::<Vec<_>>()[..] else {
            return None;
        };
        let [generation, index, ..] = ps
            .split(':')
            .flat_map(|x| x.parse::<i32>())
            .collect::<Vec<_>>()[..]
        else {
            return None;
        };
        let family = family.split(':').collect::<Vec<_>>();
        Some((generation, index, family))
    }

    fn are_childern_code(&self) -> Option<(&str, &String)> {
        if self.name != "pre" {
            return None;
        }
        let [one_node] = &self.children[..] else {
            return None;
        };
        let element = match one_node {
            Node::Element(element) => element,
            _ => return None,
        };
        if element.name != "code" {
            return None;
        }
        let Some(language) = element
            .get_attirbute("class")
            .and_then(|x| x.split('-').last())
        else {
            return None;
        };
        let [Node::Text(code)] = &element.children[..] else {
            return None;
        };
        Some((language, code))
    }

    fn code_children(&mut self, syntax_set: &SyntaxSet) {
        let Some((language, code)) = self.are_childern_code() else {
            return;
        };
        let attrs = self
            .attrs
            .iter()
            .filter(|x| x.0 != "class")
            .chain(&vec![(String::from("class"), String::from("code"))])
            .cloned()
            .collect::<Vec<_>>();

        let Ok(code) = generate_html_from_code(code, language, syntax_set) else {
            return;
        };
        let children = parse(&code).unwrap_or_default();
        self.children = children;
        self.attrs = attrs;
    }

    fn seperate_html_elements(&mut self) -> &mut Self {
        let mapping = |element: Element| {
            if element
                .children
                .iter()
                .any(|node| matches!(node, Node::Element(_)))
            {
                Element {
                    children: vec![],
                    ..element
                }
                .into_node()
            } else {
                element.into_node()
            }
        };
        self.children = self
            .query_all(&Selector::from(HTML_TYPES))
            .into_iter()
            .cloned()
            .map(mapping)
            .collect::<Vec<_>>();
        self
    }

    fn sort_html_elements(&mut self) -> Self {
        let comparing = |x: &&Node, y: &&Node| {
            let (Some(x), Some(y)) = (x.as_element(), y.as_element()) else {
                return Ordering::Equal;
            };
            let (Some((x_generation, x_index, x_family)), Some((y_generation, y_index, y_family))) =
                (x.split_id(), y.split_id())
            else {
                return Ordering::Equal;
            };

            let x_family = x_family.len();
            let y_family = y_family.len();

            if x_family.cmp(&y_family) != Ordering::Equal {
                x_family.cmp(&y_family)
            } else if x_generation.cmp(&y_generation) != Ordering::Equal {
                x_generation.cmp(&y_generation)
            } else {
                x_index.cmp(&y_index)
            }
        };
        self.children = self.children.iter().sorted_by(comparing).cloned().collect();
        self.to_owned()
    }

    fn to_html(self) -> Vec<String> {
        self.children
            .into_iter()
            .map(|x| x.html())
            .collect::<Vec<_>>()
    }

    fn container(nodes: Vec<Node>) -> Self {
        Element::new("FakeElement", vec![], nodes)
    }
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

const HTML_TYPES: &str = "span,pre,li,ul,ol,a,div,h1,h2,h3,h4,h5,h6,section,code";

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


const NEW_LINE: &str = "THENEWLINESYMPOLE";

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

        let dom = state.get_markdown(&spans);
        state.set_markdown(spans.clone());
        dom
    } else {
        let html = generate_html_from_code(&new_lines, extension, &syntax_set)?;
        let html = replace_new_lines(html);

        let dom = parse(&html).unwrap_or_default();
        let dom = wrap_with_id_spans(dom, 0, "-2:-1".to_string(), &syntax_set);
        let mut dom = Element::container(dom);
        let spans = dom.seperate_html_elements().sort_html_elements().to_html();

        let dom = state.get_old_code(&spans);
        state.set_old_code(spans.clone());
        dom
    };

    Ok(result)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let keys_manager = GlobalHotKeyManager::new()?;

    let open_lesson = HotKey::new(None, Code::KeyO);
    let quit_lesson = HotKey::new(None, Code::KeyQ);
    let font_increase = HotKey::new(None, Code::Equal);
    let font_decrease = HotKey::new(None, Code::Minus);
    let next_snippet = HotKey::new(None, Code::KeyL);
    let previous_snippet = HotKey::new(None, Code::KeyH);
    let remember_toggle = HotKey::new(None, Code::KeyM);
    let next_snippet_stacked = HotKey::new(Some(Modifiers::SHIFT), Code::KeyL);

    keys_manager.register_all(&[
        open_lesson,
        quit_lesson,
        font_increase,
        font_decrease,
        next_snippet,
        previous_snippet,
        next_snippet_stacked,
        remember_toggle,
    ])?;

    let state = AppState{
        code: Mutex::new(vec![]),
        markdown: Mutex::new(vec![]),
    };

    let syntax_set = SyntaxSet::load_defaults_newlines();
    tauri::Builder::default()
        .setup(move |app| {
            let main_window = app.get_window("main").unwrap();
            std::thread::spawn(move || loop {
                if let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
                    if event.state == HotKeyState::Pressed {
                        if event.id == open_lesson.id() {
                            main_window.emit(stringify!(open_lesson), ()).unwrap();
                        } else if event.id == quit_lesson.id() {
                            main_window.emit(stringify!(quit_lesson), ()).unwrap();
                        } else if event.id == next_snippet.id() {
                            main_window.emit(stringify!(next_snippet), ()).unwrap();
                        } else if event.id == previous_snippet.id() {
                            main_window.emit(stringify!(previous_snippet), ()).unwrap();
                        } else if event.id == font_increase.id() {
                            main_window.emit(stringify!(font_increase), ()).unwrap();
                        } else if event.id == font_decrease.id() {
                            main_window.emit(stringify!(font_decrease), ()).unwrap();
                        } else if event.id == remember_toggle.id() {
                            main_window.emit(stringify!(remember_toggle), ()).unwrap();
                        }
                    }
                }
            });
            Ok(())
        })
        .manage(syntax_set)
        .manage(state)
        .invoke_handler(tauri::generate_handler![open_config, read_file])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    Ok(())
}

struct AppState{
    code :Mutex<Vec<String>>,
    markdown :Mutex<Vec<String>>,
}

impl AppState {
    fn set_old_code(&self,spans : Vec<String>) {
        *self.code.lock().unwrap() = spans;
    }
    fn set_markdown(&self,spans : Vec<String>) {
        *self.markdown.lock().unwrap() = spans;
    }

    fn get_markdown(&self,markdown: &[String]) -> Vec<(String, String)> {
        let lines = self.markdown.lock().unwrap();
        let lines1 = Itertools::intersperse(lines.clone().into_iter(), "\n".to_string())
            .collect::<String>();
        let lines2 = Itertools::intersperse(markdown.iter(), &"\n".to_string())
            .cloned()
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

    fn get_old_code(&self,new_code: &[String]) -> Vec<(String, String)> {
        let lines = self.code.lock().unwrap();
        let lines1 = Itertools::intersperse(lines.iter(), &"\n".to_string())
            .cloned()
            .collect::<String>();
        let lines2 = Itertools::intersperse(new_code.iter(), &"\n".to_string())
            .cloned()
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
