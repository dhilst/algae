// Dungeon Proof Crawler — main controller. Boots the reused Algae editor + wasm
// kernel, builds a deterministic dungeon from a seed, and drives the screen
// state machine (title → dungeon ⇄ combat/chest → win/lose). Progress persists
// to LocalStorage; hunger drains 1 HP per real minute while the page is open.

import { makeRng, randomSeedString } from "./rng.js";
import { generateDungeon, FLOORS, FLEE_FAIL } from "./dungeon.js";
import { loadManifest, loadChallenge } from "./challenges.js";
import { saveRun, loadRun, hasSave, clearRun } from "./save.js";
import { startCombat } from "./combat.js";

const app = document.getElementById("app");
const el = (tag, cls, text) => {
  const n = document.createElement(tag);
  if (cls) n.className = cls;
  if (text != null) n.textContent = text;
  return n;
};
const rk = (f, r) => f + ":" + r; // room key for the "solved / opened" sets

let wasm = null;
let mountAlgaeEditor = null;
let manifest = null;
let run = null;
let hungerTimer = null;

// ---- Boot ---------------------------------------------------------------

async function boot() {
  app.innerHTML = "";
  app.appendChild(el("div", "boot", "Lighting the torches…"));
  try {
    const editorMod = await import(new URL("../_static/algae-editor.js", import.meta.url));
    mountAlgaeEditor = editorMod.mountAlgaeEditor;
    const wasmMod = await import(new URL("../_static/algae_wasm.js", import.meta.url));
    await wasmMod.default();
    wasm = wasmMod;
    manifest = await loadManifest();
  } catch (err) {
    app.innerHTML = "";
    const box = el("div", "boot boot-error");
    box.appendChild(el("h1", null, "The torches won't catch."));
    box.appendChild(el("p", null, "The proof kernel or challenge files failed to load. This game must be served over HTTP (not opened as a file), with the Algae wasm build present in ../_static/."));
    box.appendChild(el("pre", null, String(err && err.message ? err.message : err)));
    app.appendChild(box);
    return;
  }
  titleScreen();
}

// ---- Title --------------------------------------------------------------

function titleScreen() {
  stopHunger();
  app.innerHTML = "";
  const wrap = el("div", "screen title");
  wrap.appendChild(el("div", "title-emblem", "💍"));
  wrap.appendChild(el("h1", "title-name", "Dungeon Proof Crawler"));
  wrap.appendChild(el("p", "title-tag", "Every monster is a proof. Bring back the ring before sunrise."));

  const note = el("p", "title-note");
  note.appendChild(document.createTextNode("This is a game about Algae, an algebraic specification language. If you don't know what Algae is, "));
  const tut = el("a", "title-note-link", "check out the tutorial first");
  tut.href = "../tutorial/index.html";
  tut.target = "_blank";
  tut.rel = "noopener noreferrer";
  note.appendChild(tut);
  note.appendChild(document.createTextNode("."));
  wrap.appendChild(note);

  const story = el("div", "title-story");
  story.appendChild(el("p", null,
    "On the eve of your wedding, a demon breaks into the house and steals the wedding ring. Miriam — the purest, most luminous soul in the kingdom — does not weep. She takes your hand and says only, with a faith in you that you have never quite been able to share: “Bring it back before sunrise.”"));
  story.appendChild(el("p", null, "You go down after it, into an underworld that somehow seems to remember you. Defeat each monster by completing the unfinished proof it guards. Recover the ring. Return before sunrise."));
  wrap.appendChild(story);

  const form = el("div", "title-form");
  const seedRow = el("div", "seed-row");
  const seedInput = el("input", "seed-input");
  seedInput.type = "text";
  seedInput.placeholder = "seed (leave blank for random)";
  seedInput.spellcheck = false;
  const rollBtn = el("button", "btn", "🎲");
  rollBtn.title = "Random seed";
  rollBtn.addEventListener("click", () => (seedInput.value = randomSeedString()));
  seedRow.appendChild(seedInput);
  seedRow.appendChild(rollBtn);
  form.appendChild(seedRow);

  const newBtn = el("button", "btn btn-primary btn-wide", "⚔ Begin the Descent");
  newBtn.addEventListener("click", () => {
    const seed = seedInput.value.trim() || randomSeedString();
    startNewRun(seed);
  });
  form.appendChild(newBtn);

  if (hasSave()) {
    const contBtn = el("button", "btn btn-wide", "🕯 Continue your descent");
    contBtn.addEventListener("click", () => {
      const saved = loadRun();
      if (saved) resumeRun(saved);
      else startNewRun(randomSeedString());
    });
    form.appendChild(contBtn);
  }
  wrap.appendChild(form);
  wrap.appendChild(el("p", "title-foot", "A love letter to formal proof · powered by the Algae kernel, compiled to WebAssembly."));
  app.appendChild(wrap);
}

// ---- Run lifecycle ------------------------------------------------------

function startNewRun(seed) {
  const dungeon = generateDungeon(seed, manifest);
  run = {
    seed,
    dungeon,
    floorIndex: 0,
    roomId: dungeon.floors[0].entryId,
    player: { hp: 10, maxHp: 10, food: 0, keys: new Set(), ring: false },
    solved: new Set(),
    openedChests: new Set(),
    defeatedBosses: new Set(),
    seenLore: new Set(),
    lastCause: null,
  };
  persist();
  startHunger();
  dungeonScreen();
}

function resumeRun(saved) {
  const dungeon = generateDungeon(saved.seed, manifest);
  run = {
    seed: saved.seed,
    dungeon,
    floorIndex: saved.floorIndex,
    roomId: saved.roomId,
    player: saved.player,
    solved: saved.solved,
    openedChests: saved.openedChests,
    defeatedBosses: saved.defeatedBosses,
    seenLore: saved.seenLore,
    lastCause: null,
  };
  startHunger();
  dungeonScreen();
}

function persist() {
  if (run) saveRun(run);
}

// ---- Hunger -------------------------------------------------------------

function startHunger() {
  stopHunger();
  // 1 HP per real minute, only while the page is open (closing pauses it).
  hungerTimer = setInterval(() => {
    damage(1, "hunger");
  }, 60000);
}
function stopHunger() {
  if (hungerTimer) clearInterval(hungerTimer);
  hungerTimer = null;
}

// ---- Player helpers -----------------------------------------------------

function damage(n, cause) {
  run.player.hp -= n;
  run.lastCause = cause;
  if (run.player.hp <= 0) {
    run.player.hp = 0;
    persist();
    gameOver(cause);
    return true;
  }
  persist();
  if (currentScreen === "dungeon") refreshHud();
  return false;
}
function heal(n) {
  run.player.hp = Math.min(run.player.maxHp, run.player.hp + n);
}
function grow(n) {
  run.player.maxHp += n;
  run.player.hp += n;
}

const curFloor = () => run.dungeon.floors[run.floorIndex];
const curRoom = () => curFloor().byId.get(run.roomId);
const isCleared = (room) =>
  room.type === "chest" || room.type === "surface" || run.solved.has(rk(run.floorIndex, room.id));

// ---- Dungeon screen -----------------------------------------------------

let currentScreen = "title";

function dungeonScreen() {
  currentScreen = "dungeon";
  app.innerHTML = "";
  const wrap = el("div", "screen dungeon");
  wrap.appendChild(buildHud());
  const main = el("div", "dungeon-main");
  main.appendChild(buildMap());
  main.appendChild(buildRoomPanel());
  wrap.appendChild(main);
  app.appendChild(wrap);
}

function buildHud() {
  const hud = el("div", "hud");
  const p = run.player;
  const stats = el("div", "hud-stats");
  const hpPct = Math.max(0, Math.min(100, (p.hp / p.maxHp) * 100));
  const hpWrap = el("div", "hud-hp");
  hpWrap.appendChild(el("span", "hud-label", "❤"));
  const bar = el("div", "hp-bar");
  const fill = el("div", "hp-fill");
  fill.style.width = hpPct + "%";
  if (hpPct <= 30) fill.classList.add("low");
  bar.appendChild(fill);
  hpWrap.appendChild(bar);
  hpWrap.appendChild(el("span", "hud-num", `${p.hp}/${p.maxHp}`));
  stats.appendChild(hpWrap);

  const chips = el("div", "hud-chips");
  const foodChip = el("button", "chip chip-food", `🍖 ${p.food}`);
  foodChip.title = "Eat food (+5 HP)";
  foodChip.disabled = p.food <= 0 || p.hp >= p.maxHp;
  foodChip.addEventListener("click", () => {
    if (p.food > 0 && p.hp < p.maxHp) {
      p.food--;
      heal(5);
      persist();
      refreshHud();
    }
  });
  chips.appendChild(foodChip);
  chips.appendChild(el("span", "chip", `🗝 ${p.keys.size}`));
  chips.appendChild(el("span", "chip" + (p.ring ? " chip-ring" : ""), p.ring ? "💍 ring" : "💍 —"));
  stats.appendChild(chips);
  hud.appendChild(stats);

  const right = el("div", "hud-right");
  right.appendChild(el("div", "hud-floor", FLOORS[run.floorIndex].label));
  right.appendChild(el("div", "hud-seed", "seed: " + run.seed));
  const menuBtn = el("button", "chip", "☰");
  menuBtn.title = "Abandon run";
  menuBtn.addEventListener("click", () => {
    if (confirm("Abandon this descent and return to the title? Your run is saved and can be continued.")) {
      titleScreen();
    }
  });
  right.appendChild(menuBtn);
  hud.appendChild(right);
  return hud;
}

function refreshHud() {
  const old = document.querySelector(".hud");
  if (old) old.replaceWith(buildHud());
}

function buildMap() {
  const floor = curFloor();
  const rooms = floor.rooms;
  const xs = rooms.map((r) => r.x);
  const ys = rooms.map((r) => r.y);
  const minX = Math.min(...xs), maxX = Math.max(...xs);
  const minY = Math.min(...ys), maxY = Math.max(...ys);
  const grid = el("div", "map");
  grid.style.gridTemplateColumns = `repeat(${maxX - minX + 1}, var(--tile))`;
  grid.style.gridTemplateRows = `repeat(${maxY - minY + 1}, var(--tile))`;

  // Rooms one door-step from where you stand: clicking any of them walks there.
  const here = curRoom();
  const reachable = new Set(
    ["N", "S", "E", "W"].map((d) => here.doors[d]).filter((id) => id != null)
  );

  for (const r of rooms) {
    const cell = el("div", "tile");
    cell.style.gridColumn = r.x - minX + 1;
    cell.style.gridRow = r.y - minY + 1;
    for (const dir of ["N", "S", "E", "W"]) {
      if (r.doors[dir]) cell.classList.add("door-" + dir.toLowerCase());
    }
    let glyph = "·";
    if (r.type === "surface") glyph = "🏔";
    else if (r.type === "chest") glyph = run.openedChests.has(rk(floor.index, r.id)) ? "📦" : "🎁";
    else if (!isCleared(r)) glyph = r.isBoss ? "🐉" : "🦁";
    else glyph = "•";
    cell.appendChild(el("span", "tile-glyph", glyph));
    if (r.isExitDown && floor.index !== 0) cell.appendChild(el("span", "tile-stair down", "⬇"));
    if (r.isEntryUp && floor.index !== 0) cell.appendChild(el("span", "tile-stair up", "⬆"));
    if (r.id === run.roomId) {
      cell.classList.add("here");
    } else if (reachable.has(r.id)) {
      cell.classList.add("reachable");
      cell.title = "Walk here";
      cell.addEventListener("click", () => moveTo(r.id));
    }
    grid.appendChild(cell);
  }
  const frame = el("div", "map-frame");
  frame.appendChild(grid);
  return frame;
}

function buildRoomPanel() {
  const panel = el("div", "room-panel");
  const floor = curFloor();
  const room = curRoom();

  // Movement compass.
  const compass = el("div", "compass");
  for (const [dir, label] of [["N", "North"], ["W", "West"], ["E", "East"], ["S", "South"]]) {
    const nb = room.doors[dir];
    const b = el("button", "btn compass-" + dir.toLowerCase(), label);
    if (nb == null) b.disabled = true;
    else b.addEventListener("click", () => moveTo(nb));
    compass.appendChild(b);
  }
  panel.appendChild(compass);

  const info = el("div", "room-info");
  info.appendChild(el("h2", "room-title", roomTitle(room)));
  info.appendChild(el("p", "room-desc", roomDesc(room, floor)));

  const actions = el("div", "room-actions");

  // Encounter.
  if ((room.type === "monster") && !isCleared(room)) {
    const label = room.isBoss && floor.final ? "😈 Face the demon" : room.isBoss ? "🐉 Face the dragon" : "🦁 Face the sphinx";
    const b = el("button", "btn btn-primary", label);
    b.addEventListener("click", () => enterCombat(room));
    actions.appendChild(b);
  }

  // Chest.
  if (room.type === "chest" && !run.openedChests.has(rk(floor.index, room.id))) {
    const b = el("button", "btn btn-primary", "🎁 Open the chest");
    b.addEventListener("click", () => openChest(room));
    actions.appendChild(b);
  }

  // Stairs.
  if (room.isExitDown) {
    if (floor.index === 0) {
      const b = el("button", "btn btn-primary", "⬇ Descend into the dark");
      b.addEventListener("click", () => descend());
      actions.appendChild(b);
    } else {
      const gated = room.isBoss && !isCleared(room);
      const b = el("button", "btn btn-primary", "⬇ Take the hatch down");
      if (gated) {
        b.disabled = true;
        b.textContent = "⬇ Hatch locked — defeat the dragon";
      } else if (!isCleared(room)) {
        b.disabled = true;
        b.textContent = "⬇ Clear this room to descend";
      } else {
        b.addEventListener("click", () => descend());
      }
      actions.appendChild(b);
    }
  }
  if (room.isEntryUp) {
    const b = el("button", "btn", floor.index === 0 ? "☀ Ascend to the world" : "⬆ Climb back up");
    b.addEventListener("click", () => ascend());
    actions.appendChild(b);
  }

  panel.appendChild(info);
  panel.appendChild(actions);
  return panel;
}

function roomTitle(room) {
  if (room.type === "surface") return "The Mouth of the Dungeon";
  if (room.type === "chest") return "Treasure Room";
  if (room.isBoss && curFloor().final) return isCleared(room) ? "The Demon's Ashes" : "The Demon's Throne";
  if (room.isBoss) return isCleared(room) ? "The Dragon's Ashes" : "The Dragon's Lair";
  return isCleared(room) ? "A Silent Room" : "A Sphinx's Chamber";
}

function roomDesc(room, floor) {
  if (room.type === "surface") return "Cold air rises from the stair below. Somewhere above, Miriam waits by the fire.";
  if (room.type === "chest") {
    return run.openedChests.has(rk(floor.index, room.id)) ? "The chest lies open and empty." : "A heavy chest rests against the wall, its lid still shut.";
  }
  if (room.isBoss && floor.final) {
    return isCleared(room)
      ? "Where the demon stood there is only quiet now, and the ring warm in your hand."
      : "The demon waits at the end of everything, the wedding ring glinting between its claws.";
  }
  if (room.isBoss) {
    return isCleared(room) ? "Only scorch marks and settling dust remain where the dragon coiled." : "A dragon guards the hatch, its breath heavy with every doubt you have ever swallowed.";
  }
  return isCleared(room) ? "The sphinx is gone; its riddle answered. The room is quiet." : "A sphinx blocks the way, a half-finished proof glowing in the air before it.";
}

// ---- Movement / floors --------------------------------------------------

function moveTo(roomId) {
  run.roomId = roomId;
  persist();
  // Wandering the maze, an unresolved insecurity may seize you on its own: a
  // sphinx has a 1-in-3 chance of engaging as you step into its room, so you
  // cannot simply slip past them to the stairs. (Dragons are faced by choice.)
  const room = curRoom();
  if (room.type === "monster" && !room.isBoss && !isCleared(room) && Math.random() < 0.33) {
    enterCombat(room);
    return;
  }
  dungeonScreen();
}

// Enter on the dungeon map: interact with whatever the room holds, using the
// same precedence as the on-screen action buttons in buildRoomPanel.
function interact() {
  const floor = curFloor();
  const room = curRoom();
  // A mob (uncleared sphinx/dragon/demon) must be faced first — a boss room's
  // hatch stays locked until it is cleared.
  if (room.type === "monster" && !isCleared(room)) { enterCombat(room); return; }
  // An unopened chest.
  if (room.type === "chest" && !run.openedChests.has(rk(floor.index, room.id))) {
    openChest(room);
    return;
  }
  // Stairs: prefer descending, else climb back up. descend()/ascend() self-guard.
  if (room.isExitDown && (floor.index === 0 || isCleared(room))) { descend(); return; }
  if (room.isEntryUp) { ascend(); return; }
  // Otherwise: nothing to interact with.
}

function descend() {
  const floor = curFloor();
  const room = curRoom();
  if (!room.isExitDown) return;
  if (floor.index !== 0 && !isCleared(room)) return;
  if (run.floorIndex >= run.dungeon.floors.length - 1) return; // no floor below the last
  run.floorIndex += 1;
  run.roomId = run.dungeon.floors[run.floorIndex].entryId;
  persist();
  dungeonScreen();
}

function ascend() {
  const floor = curFloor();
  if (floor.index === 0) {
    // Leaving the dungeon. Miriam's condition: do not come back without the ring.
    if (!run.player.ring) {
      guardScreen();
      return;
    }
    winScreen();
    return;
  }
  run.floorIndex -= 1;
  run.roomId = run.dungeon.floors[run.floorIndex].exitId;
  persist();
  dungeonScreen();
}

// ---- Combat -------------------------------------------------------------

async function enterCombat(room) {
  currentScreen = "combat";
  const floor = curFloor();
  app.innerHTML = "";
  const wrap = el("div", "screen combat-screen");
  wrap.appendChild(buildHud());
  const host = el("div", "combat-host");
  wrap.appendChild(host);
  host.appendChild(el("div", "boot", "The sphinx clears its throat…"));
  app.appendChild(wrap);

  let challenge;
  try {
    challenge = await loadChallenge(room.challengeId);
  } catch (_e) {
    host.textContent = "This challenge failed to load. Retreating.";
    setTimeout(() => dungeonScreen(), 900);
    return;
  }

  const canFlee = FLEE_FAIL[floor.index] < 1;
  startCombat(host, {
    wasm,
    mountAlgaeEditor,
    challenge,
    canFlee,
    fleeFail: FLEE_FAIL[floor.index] ?? 0,
    onWin: (meta) => defeatMonster(room, meta),
    onFlee: (escaped) => {
      if (escaped) {
        dungeonScreen();
      } else {
        // The monster catches you as you turn — a bite, then the fight resumes.
        const bite = room.isBoss ? 4 : 2;
        if (!damage(bite, "monster")) enterCombat(room);
      }
    },
  });
}

function defeatMonster(room, meta) {
  const floor = curFloor();
  run.solved.add(rk(floor.index, room.id));
  if (room.isBoss) {
    grow(5);
    run.defeatedBosses.add(floor.index);
    run.player.keys.add(floor.index);
    if (floor.final) run.player.ring = true;
  } else {
    grow(2);
  }
  if (meta.food) run.player.food += 1;
  persist();

  // Reveal any lore this monster was hiding, then return to the room.
  const loreKey = "c:" + room.challengeId;
  if (meta.lore && !run.seenLore.has(loreKey)) {
    run.seenLore.add(loreKey);
    persist();
    loreScreen(meta.lore, () => afterDefeat(room, floor));
  } else {
    afterDefeat(room, floor);
  }
}

function afterDefeat(room, floor) {
  if (floor.final && room.isBoss) {
    ringScreen();
    return;
  }
  dungeonScreen();
}

// ---- Chests -------------------------------------------------------------

function openChest(room) {
  const floor = curFloor();
  run.openedChests.add(rk(floor.index, room.id));
  let msg;
  if (room.chestKind === "food") {
    run.player.food += 2;
    msg = "🍖 Two rations of food. Something to hold back the hunger.";
  } else if (room.chestKind === "maxhp") {
    grow(5);
    msg = "🧪 A Potion of Vigor. Your maximum health swells (+5 max HP, +5 HP).";
  } else {
    run.player.food += 1;
    msg = "📜 A letter, and a crust of bread beside it (+1 food).";
  }
  persist();
  currentScreen = "chest";
  app.innerHTML = "";
  const wrap = el("div", "screen message-screen");
  wrap.appendChild(el("div", "big-emoji", "🎁"));
  wrap.appendChild(el("h1", null, "The chest creaks open"));
  wrap.appendChild(el("p", "message-body", msg));
  if (room.letter) {
    const note = el("blockquote", "letter", room.letter);
    wrap.appendChild(note);
  }
  const b = el("button", "btn btn-primary", "Take it and go on");
  b.addEventListener("click", () => dungeonScreen());
  wrap.appendChild(b);
  app.appendChild(wrap);
}

// ---- Lore / message screens --------------------------------------------

function loreScreen(text, next) {
  currentScreen = "lore";
  app.innerHTML = "";
  const wrap = el("div", "screen message-screen");
  wrap.appendChild(el("div", "big-emoji", "📜"));
  wrap.appendChild(el("h1", null, "As the dust settles, it whispers…"));
  wrap.appendChild(el("blockquote", "letter", text));
  const b = el("button", "btn btn-primary", "Go on");
  b.addEventListener("click", next);
  wrap.appendChild(b);
  app.appendChild(wrap);
}

function guardScreen() {
  currentScreen = "guard";
  app.innerHTML = "";
  const wrap = el("div", "screen message-screen");
  wrap.appendChild(el("div", "big-emoji", "🚪"));
  wrap.appendChild(el("h1", null, "You stop at the threshold"));
  wrap.appendChild(el("p", "message-body", "Not without it. I can't stand before her empty-handed — not while I still fear I am not enough for her."));
  const b = el("button", "btn btn-primary", "Turn back into the dark");
  b.addEventListener("click", () => dungeonScreen());
  wrap.appendChild(b);
  app.appendChild(wrap);
}

function ringScreen() {
  currentScreen = "ring";
  app.innerHTML = "";
  const wrap = el("div", "screen message-screen ring-screen");
  wrap.appendChild(el("div", "big-emoji", "💍"));
  wrap.appendChild(el("h1", null, "The demon falls"));
  wrap.appendChild(el("p", "message-body", "The last of its fire gutters out, and there in the ash lies the wedding ring, unburnt and bright. You close your fist around it. Far above, at the top of the long stair, the first grey of dawn is waiting."));
  wrap.appendChild(el("p", "message-body dim", "Climb back to the surface with the ring before sunrise."));
  const b = el("button", "btn btn-primary", "Begin the long climb");
  b.addEventListener("click", () => dungeonScreen());
  wrap.appendChild(b);
  app.appendChild(wrap);
}

// ---- Endings ------------------------------------------------------------

function gameOver(cause) {
  stopHunger();
  currentScreen = "over";
  clearRun();
  app.innerHTML = "";
  const wrap = el("div", "screen message-screen over-screen");
  wrap.appendChild(el("div", "big-emoji", cause === "hunger" ? "🍂" : "💀"));
  wrap.appendChild(el("h1", null, cause === "hunger" ? "You starved in the dark" : "You fell in the dark"));
  wrap.appendChild(el("p", "message-body",
    cause === "hunger"
      ? "Nothing in the dark could feed what you were truly hungry for. Far above, a candle burns beside the bed, and Miriam sleeps on, trusting the morning."
      : "The fear grew heavier than you could carry, and the dark closed over you. Far above, in the last hour before dawn, Miriam sleeps on, certain you will be beside her when she wakes."));
  const b = el("button", "btn btn-primary", "Descend again");
  b.addEventListener("click", () => titleScreen());
  wrap.appendChild(b);
  app.appendChild(wrap);
}

function winScreen() {
  stopHunger();
  currentScreen = "win";
  clearRun();
  app.innerHTML = "";
  const wrap = el("div", "screen message-screen win-screen");
  wrap.appendChild(el("div", "big-emoji", "🌅"));
  wrap.appendChild(el("h1", null, "You wake"));
  wrap.appendChild(el("p", "message-body", "Dawn — the wedding morning. Miriam sleeps beside you, calm as still water, her black hair spilled across the pillow. There was never any dungeon: only a nightmare, spun from the last doubt of a man about to be married. On the nightstand the drawer hangs open, and the ring is inside, exactly where it always was, catching the first light."));
  wrap.appendChild(el("p", "message-body", "But you are not the man who fell asleep. Down in the dark you met the thing that kept whispering you were not enough for her — and you answered it, step by patient step, until it had nothing left to say. The doubt is gone, and it is not coming back."));
  wrap.appendChild(el("p", "message-body dim", "You would go down a thousand dungeons for this woman and climb back up smiling. You take the ring, and you wait for her to wake — certain, for the first time and for good, that you are exactly the man she chose. Whatever the morning asks of you, the answer is already yes."));
  const b = el("button", "btn btn-primary", "Wake again");
  b.addEventListener("click", () => titleScreen());
  wrap.appendChild(b);
  app.appendChild(wrap);
}

// ---- Keyboard ------------------------------------------------------------

// WASD / arrow keys walk through a door; Enter dismisses a message screen, or
// interacts with the current room when the dungeon map is up.
const MOVE_KEYS = {
  w: "N", arrowup: "N",
  a: "W", arrowleft: "W",
  s: "S", arrowdown: "S",
  d: "E", arrowright: "E",
};

document.addEventListener("keydown", (e) => {
  if (e.ctrlKey || e.metaKey || e.altKey) return; // leave chords (e.g. Ctrl-Enter) alone
  const t = e.target;
  const typing =
    t && (t.tagName === "INPUT" || t.tagName === "TEXTAREA" || t.isContentEditable ||
      (t.closest && t.closest(".cm-editor")));

  // Enter dismisses whatever message screen is up — the lore a sphinx leaves
  // behind, a chest, the guard, the ring, or an ending.
  if (e.key === "Enter" && !typing) {
    const btn = document.querySelector(".message-screen .btn-primary");
    if (btn) {
      e.preventDefault();
      btn.click();
      return;
    }
    // No message up: on the dungeon map, Enter interacts with the room.
    if (currentScreen === "dungeon" && !document.querySelector(".help-overlay")) {
      e.preventDefault();
      interact();
      return;
    }
  }

  // Movement only on the dungeon map, and not while the help manual is open.
  if (currentScreen !== "dungeon" || typing) return;
  if (document.querySelector(".help-overlay")) return;
  const dir = MOVE_KEYS[e.key.toLowerCase()];
  if (!dir) return;
  const nb = curRoom().doors[dir];
  if (nb != null) {
    e.preventDefault();
    moveTo(nb);
  }
});

// Pause hunger persistence cleanly if the tab is hidden/closed.
window.addEventListener("beforeunload", () => persist());

// Freeze the hunger clock while the help modal is open, so reading the manual
// never costs the player health. Resume when it closes (mid-run only).
document.addEventListener("help-open", () => stopHunger());
document.addEventListener("help-close", () => {
  if (run && currentScreen !== "over" && currentScreen !== "win") startHunger();
});

boot();
