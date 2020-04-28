// Note that a dynamic `import` statement here is required due to
// webpack/webpack#6615, but in theory `import { greet } from './pkg/hello_world';`
// will work here one day as well!
const rust = import("./boa_wasm/pkg");
import * as monaco from "monaco-editor";

self.MonacoEnvironment = {
  getWorkerUrl: function (moduleId, label) {
    if (label === "json") {
      return "./json.worker.bundle.js";
    }
    if (label === "css") {
      return "./css.worker.bundle.js";
    }
    if (label === "html") {
      return "./html.worker.bundle.js";
    }
    if (label === "typescript" || label === "javascript") {
      return "./ts.worker.bundle.js";
    }
    return "./editor.worker.bundle.js";
  },
};

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

rust.then((m) => {
  window.evaluate = m.evaluate;

  editor.getModel().onDidChangeContent(inputHandler);
  inputHandler(); // Evaluate initial code
});

function inputHandler(evt) {
  const text = editor.getValue();
  let p = document.querySelector("p.output");
  let result = window.evaluate(text);
  p.textContent = `> ${result}`;
}
