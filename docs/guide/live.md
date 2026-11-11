# Live updates

A screen that shows something other people change (a kitchen board, a
diner's bill, a checkout waiting for payment) shouldn't poll. The API
templates (axum, actix and the monorepo's API) push a **notice** on a
**Topic** instead, and the App refetches what the notice concerns.

## Publishing

A feature publishes on its own transaction, beside the change:

<!-- not compiled: a feature's service, which the reader writes -->
```rust
use crate::shared::live;

let mut tx = self.pool.begin().await?;
let order = Order::create(&mut *tx, new_order).await?;
live::publish(&mut tx, tenant, "orders.new", json!({"id": order.id})).await?;
tx.commit().await?;
```

`publish` is a PostgreSQL `NOTIFY`, which the database delivers only when
the transaction commits: a rolled-back change sends nothing, the way a job
enqueued on a transaction never runs for a rollback. Outside a
transaction (`&mut *pool.acquire().await?`) it is sent at once.

A notice says *what* changed, not the new rows. `NOTIFY` carries at most
8000 bytes, and the client refetches what it shows anyway, through the
same routes and the same authorization as before. Name Topics for the
news, `orders.new`, `bills.paid`, and send ids in the payload.

## Who hears it: the tenant and the guard

Every notice is for one **tenant**, the first argument of `publish`, and
only that tenant's subscribers hear it. Who may subscribe, and to which
tenant, is a [guard](guards.md): `features/live/service.rs`'s
`Subscriber`, whose grant is the tenant. The template's lets every
signed-in caller subscribe to their own account's Topics, so until the
project has tenants of its own a feature publishes to the user it
concerns (`publish(&mut tx, user_id, ...)`).

With organizations, answer the caller's organization there (one read in
`queries/shared/`), or `None` for a `403`:

<!-- not compiled: a project's own tenant and its query -->
```rust
impl Guard for Subscriber {
    type Grant = OrganizationId;
    type Path = ();

    async fn check(
        caller: &AuthUser,
        state: &AppState,
        _path: &(),
    ) -> Result<Option<OrganizationId>, AppError> {
        Ok(organization_of(&state.pool, caller.id.into()).await?)
    }
}
```

## Subscribing

Two routes, left out of the OpenAPI document (an event stream has no
schema a generated client could call):

- `POST /api/v1/live/tickets`, with the session's bearer token: a ticket
  to the tenant the guard grants, `{"ticket": "...", "expires_in": 60}`.
  A caller the guard refuses gets `403`. A browser's `EventSource` can't
  send an `Authorization` header, so the stream is opened with the ticket.
- `GET /api/v1/live?topics=orders.new,bills.paid&ticket=...`: the notices
  on those Topics, as server-sent events whose data is the notice,
  `{"topic": "orders.new", "payload": {"id": "..."}}`, with a keep-alive
  comment every 15 seconds. A missing, forged or expired ticket is a `401`.
  The ticket only opens the stream; an open stream outlives it.

### The web App

`src/lib/live.ts` has `subscribe` and, for a component, `useTopics`. Each
notice goes to your function, which returns the query keys to
invalidate:

```tsx
useTopics(["orders.new", "orders.ready"], () => [["orders"]])
```

A dropped connection is reopened with a fresh ticket, after 1 s, then twice
as long each time, up to 30 s.

### The mobile App

`LiveRepository` (`lib/app/data/repositories/live.repository.dart`) gives
a `Stream<Notice>`; a cubit reloads on each notice it cares about:

```dart
final live = LiveRepository(url: api.url, session: authService);
_subscription = live.notices(['bills.paid']).listen((_) => load());
```

It reconnects the same way, and never ends until the subscription is
cancelled. A notice published while it was reconnecting is missed: reload
when the screen comes back if that matters.

## How it scales

Each API instance listens with one connection (`Topics::start`, from
`app::serve`) and fans every notice out to its own subscribers, so every
instance sees every notice and a client may connect to any of them. Each
subscriber has a buffer of 256 notices; one that falls further behind is
dropped, and its client reconnects and refetches. On `SIGTERM` the open
streams end at once, so they don't hold the drain up, and their clients
reconnect to an instance that is ready.

A subscription holds an HTTP connection open, not a database connection.
