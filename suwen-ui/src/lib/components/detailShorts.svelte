<script lang="ts">
	import { ChevronRight, BookImage } from '@lucide/svelte';
	import ShortItem from './shortItem.svelte';
	import Sortbar from './sortbar.svelte';
	import DetailShortItem from './detailShortItem.svelte';

	let { shorts } = $props();
</script>

<Sortbar
	sortTabs={[
		{ name: '最新', query: '' },
		{ name: '最热', query: 'hot' },
		{ name: '最多评论', query: 'most-commented' }
	]}
/>
<div>
	<div class="grid gap-3 grid-cols-1 sm:grid-cols-4">
		{#each shorts as short}
			<DetailShortItem
				key={short.key}
				images={short.images}
				title={short.title}
				content={short.content}
			/>
		{/each}
	</div>
</div>
