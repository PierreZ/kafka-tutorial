//! Kafka Tutorial Classroom Simulator
//!
//! Simulates all 15 teams consuming from `new_users`, applying team-specific filters,
//! and producing to `actions` and `team_stats` topics.

pub mod config;
pub mod filters;
pub mod stats;
pub mod team;
