==============
The editor
==============

Before any syntax, meet the tool you'll spend the whole tutorial in. Every Algae
block on this site is a live editor running the real kernel in your browser.
Here's one — a small, already-finished proof:

.. code-block:: alg

   import nat;
   import core(refl);

   lemma zero_is_zero
     ⊢ 0 = 0;
   proof
     by refl(Nat, 0);
   qed;

.. admonition:: Don't read the syntax yet
   :class: note

   Genuinely — don't try to parse ``by refl(Nat, 0)`` or ``⊢`` right now. The
   next few chapters build all of that up from scratch. For this chapter, the
   editor itself is the subject. Just click the buttons and watch what happens.

Check ▶
=======

The **Check ▶** button (or **Ctrl-Enter**, **⌘-Enter** on a Mac) runs the
kernel on whatever is in the editor. A results line appears underneath:

- **✓ checked N proof obligation(s)** — success. Every claim in the buffer holds.
- **✗ …** with red underlines — something failed. The message reads
  ``line:col  what went wrong``, and the offending spot is underlined in the
  editor. Fix the *first* error first; later ones are often just fallout.

Press **Check ▶** on the proof above — it's complete, so you should see the
green success line. Now change ``refl(Nat, 0)`` to ``refl(Nat, 1)`` and check
again: the kernel objects, because ``0 = 0`` is not ``1 = 1``. Change it back.

Holes: ``wip`` and ``?name``
============================

You rarely write a proof top-to-bottom in one go. Algae lets you leave the parts
you haven't figured out as **holes**, and it still checks everything around them.

- ``by wip;`` **admits** a goal — it tells the kernel "trust me here for now." A
  proof that leans on ``wip`` checks as *in progress* rather than done, and the
  results line says so.
- ``by wip(?goal);`` admits the goal **and prints it** — the exact thing you
  still owe, the assumptions in scope, and a list of rules that might discharge
  it. Stuck? Drop in a named hole, check, and read what it asks for.

Try it: replace ``by refl(Nat, 0);`` above with ``by wip(?here);`` and press
**Check ▶**. Instead of an error you'll get a *report* of the open goal.

Suggestions
===========

When a check flags something fixable — a hole with candidate rules, a mismatched
``qed``/``wip``, or a half-applied step — the editor can *offer the fix* and
apply it for you. **Click the underlined spot** (or press **Ctrl-Space**) to see
the suggestions; pick one and it rewrites the source, then re-checks
automatically so the next suggestion is ready. It's the fastest way to make
progress when you're not sure what to type — accept a suggestion, see what the
kernel says next, repeat.

Format
======

On the docs playground (and from the CLI) a **Format** button tidies a buffer:
it normalises operator glyphs so, for example, the ASCII ``|-`` becomes the
Unicode ``⊢`` and ``/\`` becomes ``∧``. Every operator has both an ASCII and a
Unicode spelling and they mean exactly the same thing to the kernel — type
whichever you find comfortable and let Format make it pretty.

.. tip::

   That's the whole toolbox: **Check** to run the kernel, **holes** to leave
   gaps and ask questions, **Suggestions** to fill them, and **Format** to tidy
   up. Keep this chapter's editor open in another tab if you like — from here on
   we start explaining what the proofs actually *say*.
