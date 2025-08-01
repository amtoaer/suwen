<script lang="ts">
	import type { PageProps } from './$types';
	import DetailShorts from '@/components/detailShorts.svelte';

	let { data }: PageProps = $props();

	let { shorts } = data;
</script>

<DetailShorts {shorts} />
