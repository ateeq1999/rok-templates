ALTER TABLE "users" ADD COLUMN "token_version" INTEGER NOT NULL DEFAULT 1;
CREATE TABLE "auth_tokens" (
    "id" UUID NOT NULL,
    "user_id" UUID NOT NULL,
    "purpose" TEXT NOT NULL,
    "token_hash" TEXT NOT NULL,
    "new_email" TEXT,
    "expires_at" TIMESTAMPTZ NOT NULL,
    "used_at" TIMESTAMPTZ,
    "created_at" TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT "auth_tokens_pkey" PRIMARY KEY ("id"),
    CONSTRAINT "auth_tokens_token_hash_key" UNIQUE ("token_hash"),
    CONSTRAINT "auth_tokens_purpose_check" CHECK (purpose IN ('email_verification', 'password_reset', 'email_change', 'invitation'))
);
CREATE TABLE "outbound_mail" (
    "id" UUID NOT NULL,
    "payload" JSONB NOT NULL,
    "status" TEXT NOT NULL DEFAULT 'pending',
    "attempts" INTEGER NOT NULL DEFAULT 0,
    "next_attempt_at" TIMESTAMPTZ NOT NULL DEFAULT now(),
    "last_error" TEXT,
    "created_at" TIMESTAMPTZ NOT NULL DEFAULT now(),
    "updated_at" TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT "outbound_mail_pkey" PRIMARY KEY ("id"),
    CONSTRAINT "outbound_mail_status_check" CHECK (status IN ('pending', 'dead'))
);
ALTER TABLE "auth_tokens" ADD CONSTRAINT "auth_tokens_user_id_fkey" FOREIGN KEY ("user_id") REFERENCES "users" ("id") ON DELETE CASCADE;
CREATE INDEX auth_tokens_user_purpose ON auth_tokens(user_id, purpose);
CREATE INDEX outbound_mail_due ON outbound_mail(next_attempt_at) WHERE status = 'pending';
CREATE TRIGGER outbound_mail_touch_updated_at BEFORE UPDATE ON outbound_mail
    FOR EACH ROW EXECUTE FUNCTION touch_updated_at();
