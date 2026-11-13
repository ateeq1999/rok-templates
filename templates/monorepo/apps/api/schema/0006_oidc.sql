-- Sign-in with an OpenID provider (the `oidc` feature, workspace ADR 0035).
-- A provider is named in the settings (`OIDC_PROVIDERS`), never here: the
-- tables hold its name as text.

-- A User's Linked identities: who they are at a provider. One identity is
-- one account's, matched by its provider and subject, never by email.
CREATE TABLE user_identities (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    -- The provider's name, such as `google`.
    provider TEXT NOT NULL,
    -- The provider's id for the person (the ID token's `sub`).
    subject TEXT NOT NULL,
    -- The email the provider gave when the identity was linked, shown in
    -- the settings; it is never matched on.
    email TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT user_identities_provider_subject_key UNIQUE (provider, subject)
);

CREATE INDEX user_identities_user ON user_identities (user_id);

-- A sign-in (or, with `user_id`, a link) waiting for the provider's
-- callback: single-use, found by the hash of its `state`, with the PKCE
-- verifier and the nonce the callback redeems the code with.
CREATE TABLE oidc_requests (
    id UUID PRIMARY KEY,
    provider TEXT NOT NULL,
    state_hash TEXT NOT NULL,
    pkce_verifier TEXT NOT NULL,
    nonce TEXT NOT NULL,
    -- The signed-in account linking an identity; none for a sign-in.
    user_id UUID REFERENCES users (id) ON DELETE CASCADE,
    -- A mobile App's (`start?client=mobile`): the callback hands the
    -- outcome over at `<APP_SCHEME>://auth/oidc`, a session as a one-minute
    -- code (`oidc_handoffs`), instead of sending the browser to `APP_URL`.
    hands_over BOOLEAN NOT NULL DEFAULT false,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT oidc_requests_state_hash_key UNIQUE (state_hash)
);

-- A finished sign-in handed over to a mobile App: a one-minute,
-- single-use code the App trades for a session (only its hash is kept).
CREATE TABLE oidc_handoffs (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    code_hash TEXT NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT oidc_handoffs_code_hash_key UNIQUE (code_hash)
);
