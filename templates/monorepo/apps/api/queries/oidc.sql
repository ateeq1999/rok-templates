-- Sign-in with an OpenID provider's SQL. `rok sqlc g` generates
-- `src/db/queries/oidc.rs`, one function per query, which only the `oidc`
-- feature calls.

-- name: InsertRequest :exec
-- A sign-in at the provider `$2` waiting for its callback: the hash of its
-- state, the PKCE verifier and the nonce the code is redeemed with, and,
-- for a link, the signed-in account `$6`. It waits until `$7`; `$8` hands
-- its outcome over to a mobile App.
INSERT INTO oidc_requests (id, provider, state_hash, pkce_verifier, nonce, user_id, expires_at, hands_over)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8);

-- name: ForgetExpiredRequests :exec
-- Forgets every request whose callback never came in time.
DELETE FROM oidc_requests
WHERE expires_at <= now();

-- name: TakeRequest :maybe_one
-- Spends the unexpired request at the provider `$1` whose state hashes to
-- `$2`: one statement, so a callback can't be redeemed twice.
DELETE FROM oidc_requests
WHERE provider = $1 AND state_hash = $2 AND expires_at > now()
RETURNING pkce_verifier, nonce, user_id, hands_over;

-- name: FindIdentity :maybe_one
-- The account the identity `$2` at the provider `$1` is linked to.
SELECT user_id
FROM user_identities
WHERE provider = $1 AND subject = $2;

-- name: FindUserByEmail :maybe_one
-- The live account with `email`, whatever its case.
-- param: $1 email String
SELECT id
FROM users
WHERE lower(email) = lower($1) AND deleted_at IS NULL;

-- name: CreateUser :one
-- A new account signing up through a provider: no password, its email
-- verified when the provider says so. A live account with the same email
-- is a unique violation of `users_email_key`.
INSERT INTO users (id, email, email_verified)
VALUES ($1, $2, $3)
RETURNING id;

-- name: InsertIdentity :exec
-- Links the identity `$4` at the provider `$3` to the account `$2`. One
-- linked already is a unique violation of
-- `user_identities_provider_subject_key`.
INSERT INTO user_identities (id, user_id, provider, subject, email)
VALUES ($1, $2, $3, $4, $5);

-- name: ListIdentities :many
-- The account `$1`'s Linked identities, oldest first.
SELECT id, provider, email, created_at
FROM user_identities
WHERE user_id = $1
ORDER BY created_at, id;

-- name: FindLinked :maybe_one
-- The identity `$1`, when it is the account `$2`'s.
SELECT id
FROM user_identities
WHERE id = $1 AND user_id = $2;

-- name: Unlink :execrows
-- Unlinks the account `$2`'s identity `$1`, unless it is the account's
-- last way in: nothing (`0`) when the account has no password and no
-- other identity. One statement, so two unlinks at once can't both pass.
DELETE FROM user_identities
WHERE id = $1 AND user_id = $2
    AND (
        EXISTS (SELECT 1 FROM users WHERE users.id = $2 AND users.has_password)
        OR EXISTS (
            SELECT 1 FROM user_identities AS other WHERE other.user_id = $2 AND other.id <> $1
        )
    );

-- name: InsertHandoff :exec
-- A sign-in handed over to a mobile App as the code hashing to `$3`: the
-- account `$2` signs in with it once, until `$4`.
INSERT INTO oidc_handoffs (id, user_id, code_hash, expires_at)
VALUES ($1, $2, $3, $4);

-- name: TakeHandoff :maybe_one
-- Spends the unused, unexpired hand-over whose code hashes to `$1`: the
-- account it signs in. One statement, so a code can't be spent twice.
UPDATE oidc_handoffs
SET used_at = now()
WHERE code_hash = $1 AND used_at IS NULL AND expires_at > now()
RETURNING user_id;

-- name: ForgetExpiredHandoffs :exec
-- Forgets every hand-over past its minute, used or not.
DELETE FROM oidc_handoffs
WHERE expires_at <= now();
