//! Persona repository - Data access for personas

use chrono::Utc;
use rusqlite::{params, Connection};

use crate::domain::persona::{
    CreatePersonaRequest, GenerationParams, Persona, UpdatePersonaRequest,
};
use crate::error::AppError;

/// Repository for persona database operations
pub struct PersonaRepository;

impl PersonaRepository {
    /// Insert a new persona into the database
    pub fn insert(conn: &Connection, persona: &Persona) -> Result<(), AppError> {
        let tags_json = serde_json::to_string(&persona.tags)?;

        conn.execute(
            r"
            INSERT INTO personas (id, name, description, tags, ai_provider_id, ai_model_id, ai_instructions, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            ",
            params![
                persona.id,
                persona.name,
                persona.description,
                tags_json,
                persona.ai_provider_id,
                persona.ai_model_id,
                persona.ai_instructions,
                persona.created_at.to_rfc3339(),
                persona.updated_at.to_rfc3339(),
            ],
        )?;

        // Also create default generation params
        let params = GenerationParams::default_for_persona(&persona.id);
        Self::insert_generation_params(conn, &params)?;

        Ok(())
    }

    /// Insert generation parameters for a persona
    pub fn insert_generation_params(
        conn: &Connection,
        params: &GenerationParams,
    ) -> Result<(), AppError> {
        conn.execute(
            r"
            INSERT INTO generation_params (persona_id, model_id, seed, steps, cfg_scale, sampler, scheduler)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            ",
            params![
                params.persona_id,
                params.model_id,
                params.seed,
                params.steps,
                params.cfg_scale,
                params.sampler,
                params.scheduler,
            ],
        )?;
        Ok(())
    }

    /// Find a persona by ID
    pub fn find_by_id(conn: &Connection, id: &str) -> Result<Persona, AppError> {
        conn.query_row(
            r"
            SELECT id, name, description, tags, ai_provider_id, ai_model_id, ai_instructions, created_at, updated_at
            FROM personas WHERE id = ?1
            ",
            [id],
            Self::row_to_persona,
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => {
                AppError::NotFound(format!("Persona with id '{id}' not found"))
            }
            _ => AppError::Database(e),
        })
    }

    /// Helper to convert a row to Persona
    ///
    /// Column mapping:
    /// 0: id, 1: name, 2: description, 3: tags (JSON),
    /// 4: `ai_provider_id`, 5: `ai_model_id`, 6: `ai_instructions`,
    /// 7: `created_at`, 8: `updated_at`
    fn row_to_persona(row: &rusqlite::Row) -> rusqlite::Result<Persona> {
        // Tags stored as JSON array; fallback to empty vec if parsing fails
        let tags_json: String = row.get(3)?;
        let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();

        Ok(Persona {
            id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            tags,
            ai_provider_id: row.get(4)?,
            ai_model_id: row.get(5)?,
            ai_instructions: row.get(6)?,
            // Timestamps stored as RFC3339 strings; fallback to now if parsing fails
            created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(7)?).map_or_else(|_| Utc::now(), |dt| dt.with_timezone(&Utc)),
            updated_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(8)?).map_or_else(|_| Utc::now(), |dt| dt.with_timezone(&Utc)),
        })
    }

    /// Find generation parameters for a persona
    pub fn find_generation_params(
        conn: &Connection,
        persona_id: &str,
    ) -> Result<GenerationParams, AppError> {
        conn.query_row(
            r"
            SELECT persona_id, model_id, seed, steps, cfg_scale, sampler, scheduler
            FROM generation_params WHERE persona_id = ?1
            ",
            [persona_id],
            |row| {
                Ok(GenerationParams {
                    persona_id: row.get(0)?,
                    model_id: row.get(1)?,
                    seed: row.get(2)?,
                    steps: row.get(3)?,
                    cfg_scale: row.get(4)?,
                    sampler: row.get(5)?,
                    scheduler: row.get(6)?,
                })
            },
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => AppError::NotFound(format!(
                "Generation params for persona '{persona_id}' not found"
            )),
            _ => AppError::Database(e),
        })
    }

    /// Find all personas
    pub fn find_all(conn: &Connection) -> Result<Vec<Persona>, AppError> {
        let mut stmt = conn.prepare(
            r"
            SELECT id, name, description, tags, ai_provider_id, ai_model_id, ai_instructions, created_at, updated_at
            FROM personas ORDER BY created_at DESC
            ",
        )?;

        let personas = stmt
            .query_map([], Self::row_to_persona)?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(personas)
    }

    /// Search personas by name or description
    pub fn search(conn: &Connection, query: &str) -> Result<Vec<Persona>, AppError> {
        let search_term = format!("%{query}%");
        let mut stmt = conn.prepare(
            r"
            SELECT id, name, description, tags, ai_provider_id, ai_model_id, ai_instructions, created_at, updated_at
            FROM personas
            WHERE name LIKE ?1 OR description LIKE ?1
            ORDER BY created_at DESC
            ",
        )?;

        let personas = stmt
            .query_map([&search_term], Self::row_to_persona)?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(personas)
    }

    /// Update a persona
    pub fn update(
        conn: &Connection,
        id: &str,
        request: &UpdatePersonaRequest,
    ) -> Result<Persona, AppError> {
        // First fetch the existing persona
        let mut persona = Self::find_by_id(conn, id)?;

        // Apply updates
        persona.update(request);

        let tags_json = serde_json::to_string(&persona.tags)?;

        // Update in database
        conn.execute(
            r"
            UPDATE personas
            SET name = ?1, description = ?2, tags = ?3, ai_provider_id = ?4, ai_model_id = ?5, ai_instructions = ?6, updated_at = ?7
            WHERE id = ?8
            ",
            params![
                persona.name,
                persona.description,
                tags_json,
                persona.ai_provider_id,
                persona.ai_model_id,
                persona.ai_instructions,
                persona.updated_at.to_rfc3339(),
                id,
            ],
        )?;

        Ok(persona)
    }

    /// Update generation parameters for a persona
    pub fn update_generation_params(
        conn: &Connection,
        params: &GenerationParams,
    ) -> Result<(), AppError> {
        conn.execute(
            r"
            UPDATE generation_params
            SET model_id = ?1, seed = ?2, steps = ?3, cfg_scale = ?4, sampler = ?5, scheduler = ?6
            WHERE persona_id = ?7
            ",
            params![
                params.model_id,
                params.seed,
                params.steps,
                params.cfg_scale,
                params.sampler,
                params.scheduler,
                params.persona_id,
            ],
        )?;
        Ok(())
    }

    /// Delete a persona (cascades to tokens and generation params)
    pub fn delete(conn: &Connection, id: &str) -> Result<(), AppError> {
        let rows = conn.execute("DELETE FROM personas WHERE id = ?1", [id])?;
        if rows == 0 {
            return Err(AppError::NotFound(format!(
                "Persona with id '{id}' not found"
            )));
        }
        Ok(())
    }

    /// Check if a persona name already exists
    pub fn name_exists(
        conn: &Connection,
        name: &str,
        exclude_id: Option<&str>,
    ) -> Result<bool, AppError> {
        let exists: bool = match exclude_id {
            Some(id) => conn.query_row(
                "SELECT EXISTS(SELECT 1 FROM personas WHERE name = ?1 AND id != ?2)",
                params![name, id],
                |row| row.get(0),
            )?,
            None => conn.query_row(
                "SELECT EXISTS(SELECT 1 FROM personas WHERE name = ?1)",
                [name],
                |row| row.get(0),
            )?,
        };
        Ok(exists)
    }

    /// Create a persona from a request
    pub fn create(conn: &Connection, request: &CreatePersonaRequest) -> Result<Persona, AppError> {
        // Check if name already exists
        if Self::name_exists(conn, &request.name, None)? {
            return Err(AppError::Validation(format!(
                "A persona with name '{}' already exists",
                request.name
            )));
        }

        let persona = Persona::new(
            request.name.clone(),
            request.description.clone(),
            request.tags.clone(),
        );

        Self::insert(conn, &persona)?;

        Ok(persona)
    }
}
