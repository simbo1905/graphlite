/// Rust code emitter — generates standalone serde_json::Value validators.
mod context;
mod emit;
mod types;

pub use emit::emit;
