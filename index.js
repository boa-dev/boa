import { evaluate } from "./boa_wasm/pkg";

import * as monaco from "monaco-editor/esm/vs/editor/editor.api";

const initialCode = `\
function greet(targetName) {
  return 'Hello, ' + targetName + '!';
}

greet('World')
`;

const editor = monaco.editor.create(
  document.getElementsByClassName("textbox")[0],
  {
    value: initialCode,
    language: "javascript",
    theme: "vs",
    minimap: {
      enabled: false,
    },
  }
);

// Fix size of Monaco Editor when window resize
window.addEventListener("resize", () => {
  editor.layout();
});

window.evaluate = evaluate;

editor.getModel().onDidChangeContent(inputHandler);
inputHandler(); // Evaluate initial code

function inputHandler(evt) {
  const text = editor.getValue();
  let p = document.querySelector("p.output");

  try {
    let result = window.evaluate(text);
    p.textContent = `> ${result}`;
  } catch (err) {
    console.error(err);
    p.innerHTML = `<span style="color:red">${err}</span>`;
  }
}
