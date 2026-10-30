DROP INDEX IF EXISTS "totp_recovery_codes_user";
ALTER TABLE "totp_recovery_codes" DROP CONSTRAINT "totp_recovery_codes_user_id_fkey";
ALTER TABLE "auth_tokens" DROP CONSTRAINT "auth_tokens_purpose_check";
DROP TABLE "totp_recovery_codes";
ALTER TABLE "users" DROP COLUMN "totp_last_step";
ALTER TABLE "auth_tokens" ADD CONSTRAINT "auth_tokens_purpose_check" CHECK (purpose IN ('email_verification', 'password_reset', 'email_change', 'invitation'));
