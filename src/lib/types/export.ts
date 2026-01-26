/**
 * Export/Import types for SQLite database operations
 */

/** Result of a database export operation */
export interface ExportResult {
	/** Whether the export completed successfully */
	success: boolean;
	/** Path where the file was saved (if successful) */
	path?: string | null;
	/** Error message (if failed) */
	error?: string | null;
}

/** Result of a database import operation */
export interface ImportResult {
	/** Whether the import was successful */
	success: boolean;
	/** Number of personas in the imported database */
	personas_count: number;
	/** Error message (if failed) */
	error?: string | null;
}
