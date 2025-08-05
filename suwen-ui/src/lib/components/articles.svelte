<script lang="ts">
	import ArticleItem from './articleItem.svelte';
	import Sortbar from './sortbar.svelte';

	let activeFilter = $state('最新');

	let { articles } = $props();
</script>

<Sortbar
	sortTabs={[
		{ name: '最新', query: '' },
		{ name: '最热', query: 'trending' },
		{ name: '最多评论', query: 'top-comments' }
	]}
/>

<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
	{#each articles as article}
		<ArticleItem {...article} />
	{/each}
</div>
