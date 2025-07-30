<script lang="ts">
	import ArticleItem from './articleItem.svelte';

	let activeFilter = $state('最新');

	let { articles } = $props();
</script>

<div class="flex gap-4 mb-6">
	{#each ['最新', '最热', '最多评论'] as filter}
		<button
			onclick={() => {
				activeFilter = filter;
			}}
			class="px-4 py-2 rounded-full text-sm font-medium transition-colors {activeFilter === filter
				? 'bg-gray-900 text-white'
				: 'bg-gray-100 text-gray-600 hover:bg-gray-200'}"
		>
			{filter}
		</button>
	{/each}
</div>

<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
	{#each articles as article}
		<ArticleItem {...article} />
	{/each}
</div>
