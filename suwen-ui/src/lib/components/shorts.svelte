<script lang="ts">
	import { ChevronRight, BookImage } from '@lucide/svelte';
	import * as Tooltip from '$lib/components/ui/tooltip/index.js';
	import ShortItem from './shortItem.svelte';

	let { shorts } = $props();
</script>

<div class="mb-8">
	<div class="flex items-center justify-between mb-4">
		<h2 class="text-lg font-semibold text-gray-900 flex items-center gap-2">
			<BookImage class="size-4" />
			图文
		</h2>
		<Tooltip.Provider delayDuration={0}>
			<Tooltip.Root>
				<Tooltip.Trigger>
					<a href="/shorts" class="text-sm text-gray-500 hover:underline">
						<ChevronRight class="w-5 h-5 text-gray-400" />
					</a>
				</Tooltip.Trigger>
				<Tooltip.Content>
					<p>更多图文</p>
				</Tooltip.Content>
			</Tooltip.Root>
		</Tooltip.Provider>
	</div>
	<div class="grid grid-cols-4 sm:grid-cols-8 gap-2 sm:gap-4">
		{#each shorts as short}
			<Tooltip.Provider delayDuration={0}>
				<Tooltip.Root>
					<Tooltip.Trigger
						><ShortItem
							slug={short.slug}
							image={short.coverImages[0]}
							title={short.title}
						/></Tooltip.Trigger
					>
					<Tooltip.Content>
						<p>{short.title}</p>
					</Tooltip.Content>
				</Tooltip.Root>
			</Tooltip.Provider>
		{/each}
	</div>
</div>
