<script lang="ts">
	import { goto } from '$app/navigation';
	import { Avatar, AvatarImage } from '@/components/ui/avatar';
	import { Bell, Settings, ChevronDown } from '@lucide/svelte';

	let { avatar, naviTabs } = $props();
	let activeTab = $state('主页');
</script>

<nav class="px-8 py-4">
	<div class="flex items-center justify-between">
		<div class="flex items-center gap-8">
			{#each naviTabs as tab}
				<button
					onclick={() => {
						activeTab = tab.name;
						goto(tab.path);
					}}
					class="pb-2 text-sm font-medium border-b-2 transition-colors {activeTab === tab.name
						? 'text-red-500 border-red-500'
						: 'text-gray-600 border-transparent hover:text-gray-900'}"
				>
					{tab.name}
				</button>
			{/each}
		</div>
		<div class="flex items-center gap-4">
			<Bell class="w-5 h-5 text-gray-600" />
			<Settings class="w-5 h-5 text-gray-600" />
			<div class="flex items-center gap-2">
				<Avatar class="w-6 h-6">
					<AvatarImage src={avatar} alt="User" />
				</Avatar>
				<span class="text-sm text-gray-600">amtoaer</span>
				<ChevronDown class="w-4 h-4 text-gray-600" />
			</div>
		</div>
	</div>
</nav>
