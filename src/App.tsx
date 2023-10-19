import { createEffect, createMemo, createResource, createSignal } from "solid-js";
import "./App.css";
import "./styles.css";
import { invoke } from "@tauri-apps/api";
import { open } from "@tauri-apps/api/dialog";

type Config = {
  name : string,
  lessons : Object// Map<string,string>,
}

const CODE_BLOCK_ID = "code_id";
const CONFIG_NAME = "config.json";

async function opene_folder() {
  return await open({
    title:"choose lesson",
    directory:true,
    multiple:false
  }) as string | null;
}

const BASIC_MESSAGE = [`
<span class="source rust">
<span class="support macro rust">println!</span><span class="meta group rust"><span class="punctuation section group begin rust">(</span></span><span class="meta group rust"><span class="string quoted double rust"><span class="punctuation definition string begin rust">&quot;</span>don,t panic<span class="punctuation definition string end rust">&quot;</span></span></span><span class="meta group rust"><span class="punctuation section group end rust">)</span></span><span class="punctuation terminator rust">;</span>
`];

async function read_file(path: string | null) {
  if(!path) {
    return BASIC_MESSAGE;
  }
  try {
    return await invoke<string[]>(
        "read_file",
        {path},
    );
  } catch(_err) {
    return BASIC_MESSAGE;
  }
}

function App() {
  const [opened_folder,set_opened_folder] = createSignal<string | null>(null);
  const [folder_config,set_folder_config] = createSignal<Config | null>(null);
  const lessons_keys = createMemo((_) => {
    const fc = folder_config();
    let indexs : number[] = [];
    if(fc) {
      for (const key of Object.keys(fc.lessons)) {
        indexs.push(+key);
      };
      indexs.sort((x,y) =>  x - y);
    }
    return indexs;
  });
  const last_lesson_index = createMemo((_) => lessons_keys().length - 1);
  const [current_lesson_index,set_current_lesson_index] = createSignal(0);
  const current_lesson_key = createMemo((_) => lessons_keys().at(current_lesson_index()));
  const current_lesson_path = createMemo((_) => {
    let lk = current_lesson_key();
    if (!lk) {
      return null;
    }
    let lesson_key = lk.toString();
    let ln = folder_config();
    if (!ln) {
      return null;
    }
    let lesson_name = ln.lessons[lesson_key] as string | null;
    if (!lesson_name) {
      return null;
    }
    let path = opened_folder();
    if (path) {
      return path + "/" + lesson_name
    } else {
      return null;
    }
  });
  let [font_size,set_font_size] = createSignal(1.0);
  addEventListener("keypress", async (ev) => {
    let key_code = ev.code;
    console.log(key_code)

    switch (key_code) {
      case "Equal" : {
        set_font_size((x) => x + 0.05);
        break;
      }
      case "Minus" : {
        set_font_size((x) => x - 0.05);
        break;
      }
      case "KeyQ" : {
          set_opened_folder(null);
          set_current_lesson_index(0);
          break;
      }
      case "KeyL" : {
        set_current_lesson_index((index) => {
          let lli = last_lesson_index();
          if (lli && lli > index) {
            return index + 1;
          } else {
            return index;
          }
        });
        break;
      }
      case "KeyH" : {
        set_current_lesson_index((x) => {
          if (x > 0) {
            return x - 1;
          } else {
            return x;
          }
        });
        break;
      }
      case "KeyO" : {
        let path = await opene_folder();
        if(!path) {
          return;
        }
        let config_path = path + "/" + CONFIG_NAME;
        try {
          let config = await invoke<Config>("open_config",{path:config_path});
          set_opened_folder(path);
          set_folder_config(config);
        } catch (err) {
          console.log(err);
        }
        break;
      } 
      default : {
        break;
      }
    }
  });

  const [the_code] = createResource(() => current_lesson_path(), read_file);

  createEffect(() => {
    let code = the_code();
    if(!code) {
      return;
    }
    let code_node = document.getElementById(CODE_BLOCK_ID);
    if(!code_node) {
      return;
    }
    code_node.innerHTML = "";
    for(let i = 0; i < code.length; i++) {
      setTimeout(() => {
        code_node?.insertAdjacentHTML("beforeend",code![i]);
      },i * 400);
    }
  })

  
  const containerDynamicStyle = () => `font-size: ${font_size()}rem;`;

  return (
      <pre
        id={CODE_BLOCK_ID} 
        class="code custom"  
        style={containerDynamicStyle()}></pre>
  );
}

export default App;
