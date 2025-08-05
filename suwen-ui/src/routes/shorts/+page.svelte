<script lang="ts">
	import type { PageProps } from './$types';
	import DetailShorts from '@/components/detailShorts.svelte';

	let { data }: PageProps = $props();

	let shorts = $derived(data.shorts);
</script>

<DetailShorts {shorts} />
