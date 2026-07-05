# Algae CodeMirror 6 editor

A browser editor for the Algae proof language, built on [CodeMirror 6]. It
provides:

- **Syntax highlighting** — a token-level `StreamLanguage` ported from the
  tree-sitter grammar in [`../tree-sitter`](../tree-sitter) (keywords, ASCII +
  Unicode operators, the 24+-dash inference separator, numbers, holes,
  declaration names).
- **Proof checking in the browser** — runs the real kernel compiled to
  WebAssembly by the [`algae-wasm`](../../algae-wasm) crate.
- **Error reporting** — kernel diagnostics rendered as inline squiggles, gutter
  markers, and a results pane.
- **Emacs keybindings** — the `@replit/codemirror-emacs` keymap (`C-a`/`C-e`,
  `C-k`, `C-w`/`M-w`/`C-y`, `C-s`/`C-r`, mark & kill-ring, …). `Ctrl-Enter` still
  runs the checker.

## Build

```sh
npm ci
npm run build       # → dist/algae-editor.js  (minified ESM)
npm run build:dev   # → dist/algae-editor.js  (with sourcemap)
```

The output `dist/algae-editor.js` is a self-contained ES module (CodeMirror is
bundled in; CSS chrome is injected at runtime).

## Usage

The page owns wasm loading and passes the initialized module in, so one wasm
instance is shared by every editor on the page:

```html
<script type="module">
  import initWasm, * as wasm from "./algae_wasm.js";     // from algae-wasm/pkg
  import { mountAlgaeEditor } from "./algae-editor.js";

  await initWasm();                                        // load the .wasm
  mountAlgaeEditor(document.getElementById("host"), {
    wasm,                                                  // { check, format }
    moduleName: "playground",
    doc: "import core(refl);\n\nsort T : Sort;\nop a : -> T;\n\nlemma a_refl\n  |- a = a;\nproof\n  by refl(T, a);\nqed;\n",
  });
</script>
```

### `mountAlgaeEditor(parent, opts)`

| option | meaning |
| --- | --- |
| `doc` | initial source string |
| `wasm` | initialized algae-wasm module (`{ check, format }`); omit for highlight-only |
| `moduleName` | unit name passed to the checker (default `"playground"`) |
| `extra` | optional `[[name, source], …]` pairs exposed to `import` |
| `readOnly` | make the editor non-editable |
| `showToolbar` | set `false` to hide the Check/Format buttons |

Returns `{ view, container, check() }`. Highlighting-only editors (no `wasm`)
skip the linter and toolbar entirely.

The named exports `algae()`, `algaeStreamLanguage`, `algaeHighlightStyle`, and
`algaeLinter()` are also available if you want to assemble your own `EditorView`.

[CodeMirror 6]: https://codemirror.net/
