extern crate serde;
extern crate serde_json;
extern crate slack_morphism;
extern crate tracing;

pub mod webhook;

pub use webhook::{AppState, slack_router};

#[cfg(test)]
mod webhook_test;
