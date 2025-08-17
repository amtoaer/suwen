<script lang="ts">
	import type { PageProps } from './$types';
	import DetailShorts from '@/components/detailShorts.svelte';

	let { data }: PageProps = $props();
	let { siteName, intro } = data;

	let shorts = $derived(data.shorts);
</script>

<svelte:head>
	<title>图文 - {siteName}</title>
	<meta name="description" content={intro} />
</svelte:head>

<DetailShorts {shorts} />
