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

// Sphinx's doctools.js installs global single-key shortcuts on `document`
// (`/` focuses the search box; ← / → page-navigate) and only exempts
// <input>/<textarea>/<select> elements. CodeMirror's editable area is a
// contenteditable <div>, so those shortcuts fire mid-typing. Stopping keydown
// events from bubbling out of the mounted editor keeps them from reaching the
// document handler, while CodeMirror (whose handlers live on the inner
// .cm-content) still processes the key at the target first.
function shieldGlobalKeys(host) {
  host.addEventListener("keydown", (event) => event.stopPropagation());
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
    shieldGlobalKeys(wrapper);
    block.replaceWith(wrapper);
  }
}

async function upgradePlayground(mountAlgaeEditor) {
  const host = document.getElementById("algae-playground");
  if (!host) return;

  // A `?q=<url>` query parameter overrides the page's default seed, so any proof
  // file — a relative path (a game room) or an absolute URL (e.g. a
  // raw.githubusercontent.com link) — can be opened straight in the playground.
  // This makes proofs shareable: whatever is in the box goes into `?q=`, and a
  // `?q=` on load is fetched automatically. `?module=` optionally names the
  // checker unit; otherwise it's derived from the file name.
  const params = new URLSearchParams(window.location.search);
  const qParam = params.get("q");
  const seedUrl = qParam || host.getAttribute("data-seed-url");
  let moduleName = params.get("module") || host.getAttribute("data-module") || "playground";
  if (qParam && !params.get("module")) {
    const stem = qParam.split(/[?#]/)[0].split(/[\/\\]/).pop().replace(/\.alg$/, "");
    if (stem) moduleName = stem;
  }

  let doc = host.getAttribute("data-seed") || "";
  if (seedUrl) {
    try {
      // An absolute URL ignores the base; a relative one resolves against the
      // page. Cross-origin fetches (e.g. raw.githubusercontent.com, which sends
      // `Access-Control-Allow-Origin: *`) work; a host without CORS headers will
      // be blocked and surfaced below.
      const resp = await fetch(new URL(seedUrl, document.baseURI));
      if (resp.ok) doc = await resp.text();
      else if (qParam) doc = "# Could not load " + seedUrl + " — HTTP " + resp.status + "\n";
    } catch (_e) {
      // A `?q` fetch that fails (bad URL, or a cross-origin host without CORS)
      // should be visible, not silently ignored.
      if (qParam) doc = "# Could not load " + seedUrl + " (network error or blocked by CORS)\n";
    }
  }

  const wasm = await loadWasm().catch(() => null);
  host.textContent = "";
  mountAlgaeEditor(host, { doc, wasm: wasm || undefined, moduleName });
  shieldGlobalKeys(host);

  // Wire the "Load from URL" control (see playground.md): typing a URL and
  // pressing Load / Enter reloads the page with `?q=<url>`, so the resulting
  // address is itself a shareable link to that proof.
  const urlInput = document.getElementById("algae-load-url");
  if (urlInput) {
    if (qParam) urlInput.value = qParam;
    const go = () => {
      const v = urlInput.value.trim();
      if (v) window.location.search = "?q=" + encodeURIComponent(v);
    };
    const loadBtn = document.getElementById("algae-load-btn");
    if (loadBtn) loadBtn.addEventListener("click", go);
    urlInput.addEventListener("keydown", (event) => {
      if (event.key === "Enter") {
        event.preventDefault();
        go();
      }
    });
  }
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
