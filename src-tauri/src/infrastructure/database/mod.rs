//! Database Module - `SQLite` Persistence Layer
//!
//! This module provides all database operations for the application, including
//! connection management, schema migrations, and repository implementations.
//!
//! # Architecture
//!
//! The database layer follows the Repository pattern:
//! - **Connection**: Single `SQLite` connection with WAL mode
//! - **Migrations**: Version-controlled schema evolution
//! - **Repositories**: Type-safe data access objects
//!
//! # `SQLite` Configuration
//!
//! The database is configured for desktop application use:
//! - **WAL Mode**: Write-Ahead Logging for better concurrent access
//! - **Foreign Keys**: Enabled for referential integrity
//! - **Location**: `{app_data_dir}/ppm.db`
//!
//! # Schema Overview
//!
//! - `personas`: Core persona entities with metadata
//! - `generation_params`: Image generation settings (1:1 with personas)
//! - `tokens`: Prompt tokens with granularity, polarity, and weights

pub mod connection;
pub mod migrations;
pub mod repositories;

pub use connection::Database;
