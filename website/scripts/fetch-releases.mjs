// Pulls release data from the GitHub API at build time, avoiding a client-side fetch.
import { writeFileSync, mkdirSync } from "node:fs";
import { fileURLToPath } from "node:url";
import path from "node:path";

const REPO = "xacnio/radiocove";
const OUT_DIR = path.join(path.dirname(fileURLToPath(import.meta.url)), "..", "src", "data");
const OUT_FILE = path.join(OUT_DIR, "releases.json");

// release.yml's body has a download table + commit list + CI footer; keep just the commit list.
function extractChangelog(body) {
  const match = body.match(/What's New\s*\n([\s\S]*?)(\n---|\n\*Automatically generated|$)/);
  if (!match) return body.trim();
  return match[1].trim();
}

function classifyAsset(name) {
  const lower = name.toLowerCase();
  const arch = /arm64|aarch64/.test(lower) ? "arm64" : "x64";
  if (lower.endsWith(".exe") || lower.endsWith(".msi")) return { platform: "windows", arch };
  if (lower.endsWith(".dmg")) return { platform: "macos", arch: "universal" };
  if (lower.endsWith(".deb") || lower.endsWith(".rpm") || lower.endsWith(".appimage")) {
    return { platform: "linux", arch };
  }
  return null;
}

async function main() {
  let releases = [];
  try {
    const res = await fetch(`https://api.github.com/repos/${REPO}/releases`, {
      headers: { Accept: "application/vnd.github+json" },
    });
    if (!res.ok) throw new Error(`GitHub API responded ${res.status}`);
    const raw = await res.json();

    releases = raw
      .filter((r) => !r.draft)
      .map((r) => {
        const downloads = [];
        for (const asset of r.assets || []) {
          const kind = classifyAsset(asset.name);
          if (!kind) continue;
          downloads.push({
            ...kind,
            name: asset.name,
            url: asset.browser_download_url,
            size: asset.size,
          });
        }
        return {
          tag: r.tag_name,
          name: r.name || r.tag_name,
          publishedAt: r.published_at,
          changelog: extractChangelog(r.body || ""),
          url: r.html_url,
          prerelease: r.prerelease,
          downloads,
        };
      });
  } catch (err) {
    console.warn(`[fetch-releases] could not reach GitHub API (${err.message}); writing empty dataset.`);
  }

  mkdirSync(OUT_DIR, { recursive: true });
  writeFileSync(OUT_FILE, JSON.stringify({ releases, fetchedAt: new Date().toISOString() }, null, 2));
  console.log(`[fetch-releases] wrote ${releases.length} release(s) to ${path.relative(process.cwd(), OUT_FILE)}`);
}

main();
