// Download the deep-whale-day-night-theme v0.1.6 runtime files from the
// official GitHub raw CDN (tag v0.1.6) and reconstruct the package locally.
import { createWriteStream, mkdirSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { createHash } from "node:crypto";
import { Readable } from "node:stream";
import { pipeline } from "node:stream/promises";

const TAG = "v0.1.6";
const BASE = "https://raw.githubusercontent.com/GGBond2424648901/deep-whale-day-night-theme/" + TAG + "/";
const OUT = new URL("../deep-whale-day-night-theme/", import.meta.url).pathname.replace(/^\/([A-Za-z]:)/, "$1");
const FILES = [
  "package.json",
  "cordis.patch.yml",
  "skin.json",
  "LICENSE",
  "NOTICE",
  "lib/index.js",
  "lib/client.js",
];

async function fetchWithRetry(url, tries = 4) {
  let last;
  for (let i = 0; i < tries; i++) {
    try {
      const res = await fetch(url, { headers: { "User-Agent": "dsh-installer" } });
      if (res.ok) return res;
      if (res.status === 404) throw new Error("404 Not Found: " + url);
      last = new Error("HTTP " + res.status + " for " + url);
    } catch (e) {
      last = e;
    }
    const wait = 3000 * (i + 1);
    console.log("[retry]", url, "in", wait, "ms —", last.message);
    await new Promise((r) => setTimeout(r, wait));
  }
  throw last;
}

for (const rel of FILES) {
  const url = BASE + rel;
  console.log("fetching", rel, "...");
  const res = await fetchWithRetry(url);
  const dest = join(OUT, rel);
  mkdirSync(dirname(dest), { recursive: true });
  const hash = createHash("sha256");
  const counter = { n: 0 };
  await pipeline(
    Readable.fromWeb(res.body),
    async function* (src) {
      for await (const chunk of src) {
        counter.n += chunk.length;
        if (counter.n % (2 << 20) < chunk.length) console.log("  ", rel, (counter.n / 1048576).toFixed(1), "MB");
        hash.update(chunk);
        yield chunk;
      }
    },
    createWriteStream(dest)
  );
  console.log("saved", rel, "—", counter.n, "bytes, sha256", hash.digest("hex"));
}
console.log("DONE");
