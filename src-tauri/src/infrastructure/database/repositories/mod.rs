//! Repository Layer - Data Access Objects
//!
//! This module provides repository implementations following the Repository pattern,
//! encapsulating all database operations for each domain entity type.
//!
//! # Design Principles
//!
//! - **Stateless**: Repository structs contain no state; methods take connection references
//! - **Type-Safe**: All queries return strongly-typed domain entities
//! - **SQL Encapsulation**: Raw SQL is contained within repository methods
//! - **Transaction Support**: Methods can be composed within external transactions
//!
//! # Available Repositories
//!
//! - [`PersonaRepository`]: CRUD operations for personas and generation parameters
//! - [`TokenRepository`]: Token management including batch operations and reordering

pub mod persona;
pub mod token;

pub use persona::PersonaRepository;
pub use token::TokenRepository;
