<script lang="ts">
	import { Avatar, AvatarImage } from '$lib/components/ui/avatar';
	import { Button } from '$lib/components/ui/button';
	import { Rss } from '@lucide/svelte';
	let { avatar, title, description, relatedLinks } = $props();
</script>

<header class="px-4 sm:px-8 pt-15 sm:pt-10 pb-4">
	<div class="flex items-start justify-between mb-4 sm:mb-6">
		<div class="flex items-start gap-4 sm:gap-6">
			<Avatar class="w-20 h-20 sm:w-30 sm:h-30 border-3 sm:border-4 border-white shadow-lg">
				<AvatarImage fetchpriority="high" src={avatar} alt="Profile" draggable="false" />
			</Avatar>
			<div class="flex-1">
				<h1 class="text-xl sm:text-2xl font-bold text-gray-900 mb-2">{title}</h1>
				<p class="text-sm sm:text-base text-gray-600 mt-2 mb-3 sm:mb-5">{description}</p>
				<div class="flex items-center gap-3 pt-1 sm:pt-0">
					{#each relatedLinks as relatedLink}
						<Button
							variant="ghost"
							size="icon"
							href={relatedLink.url}
							aria-label={relatedLink.name}
							target="_blank"
							class="h-5 w-5 sm:h-10 sm:w-10"
						>
							<img
								fetchpriority="high"
								src={relatedLink.icon}
								alt={relatedLink.name}
								class="w-4 h-4"
							/>
						</Button>
					{/each}
				</div>
			</div>
		</div>
		<div class="flex items-center pr-3 mt-1">
			<Button variant="ghost" size="icon" class="h-5 w-5 sm:h-10 sm:w-10">
				<Rss class="w-4 h-4" />
			</Button>
		</div>
	</div>
</header>
