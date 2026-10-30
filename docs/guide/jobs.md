# Jobs and schedules

Work that need not happen inside the request — sending mail, building a
report, calling a slow API — runs as a **job** on PostgreSQL itself
(`src/shared/jobs/`, workspace ADR 0024). There is no other datastore to
run, and a job is written on the same transaction as the change that asks
for it, so a rolled-back request enqueues nothing.

- [How it works](#how-it-works)
- [Adding a job](#adding-a-job)
- [Scheduled jobs](#scheduled-jobs)
- [Processes: `APP_ROLE`](#processes-app_role)
- [Failed jobs](#failed-jobs)
- [Testing jobs](#testing-jobs)
- [Settings](#settings)

## How it works

A job is a row in `jobs`: its kind, its payload (JSON), its status
(`pending` or `dead`), its attempts and when it is next due. The worker
running beside the server claims the due job waiting longest with
`FOR UPDATE SKIP LOCKED` — two workers, in one process or many, never take
the same row — runs it, and deletes it, all in the claiming transaction.
A failed attempt is retried after 30 seconds, then 1, 2 and 4 minutes;
after its kind's last attempt (5 by default) the job is `dead`, kept with
its `last_error`, never tried again.

The worker wakes as soon as a job is committed: in-process at once, from
another process through `NOTIFY jobs`, and every 5 seconds otherwise.

Delivery is **at least once**: a crash after a job ran but before its
claim committed runs it again. Make each job safe to repeat (an
idempotency key on the call it makes, a check that the work is not done
already). A running job holds its claim — one pool connection — until it
settles, so keep `JOBS_CONCURRENCY` well under `DB_MAX_CONNECTIONS`.

A job's span (`job`, with its kind, id and attempt) continues the trace of
the request that enqueued it, so a trace backend shows the request and the
mail it sent as one trace.

## Adding a job

A job is a payload type and what to do with it. Mail is the first one:

<!-- source: templates/axum-api/src/shared/mail/outbox.rs -->
```rust
#[async_trait]
impl Job for SendMail {
    const KIND: &'static str = "mail.send";
    const MAX_ATTEMPTS: i32 = MAX_ATTEMPTS;
    type Context = Arc<dyn Mailer>;

    async fn run(self, mailer: &Arc<dyn Mailer>) -> Result<(), JobError> {
        mailer.send(&self.0).await?;
        Ok(())
    }

    /// A dead mail's bodies are dropped, since a link they carry must not
    /// outlive the attempts to send it.
    fn on_dead(mut payload: serde_json::Value) -> serde_json::Value {
        for body in ["text", "html"] {
            if let Some(value) = payload.get_mut(body) {
                *value = serde_json::Value::from(REMOVED);
            }
        }
        payload
    }
}
```

1. **The payload**, `#[derive(Serialize, Deserialize)]`: what the job needs
   to know, small (ids, not rows). It is stored as JSON until the job runs.
2. **`impl Job`**: `KIND` names it in the table and the metrics (never
   rename one with jobs queued); `Context` is what `run` needs, such as a
   service; `run` does the work, and any error converts with `?`.
   `MAX_ATTEMPTS`, `TIMEOUT` (5 minutes) and `on_dead` (what a dead job's
   payload keeps; strip what must not outlive the attempts) have
   defaults. The job's type lives in the feature that owns the work, in
   its `service.rs`.
3. **Register it** in `app/run.rs`'s `worker`, with its context:

   <!-- source: templates/axum-api/src/app/run.rs -->
   ```rust
   pub fn worker(state: &AppState, mailer: Arc<dyn Mailer>) -> Worker {
       Worker::new(state.pool.clone(), &state.queue)
           .with_concurrency(state.jobs_concurrency)
           .register::<SendMail>(mailer)
           .register::<SweepUploads>(MediaService::clone(&state.media))
   }
   ```

   A job of a kind no worker registers fails at once and is `dead`, so
   register every kind in this one place.
4. **Enqueue it** on the request's transaction, from the service that
   holds a `Queue` (`AppState.queue`): `queue.enqueue(&mut tx, &job)`, then
   `queue.commit(tx)`, which commits and wakes the worker.
   `queue.enqueue_with(&mut tx, &job, Enqueue { delay, unique_key })`
   delays it, or enqueues it only when no job holds the same `unique_key`
   (it answers `false` then).

## Scheduled jobs

A job on a clock is one line of `app/run.rs`'s `schedule`:

<!-- source: templates/axum-api/src/app/run.rs -->
```rust
pub fn schedule() -> Result<Schedule, ScheduleError> {
    Schedule::new().cron::<SweepUploads>("*/5 * * * *")
}
```

`.cron::<Kind>(pattern)?` enqueues `Kind::default()` at every tick of a
cron pattern (`minute hour day month weekday`, UTC, `croner`'s syntax);
the kind must be registered in `worker` too. A pattern that doesn't parse
stops the start-up, naming the kind.

Every process running jobs runs the schedule, and each tick runs once
however many there are: at a tick, each tries to move the kind's
`last_tick` forward in `job_schedules`, and the row lock lets one
transaction win — that one enqueues the job. A tick missed while nothing
ran (a deploy, an outage) runs once when a process is back, not once per
missed tick; a schedule seen for the first time waits for its next tick
rather than running at once, so a nightly job does not fire at the first
boot.

## Processes: `APP_ROLE`

One image runs any part of the application:

| `APP_ROLE` | Serves | Runs jobs and the schedule |
|---|---|---|
| `all` (the default) | the API | yes |
| `web` | the API | no: its jobs run in a `worker` |
| `worker` | `/health/live` and `/health/ready` only | yes |

Run `all` in one process while it is small; split into `web` (scaled with
traffic) and `worker` (scaled with `jobs_queued{state="due"}`) when it is
not. On `SIGTERM` the worker stops taking jobs; a job still running past
`SHUTDOWN_GRACE_SECS` is abandoned, its transaction rolled back, so it runs
again on another worker, no attempt counted.

## Failed jobs

A dead job stays in `jobs` until someone acts. The `jobs` feature
(`src/features/jobs/`, admins only) is that someone:

- `GET /api/v1/jobs?status=dead&kind=mail.send` lists them, newest first,
  paginated. A job's payload is never answered: a pending mail's holds its
  link.
- `POST /api/v1/jobs/{id}/retry` puts a dead job back in the queue, its
  attempts at zero.
- `DELETE /api/v1/jobs/{id}` cancels a pending or dead job; `409` while a
  worker is running it.

Retries and cancellations are in the audit log (`job.retried`,
`job.cancelled`). Alert on `jobs_queued{state="dead"} > 0`
([Observability](observability.md#metrics)).

## Testing jobs

Tests run jobs without a background task and without sleeping:
`Worker::run_due()` runs every due job, one after another, until none is
due (a failing job with `with_backoff(Duration::ZERO)` runs until it is
dead), and answers how many ran and failed. `tests/support/mod.rs`'s
`mailed(pool)` sends the queued mail through a `MemoryMailer` that way.
Schedules are driven with the clock a test wants:
`schedule.tick_at(&pool, &queue, now)`. `tests/jobs.rs` and
`tests/schedule.rs` show both.

## Settings

| Variable | Default | |
|---|---|---|
| `JOBS_CONCURRENCY` | 4 | jobs a worker runs at once; each holds a pool connection |
| `APP_ROLE` | `all` | what the process runs |
| `SHUTDOWN_GRACE_SECS` | 25 | how long running jobs have to finish on `SIGTERM` |
