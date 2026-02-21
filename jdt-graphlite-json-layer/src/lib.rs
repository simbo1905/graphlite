//! JDT-GraphLite JSON Layer
//!
//! A JSON document persistence layer for GraphLite that integrates:
//!
//! - **JTD validation** (RFC 8927) — validate JSON before persisting
//! - **JDT transformation** — transform JSON documents on read
//! - **GraphLite storage** — persist JSON as graph nodes with GQL queries
//!
//! Based on simbo1905/jtd-wasm and simbo1905/jdt-wasm.

pub mod jdt;
pub mod json_store;
pub mod jtd;

pub use jdt::{apply as jdt_apply, JdtError};
pub use json_store::{InsertResult, JsonDocument, JsonQueryResult, JsonStore, JsonStoreError};
pub use jtd::{JtdError, JtdSchema, ValidationError};
