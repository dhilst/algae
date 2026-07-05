# Dungeon Crawler

**Dungeon Proof Crawler** is a browser roguelike where every monster is an Algae
proof. A demon stole your wedding ring on the eve of your wedding; Miriam wants
it back before sunrise. Descend seven levels, and defeat each sphinx and dragon
by *completing* the unfinished proof it guards — checked live by the same kernel
that powers the [Playground](playground.md), compiled to WebAssembly.

```{admonition} It opens full-screen
:class: tip
The game runs as its own page, outside the documentation chrome. Everything —
world generation, your run, saved progress — stays in your browser. Proofs are
checked locally; nothing leaves the page.
```

```{raw} html
<p style="text-align:center;margin:2rem 0;">
  <a href="game/index.html"
     style="display:inline-block;padding:0.9rem 1.6rem;font-weight:700;
            text-decoration:none;color:#241706;background:#e8a33d;
            border:3px solid #6b4413;box-shadow:4px 4px 0 #201a16;
            text-transform:uppercase;letter-spacing:1px;">
    ⚔ Enter the Dungeon
  </a>
</p>
```

## How you fight

Each room holds a monster and an unfinished proof — one that ends in `wip`, the
Algae marker for an admitted (not-yet-proven) goal. Replace every `wip` with the
real step and close the block with `qed`; press **Cast Proof**. If the kernel
reports no errors and nothing left admitted, the monster falls and the room is
yours.

Difficulty rises as you descend — from a single `refl` on Level -1 to full
proofs by induction near the bottom. Defeat monsters to grow your maximum health,
open chests for food and lore, and beware the hunger that drains you while you
linger. Reach Level -6, recover the ring, and climb back before sunrise.

If you are new to writing Algae proofs, work through the
[tutorial](tutorial/index) first — the crawler assumes you can read and finish a
proof tree.

