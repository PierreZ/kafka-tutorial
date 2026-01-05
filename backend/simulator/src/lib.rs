//! Kafka Tutorial Classroom Simulator
//!
//! Simulates all 15 teams consuming from `new_users`, applying team-specific filters,
//! and producing to `actions` and `watchlist` topics.

pub mod config;
pub mod filters;
pub mod team;
pub mod watchlist;
