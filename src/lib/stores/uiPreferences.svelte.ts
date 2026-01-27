/**
 * UI Preferences Store - Svelte 5 runes-based state management
 *
 * Manages both session-only preferences (lost on app close) and
 * persistent preferences (stored in JSON file via tauri-plugin-store).
 *
 * Session preferences: personaListTags (filter state, survives navigation)
 * Persistent preferences: personaListSort (database-backed, survives restart)
 */

import { LazyStore } from '@tauri-apps/plugin-store';

/** Preference keys for file storage */
const Keys = {
	PERSONA_LIST_SORT: 'personaListSort'
} as const;

/** Default values for preferences */
const DEFAULTS = {
	personaListSort: 'updated_at-desc' as string,
	personaListTags: [] as string[]
};

/** Create the UI preferences store */
function createUiPreferencesStore() {
	// Lazy store for file-based persistence (connects on first access)
	const fileStore = new LazyStore('preferences.json', {
		autoSave: 100,
		defaults: {
			[Keys.PERSONA_LIST_SORT]: DEFAULTS.personaListSort
		}
	});

	// === Session-only state (survives navigation, lost on app close) ===
	let personaListTags = $state<string[]>([...DEFAULTS.personaListTags]);

	// === Persistent state (file-backed) ===
	let personaListSort = $state(DEFAULTS.personaListSort);

	// Loading state
	let isInitialized = $state(false);

	/**
	 * Initialize the store by loading persistent preferences from file.
	 * Should be called once at app startup.
	 */
	async function initialize(): Promise<void> {
		if (isInitialized) return;

		try {
			const storedSort = await fileStore.get<string>(Keys.PERSONA_LIST_SORT);
			if (storedSort) {
				personaListSort = storedSort;
			}
			isInitialized = true;
		} catch (err) {
			console.error('Failed to load UI preferences:', err);
			isInitialized = true;
		}
	}

	// === Session Preference Actions ===

	/**
	 * Set the selected tags for persona list filtering.
	 * Session-only - survives navigation but lost on app close.
	 */
	function setPersonaListTags(tags: string[]): void {
		personaListTags = [...tags];
	}

	// === Persistent Preference Actions ===

	/**
	 * Set the sort value for persona list.
	 * Persisted to file - survives app restart.
	 */
	async function setPersonaListSort(value: string): Promise<void> {
		personaListSort = value;
		try {
			await fileStore.set(Keys.PERSONA_LIST_SORT, value);
		} catch (err) {
			console.error('Failed to save sort preference:', err);
		}
	}

	return {
		// State getters
		get personaListTags() {
			return personaListTags;
		},
		get personaListSort() {
			return personaListSort;
		},
		get isInitialized() {
			return isInitialized;
		},

		// Actions
		initialize,
		setPersonaListTags,
		setPersonaListSort
	};
}

// Export a singleton instance
export const uiPreferencesStore = createUiPreferencesStore();
