<script lang="ts">
	import { Avatar, AvatarFallback, AvatarImage } from '$lib/components/ui/avatar';
	let { avatar, displayName, class: className = '' } = $props();
</script>

<Avatar class={className}>
	{#if avatar}
		<AvatarImage fetchpriority="high" src={avatar} alt="User" />
	{/if}
	<AvatarFallback>{displayName.at(0)}</AvatarFallback>
</Avatar>
