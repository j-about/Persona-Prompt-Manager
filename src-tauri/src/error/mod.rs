//! Error Handling Module
//!
//! This module defines the application's error types and provides a unified
//! error handling strategy across all layers.
//!
//! # Design Goals
//!
//! - **Type Safety**: All error conditions are represented in the type system
//! - **User-Friendly**: Error messages are clear and actionable
//! - **IPC Compatible**: Errors serialize cleanly for Tauri frontend communication
//!
//! # Usage
//!
//! Functions return `Result<T, AppError>`, and the `?` operator propagates
//! errors automatically thanks to `From` implementations for common error types.

mod app_error;

pub use app_error::AppError;
