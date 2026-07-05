// A proof encounter. Mounts the real Algae CodeMirror editor (reused from the
// docs) seeded with the monster's unfinished proof, and wires a "Cast Proof"
// button that runs the WebAssembly kernel. A proof defeats the monster only when
// it checks with no errors AND nothing left admitted (result.ok && result.wip === 0).

import { openHelp } from "./help.js";

function el(tag, cls, text) {
  const n = document.createElement(tag);
  if (cls) n.className = cls;
  if (text != null) n.textContent = text;
  return n;
}

export function startCombat(host, opts) {
  const { wasm, mountAlgaeEditor, challenge, canFlee, fleeFail, onWin, onFlee } = opts;
  const meta = challenge.meta;
  const isDragon = meta.monster === "dragon" || meta.tier === "boss";
  const face = meta.tier === "boss" ? "😈" : isDragon ? "🐉" : "🦁";

  host.textContent = "";
  const panel = el("div", "combat");

  const header = el("div", "combat-head");
  header.appendChild(el("span", "combat-face", face));
  const titleWrap = el("div", "combat-titlewrap");
  titleWrap.appendChild(el("div", "combat-title", meta.title));
  titleWrap.appendChild(el("div", "combat-sub", isDragon ? "A dragon blocks the hatch." : "A sphinx bars the way."));
  header.appendChild(titleWrap);
  panel.appendChild(header);

  panel.appendChild(el("p", "combat-prompt", meta.prompt));

  const editorSlot = el("div", "combat-editor");
  panel.appendChild(editorSlot);

  const message = el("div", "combat-message");
  panel.appendChild(message);

  const buttons = el("div", "combat-buttons");
  const castBtn = el("button", "btn btn-primary", "⚔ Cast Proof");
  buttons.appendChild(castBtn);
  const helpBtn = el("button", "btn", "❓ Help");
  helpBtn.addEventListener("click", () => openHelp(challenge));
  buttons.appendChild(helpBtn);
  let fleeBtn = null;
  if (canFlee) {
    fleeBtn = el("button", "btn", `🏃 Flee (${Math.round(fleeFail * 100)}% risk)`);
    buttons.appendChild(fleeBtn);
  } else {
    buttons.appendChild(el("span", "combat-noflee", "There is no fleeing this one."));
  }
  panel.appendChild(buttons);
  host.appendChild(panel);

  // The reused editor gives syntax highlighting + inline diagnostics for free.
  const editor = mountAlgaeEditor(editorSlot, {
    doc: challenge.src,
    wasm,
    moduleName: meta.module || "room" + challenge.id,
    showToolbar: false, // the game supplies its own "Cast Proof" button
  });

  let done = false;

  const cast = () => {
    if (done || !wasm) return;
    const src = editor.view.state.doc.toString();
    let result;
    try {
      result = wasm.check(src, meta.module || "room" + challenge.id, undefined);
    } catch (err) {
      message.className = "combat-message bad";
      message.textContent = "The kernel recoils: " + (err && err.message ? err.message : String(err));
      return;
    }
    editor.check(); // paint inline diagnostics in the editor

    if (result.ok && result.wip === 0) {
      done = true;
      message.className = "combat-message good";
      message.textContent = "✦ The proof holds. The " + (isDragon ? "dragon" : "sphinx") + " dissolves into light.";
      castBtn.disabled = true;
      if (fleeBtn) fleeBtn.disabled = true;
      setTimeout(() => onWin(meta), 700);
      return;
    }

    message.className = "combat-message bad";
    if (result.ok && result.wip > 0) {
      message.textContent = "The proof is unfinished — " + result.wip + " goal still admitted (wip). Replace every `wip` and close with `qed`.";
    } else {
      const n = result.diagnostics.length;
      const first = result.diagnostics[0];
      message.textContent = "The proof falters (" + n + " error" + (n === 1 ? "" : "s") + ")" + (first ? ": " + first.message : ".");
    }
  };

  castBtn.addEventListener("click", cast);

  // Ctrl/Cmd-Enter casts the proof (win check), not just the editor's internal
  // check. Caught in the capture phase on the editor host so it runs before
  // CodeMirror's own Mod-Enter binding, which we then suppress to avoid a
  // redundant second check.
  editorSlot.addEventListener(
    "keydown",
    (e) => {
      if ((e.ctrlKey || e.metaKey) && e.key === "Enter") {
        e.preventDefault();
        e.stopPropagation();
        cast();
      }
    },
    true
  );

  if (fleeBtn) {
    fleeBtn.addEventListener("click", () => {
      if (done) return;
      done = true;
      onFlee(Math.random() >= fleeFail); // true = escaped, false = the monster catches you
    });
  }

  // Focus the editor so the player can start typing immediately.
  setTimeout(() => editor.view.focus(), 30);
}
