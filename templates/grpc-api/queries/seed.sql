-- The development seed's SQL (`app/seed.rs`, run by `cargo run --bin
-- seed`). `rok sqlc g` generates `src/db/queries/seed.rs`; it is named
-- after no feature, so no feature calls it.

-- name: ResetSeedAccount :maybe_one
-- The live account with `email`, whatever its case, made verified and
-- active with the password hash `$2` and `role`, its failed sign-ins and
-- lock cleared and its sessions ended; none when there is no such account.
-- param: $1 email String
UPDATE users
SET password_hash = $2,
    role = $3,
    email_verified = true,
    is_active = true,
    failed_logins = 0,
    locked_until = NULL,
    token_version = token_version + 1
WHERE lower(email) = lower($1) AND deleted_at IS NULL
RETURNING id, email, role;

-- name: CreateSeedAccount :one
-- A new verified account with `email`, the password hash `$3` and `role`.
INSERT INTO users (id, email, password_hash, role, email_verified)
VALUES ($1, $2, $3, $4, true)
RETURNING id, email, role;

-- name: PromoteToAdmin :maybe_one
-- Makes the live account with `email`, whatever its case, an admin; none
-- when there is no such account.
-- param: $1 email String
UPDATE users
SET role = 'admin'
WHERE lower(email) = lower($1) AND deleted_at IS NULL
RETURNING id, email, role;
