<script lang="ts">
	import Sortbar from './sortbar.svelte';
	import DetailShortItem from './detailShortItem.svelte';

	let { shorts } = $props();
</script>

<Sortbar
	sortTabs={[
		{ name: '最新', query: '' },
		{ name: '最热', query: 'trending' },
		{ name: '最多评论', query: 'top-comments' }
	]}
/>
<div>
	<div class="grid grid-cols-1 sm:grid-cols-4 gap-8 sm:gap-5">
		{#each shorts as short}
			<DetailShortItem
				slug={short.slug}
				coverImages={short.coverImages}
				title={short.title}
				content={short.content}
			/>
		{/each}
	</div>
</div>
