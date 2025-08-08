<script lang="ts">
	import { Avatar, AvatarImage } from '@/components/ui/avatar';
	import { Bell, Settings, ChevronDown } from '@lucide/svelte';
	import { page } from '$app/state';
	let { avatar, naviTabs } = $props();
</script>

<nav class="px-8 py-4">
	<div class="flex items-center justify-between">
		<div class="flex items-center gap-8">
			{#each naviTabs as tab}
				<a
					href={tab.url}
					class="pb-2 text-sm font-medium border-b-2 transition-colors {page.url.pathname ===
					tab.url
						? 'text-red-500 border-red-500'
						: 'text-gray-600 border-transparent hover:text-gray-900'}"
				>
					{tab.name}
				</a>
			{/each}
		</div>
		<div class="flex items-center gap-4">
			<Bell class="w-5 h-5 text-gray-600" />
			<Settings class="w-5 h-5 text-gray-600" />
			<div class="flex items-center gap-2">
				<Avatar class="w-6 h-6">
					<AvatarImage fetchpriority="high" src={avatar} alt="User" />
				</Avatar>
				<span class="text-sm text-gray-600">amtoaer</span>
				<ChevronDown class="w-4 h-4 text-gray-600" />
			</div>
		</div>
	</div>
</nav>
