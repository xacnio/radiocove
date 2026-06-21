# Radiocove website

Marketing/docs site for Radiocove. Vite + React, same accent color and font
family as the desktop app (`src/index.css`). Deployed to GitHub Pages from
`.github/workflows/deploy-docs.yml`.

```bash
npm install
npm run dev     # fetches latest release data, then starts the dev server
npm run build   # fetches latest release data, then builds to dist/
```

Two scripts run before every dev/build (via the `fetch-data` npm script):

- `scripts/fetch-releases.mjs` pulls release + asset data from the GitHub API
  into `src/data/releases.json`, so download links and the changelog stay in
  sync with whatever is actually published — no client-side fetch, no manual
  edits needed when a new version ships.
- `scripts/build-legal.mjs` renders the repo's `PRIVACY.md` and `LICENSE`
  (one level up) to HTML into `src/data/legal.json`, served as `privacy.html`
  and `license.html`. The website never holds a second copy of that text —
  edit those files at the repo root and the site picks it up on next build.
  The `deploy-docs.yml` workflow also redeploys on changes to either file,
  not just to `website/**`. There is no Terms of Service page since the repo
  doesn't have a `TERMS.md`.

## One-time setup

In the GitHub repo: **Settings → Pages → Source → GitHub Actions**. After
that, every push to `master` touching `website/**`, every published release,
and manual `workflow_dispatch` runs redeploy the site.

If the repo isn't `xacnio/radiocove` or isn't served at the project-pages path
(`https://<user>.github.io/<repo>/`), update `base` in `vite.config.js` and
`REPO` in `scripts/fetch-releases.mjs` accordingly. For a custom domain, add a
`public/CNAME` file and set `base: "/"`.

## Screenshots

`public/screenshots/*.png` are real app captures pulled from
`docs/screenshots`, referenced by `src/components/Screenshot.jsx` from the
Hero, Screenshots and Features sections. If one of those files is ever
missing, the component falls back to a dashed placeholder with capture
instructions instead of a broken image.
