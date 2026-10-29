//! The request metrics: every answer counted by method, matched route and
//! status, and timed. Its own test binary, since the recorder is the
//! process's one. No database: the pool never connects.

mod support;

use actix_web::http::{Method, StatusCode};
use metrics_exporter_prometheus::PrometheusBuilder;
use sqlx::PgPool;
use support::{TestResult, send, state_with};

/// The value of the series of `metric` whose labels include every one of
/// `labels` (`name="value"`), as `/metrics` renders it.
fn sample(rendered: &str, metric: &str, labels: &[&str]) -> Option<f64> {
    rendered
        .lines()
        .filter(|line| line.starts_with(&format!("{metric}{{")))
        .find(|line| labels.iter().all(|label| line.contains(label)))
        .and_then(|line| line.rsplit(' ').next()?.parse().ok())
}

#[actix_web::test]
async fn requests_are_counted_by_route_and_status() -> TestResult {
    let metrics = PrometheusBuilder::new().install_recorder()?;
    let state = state_with(&PgPool::connect_lazy("postgres://localhost/unused")?, &[])?;
    for _ in 0..2 {
        let live = send(&state, None, Method::GET, "/health/live", &[], Vec::new()).await?;
        assert_eq!(live.status, StatusCode::OK);
    }
    send(
        &state,
        None,
        Method::GET,
        "/no/such/route/42",
        &[],
        Vec::new(),
    )
    .await?;
    let rendered = metrics.render();
    let live = [
        r#"method="GET""#,
        r#"route="/health/live""#,
        r#"status="200""#,
    ];
    assert_eq!(
        sample(&rendered, "http_requests_total", &live),
        Some(2.0),
        "{rendered}"
    );
    // Paths no route matched are one series, not one per path.
    let unmatched = [r#"route="unmatched""#, r#"status="404""#];
    assert_eq!(
        sample(&rendered, "http_requests_total", &unmatched),
        Some(1.0),
        "{rendered}"
    );
    assert!(
        sample(
            &rendered,
            "http_request_duration_seconds_count",
            &[r#"route="/health/live""#]
        )
        .is_some_and(|count| count >= 2.0),
        "{rendered}"
    );
    Ok(())
}
