// Upgrades static `.alg` code blocks (and the playground container) into live
// CodeMirror editors backed by algae-wasm. Loaded as an ES module by Sphinx
// (see conf.py `html_js_files`). It sits in _static/, so the sibling bundles
// resolve as `./algae-editor.js` and `./algae_wasm.js`.
//
// Degrades gracefully: if the wasm / editor bundles are missing (e.g. a docs
// build that skipped the asset copy), the original Pygments-highlighted blocks
// are left untouched.

const EDITOR_URL = new URL("./algae-editor.js", import.meta.url).href;
const WASM_URL = new URL("./algae_wasm.js", import.meta.url).href;

// A block is worth wiring to the checker when it actually contains a proof to
// check; other `.alg` snippets (declarations, propositions, grammar fragments)
// become editable, highlighted editors without a Check button.
function isCheckable(src) {
  return /\bproof\b/.test(src) && /\b(qed|wip)\b/.test(src);
}

function blockSource(block) {
  const pre = block.querySelector("pre");
  const text = pre ? pre.textContent : block.textContent;
  return text.replace(/\n$/, "");
}

let editorModulePromise = null;
function loadEditor() {
  if (!editorModulePromise) editorModulePromise = import(EDITOR_URL);
  return editorModulePromise;
}

let wasmPromise = null;
function loadWasm() {
  if (!wasmPromise) {
    wasmPromise = import(WASM_URL).then(async (mod) => {
      await mod.default(); // instantiate the .wasm
      return mod; // exposes check(), format()
    });
  }
  return wasmPromise;
}

async function upgradeCodeBlocks(mountAlgaeEditor) {
  const blocks = Array.from(document.querySelectorAll(".highlight-alg"));
  if (blocks.length === 0) return;

  const anyCheckable = blocks.some((b) => isCheckable(blockSource(b)));
  const wasm = anyCheckable ? await loadWasm().catch(() => null) : null;

  for (const block of blocks) {
    const src = blockSource(block);
    const checkable = wasm && isCheckable(src);
    const wrapper = document.createElement("div");
    mountAlgaeEditor(wrapper, {
      doc: src,
      wasm: checkable ? wasm : undefined,
      moduleName: "example",
    });
    block.replaceWith(wrapper);
  }
}

async function upgradePlayground(mountAlgaeEditor) {
  const host = document.getElementById("algae-playground");
  if (!host) return;

  const seedUrl = host.getAttribute("data-seed-url");
  const moduleName = host.getAttribute("data-module") || "playground";
  let doc = host.getAttribute("data-seed") || "";
  if (seedUrl) {
    try {
      const resp = await fetch(new URL(seedUrl, document.baseURI));
      if (resp.ok) doc = await resp.text();
    } catch (_e) {
      /* fall back to inline/empty seed */
    }
  }

  const wasm = await loadWasm().catch(() => null);
  host.textContent = "";
  mountAlgaeEditor(host, { doc, wasm: wasm || undefined, moduleName });
}

async function main() {
  const hasBlocks = document.querySelector(".highlight-alg");
  const hasPlayground = document.getElementById("algae-playground");
  if (!hasBlocks && !hasPlayground) return;

  let mod;
  try {
    mod = await loadEditor();
  } catch (_e) {
    // Editor bundle unavailable — leave static highlighting in place.
    return;
  }
  const { mountAlgaeEditor } = mod;

  await upgradeCodeBlocks(mountAlgaeEditor);
  await upgradePlayground(mountAlgaeEditor);
}

if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", main);
} else {
  main();
}
