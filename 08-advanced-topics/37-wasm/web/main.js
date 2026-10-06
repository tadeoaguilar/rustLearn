// Glue for index.html: load the wasm-pack output and use the exports.
import init, { markdown_to_html, Universe } from "./pkg/m37_wasm_solution.js";

const wasm = await init(); // instantiates the module; `wasm.memory` is its linear memory

const md = document.getElementById("md");
const preview = document.getElementById("preview");
const render = () => { preview.innerHTML = markdown_to_html(md.value); }; // safe: Rust escapes the input
md.addEventListener("input", render);
render();

const size = 64, scale = 5;
const universe = new Universe(size, size);
for (let i = 0; i < size * size / 4; i++) {
  universe.set(Math.floor(Math.random() * size), Math.floor(Math.random() * size), true);
}
const ctx = document.getElementById("life").getContext("2d");
function frame() {
  universe.tick();
  // read the cells in place from WASM memory: no copy per frame
  const cells = new Uint8Array(wasm.memory.buffer, universe.cells_ptr(), size * size);
  ctx.fillStyle = "#fff";
  ctx.fillRect(0, 0, size * scale, size * scale);
  ctx.fillStyle = "#b7410e";
  for (let r = 0; r < size; r++) {
    for (let c = 0; c < size; c++) {
      if (cells[r * size + c]) ctx.fillRect(c * scale, r * scale, scale, scale);
    }
  }
  requestAnimationFrame(frame);
}
requestAnimationFrame(frame);
