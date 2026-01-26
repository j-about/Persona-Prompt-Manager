//! Token repository - Data access for tokens

use chrono::Utc;
use rusqlite::{params, Connection};

use crate::domain::token::{CreateTokenRequest, Token, TokenPolarity, UpdateTokenRequest};
use crate::error::AppError;

/// Repository for token database operations
pub struct TokenRepository;

impl TokenRepository {
    /// Insert a new token into the database
    pub fn insert(conn: &Connection, token: &Token) -> Result<(), AppError> {
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

    /// Find a token by ID
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

    /// Find all tokens for a persona
    pub fn find_by_persona(conn: &Connection, persona_id: &str) -> Result<Vec<Token>, AppError> {
        let mut stmt = conn.prepare(
            r"
            SELECT id, persona_id, granularity_id, polarity, content, weight, display_order, created_at, updated_at
            FROM tokens
            WHERE persona_id = ?1
            ORDER BY granularity_id, polarity, display_order
            ",
        )?;

        let tokens = stmt
            .query_map([persona_id], Self::row_to_token)?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(tokens)
    }

    /// Update a token
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

    /// Delete a token
    pub fn delete(conn: &Connection, id: &str) -> Result<(), AppError> {
        let rows = conn.execute("DELETE FROM tokens WHERE id = ?1", [id])?;
        if rows == 0 {
            return Err(AppError::NotFound(format!(
                "Token with id '{id}' not found"
            )));
        }
        Ok(())
    }

    /// Get the next display order for a new token
    pub fn get_next_display_order(
        conn: &Connection,
        persona_id: &str,
        granularity_id: &str,
        polarity: TokenPolarity,
    ) -> Result<i32, AppError> {
        let max_order: Option<i32> = conn
            .query_row(
                r"
                SELECT MAX(display_order) FROM tokens
                WHERE persona_id = ?1 AND granularity_id = ?2 AND polarity = ?3
                ",
                params![persona_id, granularity_id, polarity.as_str()],
                |row| row.get(0),
            )
            .ok();

        Ok(max_order.unwrap_or(-1) + 1)
    }

    /// Create a token from a request
    pub fn create(conn: &Connection, request: &CreateTokenRequest) -> Result<Token, AppError> {
        let display_order = Self::get_next_display_order(
            conn,
            &request.persona_id,
            &request.granularity_id,
            request.polarity,
        )?;

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

    /// Create multiple tokens from comma-separated content
    pub fn create_batch(
        conn: &Connection,
        persona_id: &str,
        granularity_id: &str,
        polarity: TokenPolarity,
        contents: &[String],
        weight: f64,
    ) -> Result<Vec<Token>, AppError> {
        let mut tokens = Vec::new();
        let mut display_order =
            Self::get_next_display_order(conn, persona_id, granularity_id, polarity)?;

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
