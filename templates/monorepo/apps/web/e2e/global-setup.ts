import { execFileSync } from "node:child_process"
import { fileURLToPath } from "node:url"

/** The API the dev server forwards to (`vite.config.ts`). */
const API = process.env.API_PROXY_TARGET ?? "http://127.0.0.1:8080"

/** The account the smoke flow signs in with: the dev seed's ordinary user. */
const SEED_EMAIL = "user@example.com"

/**
 * Before the smoke flow: the API must answer, or the flow skips with a note
 * (on CI, where the API is always started, it fails instead). Then the
 * account to sign in with: `E2E_EMAIL` and `E2E_PASSWORD` when set, else
 * the dev seed's user, seeded afresh (`cargo run --bin seed` in `apps/api`
 * gives it a new password every run).
 */
export default async function globalSetup() {
  if (!(await answers(`${API}/health/live`))) {
    const note = `no API answers at ${API}: start it (\`cargo run\` in apps/api) to run the smoke flow`
    if (process.env.CI) throw new Error(note)
    console.log(`note: skipping the smoke flow: ${note}`)
    process.env.E2E_SKIP = note
    return
  }
  if (process.env.E2E_PASSWORD) {
    process.env.E2E_EMAIL ??= SEED_EMAIL
    return
  }
  const { email, password } = seededUser()
  process.env.E2E_EMAIL = email
  process.env.E2E_PASSWORD = password
}

async function answers(url: string) {
  try {
    const response = await fetch(url, { signal: AbortSignal.timeout(3_000) })
    return response.ok
  } catch {
    return false
  }
}

/** Runs the dev seed and reads the user's line: `  user   <email>  <password>`. */
function seededUser() {
  const api = fileURLToPath(new URL("../../api", import.meta.url))
  const output = execFileSync("cargo", ["run", "--quiet", "--bin", "seed"], {
    cwd: api,
    encoding: "utf8",
    stdio: ["ignore", "pipe", "inherit"],
  })
  const line = /^\s*user\s+(\S+)\s+(\S+)\s*$/m.exec(output)
  if (!line) {
    throw new Error(`the dev seed printed no user to sign in with:\n${output}`)
  }
  return { email: line[1], password: line[2] }
}
