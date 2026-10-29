-- The schema of the target API (api-starter-axum's OpenAPI document), as
-- far as generation covers it. `users` and `audit_log` have API features;
-- `media` and `api_keys` belong to the hand-written `media` and `api_keys`
-- features, so `rok-data-sqlc.toml` excludes them from the generated API.

CREATE TYPE role AS ENUM ('user', 'admin');
CREATE TYPE api_key_scope AS ENUM ('read', 'write');

-- Sets `updated_at` on every update.
CREATE FUNCTION touch_updated_at() RETURNS trigger
LANGUAGE plpgsql AS $$
begin
  new.updated_at := now();
  return new;
end;
$$;

-- rok:down
DROP FUNCTION touch_updated_at();

CREATE TABLE users (
    id UUID PRIMARY KEY,
    email TEXT NOT NULL,
    display_name TEXT,
    -- Write-only: set by the password flow, never answered.
    password_hash TEXT,
    has_password BOOLEAN NOT NULL GENERATED ALWAYS AS (password_hash IS NOT NULL) STORED,
    role role NOT NULL DEFAULT 'user',
    is_active BOOLEAN NOT NULL DEFAULT true,
    email_verified BOOLEAN NOT NULL DEFAULT false,
    totp_secret TEXT,
    totp_enabled BOOLEAN NOT NULL DEFAULT false,
    -- The profile photo's storage key; `avatar_url` below is its URL.
    avatar_key TEXT,
    failed_logins INTEGER NOT NULL DEFAULT 0,
    locked_until TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at TIMESTAMPTZ,
    -- Bumped by a password change or a reset. A token whose `tv` claim no
    -- longer matches is rejected on its next request.
    token_version INTEGER NOT NULL DEFAULT 1,
    -- The time step of the last two-factor code accepted, so a code is
    -- never accepted twice.
    totp_last_step BIGINT,
    -- The profile photo's public URL, `/api/v1/avatars/<random>.jpg` (`GET
    -- /avatars/{file}`), or `NULL` without a photo; `avatar_key` is its
    -- storage key, set by `PUT /users/me/avatar`. Computed here, so the
    -- generated `users` feature answers it too; `rok-data-sqlc.toml` hides
    -- the key.
    avatar_url TEXT GENERATED ALWAYS AS ('/api/v1/' || avatar_key) STORED
);

-- One live account per email, whatever its case.
CREATE UNIQUE INDEX users_email_key ON users (lower(email)) WHERE deleted_at IS NULL;

CREATE TRIGGER users_touch_updated_at BEFORE UPDATE ON users
    FOR EACH ROW EXECUTE FUNCTION touch_updated_at();

-- rok:down
DROP TRIGGER users_touch_updated_at ON users;

-- Administrative actions, newest first; written by the application.
CREATE TABLE audit_log (
    id UUID PRIMARY KEY,
    actor_user_id UUID NOT NULL REFERENCES users (id),
    target_user_id UUID REFERENCES users (id),
    action TEXT NOT NULL,
    details JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Uploaded files, each owned by its uploader. A file uploaded straight to
-- the store (`POST /media/uploads`) starts as an upload intent: no
-- `completed_at` yet, and an `expires_at` past which the sweep removes it.
-- Every read but the completion's sees completed files only.
CREATE TABLE media (
    id UUID PRIMARY KEY,
    owner_id UUID NOT NULL REFERENCES users (id),
    content_type TEXT NOT NULL,
    size_bytes BIGINT NOT NULL,
    original_filename TEXT,
    storage_key TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    completed_at TIMESTAMPTZ DEFAULT now(),
    expires_at TIMESTAMPTZ
);

-- API keys: only a hash of the key is kept.
CREATE TABLE api_keys (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users (id),
    name TEXT NOT NULL,
    key_prefix TEXT NOT NULL,
    key_hash TEXT NOT NULL,
    scope api_key_scope NOT NULL DEFAULT 'read',
    expires_at TIMESTAMPTZ,
    last_used_at TIMESTAMPTZ,
    revoked_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
