import { createApp } from "vue";
import App from "./App.vue";
import "./styles/global.css";

/** 输入类控件仍允许编辑和选择文字，其余界面按桌面应用习惯禁止选中 */
function isEditable(target: EventTarget | null): boolean {
  const element = target instanceof Element ? target : null;
  return Boolean(element?.closest("input, textarea, select, [contenteditable='true']"));
}

document.addEventListener(
  "contextmenu",
  (event) => {
    event.preventDefault();
  },
  true
);

document.addEventListener(
  "selectstart",
  (event) => {
    if (!isEditable(event.target)) event.preventDefault();
  },
  true
);

document.addEventListener(
  "dragstart",
  (event) => {
    event.preventDefault();
  },
  true
);

document.addEventListener(
  "keydown",
  (event) => {
    if (event.key === "F5" || ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "r")) {
      event.preventDefault();
      window.dispatchEvent(new Event("appshortcut:refresh"));
    }
  },
  true
);

createApp(App).mount("#app");
