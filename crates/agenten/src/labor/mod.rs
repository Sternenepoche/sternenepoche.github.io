//! Persistent autonomous players. Separate protocol version; legacy runs remain readable.
pub mod audit;
pub mod balance;
pub mod broker;
pub mod capacity;
pub mod config;
pub mod experiment;
pub mod gateway;
pub mod memory;
pub mod research;
pub mod runtime;
pub mod sandbox;
pub mod native_sandbox;
pub mod world;
pub use runtime::run;
pub type Result<T> = std::result::Result<T, String>;
