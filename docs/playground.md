# Playground

A full-page editor for experimenting with Algae. It is seeded with the standard
library's `nat` module — a complete, checked development of the natural numbers,
including a proof by induction. Edit anything and press **Check ▶** to re-run the
kernel.

```{admonition} Everything runs locally
:class: note
The proof checker is [`algae-kernel`](https://github.com/danielhilst/algae)
compiled to WebAssembly. Your code never leaves the page. The other standard
library modules (`core`, `list`, `option`, `result`, `group`, `monad`, `adt`)
are available via `import`.
```

```{raw} html
<div id="algae-playground"
     data-seed-url="_static/examples/nat.alg"
     data-module="nat"></div>
```

```{admonition} If a proof is incomplete
:class: tip
A proof that ends in `wip`, or that leaves a hole `_` where a real step is
required, checks as *in progress* rather than done — the results pane tells you
so. Fill in the holes and check again.
```

```{admonition} Suggested fixes
:class: tip
When a check flags a fixable error — a `wip`/`qed` terminator that doesn't match
the proof, a `by wip(?goal)` hole with candidate tactics, or a tactic hole that
suggests a continuation — you can apply the fix without retyping it. **Click the
flagged text** (or press **Ctrl-Space**) to see the suggestions, then pick one to
rewrite the source; the checker re-runs automatically so the next suggestion is
ready. Hole candidates seed a complete `by <tactic>?;` step, so the next check
guides you to its arguments.
```
