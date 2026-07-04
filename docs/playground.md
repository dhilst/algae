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
