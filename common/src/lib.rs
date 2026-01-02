//! Common types and utilities shared between the Kafka tutorial producer and leaderboard.

pub mod kafka;
pub mod messages;
pub mod topics;

// Re-export commonly used types at crate root
pub use messages::{Action, User, WatchlistEntry};
