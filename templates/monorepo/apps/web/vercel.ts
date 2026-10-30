import { routes } from "@vercel/config/v1"
import type { VercelConfig } from "@vercel/config/v1"

// The Rust API's deployed origin. Vercel proxies /api and /health to it, so
// the browser only ever talks to this deployment: no CORS configuration is
// needed on the API side, matching this project's dev-mode Vite proxy.
// To call it cross-origin from the browser instead, set VITE_API_URL to this
// value as a Vercel project environment variable and drop the two rewrites
// below (see .env.example).
const API_ORIGIN = "https://3-216-222-160.sslip.io"

export const config: VercelConfig = {
  // "Other": a plain static SPA build, not a framework Vercel has a preset for.
  framework: null,
  buildCommand: "bun run build",
  installCommand: "bun install",
  outputDirectory: "dist/client",

  rewrites: [
    routes.rewrite("/api/:path*", `${API_ORIGIN}/api/:path*`),
    routes.rewrite("/health/:path*", `${API_ORIGIN}/health/:path*`),
    // SPA fallback: every other path is rendered client-side by the router.
    // Static files (assets, favicon, ...) are matched by the filesystem
    // first, so this only ever catches app routes.
    routes.rewrite("/(.*)", "/_shell.html"),
  ],

  headers: [
    routes.header("/(.*)", [
      { key: "X-Content-Type-Options", value: "nosniff" },
      { key: "Referrer-Policy", value: "no-referrer" },
      { key: "X-Frame-Options", value: "DENY" },
    ]),
    // Every file under /assets is content-hashed by Vite, so it is safe to
    // cache forever.
    routes.cacheControl("/assets/(.*)", {
      public: true,
      maxAge: "1 year",
      immutable: true,
    }),
  ],
}
