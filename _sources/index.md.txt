# Algae

**Algae** is a small proof and algebraic-specification language. You declare a
vocabulary (sorts, operators), assert facts (axioms) and inference rules, and
then prove lemmas by writing **explicit proof trees** that a tiny trusted kernel
re-checks.

This site is **interactive**: every example below is a live editor. Edit a
proof and press **Check ▶** — it runs the real Algae kernel, compiled to
WebAssembly, right in your browser. No install, no server.

```alg
import core(refl);

sort T : Sort;
op a : -> T;

lemma a_refl
  |- a = a;
proof
  by refl(T, a);
qed;
```

```{admonition} Try it
:class: tip
The block above is editable. Change `refl(T, a)` to `refl(T, T)` and press
**Check ▶** to see the kernel reject the proof, with the error underlined
inline.
```

## How checking works

The toolchain is a pipeline — **Parse → Elaborate → IR → Check** — and the
kernel is deliberately *environment-free* (no threads, filesystem, or terminal
I/O), which is exactly what makes it portable to WebAssembly. The checker never
trusts a proof: it re-derives every step locally and verifies the goal linkage,
so what you see checked here is checked by the same logic the command-line tool
uses.

## Contents

```{toctree}
:maxdepth: 2

tutorial/index
playground
game
```
