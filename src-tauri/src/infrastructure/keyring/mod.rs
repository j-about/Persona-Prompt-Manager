//! Keyring Module - Secure Credential Storage
//!
//! This module provides secure storage for API keys using the operating system's
//! native credential management facilities:
//!
//! | Platform | Backend                   |
//! |----------|---------------------------|
//! | Windows  | Windows Credential Manager |
//! | macOS    | Keychain                  |
//! | Linux    | Secret Service (libsecret) |
//!
//! # Security Model
//!
//! API keys are stored separately from application data to prevent accidental
//! exposure through database exports or file system access. The OS keyring
//! provides:
//! - Encryption at rest
//! - Per-user access control
//! - Session-based unlocking
//!
//! # Linux Requirements
//!
//! On Linux, a Secret Service daemon must be running (e.g., gnome-keyring or kwallet).
//! The application checks for availability at startup via `check_credential_store_available()`.

pub mod secrets;
pub use secrets::*;
