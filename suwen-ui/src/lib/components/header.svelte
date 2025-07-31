<script lang="ts">
	import { Avatar, AvatarImage } from '$lib/components/ui/avatar';
	import { Button } from '$lib/components/ui/button';
	import { Rss } from '@lucide/svelte';
	let { avatar, title, description, socials } = $props();
</script>

<header class="px-8 pt-10 pb-4">
	<div class="flex items-start justify-between mb-6">
		<div class="flex items-start gap-6">
			<Avatar class="w-30 h-30 border-4 border-white shadow-lg">
				<AvatarImage src={avatar} alt="Profile" draggable="false" />
			</Avatar>
			<div class="flex-1">
				<h1 class="text-2xl font-bold text-gray-900 mb-2">{title}</h1>
				<p class="text-gray-600 mt-2 mb-5">{description}</p>
				<div class="flex items-center gap-3">
					{#each socials as social}
						<Button
							variant="ghost"
							size="icon"
							href={social.url}
							aria-label={social.name}
							target="_blank"
						>
							<img src={social.icon} alt={social.name} class="w-4 h-4" />
						</Button>
					{/each}
				</div>
			</div>
		</div>
		<div class="flex items-center gap-3">
			<Button variant="ghost" size="icon">
				<Rss class="w-5 h-5" />
			</Button>
		</div>
	</div>
</header>
