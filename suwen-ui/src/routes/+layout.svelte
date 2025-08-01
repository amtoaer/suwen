<script lang="ts">
	import '../app.css';
	import type { LayoutProps } from './$types';
	import Header from '@/components/header.svelte';
	import Navibar from '@/components/navibar.svelte';
	import Footer from '@/components/footer.svelte';
	import { ProgressBar } from '@prgm/sveltekit-progress-bar';
	let { data, children }: LayoutProps = $props();
	const { title, description, avatar, socials, author, naviTabs } = data;
</script>

<ProgressBar color="#7F57F1" zIndex={100} intervalTime={100} />
<div class="min-h-screen bg-white/80">
	<div class="max-w-5xl mx-auto min-h-screen flex flex-col">
		<Header {avatar} {title} {description} {socials} />
		<Navibar {avatar} {naviTabs} />
		<div class="px-8 py-3">
			{@render children()}
		</div>
		<Footer {author} {socials} />
	</div>
</div>
