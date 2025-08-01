<script lang="ts">
	import Articles from '@/components/articles.svelte';
	import type { PageProps } from './$types';
	import { Tag } from '@lucide/svelte';
	let { data }: PageProps = $props();

	let { tag, articles } = data;
</script>

<div class="flex items-center mb-6">
	<Tag class="size-7 mr-2 " />
	<h1 class="text-center text-xl sm:text-3xl font-extrabold">{tag}</h1>
</div>

<Articles {articles} />
