//! The server as a client sees it: `grpc.health.v1.Health` and server
//! reflection answer without credentials, the API's services need a
//! caller, and on shutdown the health service answers `NOT_SERVING` while
//! the server drains. None of it reaches the database.

mod support;

use std::time::Duration;

use grpc_api::app::{self, AppState};
use grpc_api::config::AppRole;
use grpc_api::features::proto;
use grpc_api::features::proto::users_service_client::UsersServiceClient;
use grpc_api::shared::mail::transport::LogMailer;
use support::{TestResult, channel, config, offline, state};
use tokio::net::TcpListener;
use tokio_stream::StreamExt;
use tonic::Code;
use tonic::transport::Channel;
use tonic_health::pb::HealthCheckRequest;
use tonic_health::pb::health_check_response::ServingStatus;
use tonic_health::pb::health_client::HealthClient;
use tonic_health::server::HealthReporter;
use tonic_reflection::pb::v1::ServerReflectionRequest;
use tonic_reflection::pb::v1::server_reflection_client::ServerReflectionClient;
use tonic_reflection::pb::v1::server_reflection_request::MessageRequest;
use tonic_reflection::pb::v1::server_reflection_response::MessageResponse;
use uuid::Uuid;

/// A channel to the whole application over a pool that never connects: the
/// API behind the caller check, health and reflection beside it.
async fn application() -> Result<Channel, Box<dyn std::error::Error>> {
    let routes = app::routes(&state(&offline()?)?, HealthReporter::new())?;
    channel(routes).await
}

/// What the health service answers for the whole server (`""`).
async fn serving(channel: Channel) -> Result<ServingStatus, Box<dyn std::error::Error>> {
    let answer = HealthClient::new(channel)
        .check(HealthCheckRequest {
            service: String::new(),
        })
        .await?;
    Ok(answer.into_inner().status())
}

/// The health service answers `SERVING` for the server, to a caller with no
/// credentials.
#[tokio::test]
async fn the_health_service_answers_serving() -> TestResult {
    assert_eq!(serving(application().await?).await?, ServingStatus::Serving);
    Ok(())
}

/// Reflection lists every service the server serves, so `grpcurl` and
/// Postman reach them without the `.proto` files.
#[tokio::test]
async fn reflection_lists_the_services() -> TestResult {
    let mut reflection = ServerReflectionClient::new(application().await?);
    let request = ServerReflectionRequest {
        host: String::new(),
        message_request: Some(MessageRequest::ListServices(String::new())),
    };
    let mut answers = reflection
        .server_reflection_info(tokio_stream::once(request))
        .await?
        .into_inner();
    let answer = answers
        .next()
        .await
        .ok_or("reflection answered nothing")??;
    let Some(MessageResponse::ListServicesResponse(listed)) = answer.message_response else {
        return Err(format!("not a list of services: {answer:?}").into());
    };
    let names: Vec<&str> = listed
        .service
        .iter()
        .map(|service| service.name.as_str())
        .collect();
    for name in [
        "grpc_api.v1.UsersService",
        "grpc_api.v1.AuditLogEntryService",
        "grpc.health.v1.Health",
        "grpc.reflection.v1.ServerReflection",
    ] {
        assert!(names.contains(&name), "`{name}` is not listed: {names:?}");
    }
    Ok(())
}

/// The API's services sit behind the caller check, which names no caller
/// yet: a call without credentials is `UNAUTHENTICATED`.
#[tokio::test]
async fn a_call_without_credentials_is_unauthenticated() -> TestResult {
    let mut users = UsersServiceClient::new(application().await?);
    let request = proto::GetUserRequest {
        id: Uuid::now_v7().to_string(),
    };
    let status = users
        .get(request)
        .await
        .err()
        .ok_or("a call without credentials was answered")?;
    assert_eq!(status.code(), Code::Unauthenticated, "{status:?}");
    Ok(())
}

/// Served on a port for `role`, until `stop` resolves.
async fn served(
    role: AppRole,
    state: AppState,
    stop: tokio::sync::oneshot::Receiver<()>,
) -> Result<(Channel, tokio::task::JoinHandle<std::io::Result<()>>), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    let server = tokio::spawn(app::serve_as(
        role,
        listener,
        state,
        std::sync::Arc::new(LogMailer),
        async {
            let _ = stop.await;
        },
    ));
    let channel = Channel::from_shared(format!("http://{addr}"))?
        .connect()
        .await?;
    Ok((channel, server))
}

/// On shutdown the health service answers `NOT_SERVING` for
/// `SHUTDOWN_DELAY_SECS` while calls are still served, then the server
/// stops.
#[tokio::test]
async fn a_draining_server_answers_not_serving() -> TestResult {
    let config = config(&[("SHUTDOWN_DELAY_SECS", "1"), ("SHUTDOWN_GRACE_SECS", "1")])?;
    let state = AppState::new(offline()?, &config);
    let (stop, stopped) = tokio::sync::oneshot::channel();
    let (channel, server) = served(AppRole::Web, state, stopped).await?;
    assert_eq!(serving(channel.clone()).await?, ServingStatus::Serving);
    let _ = stop.send(());
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert_eq!(serving(channel).await?, ServingStatus::NotServing);
    tokio::time::timeout(Duration::from_secs(10), server).await???;
    Ok(())
}

/// A `worker` serves the health and reflection services only: the
/// platform's health check reaches it, the API doesn't.
#[tokio::test]
async fn a_worker_serves_health_only() -> TestResult {
    let (stop, stopped) = tokio::sync::oneshot::channel();
    let (channel, server) = served(AppRole::Worker, state(&offline()?)?, stopped).await?;
    assert_eq!(serving(channel.clone()).await?, ServingStatus::Serving);
    let status = UsersServiceClient::new(channel)
        .get(proto::GetUserRequest {
            id: Uuid::now_v7().to_string(),
        })
        .await
        .err()
        .ok_or("a worker answered the API")?;
    assert_eq!(status.code(), Code::Unimplemented, "{status:?}");
    let _ = stop.send(());
    tokio::time::timeout(Duration::from_secs(10), server).await???;
    Ok(())
}
