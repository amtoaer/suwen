<script lang="ts">
	import { Avatar, AvatarImage } from '@/components/ui/avatar';
	import { Bell, CirclePlus, ChevronDown } from '@lucide/svelte';
	import { page } from '$app/state';
	let { avatar, naviTabs, displayName } = $props();
</script>

<nav class="px-8 py-4">
	<div class="flex items-center justify-between">
		<div class="flex overflow-x-auto no-scrollbar space-x-8 border-b border-none">
			{#each naviTabs as tab}
				<a
					href={tab.url}
					class="inline-flex items-center h-10 whitespace-nowrap transition-colors relative text-sm font-medium {page
						.url.pathname === tab.url
						? 'text-red-500'
						: 'text-gray-600 hover:text-gray-900'}"
				>
					{tab.name}
					{#if page.url.pathname === tab.url}
						<span class="absolute inset-x-0 bottom-0 bg-red-500 h-[2px]"></span>
					{/if}
				</a>
			{/each}
		</div>
		<div class="relative flex items-center -mr-2">
			<Bell class="w-5 h-5 text-gray-500 cursor-pointer" />
			<CirclePlus class="w-5 h-5 ml-2 text-gray-500 cursor-pointer hidden sm:block" />
			<div class="h-full w-[2px] py-1 ml-3">
				<div class="w-full h-6 bg-gray-200 rounded-full"></div>
			</div>
			<button
				class="flex items-center hover:bg-gray-50 transition-colors py-1 px-2 rounded-lg ml-2"
				type="button"
			>
				<Avatar class="w-7 h-7">
					<AvatarImage fetchpriority="high" src={avatar} alt="User" />
				</Avatar>
				<span
					class="ml-2 text-left leading-none font-medium truncate text-gray-600 text-sm hidden sm:block"
					>{displayName}</span
				>
				<ChevronDown class="w-4 h-4 text-gray-600 ml-1" />
			</button>
		</div>
	</div>
</nav>
