<script lang="ts">
	import ShortContent from '@/components/shortContent.svelte';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();
</script>

<svelte:head>
	<title>{data.short.title} - {data.siteName}</title>
</svelte:head>

<ShortContent {...data.short} />
