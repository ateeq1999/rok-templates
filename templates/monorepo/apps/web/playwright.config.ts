import { defineConfig, devices } from "@playwright/test"

// `bun run e2e`: the browser smoke flow in `e2e/`, against the API the dev
// server forwards to (`API_PROXY_TARGET`, default http://127.0.0.1:8080).
// It starts `bun run dev` unless one already answers; on CI it starts its
// own. `e2e/global-setup.ts` checks the API answers and signs in with the
// dev seed's account.
export default defineConfig({
  testDir: "e2e",
  globalSetup: "./e2e/global-setup.ts",
  forbidOnly: Boolean(process.env.CI),
  retries: 0,
  reporter: "list",
  timeout: 60_000,
  expect: { timeout: 15_000 },
  use: {
    baseURL: "http://127.0.0.1:3001",
    trace: "retain-on-failure",
  },
  projects: [{ name: "chromium", use: { ...devices["Desktop Chrome"] } }],
  webServer: {
    command: "bun run dev",
    url: "http://127.0.0.1:3001",
    reuseExistingServer: !process.env.CI,
    timeout: 120_000,
  },
})
