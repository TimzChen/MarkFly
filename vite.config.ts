import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import { resolve } from "node:path";

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

const PRELOAD_ENTRY = "/assets/previewPreload.js";
const DEV_PRELOAD_ENTRY = "/src/previewPreload.ts";

function markflyDeferAppLoad() {
  return {
    name: "markfly-defer-app",
    transformIndexHtml: {
      order: "post" as const,
      handler(html: string, ctx: { server?: unknown }) {
        const entryMatch = html.match(
          /<script type="module" crossorigin src="(\/assets\/(?:index|main)-[^"]+\.js)"><\/script>/
        );
        const devEntry = '/src/main.ts';
        const preloadScript = `<script>window.__MARKFLY_PRELOAD_ENTRY__="${ctx.server ? DEV_PRELOAD_ENTRY : PRELOAD_ENTRY}"</script>`;

        if (ctx.server) {
          return html
            .replace(
              `<script type="module" src="${devEntry}"></script>`,
              `<script>window.__MARKFLY_ENTRY__="${devEntry}"</script>${preloadScript}`
            )
            .replace(
              /<link rel="modulepreload"[^>]+>\s*/g,
              ""
            )
            .replace(
              "</body>",
              '<script src="/markfly-defer.js"></script></body>'
            );
        }

        if (!entryMatch) return html;

        const entry = entryMatch[1];
        return html
          .replace(entryMatch[0], "")
          .replace(
            '<script src="/markfly-boot.js"></script>',
            `<script src="/markfly-boot.js"></script><script>window.__MARKFLY_ENTRY__="${entry}"</script>${preloadScript}`
          )
          .replace(/<link rel="modulepreload"[^>]+>\s*/g, "")
          .replace(
            /<link rel="stylesheet" crossorigin href="\/assets\/bytemd[^"]+">\s*/g,
            ""
          )
          .replace(
            "</body>",
            '<script src="/markfly-defer.js"></script></body>'
          );
      },
    },
  };
}

// https://vite.dev/config/
export default defineConfig(async () => ({
  plugins: [vue(), markflyDeferAppLoad()],

  build: {
    modulePreload: false,
    rollupOptions: {
      input: {
        main: resolve(__dirname, "index.html"),
        previewPreload: resolve(__dirname, "src/previewPreload.ts"),
      },
      output: {
        entryFileNames: (chunkInfo) => {
          if (chunkInfo.name === "previewPreload") {
            return "assets/previewPreload.js";
          }
          return "assets/[name]-[hash].js";
        },
        manualChunks(id) {
          if (id.includes("node_modules/@tauri-apps")) {
            return "tauri";
          }
          if (id.includes("node_modules/vue") || id.includes("node_modules/@vue")) {
            return "vue";
          }
          if (id.includes("node_modules/pinia")) {
            return "pinia";
          }
          if (id.includes("node_modules/bytemd") || id.includes("node_modules/@bytemd")) {
            return "bytemd";
          }
          if (
            id.includes("node_modules/unified") ||
            id.includes("node_modules/remark") ||
            id.includes("node_modules/rehype") ||
            id.includes("node_modules/micromark")
          ) {
            return "markdown";
          }
        },
      },
    },
  },

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent Vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // 3. tell Vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
  },
}));
