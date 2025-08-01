<script lang="ts">
	import { page } from '$app/state';
	let { sortTabs } = $props();

	const buildUrl = (sort: string) => {
		const url = new URL(page.url);
		if (sort) {
			url.searchParams.set('sort', sort);
		} else {
			url.searchParams.delete('sort');
		}
		return url.pathname + url.search;
	};
</script>

<div class="flex gap-4 mb-6">
	{#each sortTabs as tab}
		<a
			href={`${buildUrl(tab.query)}`}
			class="px-4 py-2 rounded-full text-sm font-medium transition-colors {(page.url.searchParams.get(
				'sort'
			) || '') === tab.query
				? 'bg-gray-900 text-white'
				: 'bg-gray-100 text-gray-600 hover:bg-gray-200'}"
		>
			{tab.name}
		</a>
	{/each}
</div>
