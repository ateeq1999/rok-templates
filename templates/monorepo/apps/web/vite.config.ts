import { defineConfig } from "vite"
import { devtools } from "@tanstack/devtools-vite"
import { tanstackStart } from "@tanstack/react-start/plugin/vite"
import viteReact from "@vitejs/plugin-react"
import tailwindcss from "@tailwindcss/vite"

const API_TARGET = process.env.API_PROXY_TARGET ?? "http://localhost:3000"

// SPA mode: a static shell is emitted and every route renders in the browser,
// so the app needs no Node server (the API is a separate service).
const config = defineConfig({
  resolve: { tsconfigPaths: true },
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
