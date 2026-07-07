================
Inference rules
================

Instead of grinding truth tables, Algae reasons with **inference rules**: small,
fixed steps you chain into a proof. This chapter is about what a rule *is* —
its shape, and how to read it — using conjunction as the running example. We
won't run the checker yet; we're learning to read.

Sequents and the turnstile ``⊢``
================================

Everything you prove is a **sequent**, written with a **turnstile** ``⊢``
(ASCII ``|-``):

.. code-block:: text

   context ⊢ proposition

Read it "under the *context*, the *proposition* holds." The proposition on the
right is the **goal** — what you're trying to establish. The **context** on the
left lists what you're allowed to assume: typed variables and named
hypotheses. When there's nothing to assume, the context is empty and you just
write ``⊢ proposition``.

.. code-block:: text

   ⊢ A ∧ B                    -- prove A ∧ B, no assumptions
   x := A ⊢ A                 -- assuming a proof x of A, prove A
   n : Nat, ih := P(n) ⊢ P(s(n))   -- given a number n and a proof of P(n), prove P(s(n))

Two kinds of thing live in a context. A **variable** like ``n : Nat`` is a term
you may use. A **hypothesis** like ``ih := P(n)`` is *evidence* — a proof you've
been handed and may cite by its name (here ``ih``). Keep those apart: ``n`` is a
number, ``ih`` is a proof.

Anatomy of a rule
=================

An **inference rule** is a fixed pattern with **premises** above a line and a
**conclusion** below it:

.. code-block:: text

   premise₁     premise₂     …     premiseₙ
   ─────────────────────────────────────────
                  conclusion

Read top-to-bottom it says: *if* you can establish all the premises, *then* you
may conclude the conclusion. Each premise and the conclusion is itself a
sequent. A rule with **no** premises is an **axiom** — an unconditional fact,
true on its own with nothing to establish first.

Conjunction, in words and in Algae
===================================

Take the rule that builds a conjunction. In words:

   **To prove ``A ∧ B`` you need a proof of ``A``, and a proof of ``B``.**

That is two premises (a proof of ``A``; a proof of ``B``) and one conclusion
(``A ∧ B``). As an inference rule:

.. code-block:: text

   ⊢ A          ⊢ B
   ─────────────────────
        ⊢ A ∧ B

Algae writes exactly this. Here is ``and_intro`` from the standard library's
``core`` module (``and`` *introduction* — the rule that introduces a ``∧``):

.. code-block:: alg

   rule and_intro(
     P Q : Prop
   )
     ⊢ P;
     ⊢ Q
     ────────────────────────
     ⊢ P ∧ Q
   end;

Piece by piece:

- ``rule and_intro(…)`` names the rule and lists its **parameters** in
  parentheses. ``P Q : Prop`` declares two propositions ``P`` and ``Q`` — the
  placeholders the rule is stated over.
- The lines above the ``────`` bar are the **premises**, separated by ``;`` —
  here ``⊢ P`` and ``⊢ Q``.
- The line below the bar is the **conclusion**, ``⊢ P ∧ Q``.
- ``end;`` closes the rule. (An axiom has no bar and no premises, and ends with a
  plain ``;`` instead — you'll meet ``refl`` that way later.)

.. admonition:: The bar is just dashes
   :class: note

   The separator is a run of ``─`` (or ASCII ``-``); its exact length carries no
   meaning. And the parameters ``P``, ``Q`` are *schematic* — the rule works for
   any propositions you plug in, not two specific ones.

The companion rules take a conjunction *apart* again. ``and_left`` recovers the
left half and ``and_right`` the right — one premise each (the conjunction), one
conclusion (the half):

.. code-block:: alg

   rule and_left(
     P Q : Prop
   )
     ⊢ P ∧ Q
     ────────────────────────
     ⊢ P
   end;

That "build one / use one" pairing — an **introduction** rule and an
**elimination** rule per connective — runs through the whole logic module.
You'll see it again in :doc:`tour/logic`.

So a rule is a template: premises on top, conclusion on the bottom, schematic
parameters standing in for the specifics. The next chapter puts one to work —
and reveals that in a *proof* you read these rules **bottom-to-top**.
