// A CodeMirror `linter` source that proof-checks the buffer with algae-wasm and
// renders diagnostics as inline squiggles + gutter markers.

import { linter } from "@codemirror/lint";

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
export function algaeLinter(opts) {
  const wasm = opts.wasm;
  const moduleName = opts.moduleName || "playground";
  const extra = opts.extra || undefined;
  const onResult = opts.onResult;

  return linter(
    (view) => {
      const doc = view.state.doc;
      const source = doc.toString();
      let result;
      try {
        result = wasm.check(source, moduleName, extra);
      } catch (err) {
        // A panic in the checker should surface, not silently pass.
        const message = "internal checker error: " + (err && err.message ? err.message : String(err));
        if (onResult) onResult({ ok: false, diagnostics: [], obligations: 0, wip: 0, error: message });
        return [{ from: 0, to: Math.min(doc.line(1).to, doc.length), severity: "error", message }];
      }
      if (onResult) onResult(result);
      return result.diagnostics.map((d) => toCmDiagnostic(doc, d));
    },
    { delay: 400 }
  );
}
