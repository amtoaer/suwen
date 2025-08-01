<script lang="ts">
	import Articles from '@/components/articles.svelte';
	import type { PageProps } from './$types';
	import { Archive } from '@lucide/svelte';
	let { data }: PageProps = $props();

	let { tags, groupedArchives } = data;
</script>

<div class="flex items-center mb-6">
	<Archive class="size-7 mr-2 " />
	<h1 class="text-center text-xl sm:text-3xl font-extrabold">归档</h1>
</div>

<h2 class="text-lg mb-4">标签</h2>
<div class="flex flex-wrap gap-2 mb-6">
	{#each tags as tag}
		<a
			href={`/tags/${tag.name}`}
			class="px-3 py-1 bg-gray-200 rounded-full text-sm text-gray-800 hover:bg-gray-300"
		>
			{tag.name} ({tag.count})
		</a>
	{/each}
</div>

{#each Object.entries(groupedArchives) as [year, articles]}
	<h2 class="text-lg mb-4">{year} 年</h2>
	{#each articles as article}
		<a
			href={`/articles/${article.key}`}
			class="flex justify-between items-center py-2 px-3 hover:bg-gray-100 rounded-lg transition-colors"
		>
			<span>{article.title}</span>
			<span class="text-gray-500 text-sm">{article.publishedDate.toLocaleDateString()}</span>
		</a>
	{/each}
{/each}
