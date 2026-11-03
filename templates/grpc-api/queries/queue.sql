-- The job queue's SQL. `rok sqlc g` generates `src/db/queries/queue.rs`;
-- `shared/jobs/` calls it: flows enqueue a job on their transaction, and a
-- worker claims, runs and settles it.

-- name: EnqueueJob :execrows
-- A job of `kind`, due `$4` seconds from now (the database's clock). With
-- a `unique_key` another job holds, nothing is written: 0 rows.
-- param: $4 delay_secs f64
INSERT INTO jobs (id, kind, payload, run_at, unique_key, traceparent)
VALUES ($1, $2, $3, now() + make_interval(secs => $4), $5, $6)
ON CONFLICT (unique_key) DO NOTHING;

-- name: NotifyJobs :exec
-- Wakes every worker listening on `jobs`, once the transaction commits.
SELECT pg_notify('jobs', '');

-- name: ClaimDueJob :maybe_one
-- The pending, due job waiting longest, locked until the claiming
-- transaction ends. A job another worker holds is skipped, so two workers
-- never run the same job.
SELECT id, kind, payload, attempts, traceparent
FROM jobs
WHERE status = 'pending' AND run_at <= now()
ORDER BY run_at, id
LIMIT 1
FOR UPDATE SKIP LOCKED;

-- name: DeleteJob :exec
-- A job that ran: nothing is left to do.
DELETE FROM jobs
WHERE id = $1;

-- name: RetryJob :exec
-- A failed attempt: its count and error, and a retry `$4` seconds from now.
-- param: $4 retry_in_secs f64
UPDATE jobs
SET attempts = $2, last_error = $3, run_at = now() + make_interval(secs => $4)
WHERE id = $1;

-- name: BuryJob :exec
-- A job out of attempts: `dead`, with its error, its payload as its kind's
-- `on_dead` left it (mail drops its bodies). It is never tried again.
UPDATE jobs
SET status = 'dead', attempts = $2, last_error = $3, payload = $4
WHERE id = $1;

-- name: CountJobs :many
-- The queue's depth: pending jobs due now, pending jobs not due yet, and
-- dead jobs.
-- column: state String
-- column: jobs i64
SELECT
    CASE
        WHEN status = 'dead' THEN 'dead'
        WHEN run_at <= now() THEN 'due'
        ELSE 'scheduled'
    END AS state,
    count(*) AS jobs
FROM jobs
GROUP BY 1;

-- name: StartSchedule :exec
-- A schedule seen for the first time starts at the tick `$2`, so it waits
-- for its next tick rather than running at once.
INSERT INTO job_schedules (kind, last_tick)
VALUES ($1, $2)
ON CONFLICT (kind) DO NOTHING;

-- name: ClaimTick :execrows
-- Claims the tick `$2` of the schedule of `$1`: 1 row for the one
-- transaction that moves it forward, 0 for any other (the row lock makes a
-- concurrent claim wait, then see the tick taken).
UPDATE job_schedules
SET last_tick = $2
WHERE kind = $1 AND last_tick < $2;
