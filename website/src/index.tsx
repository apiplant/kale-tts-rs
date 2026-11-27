import { render } from "@solidjs/web";
import { App, type Page } from "./App";

const root = document.getElementById("root");
if (root) render(() => <App page={root.dataset.page as Page} />, root);
