<script lang="ts">
	import { Eye } from '@lucide/svelte';
	import { Badge } from './ui/badge';
	import { onMount } from 'svelte';
	import { MessageSquareCode, ThumbsUp } from '@lucide/svelte';
	import type { TocItem } from '@/type';
	import { browser } from '$app/environment';

	let { title = null, content, summary = null, publishedDate, views, tags = null, toc } = $props();

	let contentElement: HTMLElement;

	let activeId = $state('');
	let tocMaxWidth = $state(0);

	let headings = $derived.by(() => {
		if (browser && toc) {
			return toc.map((item: TocItem) => document.getElementById(item.id)).filter(Boolean);
		}
		return [];
	});

	$effect(() => {
		if (!contentElement || !content) return;
		handleScroll();
	});

	const handleScroll = () => {
		for (let i = headings.length - 1; i >= 0; i--) {
			const heading = headings[i];
			if (heading && heading.getBoundingClientRect().top <= 50) {
				activeId = heading.id;
				return;
			}
		}
		activeId = '';
	};

	const handleResize = () => {
		const padding = 20;
		const availableWidth = (window.innerWidth - contentElement.clientWidth) / 2 - padding;
		tocMaxWidth = availableWidth;
	};

	onMount(() => {
		handleScroll();
		handleResize();
		window.addEventListener('scroll', handleScroll);
		window.addEventListener('resize', handleResize);
		return () => {
			window.removeEventListener('scroll', handleScroll);
			window.removeEventListener('resize', handleResize);
		};
	});
</script>

<article class="prose max-w-4xl mx-auto">
	<h1 class="text-center text-xl sm:text-3xl">
		{title}
	</h1>
	<div class="flex items-center justify-center gap-4 text-sm text-gray-500">
		<span>{new Date(publishedDate).toLocaleDateString('zh-CN')}</span>
		{#if tags && tags.length > 0}
			<div class="flex items-center gap-2">
				{#each tags as tag}
					<Badge
						variant="secondary"
						class="text-xs bg-secondary/50 hover:bg-secondary text-secondary-foreground no-underline"
						href="/tags/{tag}">#{tag}</Badge
					>
				{/each}
			</div>
		{/if}
		<div class="flex items-center gap-1">
			<Eye class="w-4 h-4" />
			<span>{views}</span>
		</div>
	</div>
	{#if summary}
		<div class="border rounded-lg my-8 bg-gray-50 px-8">
			<h3 class="text-lg font-semibold mb-2">AI 摘要</h3>
			<p class="text-gray-600">{summary}</p>
		</div>
	{/if}
	<div class="relative">
		<aside class="absolute right-full h-full lg:block hidden pr-14">
			<div class="sticky top-2/3 flex flex-col gap-4">
				<button
					class="flex flex-col items-center gap-1 p-2 hover:bg-gray-100 rounded-lg transition-colors"
				>
					<ThumbsUp class="w-6 h-6" />
					<span class="text-xs text-gray-500">点赞</span>
				</button>
				<button
					class="flex flex-col items-center gap-1 p-2 hover:bg-gray-100 rounded-lg transition-colors"
				>
					<svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<MessageSquareCode class="w-6 h-6" />
					</svg>
					<span class="text-xs text-gray-500">评论</span>
				</button>
			</div>
		</aside>
		{#if toc.length > 0}
			<aside
				class="absolute left-full h-full lg:block hidden pl-14"
				style="max-width: {tocMaxWidth}px;"
			>
				<div class="sticky top-8 truncate">
					<nav class="space-y-1">
						{#each toc as item}
							<a
								href="#{item.id}"
								class="block text-left text-sm py-1 border-l-2 transition-colors no-underline {activeId ===
								item.id
									? 'text-blue-600 border-blue-500 bg-blue-50'
									: 'text-gray-600 hover:text-gray-900 border-transparent hover:border-blue-500'}"
								style="padding-left: {item.level * 12 + 12}px;"
							>
								<span class="truncate block">{item.text}</span>
							</a>
						{/each}
					</nav>
				</div>
			</aside>
		{/if}
		<main bind:this={contentElement}>
			{@html content}
		</main>
	</div>
</article>
