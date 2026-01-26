<!--
@component
MultiSelect - Dropdown with checkboxes for multiple selection.

Uses daisyUI's details/dropdown pattern with checkboxes for selecting
multiple options. Shows count of selected items or placeholder when closed.
Supports disabled options that cannot be selected.
-->
<script lang="ts">
	/** Option with value and optional disabled state */
	interface SelectOption {
		value: string;
		disabled?: boolean;
	}

	/**
	 * @property options - Available options to select from (with optional disabled state)
	 * @property selected - Currently selected option values
	 * @property placeholder - Text shown when no options selected
	 * @property onchange - Callback when selection changes
	 */
	interface Props {
		options: SelectOption[];
		selected: string[];
		placeholder?: string;
		onchange: (selected: string[]) => void;
	}

	let { options, selected, placeholder = 'Select...', onchange }: Props = $props();

	let detailsElement = $state<HTMLDetailsElement | null>(null);

	/** Clears all selected options */
	function deselectAll() {
		onchange([]);
	}

	/** Toggles selection of a single option */
	function toggle(option: SelectOption) {
		if (option.disabled) return;
		if (selected.includes(option.value)) {
			onchange(selected.filter((s) => s !== option.value));
		} else {
			onchange([...selected, option.value]);
		}
	}

	/** Display text for the dropdown trigger */
	const displayText = $derived(
		selected.length > 0
			? selected.length === 1
				? selected[0]
				: `${selected.length} selected`
			: placeholder
	);
</script>

<details bind:this={detailsElement} class="dropdown">
	<summary class="btn w-37 gap-1 btn-sm">
		<span class="truncate">{displayText}</span>
		<svg class="h-4 w-4 fill-current" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20">
			<path
				fill-rule="evenodd"
				d="M5.293 7.293a1 1 0 011.414 0L10 10.586l3.293-3.293a1 1 0 111.414 1.414l-4 4a1 1 0 01-1.414 0l-4-4a1 1 0 010-1.414z"
				clip-rule="evenodd"
			/>
		</svg>
	</summary>
	<ul
		class="dropdown-content menu z-10 max-h-60 overflow-y-auto rounded-box bg-base-100 p-2 shadow"
	>
		<li>
			<button
				type="button"
				class="text-sm {selected.length === 0
					? 'cursor-not-allowed text-base-content/40'
					: 'text-primary'}"
				onclick={deselectAll}
				disabled={selected.length === 0}
			>
				Deselect all
			</button>
		</li>
		<li></li>
		{#each options as option (option.value)}
			<li>
				<label
					class="label cursor-pointer justify-start gap-2 py-1 {option.disabled
						? 'opacity-50'
						: ''}"
				>
					<input
						type="checkbox"
						class="checkbox checkbox-sm checkbox-primary"
						checked={selected.includes(option.value)}
						disabled={option.disabled}
						onchange={() => toggle(option)}
					/>
					<span class="label-text">{option.value}</span>
				</label>
			</li>
		{/each}
	</ul>
</details>
