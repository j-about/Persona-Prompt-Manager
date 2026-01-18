/**
 * Export service - Tauri IPC wrapper for import/export operations
 */

import { tauriInvoke } from './tauri';
import type { BulkExport, ImportOptions, ImportResult } from '$lib/types';

/** Export all personas */
export async function exportAllPersonas(): Promise<BulkExport> {
	return tauriInvoke<BulkExport>('export_all_personas');
}

/** Import multiple personas from a bulk export */
export async function importPersonas(
	exportData: BulkExport,
	options: ImportOptions
): Promise<ImportResult[]> {
	return tauriInvoke<ImportResult[]>('import_personas', { export: exportData, options });
}

/** Parse JSON string to import */
export async function parseImportJson(json: string): Promise<BulkExport> {
	return tauriInvoke<BulkExport>('parse_import_json', { json });
}

/** Download export as JSON file (browser-side) */
export function downloadAsJson(data: BulkExport, filename?: string): void {
	const json = JSON.stringify(data, null, 2);
	const blob = new Blob([json], { type: 'application/json' });
	const url = URL.createObjectURL(blob);
	const a = document.createElement('a');
	a.href = url;
	a.download = filename || `ppm-export-${new Date().toISOString().slice(0, 10)}.json`;
	document.body.appendChild(a);
	a.click();
	document.body.removeChild(a);
	URL.revokeObjectURL(url);
}
