import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";

// GitHub Pages project site: served from https://xacnio.github.io/radiocove/
export default defineConfig({
  base: "/radiocove/",
  plugins: [react(), tailwindcss()],
  build: {
    outDir: "dist",
    rollupOptions: {
      input: {
        main: "index.html",
        tr: "tr/index.html",
        de: "de/index.html",
        privacy: "privacy.html",
        license: "license.html",
      },
    },
  },
});
