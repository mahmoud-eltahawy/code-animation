use std::{collections::HashMap, path::PathBuf};

use leptos::*;
use leptos_meta::*;
use tauri_sys::{dialog::FileDialogBuilder, tauri::invoke};

use serde::{Deserialize, Serialize};

const GENERAL_STYLE: &str = include_str!("../styles.css");
const THEME_STYLE: &str = include_str!("../style.css");
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
<span class="source rust"><span class="comment line double-slash rust"><span class="punctuation definition comment rust">//</span> Rust source
</span><span class="meta function rust"><span class="meta function rust"><span class="storage type function rust">fn</span> </span><span class="entity name function rust">main</span></span><span class="meta function rust"><span class="meta function parameters rust"><span class="punctuation section parameters begin rust">(</span></span><span class="meta function rust"><span class="meta function parameters rust"><span class="punctuation section parameters end rust">)</span></span></span></span><span class="meta function rust"> </span><span class="meta function rust"><span class="meta block rust"><span class="punctuation section block begin rust">{</span>
<span class="support macro rust">println!</span><span class="meta group rust"><span class="punctuation section group begin rust">(</span></span><span class="meta group rust"><span class="string quoted double rust"><span class="punctuation definition string begin rust">&quot;</span>Hello World!<span class="punctuation definition string end rust">&quot;</span></span></span><span class="meta group rust"><span class="punctuation section group end rust">)</span></span><span class="punctuation terminator rust">;</span>
</span><span class="meta block rust"><span class="punctuation section block end rust">}</span></span></span></span>
        "#;
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

    const CODE_BLOCK_ID: &str = "code_id";

    Effect::new(move |_| {
        let Some(code) = the_code.get().map(|x| x) else {
            return;
        };
        let Some(code_block) = document().get_element_by_id(CODE_BLOCK_ID) else {
            return;
        };
        code_block.set_inner_html(&code);
    });

    view! {
    <>
    <Style>{
            String::from("") +
            THEME_STYLE +
            GENERAL_STYLE
        }</Style>
    <pre id=CODE_BLOCK_ID class="code fullpage"></pre>
    </>
    }
}
