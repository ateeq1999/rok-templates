CREATE TABLE "user_identities" (
    "id" UUID NOT NULL,
    "user_id" UUID NOT NULL,
    "provider" TEXT NOT NULL,
    "subject" TEXT NOT NULL,
    "email" TEXT,
    "created_at" TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT "user_identities_pkey" PRIMARY KEY ("id"),
    CONSTRAINT "user_identities_provider_subject_key" UNIQUE ("provider", "subject")
);
CREATE TABLE "oidc_requests" (
    "id" UUID NOT NULL,
    "provider" TEXT NOT NULL,
    "state_hash" TEXT NOT NULL,
    "pkce_verifier" TEXT NOT NULL,
    "nonce" TEXT NOT NULL,
    "user_id" UUID,
    "hands_over" BOOLEAN NOT NULL DEFAULT false,
    "expires_at" TIMESTAMPTZ NOT NULL,
    "created_at" TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT "oidc_requests_pkey" PRIMARY KEY ("id"),
    CONSTRAINT "oidc_requests_state_hash_key" UNIQUE ("state_hash")
);
CREATE TABLE "oidc_handoffs" (
    "id" UUID NOT NULL,
    "user_id" UUID NOT NULL,
    "code_hash" TEXT NOT NULL,
    "expires_at" TIMESTAMPTZ NOT NULL,
    "used_at" TIMESTAMPTZ,
    "created_at" TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT "oidc_handoffs_pkey" PRIMARY KEY ("id"),
    CONSTRAINT "oidc_handoffs_code_hash_key" UNIQUE ("code_hash")
);
ALTER TABLE "user_identities" ADD CONSTRAINT "user_identities_user_id_fkey" FOREIGN KEY ("user_id") REFERENCES "users" ("id") ON DELETE CASCADE;
ALTER TABLE "oidc_requests" ADD CONSTRAINT "oidc_requests_user_id_fkey" FOREIGN KEY ("user_id") REFERENCES "users" ("id") ON DELETE CASCADE;
ALTER TABLE "oidc_handoffs" ADD CONSTRAINT "oidc_handoffs_user_id_fkey" FOREIGN KEY ("user_id") REFERENCES "users" ("id") ON DELETE CASCADE;
CREATE INDEX user_identities_user ON user_identities(user_id);
