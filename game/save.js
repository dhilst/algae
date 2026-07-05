// Browser LocalStorage persistence for a run. Only the seed plus the mutable
// progress is stored — the dungeon layout itself is regenerated deterministically
// from the seed on load, so we never need to serialize the whole graph.

const KEY = "dungeon-proof-crawler:save:v1";

// Sets don't survive JSON, so they're stored as arrays and rehydrated on load.
export function saveRun(run) {
  const data = {
    seed: run.seed,
    player: {
      hp: run.player.hp,
      maxHp: run.player.maxHp,
      food: run.player.food,
      keys: [...run.player.keys],
      ring: run.player.ring,
    },
    floorIndex: run.floorIndex,
    roomId: run.roomId,
    solved: [...run.solved],
    openedChests: [...run.openedChests],
    defeatedBosses: [...run.defeatedBosses],
    seenLore: [...run.seenLore],
  };
  try {
    localStorage.setItem(KEY, JSON.stringify(data));
  } catch (_e) {
    /* storage full / disabled — the run simply won't persist */
  }
}

export function loadRun() {
  try {
    const raw = localStorage.getItem(KEY);
    if (!raw) return null;
    const d = JSON.parse(raw);
    return {
      seed: d.seed,
      player: {
        hp: d.player.hp,
        maxHp: d.player.maxHp,
        food: d.player.food,
        keys: new Set(d.player.keys || []),
        ring: !!d.player.ring,
      },
      floorIndex: d.floorIndex,
      roomId: d.roomId,
      solved: new Set(d.solved || []),
      openedChests: new Set(d.openedChests || []),
      defeatedBosses: new Set(d.defeatedBosses || []),
      seenLore: new Set(d.seenLore || []),
    };
  } catch (_e) {
    return null;
  }
}

export function hasSave() {
  try {
    return !!localStorage.getItem(KEY);
  } catch (_e) {
    return false;
  }
}

export function clearRun() {
  try {
    localStorage.removeItem(KEY);
  } catch (_e) {
    /* ignore */
  }
}
