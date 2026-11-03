DROP TRIGGER outbound_mail_touch_updated_at ON outbound_mail;
DROP INDEX IF EXISTS "outbound_mail_due";
DROP INDEX IF EXISTS "auth_tokens_user_purpose";
ALTER TABLE "auth_tokens" DROP CONSTRAINT "auth_tokens_user_id_fkey";
DROP TABLE "outbound_mail";
DROP TABLE "auth_tokens";
ALTER TABLE "users" DROP COLUMN "token_version";
