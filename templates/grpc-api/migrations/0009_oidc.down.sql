DROP INDEX IF EXISTS "user_identities_user";
ALTER TABLE "oidc_handoffs" DROP CONSTRAINT "oidc_handoffs_user_id_fkey";
ALTER TABLE "oidc_requests" DROP CONSTRAINT "oidc_requests_user_id_fkey";
ALTER TABLE "user_identities" DROP CONSTRAINT "user_identities_user_id_fkey";
DROP TABLE "oidc_handoffs";
DROP TABLE "oidc_requests";
DROP TABLE "user_identities";
