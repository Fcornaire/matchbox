use std::{sync::Arc, time::Duration};

use tokio::time;

use crate::{
    constants::{MONITOR_JOB_INTERVAL, UPDATE_METRICS_INTERVAL},
    ServerState,
};

async fn monitor(state: Arc<ServerState>) {
    let mut interval = time::interval(Duration::from_secs(MONITOR_JOB_INTERVAL));
    loop {
        interval.tick().await;
        tracing::info!("[Job] Monitor: {:?}", state.get_metrics());
    }
}

async fn update_metrics(state: Arc<ServerState>) {
    let mut interval = time::interval(Duration::from_secs(UPDATE_METRICS_INTERVAL));
    loop {
        interval.tick().await;
        state.update_metrics();
    }
}

pub async fn spawn_jobs(state: Arc<ServerState>) {
    tokio::select! {
        _ = (monitor(state.clone())) => {
        },
        _ = (update_metrics(state.clone())) => {
        },

    }
}
