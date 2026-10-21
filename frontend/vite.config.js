import { fileURLToPath } from "node:url";
import path from "node:path";
import { defineConfig } from "vite";
import { sveltekit } from "@sveltejs/kit/vite";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

export default defineConfig({
  plugins: [sveltekit()],
  // Keep Rust errors visible in `tauri dev`.
  clearScreen: false,
  server: {
    // Must match devUrl in app/tauri.conf.json.
    port: 1430,
    strictPort: true,
    watch: {
      // The Rust crates live beside this directory; don't reload on them.
      ignored: [`${root}/app/**`, `${root}/crates/**`, `${root}/target/**`],
    },
  },
});
