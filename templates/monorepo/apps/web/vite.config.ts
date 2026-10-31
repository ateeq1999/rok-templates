import { defineConfig } from "vite"
import { devtools } from "@tanstack/devtools-vite"
import { tanstackStart } from "@tanstack/react-start/plugin/vite"
import viteReact from "@vitejs/plugin-react"
import tailwindcss from "@tailwindcss/vite"

const API_TARGET = process.env.API_PROXY_TARGET ?? "http://127.0.0.1:8080"

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
    tanstackStart({ spa: { enabled: true } }),
    viteReact(),
  ],
})

export default config
