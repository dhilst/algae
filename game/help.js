// Full-screen, scrollable help modal for a proof encounter. Composes:
//   1. per-room help / technique (from the challenge .json `help` field),
//   2. general editor + holes + keybinding guidance,
//   3. how to read the checker's messages,
//   4. source links to every rule the room imports (GitHub, line-anchored).

import { ruleUrl } from "./rules.js";

function el(tag, cls, text) {
  const n = document.createElement(tag);
  if (cls) n.className = cls;
  if (text != null) n.textContent = text;
  return n;
}

// Pull `(module, [names])` out of every `import mod(a, b);` in the proof source.
function parseImports(src) {
  const out = [];
  const re = /import\s+([A-Za-z_]\w*)\s*\(([^)]*)\)/g;
  let m;
  while ((m = re.exec(src))) {
    const mod = m[1];
    for (const raw of m[2].split(",")) {
      const name = raw.trim();
      if (name) out.push({ mod, name });
    }
  }
  return out;
}

const GENERAL_HTML = `
  <h3>Using the editor</h3>
  <ul>
    <li><b>Check your proof:</b> press <kbd>Ctrl</kbd>+<kbd>Enter</kbd>
      (<kbd>⌘</kbd>+<kbd>Enter</kbd> on Mac) or click <b>⚔ Cast Proof</b>. The Algae
      kernel runs right here in your browser and marks any problems inline.</li>
    <li><b>Holes &amp; <code>wip</code>:</b> the proof arrives unfinished.
      <code>by wip;</code> <i>admits</i> a goal — it tells the kernel “trust me for
      now,” so the proof checks as <i>in progress</i> instead of done. To defeat the
      monster, replace every <code>by wip;</code> with a real step and change each
      closing <code>wip;</code> back to <code>qed;</code>. The monster falls only when
      nothing is left admitted.</li>
    <li><b>The <code>_</code> hole:</b> an underscore is a placeholder the kernel fills
      in for you — for example a motive written <code>_ = _</code>. Use it when the
      shape is forced and you'd rather not spell it out.</li>
    <li><b>Named holes <code>by wip(?name);</code>:</b> these admit the goal <i>and</i>
      print it (with candidate rules), so you can see exactly what's left to prove.
      Stuck? Drop in <code>by wip(?here);</code>, check, and read what it asks for.</li>
    <li><b>Tactic holes — <code>by rule?;</code>:</b> not sure which arguments a rule
      needs, or what it leaves behind? Apply it with a trailing <code>?</code> and no
      (or partial) arguments — say <code>by or_intro_left?;</code>. The kernel replies
      with the arguments it <i>inferred</i> (<code>?P = …</code>, <code>?Q = …</code>)
      and the subgoal(s) the rule produces, then admits so the rest still checks. It's
      the fastest way to learn a step — the tour calls it “let the checker tell you.”</li>
    <li><b>Hover for details:</b> after checking, hover the underlined spot — a hole
      shows the goal it still expects; an error shows what went wrong.</li>
  </ul>

  <h3>Reading the checker</h3>
  <ul>
    <li><code>✓ checked N proof obligations</code> — success. If it adds
      <code>(k in progress)</code>, you still have <code>wip</code> to fill.</li>
    <li><code>syntax error: unexpected token</code> — the parser choked. Usually a
      missing <code>;</code>, a <code>then</code>/<code>cases</code>/<code>qed</code>
      in the wrong place, or a stray token at the marked <code>line:col</code>.</li>
    <li><code>rule conclusion does not match the current goal</code> — the rule you
      applied doesn't produce this goal. Check its shape (linked below) and its
      arguments.</li>
    <li><code>found hole ?goal</code> — a named hole, reporting the goal you still owe.</li>
    <li><code>… admits a goal (by wip) but is closed with qed; use wip</code> — if any
      branch is <code>wip</code>, close the enclosing block with <code>wip</code> too;
      a fully proven block closes with <code>qed</code>.</li>
    <li>Messages read <code>line:col&nbsp;&nbsp;message</code> and underline in the
      editor. Fix the first one first — later errors are often just fallout.</li>
  </ul>

  <h3>Staying alive</h3>
  <ul>
    <li><b>Health:</b> you begin at <b>10 / 10</b> HP. Banishing a sphinx grants
      <b>+2 HP and +2 max HP</b>; felling a dragon grants <b>+5 / +5</b>. There is no
      ceiling on maximum health — the deeper you fight, the tougher you get.</li>
    <li><b>Food</b> restores <b>5 HP</b> (never above your maximum). Some sphinxes drop
      it and treasure chests hold rations; eat from the 🍖 counter in the top bar.</li>
    <li><b>Potion of Vigor</b> (found in a chest) raises your maximum by <b>+5</b> and
      heals <b>+5</b>.</li>
    <li><b>Hunger &amp; damage:</b> the dungeon is hungry — you lose <b>1 HP every real
      minute</b> you spend down here, and if you hit <b>0</b> the dark takes you.
      <b>This drain pauses while this help is open</b>, so read at your leisure.</li>
    <li><b>Engaging:</b> wandering into a room with an unbeaten sphinx has a
      <b>1-in-3 chance</b> it seizes you on the spot — you cannot simply stroll past
      them. You may also choose to <b>Face</b> any monster. <b>Fleeing</b> a sphinx can
      fail (10% on Level -1, rising to 50% on -5, impossible on -6), and a failed
      escape costs you a bite of HP. Dragons cannot be fled.</li>
    <li><b>Getting deeper:</b> a dragon guards the hatch from Level -3 down — beat it
      for the key. Reach Level -6, defeat the demon, take the ring, and climb back to
      the surface before sunrise.</li>
  </ul>
`;

let openModal = null;

export function openHelp(challenge) {
  closeHelp();
  const meta = challenge.meta || {};

  const overlay = el("div", "help-overlay");
  const modal = el("div", "help-modal");

  const bar = el("div", "help-bar");
  bar.appendChild(el("h2", "help-title", "How to banish this monster"));
  const x = el("button", "help-close", "✕");
  x.title = "Close (Esc)";
  x.addEventListener("click", closeHelp);
  bar.appendChild(x);
  modal.appendChild(bar);

  const body = el("div", "help-body");

  // 1. Per-room help / technique.
  if (meta.help) {
    const room = el("div", "help-room");
    room.appendChild(el("h3", null, meta.title ? `This room — ${meta.title}` : "This room"));
    for (const para of String(meta.help).split("\n")) {
      if (para.trim()) room.appendChild(el("p", null, para));
    }
    body.appendChild(room);
  }

  // 2 + 3. General editor / holes / error guidance (static, trusted HTML).
  const general = el("div", "help-general");
  general.innerHTML = GENERAL_HTML;
  body.appendChild(general);

  // 4. Source links for the rules this room imports.
  const imports = parseImports(challenge.src || "");
  if (imports.length) {
    const rules = el("div", "help-rules");
    rules.appendChild(el("h3", null, "The rules in this room"));
    rules.appendChild(el("p", "help-dim", "Read a rule's definition to see exactly what it proves and what arguments it needs."));
    const list = el("ul", "help-rulelist");
    const seen = new Set();
    for (const { mod, name } of imports) {
      const k = mod + ":" + name;
      if (seen.has(k)) continue;
      seen.add(k);
      const url = ruleUrl(mod, name);
      const li = el("li");
      const code = el("code", null, `${mod}(${name})`);
      li.appendChild(code);
      if (url) {
        li.appendChild(document.createTextNode(" — "));
        const a = el("a", "help-link", `${mod}.alg ↗`);
        a.href = url;
        a.target = "_blank";
        a.rel = "noopener noreferrer";
        li.appendChild(a);
      }
      list.appendChild(li);
    }
    rules.appendChild(list);
    body.appendChild(rules);
  }

  modal.appendChild(body);
  overlay.appendChild(modal);

  overlay.addEventListener("click", (e) => {
    if (e.target === overlay) closeHelp();
  });
  document.addEventListener("keydown", onKey);
  document.body.appendChild(overlay);
  openModal = overlay;
  // Let the game pause hunger/damage while the player is reading.
  document.dispatchEvent(new CustomEvent("help-open"));
  // Focus the modal so it scrolls with the keyboard and Esc is caught reliably.
  modal.tabIndex = -1;
  modal.focus();
}

function onKey(e) {
  if (e.key === "Escape") closeHelp();
}

export function closeHelp() {
  document.removeEventListener("keydown", onKey);
  if (openModal) {
    openModal.remove();
    openModal = null;
    document.dispatchEvent(new CustomEvent("help-close"));
  }
}
