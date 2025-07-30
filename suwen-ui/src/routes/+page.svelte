<script lang="ts">
	import Shorts from '@/components/shorts.svelte';
	import type { PageProps } from './$types';
	import Articles from '@/components/articles.svelte';

	let { data }: PageProps = $props();
	let { title, description, shorts, articles } = data;
</script>

<svelte:head>
	<title>{title}</title>
	<meta name="description" content={description} />
</svelte:head>

<Shorts {shorts} />
<Articles {articles} />
