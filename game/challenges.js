// Loads proof challenges from ./challenge/ (staged next to the game at build
// time). The manifest lists challenge ids per difficulty tier; individual
// .alg/.json files are fetched lazily and cached the first time a room needs one.

const base = new URL("./challenge/", document.baseURI);
let manifestPromise = null;
const cache = new Map();

export function loadManifest() {
  if (!manifestPromise) {
    manifestPromise = fetch(new URL("manifest.json", base)).then((r) => {
      if (!r.ok) throw new Error("challenge manifest missing");
      return r.json();
    });
  }
  return manifestPromise;
}

// Fetch one challenge: its unfinished proof source (.alg) and metadata (.json).
export async function loadChallenge(id) {
  if (cache.has(id)) return cache.get(id);
  const p = (async () => {
    const [alg, meta] = await Promise.all([
      fetch(new URL(`${id}-room.alg`, base)).then((r) => r.text()),
      fetch(new URL(`${id}-room.json`, base)).then((r) => r.json()),
    ]);
    return { id, src: alg, meta };
  })();
  cache.set(id, p);
  return p;
}
