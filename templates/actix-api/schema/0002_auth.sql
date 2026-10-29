-- What the account flows need: single-use tokens, and the mail outbox
-- (`users.token_version` is in `0001_init.sql`). `rok-data-sqlc.toml` keeps
-- the two tables out of the API; the auth features reach them through
-- `queries/`.

-- Single-use tokens mailed as links. Only the SHA-256 of the token is
-- kept; `used_at` spends it, and issuing a new one for the same user and
-- purpose invalidates the older ones.
CREATE TABLE auth_tokens (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    purpose TEXT NOT NULL CHECK (
        purpose IN ('email_verification', 'password_reset', 'email_change',
                    'invitation', 'two_factor_pending')
    ),
    token_hash TEXT NOT NULL UNIQUE,
    -- The address an `email_change` token moves the account to.
    new_email TEXT,
    expires_at TIMESTAMPTZ NOT NULL,
    used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX auth_tokens_user_purpose ON auth_tokens (user_id, purpose);
