<script lang="ts">
	import ShortContent from '@/components/shortContent.svelte';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();
</script>

<ShortContent {...data.short} />
