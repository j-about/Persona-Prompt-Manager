/**
 * Tauri IPC service wrapper
 *
 * Provides a typed wrapper around Tauri's invoke function
 * with error handling and logging.
 */

import { invoke } from '@tauri-apps/api/core';

/**
 * Invoke a Tauri command with typed parameters and return value
 *
 * @param command - The command name to invoke
 * @param args - Optional arguments to pass to the command
 * @returns Promise resolving to the command's return value
 * @throws Error if the command fails
 */
export async function tauriInvoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
	try {
		const result = await invoke<T>(command, args);
		return result;
	} catch (error) {
		console.error(`Tauri command "${command}" failed:`, error);
		throw error;
	}
}
