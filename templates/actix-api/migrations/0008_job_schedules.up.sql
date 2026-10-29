CREATE TABLE "job_schedules" (
    "kind" TEXT NOT NULL,
    "last_tick" TIMESTAMPTZ NOT NULL,
    "updated_at" TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT "job_schedules_pkey" PRIMARY KEY ("kind")
);
CREATE TRIGGER job_schedules_touch_updated_at BEFORE UPDATE ON job_schedules
    FOR EACH ROW EXECUTE FUNCTION touch_updated_at();
