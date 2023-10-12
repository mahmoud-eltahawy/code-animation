use std::{collections::HashMap, path::PathBuf};

use leptos::*;
use leptos_meta::*;
use tauri_sys::{dialog::FileDialogBuilder, tauri::invoke};

use serde::{Deserialize, Serialize};

const GENERAL_STYLE: &str = include_str!("../styles.css");
const CONFIG_NAME: &str = "config.json";

#[derive(Serialize)]
struct Arg<'a> {
    path: &'a str,
}

#[inline(always)]
pub async fn open_folder() -> Option<PathBuf> {
    FileDialogBuilder::new()
        .set_title("choose lesson")
        .pick_folder()
        .await
        .ok()
        .flatten()
}

#[derive(PartialEq, Clone, Debug, Deserialize)]
struct Config {
    name: String,
    lessons: HashMap<String, String>,
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    let opened_folder = RwSignal::new(None::<PathBuf>);
    let folder_config = RwSignal::new(None::<Config>);
    let lessons_keys = Memo::new(move |_| {
        folder_config.get().map(|config| {
            let mut indexs = config
                .lessons
                .keys()
                .flat_map(|num| num.parse::<i32>())
                .collect::<Vec<_>>();

            indexs.sort();
            indexs
        })
    });
    let last_lesson_index = Memo::new(move |_| lessons_keys.get().map(|xs| xs.len() - 1));
    let current_lesson_index = RwSignal::new(0_usize);
    let current_lesson_key = Memo::new(move |_| {
        lessons_keys
            .get()
            .and_then(|xs| xs.get(current_lesson_index.get()).cloned())
    });
    let current_lesson_path = Memo::new(move |_| {
        let lesson_key = current_lesson_key.get().map(|x| x.to_string());
        let Some(key) = lesson_key else {
            return None::<PathBuf>;
        };
        let lesson_name = folder_config
            .get()
            .and_then(|x| x.lessons.get(&key).cloned());
        let Some(name) = lesson_name else {
            return None::<PathBuf>;
        };
        opened_folder.get().map(|path| {
            let mut path = path;
            path.push(name);
            path
        })
    });
    window_event_listener(ev::keypress, move |ev| match ev.code().as_str() {
        "KeyL" => current_lesson_index.update(|index| {
            if last_lesson_index.get().is_some_and(|x| x > *index) {
                *index += 1;
            }
        }),
        "KeyH" => current_lesson_index.update(|x| {
            if *x > 0 {
                *x -= 1
            }
        }),
        "KeyO" => {
            spawn_local(async move {
                let Some(path) = open_folder().await else {
                    return;
                };
                let mut config_path = path.clone();
                config_path.push(CONFIG_NAME);
                let config = match invoke::<_, Config>(
                    "open_config",
                    &Arg {
                        path: config_path.display().to_string().as_str(),
                    },
                )
                .await
                {
                    Ok(config) => Some(config),
                    Err(err) => {
                        logging::log!("{}", err.to_string());
                        None
                    }
                };
                opened_folder.set(Some(path));
                folder_config.set(config);
            });
        }
        _ => logging::log!("Other key pressed"),
    });

    async fn read_file(path: Option<PathBuf>) -> String {
        const OR: &str = r#"
            fn main() {
              println!("hello world");
            }"#;
        let Some(path) = path else {
            return OR.to_string();
        };
        invoke::<_, String>(
            "read_file",
            &Arg {
                path: path.display().to_string().as_str(),
            },
        )
        .await
        .unwrap_or(OR.to_string())
    }
    let the_code = Resource::new(move || current_lesson_path.get(), read_file);

    view! {
    <>
    <Style>{GENERAL_STYLE}</Style>
    <pre>
      <code>
      {
        move || the_code.get()
      }
      </code>
    </pre>
    </>
    }
}
