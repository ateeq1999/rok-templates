ALTER TABLE "auth_tokens" DROP CONSTRAINT "auth_tokens_purpose_check";
ALTER TABLE "users" ADD COLUMN "totp_last_step" BIGINT;
CREATE TABLE "totp_recovery_codes" (
    "id" UUID NOT NULL,
    "user_id" UUID NOT NULL,
    "code_hash" TEXT NOT NULL,
    "used_at" TIMESTAMPTZ,
    "created_at" TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT "totp_recovery_codes_pkey" PRIMARY KEY ("id")
);
ALTER TABLE "auth_tokens" ADD CONSTRAINT "auth_tokens_purpose_check" CHECK (purpose IN ('email_verification', 'password_reset', 'email_change', 'invitation', 'two_factor_pending'));
ALTER TABLE "totp_recovery_codes" ADD CONSTRAINT "totp_recovery_codes_user_id_fkey" FOREIGN KEY ("user_id") REFERENCES "users" ("id") ON DELETE CASCADE;
CREATE INDEX totp_recovery_codes_user ON totp_recovery_codes(user_id);
