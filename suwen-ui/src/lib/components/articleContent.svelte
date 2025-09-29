<script lang="ts">
	import { Eye, Sparkles } from '@lucide/svelte';
	import { Badge } from './ui/badge';
	import { onMount } from 'svelte';
	import { MessageSquareCode, ThumbsUp } from '@lucide/svelte';
	import type { TocItem } from '@/type';
	import { browser } from '$app/environment';
	import { request } from '@/api';

	let {
		slug = null,
		title = null,
		content,
		summary = null,
		publishedDate,
		views,
		likes,
		tags = null,
		toc,
		comments,
		liked,
		setViews,
		setLikes,
		setLiked
	} = $props();

	let contentElement: HTMLElement | null = null;

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
		request<number>(fetch, `/api/articles/${slug}/views`, {
			method: 'POST'
		}).then((newViews) => {
			setViews(newViews);
		});
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
		const availableWidth = (window.innerWidth - contentElement!.clientWidth) / 2 - padding;
		tocMaxWidth = availableWidth;
	};

	const likeArticleContent = async () => {
		try {
			const likeCount = await request<number>(fetch, `/api/articles/${slug}/likes`, {
				method: 'POST',
				json: { like: !liked }
			});
			setLiked(!liked);
			setLikes(likeCount);
		} catch (error) {
			console.error('Error liking the article:', error);
		}
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

<article class="prose max-w-4xl mx-auto prose-video:rounded-md prose-img:rounded-md">
	<h1 class="text-center text-xl sm:text-3xl">
		{title}
	</h1>
	<div class="flex items-center justify-center gap-4 text-sm text-gray-500">
		<span>{new Date(publishedDate).toLocaleDateString('zh-CN')}</span>
		{#if tags && tags.length > 0}
			<div class="flex items-center gap-2">
				{#each tags as tag (tag)}
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
		<div class="border rounded-lg my-8 bg-gray-50 px-6">
			<h3 class="text-lg font-semibold flex items-center">
				<Sparkles class="w-4 h-4 mr-2" />
				AI 摘要
			</h3>
			<p class="text-gray-600 whitespace-pre-wrap">{summary}</p>
		</div>
	{/if}
	<div class="relative">
		<aside class="absolute right-full h-full lg:block hidden pr-14">
			<div class="sticky top-2/3 flex flex-col gap-4">
				<button
					class="flex flex-col items-center gap-1 p-2 hover:bg-gray-100 rounded-lg transition-colors"
					onclick={likeArticleContent}
				>
					<ThumbsUp class="w-6 h-6" color={liked ? 'red' : undefined} />
					<span class="text-xs text-gray-500">{likes}</span>
				</button>
				<button
					class="flex flex-col items-center gap-1 p-2 hover:bg-gray-100 rounded-lg transition-colors"
					onclick={() => {
						const commentsSection = document.getElementById('comments');
						if (commentsSection) {
							commentsSection.scrollIntoView({ behavior: 'smooth' });
						}
					}}
				>
					<svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<MessageSquareCode class="w-6 h-6" />
					</svg>
					<span class="text-xs text-gray-500">{comments.length}</span>
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
						{#each toc as item (item.id)}
							<a
								href="#{item.id}"
								class="block text-left text-sm py-1 border-l-2 transition-colors no-underline {activeId ===
								item.id
									? 'text-red-600 border-red-500 bg-red-50'
									: 'text-gray-600 hover:text-gray-900 border-transparent hover:border-red-500'}"
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
