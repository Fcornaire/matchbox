mod args;
mod constants;
mod jobs;
mod state;
mod topology;

use std::sync::Arc;

use crate::{
    state::{RequestedRoom, RoomId, ServerState},
    topology::MatchmakingDemoTopology,
};
use args::Args;
use axum::{http::StatusCode, response::IntoResponse, routing::get};
use clap::Parser;
use jobs::spawn_jobs;
use matchbox_signaling::SignalingServerBuilder;
use tracing::info;
use tracing_subscriber::prelude::*;

fn setup_logging() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "matchbox_server=info,tower_http=debug".into()),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .compact()
                .with_file(false)
                .with_target(false),
        )
        .init();
}

#[tokio::main]
async fn main() {
    setup_logging();
    let args = Args::parse();

    // Setup router
    info!("Matchbox Signaling Server: {}", args.host);

    let state = Arc::new(ServerState::default());
    let clone = state.clone();
    let server = SignalingServerBuilder::new(
        args.host,
        MatchmakingDemoTopology,
        (*state).clone(),
        args.integrity_hash,
    )
    .on_connection_request({
        let state = state.clone();
        move |connection| {
            let room_id = RoomId(connection.path.clone().unwrap_or_default());
            let next = connection
                .query_params
                .get("next")
                .and_then(|next| next.parse::<usize>().ok());
            let room = RequestedRoom { id: room_id, next };
            state.add_waiting_client(connection.origin, room);
            Ok(true) // allow all clients
        }
    })
    .on_id_assignment({
        move |(origin, peer_id)| {
            info!("Client connected {origin:?}: {peer_id:?}");
            state.assign_id_to_waiting_client(origin, peer_id);
        }
    })
    .cors()
    .trace()
    .mutate_router(|router| router.route("/health", get(health_handler)))
    .build();

    let mut server_task = tokio::spawn(async move {
        server
            .serve()
            .await
            .expect("Unable to run signaling server, is it already running?")
    });

    let mut jobs_task = tokio::spawn(async move {
        tracing::info!("[Main] Jobs started");
        spawn_jobs(clone).await;
    });

    tokio::select! {
        _ = (&mut server_task) => { jobs_task.abort() },
        _ = (&mut jobs_task) => { server_task.abort() },
    }
}

pub async fn health_handler() -> impl IntoResponse {
    StatusCode::OK
}
