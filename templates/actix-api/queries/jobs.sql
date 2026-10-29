-- The `jobs` feature's SQL: what an admin sees and does with the queue.
-- `rok sqlc g` generates `src/db/queries/jobs.rs`; only the `jobs` feature
-- calls it. The queue itself runs on `queries/queue.sql`.

-- name: ListJobs :many
-- One page of the jobs with status `$1` and kind `$2` (either `NULL`: any),
-- newest first. The payload stays out: a pending mail's holds its link.
-- param: $1 status Option<String>
-- param: $2 kind Option<String>
-- param: $3 limit i64
-- param: $4 offset i64
SELECT id, kind, status, attempts, run_at, last_error, created_at, updated_at
FROM jobs
WHERE ($1::text IS NULL OR status::text = $1) AND ($2::text IS NULL OR kind = $2)
ORDER BY created_at DESC, id DESC
LIMIT $3 OFFSET $4;

-- name: CountListedJobs :one
-- How many jobs have status `$1` and kind `$2` (either `NULL`: any).
-- param: $1 status Option<String>
-- param: $2 kind Option<String>
SELECT count(*) AS total
FROM jobs
WHERE ($1::text IS NULL OR status::text = $1) AND ($2::text IS NULL OR kind = $2);

-- name: RetryDeadJob :maybe_one
-- The dead job `$1` back in the queue: pending, due now, its attempts at
-- zero. Its kind, or no row when `$1` is no dead job.
UPDATE jobs
SET status = 'pending', attempts = 0, run_at = now()
WHERE id = $1 AND status = 'dead'
RETURNING kind;

-- name: CancelJob :maybe_one
-- Deletes the job `$1` unless a worker is running it right now (its row is
-- locked): its kind, or no row.
DELETE FROM jobs
WHERE id = $1 AND id IN (SELECT id FROM jobs WHERE id = $1 FOR UPDATE SKIP LOCKED)
RETURNING kind;

-- name: CountJob :one
-- Whether the job `$1` exists: 1, or 0.
SELECT count(*) AS present
FROM jobs
WHERE id = $1;
