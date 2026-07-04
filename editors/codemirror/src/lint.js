// Proof-checks the buffer with algae-wasm and renders diagnostics as inline
// squiggles + gutter markers. Checking is **manual** — there is no auto-linter
// watching document changes; `runAlgaeCheck` is called explicitly (Check button
// / Ctrl-Enter) and installs the diagnostics via `setDiagnostics`. Existing
// diagnostics stay put (remapped) while you type, until the next manual check.

import { setDiagnostics } from "@codemirror/lint";

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
  try {
    result = wasm.check(doc.toString(), moduleName, extra);
    cmDiags = result.diagnostics.map((d) => toCmDiagnostic(doc, d));
  } catch (err) {
    // A panic in the checker should surface, not silently pass.
    const message = "internal checker error: " + (err && err.message ? err.message : String(err));
    result = { ok: false, diagnostics: [], obligations: 0, wip: 0, error: message };
    cmDiags = [{ from: 0, to: Math.min(doc.line(1).to, doc.length), severity: "error", message }];
  }
  view.dispatch(setDiagnostics(view.state, cmDiags));
  if (onResult) onResult(result);
  return result;
}
