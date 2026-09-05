use std::net::SocketAddr;

use axum::{extract::State, routing::get, Router};
use metrics::{describe_counter, describe_gauge, describe_histogram};
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use tracing::{error, info};

use crate::state::ServerState;

pub const CONNECTION_REQUESTS_TOTAL: &str = "bvs_connection_requests_total";
pub const CONNECTIONS_ACTIVE: &str = "bvs_connections_active";
pub const ROOMS: &str = "bvs_rooms";
pub const CLIENTS_IN_QUEUE: &str = "bvs_clients_in_queue";
pub const MATCHED_GROUPS: &str = "bvs_matched_groups";
pub const MATCHES_TOTAL: &str = "bvs_matches_total";
pub const MATCHES_RELAY_TOTAL: &str = "bvs_matches_relay_total";
pub const SIGNALING_ERRORS_TOTAL: &str = "bvs_signaling_errors_total";
pub const CONNECTION_DURATION_SECONDS: &str = "bvs_connection_duration_seconds";

pub fn install() -> PrometheusHandle {
    let handle = PrometheusBuilder::new()
        .set_buckets_for_metric(
            metrics_exporter_prometheus::Matcher::Full(CONNECTION_DURATION_SECONDS.to_string()),
            &[10.0, 30.0, 60.0, 300.0, 600.0, 1800.0, 3600.0, 7200.0],
        )
        .expect("connection duration buckets")
        .install_recorder()
        .expect("failed to install metrics recorder");

    describe_counter!(
        CONNECTION_REQUESTS_TOTAL,
        "Websocket connection requests received"
    );
    describe_gauge!(
        CONNECTIONS_ACTIVE,
        "Peers currently connected to the signaling server"
    );
    describe_gauge!(ROOMS, "Rooms currently open");
    describe_gauge!(
        CLIENTS_IN_QUEUE,
        "Clients assigned an id but not yet in a room"
    );
    describe_gauge!(
        MATCHED_GROUPS,
        "Peer groups matched via ?next= still connected"
    );
    describe_counter!(MATCHES_TOTAL, "Peer groups completed via ?next=");
    describe_counter!(MATCHES_RELAY_TOTAL, "Peer groups completed via ?next= with a relay peer");
    describe_counter!(SIGNALING_ERRORS_TOTAL, "Signaling failures, by kind");
    describe_histogram!(
        CONNECTION_DURATION_SECONDS,
        "Time a peer stayed connected to signaling"
    );

    handle
}

async fn metrics_handler(State((state, handle)): State<(ServerState, PrometheusHandle)>) -> String {
    state.publish_metrics_gauges();
    handle.run_upkeep();
    handle.render()
}

pub async fn serve(state: ServerState, handle: PrometheusHandle, addr: SocketAddr) {
    let app = Router::new()
        .route("/metrics", get(metrics_handler))
        .with_state((state, handle));

    match tokio::net::TcpListener::bind(addr).await {
        Ok(listener) => {
            info!("Metrics listening on {}", addr);

            if let Err(err) = axum::serve(listener, app).await {
                error!("Metrics server stopped: {}", err);
            }
        }
        Err(err) => error!("Metrics server could not bind {}: {}", addr, err),
    }
}
