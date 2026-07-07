// Proof-checks the buffer with algae-wasm and renders diagnostics as inline
// squiggles + gutter markers. Checking is **manual** — there is no auto-linter
// watching document changes; `runAlgaeCheck` is called explicitly (Check button
// / Ctrl-Enter) and installs the diagnostics via `setDiagnostics`. Existing
// diagnostics stay put (remapped) while you type, until the next manual check.

import { setDiagnostics } from "@codemirror/lint";
import { StateEffect, StateField } from "@codemirror/state";

// Machine-applicable fixes from the last check, kept as editor state so the
// autocomplete source (see index.js) can offer them at the cursor. Each mark is
// `{ from, to, title, replacement }` in CodeMirror positions. `runAlgaeCheck`
// republishes the set on every run; between runs the positions are remapped
// through edits so a fix keeps pointing at the right text while you type.
export const setAlgaeFixes = StateEffect.define();

export const algaeFixesField = StateField.define({
  create() {
    return [];
  },
  update(marks, tr) {
    for (const e of tr.effects) {
      if (e.is(setAlgaeFixes)) return e.value;
    }
    if (tr.docChanged) {
      return marks.map((m) => ({
        ...m,
        from: tr.changes.mapPos(m.from, -1),
        to: tr.changes.mapPos(m.to, 1),
      }));
    }
    return marks;
  },
});

// Map a wasm diagnostic (1-based line/col, per algae_kernel::line_col) to an
// absolute CodeMirror position. We deliberately use line/col rather than the
// raw byte `start`/`end`, because the wasm spans are UTF-8 byte offsets while
// CodeMirror positions are UTF-16 code units — they diverge as soon as a proof
// uses Unicode operators (⊢, ∀, ∧, …). Column counts Unicode scalar values,
// which equals UTF-16 units for every glyph Algae uses (all in the BMP).
function posOf(doc, line, col) {
  const clampedLine = Math.max(1, Math.min(line, doc.lines));
  const l = doc.line(clampedLine);
  return Math.max(l.from, Math.min(l.from + (col - 1), l.to));
}

function toCmDiagnostic(doc, d) {
  let from;
  let to;
  if (d.has_span) {
    from = posOf(doc, d.line, d.col);
    to = posOf(doc, d.end_line, d.end_col);
    if (to <= from) to = Math.min(from + 1, doc.length);
  } else {
    // Span-less diagnostic: anchor to the whole first line.
    from = 0;
    to = Math.min(doc.line(1).to, doc.length);
  }
  return {
    from,
    to,
    severity: d.severity === "warning" ? "warning" : "error",
    message: d.message,
  };
}

// Build a linter extension. `opts.wasm` is the initialized algae-wasm module
// exposing `check(source, moduleName, extra)`. `opts.moduleName` names the unit
// (default "playground"). `opts.extra` is an optional array of [name, source]
// pairs exposed to `import`. `opts.onResult(result)` (optional) is called with
// the raw CheckResult after every run, so a results pane can react.
// Run the checker on the current document and install its diagnostics. Returns
// the raw CheckResult (also passed to `opts.onResult`, for a results pane).
export function runAlgaeCheck(view, opts) {
  const wasm = opts.wasm;
  const moduleName = opts.moduleName || "playground";
  const extra = opts.extra || undefined;
  const onResult = opts.onResult;
  const doc = view.state.doc;

  let result;
  let cmDiags;
  let fixMarks = [];
  try {
    result = wasm.check(doc.toString(), moduleName, extra);
    cmDiags = result.diagnostics.map((d) => toCmDiagnostic(doc, d));
    fixMarks = collectFixMarks(doc, result.diagnostics);
  } catch (err) {
    // A panic in the checker should surface, not silently pass.
    const message = "internal checker error: " + (err && err.message ? err.message : String(err));
    result = { ok: false, diagnostics: [], obligations: 0, wip: 0, error: message };
    cmDiags = [{ from: 0, to: Math.min(doc.line(1).to, doc.length), severity: "error", message }];
  }
  // One transaction installs the diagnostics and republishes the fix set.
  view.dispatch(setDiagnostics(view.state, cmDiags), { effects: setAlgaeFixes.of(fixMarks) });
  if (onResult) onResult(result);
  return result;
}

// Flatten each diagnostic's `fixes` into position-anchored marks for the
// autocomplete source. A diagnostic without fixes contributes nothing.
function collectFixMarks(doc, diagnostics) {
  const marks = [];
  for (const d of diagnostics) {
    if (!d.fixes) continue;
    for (const f of d.fixes) {
      const from = posOf(doc, f.line, f.col);
      const to = posOf(doc, f.end_line, f.end_col);
      marks.push({ from, to, title: f.title, replacement: f.replacement });
    }
  }
  return marks;
}
