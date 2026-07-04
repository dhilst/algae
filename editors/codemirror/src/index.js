// Public entry point for the Algae CodeMirror editor bundle.
//
// `mountAlgaeEditor(parent, opts)` builds an editor with Algae syntax
// highlighting, live proof checking (via an already-initialized algae-wasm
// module), a "Check" button, and a results pane. The page owns wasm loading and
// passes the module in as `opts.wasm`, so a single wasm instance can be shared
// across many editors on one page.

import { EditorState } from "@codemirror/state";
import {
  EditorView, keymap, lineNumbers, highlightActiveLine,
  highlightActiveLineGutter, drawSelection, highlightSpecialChars,
} from "@codemirror/view";
import { history, defaultKeymap, historyKeymap, indentWithTab } from "@codemirror/commands";
import { bracketMatching } from "@codemirror/language";
import { lintGutter, lintKeymap, forceLinting } from "@codemirror/lint";
import { algae } from "./algae-lang.js";
import { algaeLinter } from "./lint.js";

export { algae, algaeStreamLanguage, algaeHighlightStyle } from "./algae-lang.js";
export { algaeLinter } from "./lint.js";

// A compact, light editor surface. The highlight palette in algae-lang.js is
// tuned for a light background; we keep the editor light even on dark pages so
// tokens stay legible, and give it a card border to sit inside prose.
const baseTheme = EditorView.theme({
  "&": {
    fontSize: "14px",
    border: "1px solid rgba(128,128,128,0.35)",
    borderRadius: "6px",
    backgroundColor: "#fafafa",
    color: "#383a42",
  },
  ".cm-content": {
    fontFamily: "ui-monospace, SFMono-Regular, Menlo, Consolas, monospace",
    caretColor: "#383a42",
  },
  ".cm-gutters": {
    backgroundColor: "#f0f0f0",
    color: "#a0a1a7",
    border: "none",
  },
  ".cm-activeLine": { backgroundColor: "rgba(0,0,0,0.03)" },
  ".cm-activeLineGutter": { backgroundColor: "rgba(0,0,0,0.05)" },
  "&.cm-focused": { outline: "2px solid rgba(64,120,242,0.4)" },
});

// Minimal chrome for the toolbar / results pane, injected once. The editor
// surface itself is themed via CodeMirror's `baseTheme` above.
const CHROME_CSS = `
.algae-editor { margin: 1rem 0; }
.algae-editor-host { }
.algae-toolbar { display: flex; gap: .5rem; margin-top: .4rem; }
.algae-btn {
  font: inherit; font-size: 13px; cursor: pointer;
  padding: .25rem .7rem; border-radius: 5px;
  border: 1px solid rgba(128,128,128,0.4);
  background: #4078f2; color: #fff;
}
.algae-btn:hover { background: #345fd0; }
.algae-btn:active { transform: translateY(1px); }
.algae-result {
  margin-top: .4rem; padding: .35rem .6rem; border-radius: 5px;
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  font-size: 13px; white-space: pre-wrap; min-height: 1.2em;
}
.algae-result:empty { display: none; }
.algae-result-ok { background: rgba(80,161,79,0.15); color: #2e7d32; }
.algae-result-error { background: rgba(228,86,73,0.12); color: #c62828; }
.algae-result-line { margin-top: .15rem; }
`;

let stylesInjected = false;
function injectStyles() {
  if (stylesInjected || typeof document === "undefined") return;
  const style = document.createElement("style");
  style.setAttribute("data-algae-editor", "");
  style.textContent = CHROME_CSS;
  document.head.appendChild(style);
  stylesInjected = true;
}

function el(tag, className, text) {
  const node = document.createElement(tag);
  if (className) node.className = className;
  if (text != null) node.textContent = text;
  return node;
}

function renderResult(pane, result) {
  pane.textContent = "";
  if (!result) return;
  if (result.error) {
    pane.className = "algae-result algae-result-error";
    pane.textContent = "✗ " + result.error;
    return;
  }
  if (result.ok) {
    pane.className = "algae-result algae-result-ok";
    let msg = "✓ checked " + result.obligations + " proof obligation" + (result.obligations === 1 ? "" : "s");
    if (result.wip > 0) msg += " (" + result.wip + " in progress)";
    pane.textContent = msg;
    return;
  }
  pane.className = "algae-result algae-result-error";
  const n = result.diagnostics.length;
  const header = el("div", null, "✗ " + n + " error" + (n === 1 ? "" : "s"));
  pane.appendChild(header);
  for (const d of result.diagnostics) {
    const where = d.has_span ? d.line + ":" + d.col + "  " : "";
    pane.appendChild(el("div", "algae-result-line", where + d.message));
  }
}

// Build and mount an editor. Options:
//   doc        initial source (string)
//   moduleName unit name for the checker (default "playground")
//   wasm       initialized algae-wasm module ({ check, format })
//   extra      optional [[name, source], …] pairs exposed to `import`
//   readOnly   if true, the editor is not editable
//   showToolbar if false, omit the Check/Format buttons (default true)
// Returns { view, container, check() }.
export function mountAlgaeEditor(parent, opts = {}) {
  injectStyles();
  const wasm = opts.wasm;
  const moduleName = opts.moduleName || "playground";

  const container = el("div", "algae-editor");
  const editorHost = el("div", "algae-editor-host");
  const pane = el("div", "algae-result");
  container.appendChild(editorHost);

  const extensions = [
    lineNumbers(),
    highlightActiveLineGutter(),
    highlightSpecialChars(),
    history(),
    drawSelection(),
    highlightActiveLine(),
    bracketMatching(),
    keymap.of([...defaultKeymap, ...historyKeymap, ...lintKeymap, indentWithTab]),
    algae(),
    baseTheme,
    EditorState.tabSize.of(2),
    EditorView.editable.of(!opts.readOnly),
  ];

  // Only wire the checker when a wasm module is supplied (highlighting-only
  // editors — e.g. non-interactive doc blocks — omit it).
  if (wasm) {
    extensions.push(lintGutter());
    extensions.push(
      algaeLinter({
        wasm,
        moduleName,
        extra: opts.extra,
        onResult: (r) => renderResult(pane, r),
      })
    );
  }

  const view = new EditorView({
    state: EditorState.create({ doc: opts.doc || "", extensions }),
    parent: editorHost,
  });

  const check = () => { if (wasm) forceLinting(view); };

  if (wasm && opts.showToolbar !== false) {
    const toolbar = el("div", "algae-toolbar");
    const checkBtn = el("button", "algae-btn", "Check ▶");
    checkBtn.type = "button";
    checkBtn.addEventListener("click", check);
    toolbar.appendChild(checkBtn);

    if (wasm.format) {
      const fmtBtn = el("button", "algae-btn", "Format");
      fmtBtn.type = "button";
      fmtBtn.addEventListener("click", () => {
        const res = wasm.format(view.state.doc.toString(), false);
        if (res && res.ok && typeof res.text === "string") {
          view.dispatch({ changes: { from: 0, to: view.state.doc.length, insert: res.text } });
        }
      });
      toolbar.appendChild(fmtBtn);
    }
    container.appendChild(toolbar);
    container.appendChild(pane);
    // Run an initial check so the pane reflects the seed proof.
    requestAnimationFrame(check);
  }

  parent.appendChild(container);
  return { view, container, check };
}

// Convenience default for direct <script type="module"> use.
export default { mountAlgaeEditor };
