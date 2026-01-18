/** @type {import('prettier').Config & import('prettier-plugin-tailwindcss').PluginOptions} */
export default {
	// Plugins - prettier-plugin-tailwindcss MUST be last
	plugins: ['prettier-plugin-svelte', 'prettier-plugin-tailwindcss'],

	// Tailwind CSS v4 configuration
	tailwindStylesheet: './src/app.css',

	// Match existing code style
	printWidth: 100,
	tabWidth: 2,
	useTabs: true,
	semi: true,
	singleQuote: true,
	trailingComma: 'none',
	bracketSpacing: true,
	arrowParens: 'always',
	endOfLine: 'lf',

	// Svelte options
	svelteAllowShorthand: true,
	svelteSortOrder: 'options-scripts-markup-styles',
	svelteIndentScriptAndStyle: true,

	overrides: [
		{
			files: '*.svelte',
			options: { parser: 'svelte' }
		}
	]
};
