use std::fmt::Debug;

pub(crate) mod builder;
pub(crate) mod callbacks;
pub(crate) mod error;
pub(crate) mod handlers;
pub(crate) mod server;

/// State managed by the signaling server
pub trait SignalingState: Debug + Clone + Send + Sync + 'static {}

/// Callbacks used by the signaling server
pub trait SignalingCallbacks: Default + Clone + Send + Sync + 'static {}

/// Store no signaling callbacks
#[derive(Default, Debug, Copy, Clone)]
pub struct NoCallbacks {}
impl SignalingCallbacks for NoCallbacks {}

/// Store no state
#[derive(Clone, Debug)]
pub struct NoState {}
impl SignalingState for NoState {}
