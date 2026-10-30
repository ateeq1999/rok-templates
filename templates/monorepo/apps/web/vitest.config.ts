import { defineConfig } from "vitest/config"

export default defineConfig({
  resolve: { tsconfigPaths: true },
  // Vite loads .env for every mode by default, so a developer's local
  // VITE_API_URL (set for `bun dev`/`bun build`) would otherwise leak into
  // the test run and make apiUrl()/http() assertions machine-dependent.
  // Pointing envDir somewhere with no .env files keeps tests deterministic.
  envDir: "src",
  test: { environment: "jsdom", include: ["src/**/*.test.{ts,tsx}"] },
})
