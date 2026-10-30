-- Two-factor sign-in (TOTP). `users.totp_secret` holds the authenticator
-- secret encrypted with AES-256-GCM under `TOTP_ENCRYPTION_KEY` (base64 of
-- the nonce, then the ciphertext); it is set while a setup waits for its
-- first code, and `totp_enabled` turns on once `POST /auth/2fa/enable`
-- confirms one. `users.totp_last_step` (in `0001_init.sql`) keeps a code
-- from being accepted twice, and a right password on a two-factor account
-- issues a short-lived `two_factor_pending` token (`0002_auth.sql`), which
-- `POST /auth/2fa/verify` spends.

-- Single-use recovery codes, issued when two-factor is enabled. Only a
-- keyed hash of each is kept; `used_at` spends it.
CREATE TABLE totp_recovery_codes (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    code_hash TEXT NOT NULL,
    used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX totp_recovery_codes_user ON totp_recovery_codes (user_id);
