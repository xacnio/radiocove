// Renders PRIVACY.md to HTML at build time so the site never holds a second copy of the text.
import { readFileSync, writeFileSync, mkdirSync } from "node:fs";
import { fileURLToPath } from "node:url";
import path from "node:path";
import { marked } from "marked";

const SCRIPT_DIR = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.join(SCRIPT_DIR, "..", "..");
const OUT_DIR = path.join(SCRIPT_DIR, "..", "src", "data");
const OUT_FILE = path.join(OUT_DIR, "legal.json");

function buildDoc(filename) {
  const raw = readFileSync(path.join(REPO_ROOT, filename), "utf-8");
  const updatedMatch = raw.match(/\*Last [Uu]pdated:\s*(.+?)\*/);
  const body = raw
    .replace(/^#\s.+\n/, "") // drop the H1 — LegalPage renders its own <h1>
    .replace(/\n---\s*\n\*Last [Uu]pdated:.*\*\s*$/, ""); // drop the trailing date line
  return { html: marked.parse(body.trim()), updated: updatedMatch?.[1] ?? null };
}

// LICENSE is plain text, not markdown — rendered verbatim in a <pre>.
function buildPlainText(filename) {
  const raw = readFileSync(path.join(REPO_ROOT, filename), "utf-8").trim();
  return { text: raw, updated: null };
}

const docs = {
  privacy: buildDoc("PRIVACY.md"),
  license: buildPlainText("LICENSE"),
};

mkdirSync(OUT_DIR, { recursive: true });
writeFileSync(OUT_FILE, JSON.stringify(docs, null, 2));
console.log(`[build-legal] wrote privacy + license HTML to ${path.relative(process.cwd(), OUT_FILE)}`);
