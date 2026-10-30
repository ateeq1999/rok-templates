# Security

What the templates do for you, what you set, and what stays yours. The
account flows' own rules (argon2id, single-use tokens, lockouts,
two-factor, API keys) are in [Accounts and sign-in](auth.md).

- [On every answer](#on-every-answer)
- [CORS](#cors)
- [Rate limits](#rate-limits)
- [Secrets](#secrets)
- [Dependencies](#dependencies)
- [What stays yours](#what-stays-yours)

## On every answer

`src/app/middleware.rs` applies `src/shared/security/headers.rs` (the
same in both templates) to every answer, errors included:

| Header | Value | Why |
|---|---|---|
| `X-Content-Type-Options` | `nosniff` | a browser never guesses another type than the one sent |
| `X-Frame-Options` | `DENY` | no page frames the API (clickjacking) |
| `Content-Security-Policy` | `frame-ancestors 'none'` | the same, for current browsers |
| `Referrer-Policy` | `no-referrer` | no URL leaks to the next site |
| `Strict-Transport-Security` | `max-age=31536000; includeSubDomains` | only when `APP_URL` is `https://` |

A header a handler sets itself is kept. A handler that panics is answered
`500 internal_error`, the panic logged, instead of the connection being
dropped; the message never reaches the caller.

## CORS

`CORS_ALLOWED_ORIGINS` names the front ends that call the API from
another origin: `https://app.example.com,http://localhost:5173`, or `*`.
Unset, no other origin is allowed. A request from an origin not listed is
not refused; it gets no `Access-Control-Allow-Origin`, and the browser
keeps the answer from the page. The API's credential is a bearer token,
not a cookie, so there is no CSRF to defend against; keep it that way, or
add CSRF protection with the cookie.

## Rate limits

Two per-IP limits, counted in the process:

- `AUTH_RATE_LIMIT` (20 a minute) on `/auth`: sign-in, sign-up, the
  password and email flows, two-factor.
- `GLOBAL_RATE_LIMIT` (off by default) on every route but health, so a
  probe is never refused.

Past one, the answer is `429 too_many_requests` with `Retry-After`. Behind
a load balancer or a proxy you run, set `TRUST_PROXY=true`: the client IP
is then the last `X-Forwarded-For` address, the one your proxy appended.
Leave it off otherwise, since a client sets that header to anything. With
several replicas each counts its own; for one count across them, see the
[Redis recipe](recipes/redis-rate-limit.md).

## Secrets

- **`JWT_SECRET`** signs the session tokens (HS256, at least 32 bytes).
  Rotate it without signing anyone out: set the new value as `JWT_SECRET`
  and the old one as `JWT_SECRET_PREVIOUS`, wait one `JWT_TTL` (every
  token the old one signed has expired), then remove `JWT_SECRET_PREVIOUS`.
  The download links the API signs itself (for a store that can't sign
  its own) use a key derived from it, so those handed out before the
  switch stop working; they last minutes.
- **`TOTP_ENCRYPTION_KEY`** seals the two-factor secrets at rest (32 bytes,
  base64). It can't be rotated by setting a new one: the stored secrets
  are sealed with the old key. Keep it as safe as the database backups.
- **Where they live**: in production, in a secret store injected as
  environment variables (`deploy/terraform/`'s Secrets Manager); never in
  the image, a task definition or the repository. `Config` has no `Debug`,
  and its start-up summary prints a secret's length only.

## Dependencies

`deny.toml` is `cargo deny check`'s policy, and CI runs it: no crate with
a known advisory, licenses from an allow list, crates from crates.io only.
The advisories it ignores are listed there, each with why it can't reach
the project (HS256-only tokens, a build-time `zip`, the configured S3
store, actix's TLS-only HTTP/2). When `cargo update` clears one, drop its
entry; review the list on every dependency update. The image runs on
distroless, as a non-root user, with the one binary in it.

## What stays yours

- **TLS**: terminate it at the load balancer (`deploy/terraform`'s
  `certificate_arn`); the API serves plain HTTP behind it.
- **Authorization** beyond the two roles: who may read and change what is
  each feature's rule, in its `service.rs`.
- **Input limits** beyond the 2 MiB body limit and the upload limits: page
  sizes are capped; anything else that is expensive per request (a search
  over everything, an export) needs its own limit.
- **Audit**: admins' actions on accounts and jobs are logged
  (`audit_log`); log your own sensitive actions through the same port.
