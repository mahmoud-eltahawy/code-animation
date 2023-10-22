/* @refresh reload */
import { render } from "solid-js/web";

import "./App.css";
import "./styles.css";
import App from "./App";

const root = document.getElementById("root");
root?.setAttribute("class", "container");
render(() => <App />, root as HTMLElement);
