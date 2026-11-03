// Scaffolded once by rok-data-sqlc; this file is yours. It is never regenerated. See docs/guide/architecture.md.

//! [`routes`]: every service the server serves. The generated features'
//! services (`features::routes`) sit behind the caller check
//! ([`authenticate`]); `grpc.health.v1.Health` (`tonic-health`) and server
//! reflection (`tonic-reflection`, v1 and v1alpha) sit beside them, outside
//! it, so a load balancer's probe and `grpcurl` need no credentials.

use tonic::service::Routes;
use tonic_health::pb::health_server::HealthServer;
use tonic_health::server::{HealthReporter, HealthService};
use tonic_reflection::server::{Builder, Error};

use super::{AppState, authenticate};
use crate::features;

/// The descriptors of `proto/` (and their imports), as `build.rs` encodes
/// them: what reflection answers with.
pub const FILE_DESCRIPTOR_SET: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/file_descriptor_set.bin"));

/// The API's services over `state`, behind [`authenticate`], beside the
/// health service `health` reports through and reflection (see
/// [`services`]). Add a hand-written service to `api` here:
/// `.add_service(OrdersServiceServer::new(orders))`.
///
/// # Errors
///
/// When the embedded descriptors don't decode: a build bug.
pub fn routes(state: &AppState, health: HealthReporter) -> Result<Routes, Error> {
    let api = features::routes(&state.pool);
    services(authenticate(api, state), health)
}

/// `api` beside the health service `health` reports through and the
/// reflection services, which describe `proto/` and the health service.
///
/// # Errors
///
/// When the embedded descriptors don't decode: a build bug.
pub fn services(api: Routes, health: HealthReporter) -> Result<Routes, Error> {
    let reflection = || {
        Builder::configure()
            .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
            .register_encoded_file_descriptor_set(tonic_health::pb::FILE_DESCRIPTOR_SET)
    };
    Ok(api
        .add_service(HealthServer::new(HealthService::from_health_reporter(
            health,
        )))
        .add_service(reflection().build_v1()?)
        .add_service(reflection().build_v1alpha()?))
}
