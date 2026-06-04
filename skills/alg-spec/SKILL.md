---
name: alg-spec
description: "Algebraic specification language (.alg files) for precise code development using equational notation. Activate when: (1) user mentions specifications, specs, .alg files, or algebraic specs, (2) .alg files exist in the project and user is implementing or reviewing related code, (3) user asks to design, specify, or formalize module behavior."
version: 1.0.0
---

# Algebraic Specifications (`.alg`)

`.alg` files are lightweight algebraic specifications. They describe sorts, operation signatures, variables, and axioms without model checking or type checking.

## Quick Reference

```
sort Stack, Elem;
sort Error = {empty_error};

op empty : -> Stack;
op push : Stack × Elem -> Stack;
op pop : Stack -> Stack | Error;
op top : Stack -> Elem | Error;

var s : Stack;
var e : Elem;

axiom top(push(s, e)) = e;
axiom pop(push(s, e)) = s;
axiom top(empty()) = empty_error;
axiom pop(empty()) = empty_error;
```

**Key conventions:**
- `sort` declares carrier sets or enum-like constructor sets.
- `op` declares operation signatures; an empty domain is written `op empty : -> Stack;`.
- `|` in result types represents an algebraic sum/union, commonly for error alternatives.
- `var` declarations are read as implicitly universally quantified over all axioms.
- `axiom` gives equations or predicates that document intended behavior.
- Lowercase ASCII aliases such as `product`, `arrow`, `neq`, `in`, and `emptyset` parse as Unicode symbols.
- See [references/syntax.md](references/syntax.md) for the full grammar.
- See [references/examples.md](references/examples.md) for example specs.

## When Working Near `.alg` Files

1. Scan for `.alg` files that specify the module you are about to implement.
2. Read the sorts to identify domain concepts and error constructors.
3. Map each `op` to the implementation's public functions or methods.
4. Use `var` and `axiom` declarations as behavioral laws and test ideas.
5. Preserve the distinction between syntax and semantics: the parser accepts syntax only.

## When Reviewing Code Against A Spec

- Check that every operation has a corresponding function or method.
- Check that return/error behavior matches the declared codomain.
- Check that axioms are reflected in implementation logic or tests.
- Flag public behavior that has no corresponding operation or axiom.
