// Deterministic, seedable PRNG. Every world detail — layout, proof placement,
// chest contents, lore — is derived from a single seed string so that the same
// seed always reproduces the same dungeon (players can share seeds).

// FNV-1a style hash of a string into a 32-bit unsigned integer.
export function hashSeed(str) {
  let h = 0x811c9dc5;
  for (let i = 0; i < str.length; i++) {
    h ^= str.charCodeAt(i);
    h = Math.imul(h, 0x01000193);
  }
  return h >>> 0;
}

// mulberry32: tiny, fast, good-enough PRNG. Returns a function yielding floats
// in [0, 1). Given the same seed it always produces the same stream.
export function mulberry32(seed) {
  let a = seed >>> 0;
  return function () {
    a |= 0;
    a = (a + 0x6d2b79f5) | 0;
    let t = Math.imul(a ^ (a >>> 15), 1 | a);
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

// Build a small RNG toolkit from a seed string (optionally salted so different
// aspects of the world draw from independent-looking streams).
export function makeRng(seedStr, salt = "") {
  const rand = mulberry32(hashSeed(seedStr + "::" + salt));
  return {
    next: rand,
    int(n) {
      return Math.floor(rand() * n);
    },
    range(lo, hi) {
      return lo + Math.floor(rand() * (hi - lo + 1));
    },
    pick(arr) {
      return arr[Math.floor(rand() * arr.length)];
    },
    chance(p) {
      return rand() < p;
    },
    shuffle(arr) {
      const a = arr.slice();
      for (let i = a.length - 1; i > 0; i--) {
        const j = Math.floor(rand() * (i + 1));
        [a[i], a[j]] = [a[j], a[i]];
      }
      return a;
    },
  };
}

// A short, pronounceable random seed for the "roll me a world" button.
export function randomSeedString() {
  const syl = ["mir", "iam", "gol", "dra", "keth", "ash", "ven", "lor", "mor", "sun", "vow", "rune", "fel", "nyx"];
  // Uses Math.random deliberately — this is the *choice* of a new seed, not
  // world generation (which stays fully deterministic from the chosen string).
  let s = "";
  for (let i = 0; i < 3; i++) s += syl[Math.floor(Math.random() * syl.length)];
  return s + "-" + Math.floor(Math.random() * 9000 + 1000);
}
