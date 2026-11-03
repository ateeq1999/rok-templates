//! What the gRPC tests share: the server on one end of an in-process pipe
//! and a channel to it on the other, a stand-in for the caller check, and
//! a PostgreSQL schema of the test's own.
//!
//! The tests that reach the data layer need `DATABASE_URL`; without it they
//! print a note and pass. Each one creates a fresh schema holding the
//! tables (`db::schema`'s snapshot of `schema/`) and drops it when done, so
//! tests run in parallel without seeing each other's rows.

#![allow(dead_code, reason = "each test file uses some of the helpers")]

use std::collections::HashMap;
use std::io;
use std::str::FromStr;

use grpc_api::app::{self, AppState};
use grpc_api::config::Config;
use grpc_api::db::schema::SCHEMA_SQL;
use grpc_api::db::types::Role;
use grpc_api::features;
use grpc_api::features::proto::user_events_service_server::UserEventsServiceServer;
use grpc_api::shared::auth::{AuthUser, Credential};
use hyper_util::rt::TokioIo;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::{AssertSqlSafe, PgPool};
use tonic::service::{InterceptorLayer, Routes};
use tonic::transport::{Channel, Endpoint, Server, Uri};
use tonic::{Request, Status};
use tonic_health::server::HealthReporter;
use uuid::Uuid;

/// What a test returns.
pub type TestResult = Result<(), Box<dyn std::error::Error>>;

/// The JWT secret the test application verifies tokens with.
const SECRET: &str = "a test secret of at least 32 bytes";

/// The `TOTP_ENCRYPTION_KEY` the test application runs with: base64 of 32
/// bytes.
const TOTP_KEY: &str = "dGVzdCBrZXkgb2YgZXhhY3RseSB0aGlydHktdHdvIGI=";

/// The metadata naming the test's caller: `<id>` or `<id>/admin`.
const CALLER: &str = "x-test-caller";

/// The settings the test application runs with: [`SECRET`], uploads in
/// memory and the defaults, with `overrides` (such as
/// `("SHUTDOWN_DELAY_SECS", "1")`) set on top.
pub fn config(overrides: &[(&str, &str)]) -> Result<Config, Box<dyn std::error::Error>> {
    let mut vars: HashMap<&str, &str> = HashMap::from([
        ("DATABASE_URL", "postgres://unused"),
        ("JWT_SECRET", SECRET),
        ("TOTP_ENCRYPTION_KEY", TOTP_KEY),
        ("STORAGE_URL", "memory://"),
        // Tests stop at once: no load balancer to wait for.
        ("SHUTDOWN_DELAY_SECS", "0"),
    ]);
    vars.extend(overrides.iter().copied());
    Ok(Config::from_vars(|name| {
        vars.get(name).map(|value| (*value).to_owned())
    })?)
}

/// The application's state over `pool`, as the tests configure it.
pub fn state(pool: &PgPool) -> Result<AppState, Box<dyn std::error::Error>> {
    Ok(AppState::new(pool.clone(), &config(&[])?))
}

/// A pool that never connects: for the calls that stop before the data
/// layer.
pub fn offline() -> Result<PgPool, sqlx::Error> {
    PgPool::connect_lazy("postgres://localhost/unused")
}

/// The tests' stand-in for the caller check (`app::authenticate`): the
/// caller the metadata names, put in the request's extensions where the
/// services read it.
fn scope(mut request: Request<()>) -> Result<Request<()>, Status> {
    let caller = request
        .metadata()
        .get(CALLER)
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned);
    if let Some(caller) = caller {
        let (id, role) = match caller.strip_suffix("/admin") {
            Some(id) => (id, Role::Admin),
            None => (caller.as_str(), Role::User),
        };
        let Ok(id) = Uuid::parse_str(id) else {
            return Err(Status::invalid_argument("the test caller is not a UUID"));
        };
        request.extensions_mut().insert(AuthUser {
            id,
            role,
            credential: Credential::Session,
        });
    }
    Ok(request)
}

/// The application's services over `pool`, the API's behind [`scope`]
/// instead of `app::authenticate`, the health service reporting through
/// `health`.
pub fn scoped(pool: &PgPool, health: HealthReporter) -> Result<Routes, Box<dyn std::error::Error>> {
    let user_events = features::user_events::UserEventsService::new(pool.clone());
    let api = features::routes(pool)
        .add_service(UserEventsServiceServer::new(user_events))
        .into_axum_router()
        .layer(InterceptorLayer::new(scope));
    Ok(app::services(Routes::from(api), health)?)
}

/// A channel to `routes`, served on the other end of an in-process pipe.
pub async fn channel(routes: Routes) -> Result<Channel, Box<dyn std::error::Error>> {
    let (client, server) = tokio::io::duplex(1024 * 1024);
    tokio::spawn(async move {
        Server::builder()
            .add_routes(routes)
            .serve_with_incoming(tokio_stream::once(Ok::<_, io::Error>(server)))
            .await
    });
    let mut client = Some(client);
    let channel = Endpoint::try_from("http://in-process")?
        .connect_with_connector(tower::service_fn(move |_: Uri| {
            let client = client.take();
            async move {
                client
                    .map(TokioIo::new)
                    .ok_or_else(|| io::Error::other("the pipe connects once"))
            }
        }))
        .await?;
    Ok(channel)
}

/// `message` as the caller `id`, an admin when `role` is.
pub fn as_caller<T>(message: T, id: Uuid, role: Role) -> Request<T> {
    let mut request = Request::new(message);
    let caller = match role {
        Role::Admin => format!("{id}/admin"),
        Role::User => id.to_string(),
    };
    if let Ok(caller) = caller.parse() {
        request.metadata_mut().insert(CALLER, caller);
    }
    request
}

/// A schema of the test database, and a pool whose connections use it.
pub struct Database {
    pub pool: PgPool,
    admin: PgPool,
    schema: String,
}

impl Database {
    /// A fresh schema, or `None` (and a note) when `DATABASE_URL` is not set.
    pub async fn fresh() -> Result<Option<Database>, Box<dyn std::error::Error>> {
        let Ok(url) = std::env::var("DATABASE_URL") else {
            eprintln!("skipping: DATABASE_URL is not set");
            return Ok(None);
        };
        let schema = format!("test_{}", Uuid::now_v7().simple());
        let admin = PgPool::connect(&url).await?;
        sqlx::raw_sql(AssertSqlSafe(format!("CREATE SCHEMA {schema}")))
            .execute(&admin)
            .await?;
        let options = PgConnectOptions::from_str(&url)?.options([("search_path", schema.as_str())]);
        let pool = PgPoolOptions::new()
            .max_connections(4)
            .connect_with(options)
            .await?;
        sqlx::raw_sql(SCHEMA_SQL).execute(&pool).await?;
        Ok(Some(Database {
            pool,
            admin,
            schema,
        }))
    }

    /// Drops the schema.
    pub async fn drop(self) -> TestResult {
        self.pool.close().await;
        sqlx::raw_sql(AssertSqlSafe(format!(
            "DROP SCHEMA {} CASCADE",
            self.schema
        )))
        .execute(&self.admin)
        .await?;
        Ok(())
    }
}
