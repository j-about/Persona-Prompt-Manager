//! Token Repository
//!
//! Provides data access operations for tokens within personas.
//! All methods are stateless and take a connection reference as their first parameter.
//!
//! # Usage
//!
//! ```rust,ignore
//! let token = TokenRepository::create(&conn, &request)?;
//! let tokens = TokenRepository::find_by_persona(&conn, &persona_id)?;
//! ```

use chrono::Utc;
use rusqlite::{params, Connection};

use crate::domain::token::{
    CreateTokenRequest, ReorderTokensRequest, Token, TokenPolarity, UpdateTokenRequest,
};
use crate::error::AppError;

/// Repository for token database operations.
///
/// This struct contains no state; all methods take a connection reference
/// and can be composed within external transactions.
pub struct TokenRepository;

impl TokenRepository {
    /// Inserts a new token into the database (internal helper).
    ///
    /// Use `create()` or `create_batch()` for the public API.
    fn insert(conn: &Connection, token: &Token) -> Result<(), AppError> {
        conn.execute(
            r"
            INSERT INTO tokens (id, persona_id, granularity_id, polarity, content, weight, display_order, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            ",
            params![
                token.id,
                token.persona_id,
                token.granularity_id,
                token.polarity.as_str(),
                token.content,
                token.weight,
                token.display_order,
                token.created_at.to_rfc3339(),
                token.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    /// Finds a token by its unique identifier.
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection reference
    /// * `id` - The token's UUID
    ///
    /// # Errors
    ///
    /// Returns `AppError::NotFound` if no token exists with the given ID.
    /// Returns `AppError::Database` for other database errors.
    pub fn find_by_id(conn: &Connection, id: &str) -> Result<Token, AppError> {
        conn.query_row(
            r"
            SELECT id, persona_id, granularity_id, polarity, content, weight, display_order, created_at, updated_at
            FROM tokens WHERE id = ?1
            ",
            [id],
            Self::row_to_token,
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => {
                AppError::NotFound(format!("Token with id '{id}' not found"))
            }
            _ => AppError::Database(e),
        })
    }

    /// Retrieves all tokens for a persona.
    ///
    /// Results are ordered by global display order (user-defined sequence).
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection reference
    /// * `persona_id` - The parent persona's UUID
    ///
    /// # Errors
    ///
    /// Returns `AppError::Database` for database errors.
    pub fn find_by_persona(conn: &Connection, persona_id: &str) -> Result<Vec<Token>, AppError> {
        let mut stmt = conn.prepare(
            r"
            SELECT id, persona_id, granularity_id, polarity, content, weight, display_order, created_at, updated_at
            FROM tokens
            WHERE persona_id = ?1
            ORDER BY display_order
            ",
        )?;

        let tokens = stmt
            .query_map([persona_id], Self::row_to_token)?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(tokens)
    }

    /// Updates a token with the provided changes.
    ///
    /// Fetches the existing token, applies the update request, and persists.
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection reference
    /// * `id` - The token's UUID
    /// * `request` - The update request with optional field changes
    ///
    /// # Returns
    ///
    /// Returns the updated token entity.
    ///
    /// # Errors
    ///
    /// Returns `AppError::NotFound` if the token doesn't exist.
    /// Returns `AppError::Database` for other database errors.
    pub fn update(
        conn: &Connection,
        id: &str,
        request: &UpdateTokenRequest,
    ) -> Result<Token, AppError> {
        let mut token = Self::find_by_id(conn, id)?;
        token.update(request);

        conn.execute(
            r"
            UPDATE tokens
            SET content = ?1, weight = ?2, granularity_id = ?3, polarity = ?4, updated_at = ?5
            WHERE id = ?6
            ",
            params![
                token.content,
                token.weight,
                token.granularity_id,
                token.polarity.as_str(),
                token.updated_at.to_rfc3339(),
                id,
            ],
        )?;

        Ok(token)
    }

    /// Deletes a token from the database.
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection reference
    /// * `id` - The token's UUID
    ///
    /// # Errors
    ///
    /// Returns `AppError::NotFound` if the token doesn't exist.
    /// Returns `AppError::Database` for other database errors.
    pub fn delete(conn: &Connection, id: &str) -> Result<(), AppError> {
        let rows = conn.execute("DELETE FROM tokens WHERE id = ?1", [id])?;
        if rows == 0 {
            return Err(AppError::NotFound(format!(
                "Token with id '{id}' not found"
            )));
        }
        Ok(())
    }

    /// Calculates the next global display order for a new token (internal helper).
    ///
    /// Returns the next available position after all existing tokens in the persona.
    fn get_next_display_order(conn: &Connection, persona_id: &str) -> Result<i32, AppError> {
        let max_order: Option<i32> = conn
            .query_row(
                r"SELECT MAX(display_order) FROM tokens WHERE persona_id = ?1",
                [persona_id],
                |row| row.get(0),
            )
            .ok();

        Ok(max_order.unwrap_or(-1) + 1)
    }

    /// Creates a new token from a request.
    ///
    /// Automatically assigns the next global display order for the token
    /// within the persona.
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection reference
    /// * `request` - The creation request with token details
    ///
    /// # Returns
    ///
    /// Returns the newly created token entity.
    ///
    /// # Errors
    ///
    /// Returns `AppError::Database` if the insert fails.
    pub fn create(conn: &Connection, request: &CreateTokenRequest) -> Result<Token, AppError> {
        let display_order = Self::get_next_display_order(conn, &request.persona_id)?;

        let token = Token::new(
            request.persona_id.clone(),
            request.granularity_id.clone(),
            request.polarity,
            request.content.clone(),
            request.weight,
            display_order,
        );

        Self::insert(conn, &token)?;

        Ok(token)
    }

    /// Creates multiple tokens in batch.
    ///
    /// Each token is assigned sequential global display orders starting from the
    /// next available position within the persona. Empty content strings are skipped.
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection reference
    /// * `persona_id` - The parent persona's UUID
    /// * `granularity_id` - The granularity level ID for all tokens
    /// * `polarity` - The polarity for all tokens
    /// * `contents` - Array of token content strings
    /// * `weight` - The weight to apply to all created tokens
    ///
    /// # Returns
    ///
    /// Returns a vector of the newly created token entities.
    ///
    /// # Errors
    ///
    /// Returns `AppError::Database` if any insert fails.
    pub fn create_batch(
        conn: &Connection,
        persona_id: &str,
        granularity_id: &str,
        polarity: TokenPolarity,
        contents: &[String],
        weight: f64,
    ) -> Result<Vec<Token>, AppError> {
        let mut tokens = Vec::new();
        let mut display_order = Self::get_next_display_order(conn, persona_id)?;

        for content in contents {
            if content.trim().is_empty() {
                continue;
            }

            let token = Token::new(
                persona_id.to_string(),
                granularity_id.to_string(),
                polarity,
                content.trim().to_string(),
                weight,
                display_order,
            );

            Self::insert(conn, &token)?;
            tokens.push(token);
            display_order += 1;
        }

        Ok(tokens)
    }

    /// Reorders tokens within a persona by updating display_order values.
    ///
    /// All updates are performed atomically. The frontend computes the new
    /// ordering after drag-and-drop operations.
    ///
    /// # Arguments
    ///
    /// * `conn` - Database connection reference
    /// * `request` - Reorder request with persona_id and token_orders
    ///
    /// # Errors
    ///
    /// Returns `AppError::Validation` if any token doesn't belong to the persona.
    /// Returns `AppError::Database` for database errors.
    pub fn reorder_tokens(
        conn: &Connection,
        request: &ReorderTokensRequest,
    ) -> Result<(), AppError> {
        // Validate all tokens belong to the persona
        for order in &request.token_orders {
            let token = Self::find_by_id(conn, &order.token_id)?;
            if token.persona_id != request.persona_id {
                return Err(AppError::Validation(format!(
                    "Token '{}' does not belong to persona '{}'",
                    order.token_id, request.persona_id
                )));
            }
        }

        // Update all display_orders
        let now = Utc::now().to_rfc3339();
        for order in &request.token_orders {
            conn.execute(
                r"UPDATE tokens SET display_order = ?1, updated_at = ?2 WHERE id = ?3",
                params![order.display_order, &now, &order.token_id],
            )?;
        }

        Ok(())
    }

    /// Helper function to convert a row to a Token
    ///
    /// Column mapping:
    /// 0: id, 1: `persona_id`, 2: `granularity_id`, 3: polarity,
    /// 4: content, 5: weight, 6: `display_order`, 7: `created_at`, 8: `updated_at`
    fn row_to_token(row: &rusqlite::Row) -> Result<Token, rusqlite::Error> {
        // Parse polarity string, defaulting to positive if parsing fails
        let polarity_str: String = row.get(3)?;
        let polarity = TokenPolarity::parse(&polarity_str).unwrap_or(TokenPolarity::Positive);

        Ok(Token {
            id: row.get(0)?,
            persona_id: row.get(1)?,
            granularity_id: row.get(2)?,
            polarity,
            content: row.get(4)?,
            weight: row.get(5)?,
            display_order: row.get(6)?,
            // Timestamps stored as RFC3339 strings; fallback to now if parsing fails
            created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(7)?)
                .map_or_else(|_| Utc::now(), |dt| dt.with_timezone(&Utc)),
            updated_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(8)?)
                .map_or_else(|_| Utc::now(), |dt| dt.with_timezone(&Utc)),
        })
    }
}
