DROP TRIGGER outbound_mail_touch_updated_at ON outbound_mail;
DROP INDEX IF EXISTS "outbound_mail_due";
CREATE TABLE "jobs" (
    "id" UUID NOT NULL,
    "kind" TEXT NOT NULL,
    "payload" JSONB NOT NULL,
    "status" TEXT NOT NULL DEFAULT 'pending',
    "attempts" INTEGER NOT NULL DEFAULT 0,
    "run_at" TIMESTAMPTZ NOT NULL DEFAULT now(),
    "unique_key" TEXT,
    "last_error" TEXT,
    "traceparent" TEXT,
    "created_at" TIMESTAMPTZ NOT NULL DEFAULT now(),
    "updated_at" TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT "jobs_pkey" PRIMARY KEY ("id"),
    CONSTRAINT "jobs_status_check" CHECK (status IN ('pending', 'dead'))
);
CREATE INDEX jobs_due ON jobs(run_at) WHERE status = 'pending';
CREATE UNIQUE INDEX jobs_unique_key ON jobs(unique_key);
DROP TABLE "outbound_mail";
CREATE TRIGGER jobs_touch_updated_at BEFORE UPDATE ON jobs
    FOR EACH ROW EXECUTE FUNCTION touch_updated_at();
