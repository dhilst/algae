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
<style>
.algae-load-bar { display: flex; gap: .5rem; margin: 1rem 0 .3rem; }
.algae-load-bar input { flex: 1; font: inherit; font-size: 13px; padding: .3rem .5rem;
  border: 1px solid rgba(128,128,128,0.4); border-radius: 5px; }
.algae-load-bar button { font: inherit; font-size: 13px; cursor: pointer; padding: .3rem .8rem;
  border-radius: 5px; border: 1px solid rgba(128,128,128,0.4); background: #4078f2; color: #fff; }
</style>
<div class="algae-load-bar">
  <input id="algae-load-url" type="text"
         placeholder="Load a .alg from a URL — a game room, or a raw.githubusercontent.com link" />
  <button id="algae-load-btn" type="button">Load</button>
</div>
<div id="algae-playground"
     data-seed-url="_static/examples/nat.alg"
     data-module="nat"></div>
```

```{admonition} Load and share proofs by URL
:class: note
Paste any URL into the box above and press **Load** to open that `.alg` file in
the editor — a [game](game.md) room like `game/challenge/060-room.alg`, or an
absolute link such as a raw GitHub file
(`https://raw.githubusercontent.com/dhilst/algae/refs/heads/main/algae/v1/challenge/002-room.alg`).

The address updates to `playground.html?q=<url>`, so **that link is itself
shareable** — send it to anyone and the playground loads the proof automatically.
(Cross-origin hosts must allow it via CORS; raw GitHub does.)
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
