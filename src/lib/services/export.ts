/**
 * Export service - Tauri IPC wrapper for database import/export operations
 */

import { tauriInvoke } from './tauri';
import type { ExportResult, ImportResult } from '$lib/types';

/**
 * Export the database to a user-selected location.
 * Opens a native save dialog.
 */
export async function exportDatabase(): Promise<ExportResult> {
	return tauriInvoke<ExportResult>('export_database');
}

/**
 * Import a database from a user-selected file.
 * Opens a native open dialog. Validates schema version before import.
 * WARNING: This replaces all existing data!
 */
export async function importDatabase(): Promise<ImportResult> {
	return tauriInvoke<ImportResult>('import_database');
}
