<script lang="ts">
	import ShortContent from '@/components/shortContent.svelte';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();
</script>

<svelte:head>
	<title>{data.short.title} - {data.site.siteName}</title>
	<meta name="description" content={data.short.content} />
</svelte:head>

<ShortContent {...data.short} />
