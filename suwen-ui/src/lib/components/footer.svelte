<script lang="ts">
	import { Button } from '@/components/ui/button';
	import { Languages } from '@lucide/svelte';

	let { author, relatedLinks } = $props();
</script>

<footer class="mt-auto py-6 px-8">
	<div class="flex flex-col sm:flex-row items-start sm:items-center gap-4 sm:gap-0">
		<div class="flex items-center gap-2 text-sm text-gray-600 sm:flex-1">
			<span>© {author}</span>
			<span>•</span>
			<div class="flex items-center gap-1">
				<span>由 <a href="https://github.com/amtoaer/suwen">SuWen</a> 提供支持</span>
			</div>
		</div>

		<div class="flex items-center gap-2">
			{#each relatedLinks as relatedLink}
				<Button
					variant="ghost"
					size="icon"
					href={relatedLink.url}
					aria-label={relatedLink.name}
					target="_blank"
					class="h-5 w-5 sm:h-10 sm:w-10"
				>
					<img src={relatedLink.icon} alt={relatedLink.name} class="w-4 h-4" />
				</Button>
			{/each}
		</div>

		<div class=" flex sm:flex-1 sm:justify-end">
			<Button variant="ghost" size="icon" class="h-5 w-5 sm:h-10 sm:w-10">
				<Languages class="w-4 h-4" />
			</Button>
		</div>
	</div>
</footer>
