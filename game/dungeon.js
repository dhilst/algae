// Procedural dungeon generation. Seven floors (0 = surface down to -6 = the
// final boss). Every normal floor is 12 rooms (10 monster, 2 chest) laid out on
// a grid, connected by N/S/E/W doors, with a guaranteed-connected graph. Bosses
// (dragons) appear from floor -3 and guard the hatch down. All of it is a pure
// function of the run seed via the shared RNG, so a seed reproduces a world.

import { makeRng } from "./rng.js";

// Floor index 0..6 → depth label and difficulty tier.
export const FLOORS = [
  { index: 0, label: "Surface", tier: null, surface: true },
  { index: 1, label: "Level -1 · Trivial", tier: "trivial" },
  { index: 2, label: "Level -2 · Easy", tier: "easy" },
  { index: 3, label: "Level -3 · Medium", tier: "medium", boss: true },
  { index: 4, label: "Level -4 · Hard", tier: "hard", boss: true },
  { index: 5, label: "Level -5 · Very Hard", tier: "veryhard", boss: true },
  { index: 6, label: "Level -6 · The Demon", tier: "boss", boss: true, final: true },
];

// Escape-failure chance per floor index (design table). Level -6 is inescapable.
export const FLEE_FAIL = { 1: 0.1, 2: 0.2, 3: 0.3, 4: 0.4, 5: 0.5, 6: 1.0 };

const DIRS = {
  N: { dx: 0, dy: -1, opp: "S" },
  S: { dx: 0, dy: 1, opp: "N" },
  E: { dx: 1, dy: 0, opp: "W" },
  W: { dx: -1, dy: 0, opp: "E" },
};

const key = (x, y) => `${x},${y}`;

// Place `count` cells on a grid by a random walk from the origin, so the blob is
// always contiguous (a prerequisite for a connected door graph).
function growCells(rng, count) {
  const cells = [{ x: 0, y: 0 }];
  const seen = new Set([key(0, 0)]);
  while (cells.length < count) {
    const from = rng.pick(cells);
    const d = DIRS[rng.pick(["N", "S", "E", "W"])];
    const nx = from.x + d.dx;
    const ny = from.y + d.dy;
    if (!seen.has(key(nx, ny))) {
      seen.add(key(nx, ny));
      cells.push({ x: nx, y: ny });
    }
  }
  return cells;
}

// Connect the cells: a random spanning tree guarantees reachability, then a few
// extra doors are added for loops so floors don't feel like a single corridor.
function carveDoors(rng, rooms, byPos) {
  const inTree = new Set([rooms[0].id]);
  const frontier = [rooms[0]];
  const link = (a, b, dir) => {
    a.doors[dir] = b.id;
    b.doors[DIRS[dir].opp] = a.id;
  };

  // Prim-style spanning tree.
  while (inTree.size < rooms.length) {
    const room = rng.pick(frontier);
    const opts = [];
    for (const dir of ["N", "S", "E", "W"]) {
      const nb = byPos.get(key(room.x + DIRS[dir].dx, room.y + DIRS[dir].dy));
      if (nb && !inTree.has(nb.id)) opts.push({ dir, nb });
    }
    if (opts.length === 0) {
      frontier.splice(frontier.indexOf(room), 1);
      continue;
    }
    const { dir, nb } = rng.pick(opts);
    link(room, nb, dir);
    inTree.add(nb.id);
    frontier.push(nb);
  }

  // Extra loop doors.
  for (const room of rooms) {
    for (const dir of ["E", "S"]) {
      const nb = byPos.get(key(room.x + DIRS[dir].dx, room.y + DIRS[dir].dy));
      if (nb && !room.doors[dir] && rng.chance(0.32)) link(room, nb, dir);
    }
  }
}

// Breadth-first distances from a room, used to place the exit far from entry.
function bfsFar(rooms, byId, startId) {
  const dist = new Map([[startId, 0]]);
  const q = [startId];
  while (q.length) {
    const cur = byId.get(q.shift());
    for (const dir of ["N", "S", "E", "W"]) {
      const nid = cur.doors[dir];
      if (nid && !dist.has(nid)) {
        dist.set(nid, dist.get(cur.id) + 1);
        q.push(nid);
      }
    }
  }
  let far = startId;
  let best = -1;
  for (const [id, d] of dist) if (d > best) ((best = d), (far = id));
  return far;
}

function makeRoom(i, x, y) {
  return {
    id: i,
    x,
    y,
    type: "monster", // monster | chest | surface
    doors: { N: null, S: null, E: null, W: null },
    challengeId: null,
    tier: null,
    isBoss: false,
    isEntryUp: false,
    isExitDown: false,
    chestKind: null, // food | maxhp | letter
    letter: null,
  };
}

function generateFloor(floor, rng, manifest, letters) {
  // The surface is a single room: the mouth of the dungeon.
  if (floor.surface) {
    const r = makeRoom(0, 0, 0);
    r.type = "surface";
    r.isExitDown = true;
    r.isEntryUp = true; // "up" from the surface = the world / win exit
    return { index: floor.index, rooms: [r], byId: new Map([[0, r]]), entryId: 0, exitId: 0 };
  }

  const count = floor.final ? 5 : 12;
  const cells = growCells(rng, count);
  const rooms = cells.map((c, i) => makeRoom(i, c.x, c.y));
  const byPos = new Map(rooms.map((r) => [key(r.x, r.y), r]));
  const byId = new Map(rooms.map((r) => [r.id, r]));
  carveDoors(rng, rooms, byPos);

  const entryId = 0;
  const exitId = bfsFar(rooms, byId, entryId);
  byId.get(entryId).isEntryUp = true;
  const exit = byId.get(exitId);
  // The final floor has no hatch down — after the demon falls you climb back up.
  if (!floor.final) exit.isExitDown = true;

  // Pool of challenge ids, shuffled per floor+seed. The exit / boss room draws
  // the first one; the rest fan out round-robin over the monster rooms. On the
  // final floor the filler rooms use very-hard proofs so only the demon at the
  // exit carries the boss challenge.
  const fillerTier = floor.final ? "veryhard" : floor.tier;
  const pool = rng.shuffle(manifest[fillerTier] || manifest.trivial);
  const others = rooms.filter((r) => r.id !== exitId);

  // Two chests on non-final floors (never the entry/exit rooms).
  if (!floor.final) {
    const chestSpots = rng.shuffle(others.filter((r) => !r.isEntryUp)).slice(0, 2);
    const kinds = rng.shuffle(["food", "maxhp", "letter", "food"]).slice(0, 2);
    chestSpots.forEach((r, i) => {
      r.type = "chest";
      r.chestKind = kinds[i];
      if (kinds[i] === "letter") r.letter = rng.pick(letters);
    });
  }

  // Assign proofs to every monster room (round-robin over the shuffled pool).
  let p = 0;
  const monsters = rooms.filter((r) => r.type === "monster");
  for (const r of monsters) {
    r.challengeId = floor.final && r.id === exitId ? manifest.boss[0] : pool[p % pool.length];
    r.tier = floor.tier;
    if (r.id === exitId && floor.boss) r.isBoss = true;
    p++;
  }
  // Guarantee the boss room carries a proof even if it wasn't a "monster".
  if (floor.boss) {
    exit.isBoss = true;
    exit.tier = floor.tier;
    exit.challengeId = floor.final ? manifest.boss[0] : pool[0];
    if (exit.type !== "chest") exit.type = "monster";
  }

  return { index: floor.index, rooms, byId, entryId, exitId };
}

const LETTERS = [
  "The night you proposed she said yes before you could finish asking. You still lie awake certain you imagined it.",
  "She tucked a note into your coat pocket once: 'Stop worrying. I already chose you.' You have read it a thousand times.",
  "They all said she was too good for you. She only ever said that she was the lucky one.",
  "On bad nights she holds your face and tells you what she sees. You have never quite been able to believe her.",
  "You have loved her far longer than you have believed yourself worthy of her. Tonight, one of those finally ends.",
];

export function generateDungeon(seed, manifest) {
  const floors = FLOORS.map((f) => generateFloor(f, makeRng(seed, "floor" + f.index), manifest, LETTERS));
  return { seed, floors };
}
