DROP TRIGGER jobs_touch_updated_at ON jobs;
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
DROP INDEX IF EXISTS "jobs_unique_key";
DROP INDEX IF EXISTS "jobs_due";
DROP TABLE "jobs";
CREATE INDEX outbound_mail_due ON outbound_mail(next_attempt_at) WHERE status = 'pending';
CREATE TRIGGER outbound_mail_touch_updated_at BEFORE UPDATE ON outbound_mail
    FOR EACH ROW EXECUTE FUNCTION touch_updated_at();
