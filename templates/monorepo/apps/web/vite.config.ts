import { defineConfig } from "vite"
import { devtools } from "@tanstack/devtools-vite"
import { tanstackStart } from "@tanstack/react-start/plugin/vite"
import viteReact from "@vitejs/plugin-react"
import tailwindcss from "@tailwindcss/vite"

const API_TARGET = process.env.API_PROXY_TARGET ?? "http://127.0.0.1:8080"

// Pages rendered to HTML at build time, served before the app loads (add a
// marketing page by listing its path, e.g. "/" or "/about"). Every other
// route, the signed-in ones included, renders in the browser from the shell.
const PRERENDERED: string[] = []

// The path the SPA shell (`_shell.html`) is rendered at. No page uses it, so
// a listed page never takes the shell's place on the static host.
const SHELL_PATH = "/_shell"

const shellPage = PRERENDERED.find(
  (path) => path.replace(/\/+$/, "") === SHELL_PATH
)
if (shellPage !== undefined) {
  throw new Error(
    `vite.config.ts: PRERENDERED lists "${shellPage}", the path the SPA shell is rendered at (SHELL_PATH); prerendering it would replace _shell.html and break every other route. Remove it from PRERENDERED or change SHELL_PATH.`
  )
}

// SPA mode: a static shell is emitted and every route renders in the browser,
// so the app needs no Node server (the API is a separate service).
const config = defineConfig({
  // `dedupe`: the generated client (`../../clients/ts`, outside this App)
  // imports TanStack Query, which resolves to this App's copy.
  resolve: { tsconfigPaths: true, dedupe: ["@tanstack/react-query"] },
  server: {
    proxy: {
      "/api": { target: API_TARGET, changeOrigin: true },
      "/health": { target: API_TARGET, changeOrigin: true },
    },
  },
  plugins: [
    devtools(),
    tailwindcss(),
    tanstackStart({
      spa: { enabled: true, maskPath: SHELL_PATH },
      // Only the listed pages: no discovered static routes, and no links
      // followed from a page, so signed-in routes are never prerendered.
      prerender: {
        enabled: PRERENDERED.length > 0,
        autoStaticPathsDiscovery: false,
        crawlLinks: false,
      },
      pages: PRERENDERED.map((path) => ({ path })),
    }),
    viteReact(),
  ],
})

export default config
