<script lang="ts">
	import { Button } from '@/components/ui/button';

	let { author, relatedLinks } = $props();
</script>

<footer class="mt-auto py-6 px-8">
	<div class="flex items-center justify-between">
		<div class="flex items-center gap-2 text-sm text-gray-600">
			<span>© {author}</span>
			<span>•</span>
			<div class="flex items-center gap-1">
				<span>SuWen 提供支持</span>
			</div>
		</div>

		<div class="flex items-center gap-4">
			{#each relatedLinks as relatedLink}
				<Button
					variant="ghost"
					size="icon"
					href={relatedLink.url}
					aria-label={relatedLink.name}
					target="_blank"
				>
					<img src={relatedLink.icon} alt={relatedLink.name} class="w-4 h-4" />
				</Button>
			{/each}
		</div>

		<div class="flex items-center gap-2">
			<Button variant="ghost" size="icon" class="w-8 h-8 text-gray-600 hover:text-gray-900">
				<svg class="w-5 h-5" viewBox="0 0 24 24" fill="currentColor">
					<path
						d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-1 17.93c-3.94-.49-7-3.85-7-7.93 0-.62.08-1.21.21-1.79L9 15v1c0 1.1.9 2 2 2v1.93zm6.9-2.54c-.26-.81-1-1.39-1.9-1.39h-1v-3c0-.55-.45-1-1-1H8v-2h2c.55 0 1-.45 1-1V7h2c1.1 0 2-.9 2-2v-.41c2.93 1.19 5 4.06 5 7.41 0 2.08-.8 3.97-2.1 5.39z"
					/>
				</svg>
			</Button>
			<Button variant="ghost" size="icon" class="w-8 h-8 text-gray-600 hover:text-gray-900">
				<svg class="w-5 h-5" viewBox="0 0 24 24" fill="currentColor">
					<path d="M3 18h18v-2H3v2zm0-5h18v-2H3v2zm0-7v2h18V6H3z" />
				</svg>
			</Button>
		</div>
	</div>
</footer>
