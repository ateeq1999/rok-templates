import { defineConfig } from "vitest/config"

export default defineConfig({
  // `dedupe`: the generated client (`../../clients/ts`, outside this App)
  // imports TanStack Query, which resolves to this App's copy.
  resolve: { tsconfigPaths: true, dedupe: ["@tanstack/react-query"] },
  // Vite loads .env for every mode by default, so a developer's local
  // VITE_API_URL (set for `bun dev`/`bun build`) would otherwise leak into
  // the test run and make apiUrl()/http() assertions machine-dependent.
  // Pointing envDir somewhere with no .env files keeps tests deterministic.
  envDir: "src",
  test: { environment: "jsdom", include: ["src/**/*.test.{ts,tsx}"] },
})
