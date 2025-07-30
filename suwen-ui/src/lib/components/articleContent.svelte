<script lang="ts">
	import { Eye } from '@lucide/svelte';
	import { Badge } from './ui/badge';

	let { title, content, summary, publishedDate, views, tags } = $props();
</script>

<div class="prose max-w-4xl mx-auto">
	<h2 class="text-center items-center relative text-3xl font-bold mb-8">
		{title}
	</h2>
	<div class="flex items-center justify-center gap-4 text-sm text-gray-500">
		<span>{publishedDate.toLocaleDateString('zh-CN')}</span>
		<div class="flex items-center gap-2">
			{#each tags as tag}
				<Badge variant="secondary" class="text-xs">#{tag}</Badge>
			{/each}
		</div>
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
		<div class="absolute right-full h-full lg:block hidden pr-10">
			<div class="sticky top-2/3 flex flex-col gap-4">
				<button
					class="flex flex-col items-center gap-1 p-2 hover:bg-gray-100 rounded-lg transition-colors"
				>
					<svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="2"
							d="M4.318 6.318a4.5 4.5 0 000 6.364L12 20.364l7.682-7.682a4.5 4.5 0 00-6.364-6.364L12 7.636l-1.318-1.318a4.5 4.5 0 00-6.364 0z"
						/>
					</svg>
					<span class="text-xs text-gray-500">点赞</span>
				</button>
				<button
					class="flex flex-col items-center gap-1 p-2 hover:bg-gray-100 rounded-lg transition-colors"
				>
					<svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="2"
							d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z"
						/>
					</svg>
					<span class="text-xs text-gray-500">评论</span>
				</button>
			</div>
		</div>
		<div class="absolute left-full pl-8 w-64 h-full top-0 lg:block hidden">
			<div class="sticky top-8">
				<h4 class="text-sm font-semibold text-gray-700 mb-4">目录</h4>
				<nav class="space-y-2">
					<a
						href="#section1"
						class="block text-sm text-gray-600 hover:text-gray-900 py-1 border-l-2 border-transparent hover:border-blue-500 pl-3 transition-colors"
					>
						章节标题 1
					</a>
					<a
						href="#section2"
						class="block text-sm text-gray-600 hover:text-gray-900 py-1 border-l-2 border-transparent hover:border-blue-500 pl-3 transition-colors"
					>
						章节标题 2
					</a>
					<a
						href="#section3"
						class="block text-sm text-gray-600 hover:text-gray-900 py-1 border-l-2 border-transparent hover:border-blue-500 pl-3 transition-colors"
					>
						章节标题 3
					</a>
				</nav>
			</div>
		</div>
		<div>
			{@html content}
		</div>
	</div>
</div>
