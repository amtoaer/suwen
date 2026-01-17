<script lang="ts">
	import { Bell, CirclePlus, ChevronDown } from '@lucide/svelte';
	import { page } from '$app/state';
	import FallbackAvatar from './fallbackAvatar.svelte';
	let { avatar, naviTabs, displayName } = $props();
</script>

<nav class="px-8 py-4">
	<div class="flex items-center justify-between">
		<div class="flex overflow-x-auto no-scrollbar space-x-8 border-b border-none">
			{#each naviTabs as tab (tab.url)}
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
				class="flex items-center hover:bg-gray-50 transition-colors py-1 px-2 rounded-lg ml-2 text-gray-600"
				type="button"
			>
				<FallbackAvatar {avatar} {displayName} class="w-7 h-7" />
				<span class="ml-2 text-left leading-none font-medium truncate text-sm hidden sm:block"
					>{displayName}</span
				>
				<ChevronDown class="w-4 h-4 ml-1" />
			</button>
		</div>
	</div>
</nav>

{#if page.url.pathname === '/'}
	<div class="px-8 py-4">
		<div class="bg-red-50 rounded-md px-8 py-4 text-sm">
			<span>
				该博客系统正在开发中，开发进度见
				<a href="https://github.com/amtoaer/suwen" class="text-red-500 hover:underline">GitHub</a>。
			</span>
		</div>
	</div>
{/if}
