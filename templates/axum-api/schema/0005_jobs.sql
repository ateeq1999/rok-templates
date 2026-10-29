-- The job queue (`shared/jobs/`): a job is written on the flow's
-- transaction, claimed by one worker (`FOR UPDATE SKIP LOCKED`) and deleted
-- once it ran. A job that keeps failing becomes `dead`, keeping its error.
-- Mail is a job (`mail.send`).
CREATE TABLE jobs (
    id UUID PRIMARY KEY,
    -- The job's type, such as `mail.send`: which handler runs it.
    kind TEXT NOT NULL,
    payload JSONB NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'dead')),
    attempts INTEGER NOT NULL DEFAULT 0,
    run_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    -- Set, no second job with the same key is enqueued while one exists.
    unique_key TEXT,
    last_error TEXT,
    -- The W3C trace context of the request that enqueued it: its span
    -- continues that trace.
    traceparent TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX jobs_due ON jobs (run_at) WHERE status = 'pending';

CREATE UNIQUE INDEX jobs_unique_key ON jobs (unique_key);

CREATE TRIGGER jobs_touch_updated_at BEFORE UPDATE ON jobs
    FOR EACH ROW EXECUTE FUNCTION touch_updated_at();

-- rok:down
DROP TRIGGER jobs_touch_updated_at ON jobs;
