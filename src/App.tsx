import {
  createEffect,
  createMemo,
  createSignal,
} from "solid-js";
import { invoke } from "@tauri-apps/api";
import { open } from "@tauri-apps/api/dialog";
import { listen } from "@tauri-apps/api/event";

type Option<T> = T | null;

type Config = {
  name: string;
  lessons: object; // Map<string,string>,
};

const CONFIG_NAME = "config.json";

const ROOT_BLOCK = document.getElementById("root")!;

async function opene_folder() {
  return await open({
    title: "choose lesson",
    directory: true,
    multiple: false,
  }) as Option<string>;
}

async function read_file(path: Option<string>) {
  if (!path) {
    return [];
  }
  try {
    const spans = await invoke<["1" | "-1",string][]>(
      "read_file",
      { path },
    );
    const div = document.createElement("div");
    const elements = spans.map(([ord,span]) => {
      div.innerHTML = span;
      return [ord ,div.firstChild]  as ["1"|"-1",HTMLSpanElement];
    })
    let t = 1;
    for (const [ord,element] of elements) {
      const id = element.id;
      t++;
      setTimeout(() => {
        if(ord === "1") {
          const ele = document.getElementById(id);
          if(ele) {
            ele.replaceWith(element);
          } else {
            document.getElementById(get_father_id(id))?.insertAdjacentElement("beforeend",element);
          }
        } else if(ord === "-1") {
          document.getElementById(id)?.remove();
        }
      },t * 50);
    }
  } catch (_err) {
    return [];
  }
}

const [opened_folder, set_opened_folder] = createSignal<Option<string>>(null);
const [folder_config, set_folder_config] = createSignal<Option<Config>>(null);
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

listen("next_snippet", () =>
  set_current_lesson_index((index) => {
    const lli = last_lesson_index();
    if (lli && lli > index) {
      return index + 1;
    } else {
      return index;
    }
  }));

listen("previous_snippet", () =>
  set_current_lesson_index((index) => {
    if (index > 0) {
      return index - 1;
    } else {
      return index;
    }
  }));

const [font_size, set_font_size] = createSignal(1.0);

listen("font_increase", () => set_font_size((x) => x + 0.05));

listen("font_decrease", () => set_font_size((x) => x - 0.05));

listen("open_lesson", async () => {
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
});

listen("quit_lesson", () => {
  set_opened_folder(null);
  set_current_lesson_index(0);
});

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
  const lesson_name = ln.lessons[lesson_key] as Option<string>;
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

createEffect(() =>
  ROOT_BLOCK.setAttribute("style", `font-size: ${font_size()}rem;`)
);

function get_father_id(id : string) {
  const [gp,family_name] = id.split('@');
  const [generation] = gp.split(':');
  const family_members = family_name.split(':');
  const father_position = family_members.pop();
  return `${+generation - 1 }:${father_position}@${family_members.join(':')}`;
}

function App() {
  createEffect(() => {
    read_file(current_lesson_path());
  });

  return <></>;
}

export default App;
