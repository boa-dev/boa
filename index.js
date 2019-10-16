// Note that a dynamic `import` statement here is required due to
// webpack/webpack#6615, but in theory `import { greet } from './pkg/hello_world';`
// will work here one day as well!
const rust = import("./pkg");
import * as monaco from "monaco-editor";
// const image = import("./assets/01_rust_loves_js.png");

const editor = monaco.editor.create(
  document.getElementsByClassName("textbox")[0], 
  {
    value: "",
    language: "javascript",
    theme: "vs-dark"
  }
);


// Fix size of Monaco Editor when window resize
window.addEventListener('resize', () => {
  editor.layout();
});


rust.then(m => {
  window.evaluate = m.evaluate;

  editor.getModel().onDidChangeContent(inputHandler);
});

function inputHandler(evt) {
  const text = editor.getValue();
  let p = document.querySelector("p.output");
  let t0 = performance.now();
  let result = window.evaluate(text);
  let t1 = performance.now();
  p.textContent = `> ${result}`;
  console.log(result);
}