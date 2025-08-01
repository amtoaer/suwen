<script lang="ts">
	import { ChevronRight, BookImage } from '@lucide/svelte';
	import ShortItem from './shortItem.svelte';

	let { shorts } = $props();
</script>

<div class="mb-8">
	<div class="flex items-center justify-between mb-4">
		<h2 class="text-lg font-semibold text-gray-900 flex items-center gap-2">
			<BookImage class="size-4" />
			图文
		</h2>
		<ChevronRight class="w-5 h-5 text-gray-400" />
	</div>
	<div class="flex gap-4">
		{#each shorts as short}
			<ShortItem key={short.key} image={short.image} title={short.title} />
		{/each}
	</div>
</div>
