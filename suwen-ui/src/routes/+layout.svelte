<script lang="ts">
	import '../app.css';
	import type { LayoutProps } from './$types';
	import Header from '@/components/header.svelte';
	import Navibar from '@/components/navibar.svelte';
	import Footer from '@/components/footer.svelte';
	import { ProgressBar } from '@prgm/sveltekit-progress-bar';
	let { data, children }: LayoutProps = $props();

	let me = $state(data.me);
	const { siteName, intro, avatarUrl, relatedLinks, displayName, tabs } = data.site;
</script>

<svelte:head>
	<link rel="icon" href={avatarUrl} />
</svelte:head>

<ProgressBar
	color="var(--color-red-400)"
	displayThresholdMs={0}
	intervalTime={200}
	settleTime={200}
/>

<div class="min-h-screen bg-white/80">
	<div class="max-w-5xl mx-auto min-h-screen flex flex-col">
		<Header avatar={avatarUrl} title={siteName} description={intro} {relatedLinks} />
		<Navibar avatar={me.avatarUrl} displayName={me.displayName} naviTabs={tabs} />
		<div class="px-8 py-3">
			{@render children()}
		</div>
		<Footer author={displayName} {relatedLinks} />
	</div>
</div>
