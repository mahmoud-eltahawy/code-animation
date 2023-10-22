import {
  createEffect,
  createMemo,
  createResource,
  createSignal,
} from "solid-js";
import "./App.css";
import "./styles.css";
import { invoke } from "@tauri-apps/api";
import { open } from "@tauri-apps/api/dialog";

type Config = {
  name: string;
  lessons: object; // Map<string,string>,
};

const CODE_BLOCK_ID = "code_id";
const CONFIG_NAME = "config.json";

async function opene_folder() {
  return await open({
    title: "choose lesson",
    directory: true,
    multiple: false,
  }) as string | null;
}

function make_line_id(index: number) {
  return `CODE_LINE-${index}`;
}
function repair_next_lines_ids(old_line: HTMLElement) {
  const id = old_line.getAttribute("id")!;
  const index = +id.split("-").at(1)!;
  const next_line = document.getElementById(make_line_id(index + 1));
  if (next_line) {
    repair_next_lines_ids(next_line);
  }
  old_line.setAttribute("id", make_line_id(index + 1));
}

function fill_ids_gaps() {
  const spans = document.getElementById(CODE_BLOCK_ID)!.children;
  let i = 0;
  for (const span of spans) {
    span.setAttribute("id", make_line_id(i));
    i++;
  }
}

async function read_file(path: string | null) {
  if (!path) {
    return new Map() as Map<string, string | null>;
  }
  try {
    const result = await invoke<object>(
      "read_file",
      { path },
    );
    return new Map(Object.entries(result)) as Map<string, string | null>;
  } catch (_err) {
    return new Map() as Map<string, string | null>;
  }
}

function App() {
  const [opened_folder, set_opened_folder] = createSignal<string | null>(null);
  const [folder_config, set_folder_config] = createSignal<Config | null>(null);
  const lessons_keys = createMemo((_) => {
    const fc = folder_config();
    const indexs: number[] = [];
    if (fc) {
      for (const key of Object.keys(fc.lessons)) {
        indexs.push(+key);
      }
      indexs.sort((x, y) => x - y);
    }
    return indexs;
  });
  const last_lesson_index = createMemo((_) => lessons_keys().length - 1);
  const [current_lesson_index, set_current_lesson_index] = createSignal(0);
  const current_lesson_key = createMemo((_) =>
    lessons_keys().at(current_lesson_index())
  );
  const current_lesson_path = createMemo((_) => {
    const lk = current_lesson_key();
    if (!lk) {
      return null;
    }
    const lesson_key = lk.toString();
    const ln = folder_config();
    if (!ln) {
      return null;
    }
    const lesson_name = ln.lessons[lesson_key] as string | null;
    if (!lesson_name) {
      return null;
    }
    const path = opened_folder();
    if (path) {
      return path + "/" + lesson_name;
    } else {
      return null;
    }
  });
  const [font_size, set_font_size] = createSignal(1.0);
  addEventListener("keypress", async (ev) => {
    const key_code = ev.code;
    console.log(key_code);

    switch (key_code) {
      case "Equal": {
        set_font_size((x) => x + 0.05);
        break;
      }
      case "Minus": {
        set_font_size((x) => x - 0.05);
        break;
      }
      case "KeyQ": {
        set_opened_folder(null);
        set_current_lesson_index(0);
        break;
      }
      case "KeyL": {
        set_current_lesson_index((index) => {
          const lli = last_lesson_index();
          if (lli && lli > index) {
            return index + 1;
          } else {
            return index;
          }
        });
        break;
      }
      case "KeyH": {
        set_current_lesson_index((index) => {
          if (index > 0) {
            return index - 1;
          } else {
            return index;
          }
        });
        break;
      }
      case "KeyO": {
        const path = await opene_folder();
        if (!path) {
          return;
        }
        const config_path = path + "/" + CONFIG_NAME;
        try {
          const config = await invoke<Config>("open_config", {
            path: config_path,
          });
          set_opened_folder(path);
          set_folder_config(config);
        } catch (err) {
          console.log(err);
        }
        break;
      }
      default: {
        break;
      }
    }
  });

  const [the_code] = createResource(() => current_lesson_path(), read_file);

  createEffect(() => {
    const code = the_code();
    if (!code) {
      return;
    }
    let gap_exist = false;
    const code_node = document.getElementById(CODE_BLOCK_ID)!;
    for (const [key, new_line] of code) {
      const key_number = +key;
      const line_id = make_line_id(key_number);
      const old_line = document.getElementById(line_id);
      if (new_line) {
        const line = `<span id="${line_id}">${new_line}</span>`;
        if (old_line) {
          old_line.insertAdjacentHTML("beforebegin", line);
          repair_next_lines_ids(old_line);
        } else {
          code_node.insertAdjacentHTML("beforeend", line);
        }
      } else {
        if (old_line) {
          old_line.remove();
          gap_exist = true;
        }
      }
    }
    if (gap_exist) {
      fill_ids_gaps();
    }
  });

  const containerDynamicStyle = () => `font-size: ${font_size()}rem;`;

  return (
    <pre
      id={CODE_BLOCK_ID}
      class="code custom"
      style={containerDynamicStyle()}
    ></pre>
  );
}

export default App;
