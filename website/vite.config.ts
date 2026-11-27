import { fileURLToPath } from "node:url";
import { defineConfig } from "vite";
import solid from "@solidjs/vite-plugin";

export default defineConfig({
  plugins: [solid()],
  build: {
    target: "es2022",
    rollupOptions: {
      input: Object.fromEntries(["index", "docs", "examples", "404"].map((page) => [
        page, fileURLToPath(new URL(`./${page}.html`, import.meta.url)),
      ])),
    },
  },
});
