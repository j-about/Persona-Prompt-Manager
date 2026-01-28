<!--
@component
PhysicalCriteriaForm - Form for specifying detailed physical characteristics.

Provides categorized dropdowns for selecting physical attributes including
general body characteristics, hair, face structure, and body regions.
Used within AiPersonaForm to guide AI generation.

@property criteria - Bindable object containing selected physical criteria
-->
<script lang="ts">
	import { Card } from '$lib/components/ui';
	import type { PhysicalCriteria } from '$lib/types';

	interface Props {
		criteria: PhysicalCriteria;
	}

	let { criteria = $bindable({}) }: Props = $props();

	// Hair shade options based on main color (ordered light → dark)
	const hairShadeOptions: Record<string, string[]> = {
		Blonde: [
			'No preference',
			'Platinum',
			'Ash Blonde',
			'Champagne',
			'Golden Blonde',
			'Wheat',
			'Honey Blonde',
			'Sandy Blonde',
			'Strawberry Blonde',
			'Dirty Blonde'
		],
		Red: [
			'No preference',
			'Ginger',
			'Copper',
			'Auburn',
			'Fire Red',
			'Mahogany',
			'Cherry',
			'Ruby',
			'Burgundy'
		],
		Brown: [
			'No preference',
			'Caramel',
			'Hazelnut',
			'Light Brown',
			'Medium Brown',
			'Chestnut',
			'Mocha',
			'Dark Brown',
			'Chocolate',
			'Espresso'
		],
		Black: ['No preference', 'Brown-Black', 'Off Black', 'Soft Black', 'Blue-Black', 'Jet Black'],
		'Gray/White': [
			'No preference',
			'Pure White',
			'Platinum Gray',
			'Silver',
			'Salt & Pepper',
			'Steel Gray',
			'Charcoal'
		],
		Fantasy: ['No preference', 'Blue', 'Green', 'Pink', 'Purple', 'Teal', 'Multi-colored']
	};

	// Eye shade options based on main color (ordered light → dark)
	const eyeShadeOptions: Record<string, string[]> = {
		Gray: [
			'No preference',
			'Ice Gray',
			'Pale Gray',
			'Silver',
			'Steel Gray',
			'Slate Gray',
			'Storm Gray',
			'Ash Gray',
			'Charcoal',
			'Gunmetal'
		],
		Blue: [
			'No preference',
			'Ice Blue',
			'Powder Blue',
			'Sky Blue',
			'Cerulean',
			'Sapphire',
			'Steel Blue',
			'Royal Blue',
			'Cobalt',
			'Navy',
			'Midnight Blue'
		],
		Green: [
			'No preference',
			'Mint Green',
			'Sea Green',
			'Jade',
			'Emerald',
			'Forest Green',
			'Olive',
			'Moss Green',
			'Teal',
			'Viridian'
		],
		Hazel: [
			'No preference',
			'Golden Hazel',
			'Green-Gold Hazel',
			'Brown-Gold Hazel',
			'Amber-Flecked Hazel',
			'Copper Hazel',
			'Greenish Hazel',
			'Brownish Hazel'
		],
		Amber: [
			'No preference',
			'Honey Amber',
			'Golden Amber',
			'Copper Amber',
			'True Amber',
			'Wolf Amber'
		],
		Brown: [
			'No preference',
			'Light Brown',
			'Caramel',
			'Hazelnut',
			'Warm Brown',
			'Medium Brown',
			'Chocolate',
			'Mocha',
			'Coffee',
			'Espresso',
			'Dark Brown'
		],
		Heterochromia: [
			'No preference',
			'Complete (different colors)',
			'Central (ring around pupil)',
			'Sectoral (wedge pattern)'
		],
		Red: ['No preference', 'Pale Red', 'Crimson', 'Ruby', 'Bright Red', 'Blood Red', 'Scarlet'],
		Purple: ['No preference', 'Lavender', 'Lilac', 'Violet', 'Amethyst', 'Plum', 'Deep Purple'],
		Gold: [
			'No preference',
			'Pale Gold',
			'Bright Gold',
			'Molten Gold',
			'Burnished Gold',
			'Deep Gold'
		]
	};

	// Get available shades based on selected main color
	const availableShades = $derived(() => {
		const mainColor = criteria.hair?.color;
		if (!mainColor || mainColor === 'No preference') return [];
		return hairShadeOptions[mainColor] || [];
	});

	// Get available eye shades based on selected main eye color
	const availableEyeShades = $derived(() => {
		const mainColor = criteria.face?.eyeColor;
		if (!mainColor || mainColor === 'No preference') return [];
		return eyeShadeOptions[mainColor] || [];
	});

	// Field type definition
	interface Field {
		id: string;
		label: string;
		options: string[];
	}

	// Subgroup type definition
	interface Subgroup {
		label: string | null;
		fields: Field[];
	}

	// Category type definition
	interface Category {
		label: string;
		subgroups: Subgroup[];
	}

	// Physical criteria options organized by category with subgroups
	const granularityOptions: Record<string, Category> = {
		general: {
			label: 'General',
			subgroups: [
				{
					label: null,
					fields: [
						{
							id: 'sex',
							label: 'Sex',
							options: ['No preference', 'Male', 'Female']
						},
						{
							id: 'age',
							label: 'Age',
							options: [
								'No preference',
								'Infant (0-1)',
								'Toddler (1-3)',
								'Preschooler (3-6)',
								'Child (6-9)',
								'Preteen (10-12)',
								'Early teen (13-15)',
								'Late teen (16-19)',
								'Young adult (20-29)',
								'Adult (30-39)',
								'Early middle-aged (40-49)',
								'Late middle-aged (50-59)',
								'Early senior (60-69)',
								'Senior (70-79)',
								'Elderly (80-89)',
								'Very elderly (90+)'
							]
						}
					]
				},
				{
					label: 'Skin',
					fields: [
						{
							id: 'skinTone',
							label: 'Tone',
							options: [
								'No preference',
								'Fair',
								'Light',
								'Medium',
								'Olive',
								'Tan',
								'Brown',
								'Dark',
								'Ebony'
							]
						},
						{
							id: 'complexion',
							label: 'Complexion',
							options: ['No preference', 'Clear', 'Blemished', 'Sun-kissed', 'Rosy', 'Pale']
						},
						{
							id: 'skinTexture',
							label: 'Texture',
							options: [
								'No preference',
								'Smooth',
								'Poreless',
								'Textured',
								'Dewy',
								'Matte',
								'Glowing'
							]
						},
						{
							id: 'distinctiveMarks',
							label: 'Distinctive Marks',
							options: [
								'No preference',
								'Freckles',
								'Moles',
								'Dimples',
								'Beauty mark',
								'Scars',
								'Birthmarks',
								'Vitiligo'
							]
						}
					]
				},
				{
					label: 'Body',
					fields: [
						{
							id: 'bodyType',
							label: 'Type',
							options: [
								'No preference',
								'Petite',
								'Slim',
								'Athletic',
								'Average',
								'Muscular',
								'Curvy',
								'Plus-size'
							]
						},
						{
							id: 'height',
							label: 'Height',
							options: [
								'No preference',
								'Short',
								'Below average',
								'Average',
								'Above average',
								'Tall'
							]
						},
						{
							id: 'proportions',
							label: 'Proportions',
							options: ['No preference', 'Long-limbed', 'Proportionate', 'Compact', 'Lanky']
						},
						{
							id: 'posture',
							label: 'Posture',
							options: [
								'No preference',
								'Upright',
								'Relaxed',
								'Slouched',
								'Athletic',
								'Regal',
								'Hunched'
							]
						}
					]
				}
			]
		},
		hair: {
			label: 'Hair',
			subgroups: [
				{
					label: 'Color',
					fields: [
						{
							id: 'color',
							label: 'Main',
							options: ['No preference', 'Blonde', 'Red', 'Brown', 'Black', 'Gray/White', 'Fantasy']
						}
					]
				},
				{
					label: 'Attributes',
					fields: [
						{
							id: 'length',
							label: 'Length',
							options: [
								'No preference',
								'Bald',
								'Buzzcut',
								'Short',
								'Medium',
								'Long',
								'Very long',
								'Floor-length'
							]
						},
						{
							id: 'style',
							label: 'Style',
							options: [
								'No preference',
								'Straight',
								'Wavy',
								'Curly',
								'Braided',
								'Ponytail',
								'Bun',
								'Pigtails',
								'Messy',
								'Slicked back',
								'Mohawk',
								'Afro'
							]
						},
						{
							id: 'texture',
							label: 'Texture',
							options: ['No preference', 'Fine', 'Thick', 'Silky', 'Coarse', 'Fluffy', 'Shiny']
						}
					]
				}
			]
		},
		face: {
			label: 'Face',
			subgroups: [
				{
					label: 'Structure',
					fields: [
						{
							id: 'forehead',
							label: 'Forehead',
							options: ['No preference', 'High', 'Low', 'Wide', 'Narrow', 'Prominent']
						},
						{
							id: 'faceShape',
							label: 'Shape',
							options: [
								'No preference',
								'Oval',
								'Round',
								'Square',
								'Heart',
								'Long',
								'Diamond',
								'Oblong'
							]
						},
						{
							id: 'cheekbones',
							label: 'Cheekbones',
							options: ['No preference', 'High', 'Low', 'Prominent', 'Subtle', 'Hollow']
						},
						{
							id: 'jawline',
							label: 'Jawline',
							options: ['No preference', 'Square', 'Sharp', 'Rounded', 'V-line', 'Soft', 'Angular']
						},
						{
							id: 'chinShape',
							label: 'Chin Shape',
							options: [
								'No preference',
								'Pointed',
								'Square',
								'Round',
								'Cleft',
								'Protruding',
								'Receding'
							]
						}
					]
				},
				{
					label: 'Eyebrows',
					fields: [
						{
							id: 'eyebrowShape',
							label: 'Shape',
							options: [
								'No preference',
								'Arched',
								'Straight',
								'Rounded',
								'Angular',
								'S-shaped',
								'Soft',
								'Feathered'
							]
						}
					]
				},
				{
					label: 'Eyes',
					fields: [
						{
							id: 'eyeColor',
							label: 'Color',
							options: [
								'No preference',
								'Gray',
								'Blue',
								'Green',
								'Hazel',
								'Amber',
								'Brown',
								'Heterochromia',
								'Red',
								'Purple',
								'Gold'
							]
						},
						{
							id: 'eyeShape',
							label: 'Shape',
							options: [
								'No preference',
								'Almond',
								'Round',
								'Hooded',
								'Upturned',
								'Downturned',
								'Monolid',
								'Wide-set',
								'Close-set'
							]
						}
					]
				},
				{
					label: 'Nose',
					fields: [
						{
							id: 'noseShape',
							label: 'Shape',
							options: [
								'No preference',
								'Straight',
								'Aquiline/Roman',
								'Greek',
								'Button/Snub',
								'Hawk',
								'Celestial',
								'Bulbous',
								'Nubian/Wide',
								'Flat',
								'Crooked'
							]
						}
					]
				},
				{
					label: 'Mouth',
					fields: [
						{
							id: 'lipShape',
							label: 'Lip Shape',
							options: [
								'No preference',
								'Thin',
								'Full',
								"Cupid's bow",
								'Heart-shaped',
								'Wide',
								'Narrow',
								'Bow-shaped'
							]
						},
						{
							id: 'teeth',
							label: 'Teeth',
							options: [
								'No preference',
								'Straight',
								'Slightly crooked',
								'Gap-toothed',
								'Prominent canines',
								'Perfect'
							]
						}
					]
				}
			]
		},
		upperBody: {
			label: 'Upper Body',
			subgroups: [
				{
					label: null,
					fields: [
						{
							id: 'neck',
							label: 'Neck',
							options: ['No preference', 'Short', 'Average', 'Long', 'Slender', 'Thick']
						}
					]
				},
				{
					label: 'Torso',
					fields: [
						{
							id: 'build',
							label: 'Build',
							options: ['No preference', 'Slim', 'Toned', 'Muscular', 'Broad', 'Average']
						},
						{
							id: 'shoulders',
							label: 'Shoulders',
							options: ['No preference', 'Narrow', 'Average', 'Broad', 'Sloped', 'Square']
						},
						{
							id: 'back',
							label: 'Back',
							options: ['No preference', 'Narrow', 'Average', 'Broad', 'Athletic']
						},
						{
							id: 'chest',
							label: 'Chest/Bust',
							options: ['No preference', 'Small', 'Medium', 'Large', 'Full']
						}
					]
				},
				{
					label: 'Upper limbs',
					fields: [
						{
							id: 'arms',
							label: 'Arms',
							options: ['No preference', 'Slender', 'Toned', 'Muscular', 'Average']
						},
						{
							id: 'hands',
							label: 'Hands',
							options: [
								'No preference',
								'Slender',
								'Average',
								'Broad',
								'Delicate',
								'Strong',
								'Calloused'
							]
						},
						{
							id: 'nails',
							label: 'Nails',
							options: ['No preference', 'Short', 'Long', 'Manicured', 'Natural', 'Painted']
						}
					]
				}
			]
		},
		midsection: {
			label: 'Midsection',
			subgroups: [
				{
					label: null,
					fields: [
						{
							id: 'waist',
							label: 'Waist',
							options: ['No preference', 'Narrow', 'Average', 'Wide', 'Hourglass']
						},
						{
							id: 'hips',
							label: 'Hips',
							options: ['No preference', 'Narrow', 'Average', 'Wide', 'Curvy']
						}
					]
				}
			]
		},
		lowerBody: {
			label: 'Lower Body',
			subgroups: [
				{
					label: null,
					fields: [
						{
							id: 'legs',
							label: 'Legs',
							options: [
								'No preference',
								'Short',
								'Average',
								'Long',
								'Slender',
								'Athletic',
								'Muscular'
							]
						},
						{
							id: 'build',
							label: 'Build',
							options: ['No preference', 'Slim', 'Toned', 'Athletic', 'Average']
						},
						{
							id: 'feet',
							label: 'Feet',
							options: ['No preference', 'Small', 'Average', 'Large', 'Slender', 'Wide']
						}
					]
				}
			]
		}
	};

	function updateField(granularity: string, fieldId: string, value: string) {
		const granKey = granularity as keyof PhysicalCriteria;
		if (!criteria[granKey]) {
			criteria[granKey] = {};
		}
		if (value === 'No preference') {
			// @ts-expect-error - dynamic assignment
			delete criteria[granKey][fieldId];
		} else {
			// @ts-expect-error - dynamic assignment
			criteria[granKey][fieldId] = value;
		}

		// Reset shade when main color changes
		if (granularity === 'hair' && fieldId === 'color') {
			if (criteria.hair) {
				delete criteria.hair.colorShade;
			}
		}

		// Reset eye shade when main eye color changes
		if (granularity === 'face' && fieldId === 'eyeColor') {
			if (criteria.face) {
				delete criteria.face.eyeColorShade;
			}
		}
	}

	function updateHairShade(value: string) {
		if (!criteria.hair) {
			criteria.hair = {};
		}
		if (value === 'No preference') {
			delete criteria.hair.colorShade;
		} else {
			criteria.hair.colorShade = value;
		}
	}

	function updateEyeShade(value: string) {
		if (!criteria.face) {
			criteria.face = {};
		}
		if (value === 'No preference') {
			delete criteria.face.eyeColorShade;
		} else {
			criteria.face.eyeColorShade = value;
		}
	}

	function getFieldValue(granularity: string, fieldId: string): string {
		const granKey = granularity as keyof PhysicalCriteria;
		const granData = criteria[granKey];
		if (!granData) return 'No preference';
		// @ts-expect-error - dynamic access
		return granData[fieldId] ?? 'No preference';
	}
</script>

<Card>
	<h2 class="mb-4 text-lg font-semibold text-base-content">Physical Criteria (Optional)</h2>
	<p class="mb-4 text-xs text-base-content/60">
		Specify physical characteristics by category. Leave as "No preference" to let AI decide based on
		description.
	</p>

	<div class="space-y-6">
		{#each Object.entries(granularityOptions) as [granularityId, category] (granularityId)}
			<div class="bg-base-200/50 p-4">
				<h3 class="mb-3 text-sm font-semibold text-base-content">{category.label}</h3>

				{#each category.subgroups as subgroup, subgroupIndex (subgroupIndex)}
					{#if subgroup.label}
						<div
							class="mb-1 text-xs font-medium tracking-wide text-base-content/50 uppercase"
							class:mt-3={subgroupIndex > 0}
						>
							{subgroup.label}
						</div>
					{/if}

					<div class="grid grid-cols-4 gap-x-4 gap-y-2">
						{#each subgroup.fields as field (field.id)}
							<div class="flex items-center gap-1">
								<label
									for="{granularityId}-{field.id}"
									class="min-w-20 text-xs whitespace-nowrap text-base-content/70"
								>
									{field.label}:
								</label>
								<select
									id="{granularityId}-{field.id}"
									class="select-bordered select min-w-0 flex-1 select-xs"
									value={getFieldValue(granularityId, field.id)}
									onchange={(e) => updateField(granularityId, field.id, e.currentTarget.value)}
								>
									{#each field.options as option (option)}
										<option value={option}>{option}</option>
									{/each}
								</select>
							</div>

							<!-- Hair shade selector (shown immediately after hair color field) -->
							{#if granularityId === 'hair' && field.id === 'color' && availableShades().length > 0}
								<div class="flex items-center gap-1">
									<label
										for="hair-colorShade"
										class="min-w-20 text-xs whitespace-nowrap text-base-content/70"
									>
										Shade:
									</label>
									<select
										id="hair-colorShade"
										class="select-bordered select min-w-0 flex-1 select-xs"
										value={criteria.hair?.colorShade ?? 'No preference'}
										onchange={(e) => updateHairShade(e.currentTarget.value)}
									>
										{#each availableShades() as shade (shade)}
											<option value={shade}>{shade}</option>
										{/each}
									</select>
								</div>
							{/if}

							<!-- Eye shade selector (shown immediately after eye color field) -->
							{#if granularityId === 'face' && field.id === 'eyeColor' && availableEyeShades().length > 0}
								<div class="flex items-center gap-1">
									<label
										for="face-eyeColorShade"
										class="min-w-20 text-xs whitespace-nowrap text-base-content/70"
									>
										Shade:
									</label>
									<select
										id="face-eyeColorShade"
										class="select-bordered select min-w-0 flex-1 select-xs"
										value={criteria.face?.eyeColorShade ?? 'No preference'}
										onchange={(e) => updateEyeShade(e.currentTarget.value)}
									>
										{#each availableEyeShades() as shade (shade)}
											<option value={shade}>{shade}</option>
										{/each}
									</select>
								</div>
							{/if}
						{/each}
					</div>

					{#if subgroupIndex < category.subgroups.length - 1}
						<div class="my-3 border-t border-base-300/50"></div>
					{/if}
				{/each}
			</div>
		{/each}
	</div>
</Card>
