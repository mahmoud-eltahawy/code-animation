import {
  createEffect,
  createMemo,
  createResource,
  createSignal,
} from "solid-js";
import { invoke } from "@tauri-apps/api";
import { open } from "@tauri-apps/api/dialog";
import { emit, listen } from "@tauri-apps/api/event";

type Option<T> = T | null;

type Config = {
  name: string;
  lessons: object; // Map<string,string>,
};

const CONFIG_NAME = "config.json";

const ROOT_BLOCK = document.getElementById("root")!;
const PRE_CODE_BLOCK = document.getElementById("-1:-1@")!;
const MARKDOWN_BLOCK = document.getElementById("markdown")!;

async function opene_folder() {
  return await open({
    title: "choose lesson",
    directory: true,
    multiple: false,
  }) as Option<string>;
}

async function read_file(path: Option<string>) {
  if (!path) {
    return new Map() as Map<string, Option<string>>;
  }
  try {
    const result = await invoke<object>(
      "read_file",
      { path },
    );
    return new Map(Object.entries(result)) as Map<string, Option<string>>;
  } catch (_err) {
    return new Map() as Map<string, Option<string>>;
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

const [the_code] = createResource(() => current_lesson_path(), read_file);

createEffect(() =>
  ROOT_BLOCK.setAttribute("style", `font-size: ${font_size()}rem;`)
);

function display_code() {
  PRE_CODE_BLOCK.setAttribute("style", "display: inline-block;");
  MARKDOWN_BLOCK.setAttribute("style", "display: none;");
}

function display_markdown() {
  MARKDOWN_BLOCK.setAttribute("style", "display: inline-block;");
  PRE_CODE_BLOCK.setAttribute("style", "display: none;");
}

function number_spans(
  elements: Element[],
  generation: number,
  family_name: string,
) {
  for (let position = 0; position < elements.length; position++) {
    elements[position].setAttribute(
      "id",
      `${generation}:${position}@${family_name}`,
    );
    number_spans(
      Array.from(elements[position].children),
      generation + 1,
      `${family_name}:${position}`,
    );
  }
  return elements;
}

function get_father_id(id : string) {
  const [gp,family_name] = id.split('@');
  const [generation] = gp.split(':');
  const family_members = family_name.split(':');
  const father_position = family_members.pop();
  return `${+generation - 1 }:${father_position}@${family_members.join(':')}`;
}

function textNodesUnder(node: ChildNode | null | undefined) {
  let all : (ChildNode | null| undefined)[] = [];
  for (node = node?.firstChild;node;node=node.nextSibling) {
    if(node.nodeType==3) {
      all.push(node);
    } else {
      all = all.concat(textNodesUnder(node))
    }
  }
  return all;
}

function wrap_text_number_spans(str :string) {
  const div = document.createElement("div");
  div.innerHTML = str;

  // wrap spans around text nodes
  textNodesUnder(div).forEach((n) => {
    const rn = document.createElement("span");
    const value = n?.nodeValue;
    if (value) {
      rn.innerHTML = value;
    }
    n?.parentNode?.insertBefore(rn,n);
    n?.parentNode?.removeChild(n);
  });

  const elements = number_spans(Array.from(div.children), 0, "-1");
  div.innerHTML = "";
  for (const element of elements) {
    div.insertAdjacentElement("beforeend", element);
  }
  return div;
  
}

function seperateSpans(div : HTMLDivElement) {
  return Array.from(div.getElementsByTagName("span"));
}

function unnest_spans(spans : HTMLSpanElement[]) {
  spans.forEach((span) => {
    if(span.getElementsByTagName("span").length != 0) {
      span.innerHTML = "";
    };
  })
}

function sortSpans(spans : HTMLSpanElement[]) {
  const result = spans.sort((x, y) => {
    const [x_generation, x_position] = x.id.split("@").at(
      0,
    )!.split(":").map((x) => +x);
    const [y_generation, y_position] = y.id.split("@").at(
      0,
    )!.split(":").map((x) => +x);
    if (x_generation != y_generation) {
      return x_generation - y_generation;
    } else {
      return x_position - y_position;
    }
  });
  return result;
}

listen("new_code",(ev) => {
  display_code();
  const spans = ev.payload as ["1"|"-1",string][];
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
});

function App() {
  createEffect(() => {
    const code = the_code();
    if (!code) {
      return;
    }
    if (code.size == 1 && Array.from(code.keys()).at(0) == "-1") {
      display_markdown();
      MARKDOWN_BLOCK.innerHTML = code.get("-1")!;
    } else {
      const div = wrap_text_number_spans(code.get("0")!);
      const spans = seperateSpans(div);
      const elements = sortSpans(spans);
      unnest_spans(elements);
      emit("set_code",elements.map((x) => x.outerHTML));
    }
  });

  return <></>;
}

export default App;
