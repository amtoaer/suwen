<script lang="ts">
	import Shorts from '@/components/shorts.svelte';
	import type { PageProps } from './$types';
	import Articles from '@/components/articles.svelte';

	let { data }: PageProps = $props();
	let { siteName, intro, shorts, articles } = data;
</script>

<svelte:head>
	<title>{siteName}</title>
	<meta name="description" content={intro} />
</svelte:head>

<Shorts {shorts} />
<Articles {articles} />
