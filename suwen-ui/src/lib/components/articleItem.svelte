<script lang="ts">
	import { Card, CardContent } from '@/components/ui/card';
	import { Badge } from '@/components/ui/badge';
	import { Eye, MessageSquare } from '@lucide/svelte';
	import { goto } from '$app/navigation';

	let {
		slug,
		coverImages,
		title,
		intro,
		summary,
		tags,
		viewCount,
		commentCount,
		publishedAt,
		lazy
	} = $props();

	let formatDate = (dateString: string) => {
		const now = new Date();
		const commentDate = new Date(dateString);
		const diffInSeconds = Math.floor((now.getTime() - commentDate.getTime()) / 1000);

		if (diffInSeconds < 60) return '刚刚';
		if (diffInSeconds < 3600) return `${Math.floor(diffInSeconds / 60)} 分钟前`;
		if (diffInSeconds < 86400) return `${Math.floor(diffInSeconds / 3600)} 小时前`;
		if (diffInSeconds < 2592000) return `${Math.floor(diffInSeconds / 86400)} 天前`;
		if (diffInSeconds < 31536000) return `${Math.floor(diffInSeconds / 2592000)} 个月前`;
		return `${Math.floor(diffInSeconds / 31536000)} 年前`;
	};
</script>

<a href={`/articles/${slug}`} class="block">
	<Card
		class="overflow-hidden hover:shadow-lg transition-shadow py-0 group size-full flex flex-col"
	>
		<div class="aspect-video overflow-hidden">
			<img
				fetchpriority="high"
				src={`https://amto.cc/cdn-cgi/image/width=500,height=300,fit=cover/${coverImages[0]}`}
				alt={title}
				class="size-full object-cover sm:group-hover:scale-105 sm:transition-transform sm:duration-400 sm:ease-in-out"
				{...lazy ? { loading: 'lazy' } : {}}
			/>
		</div>
		<CardContent class="px-4 pb-4 grid grid-rows-[auto_1fr_auto_auto] gap-2">
			<h3 class="font-semibold text-gray-900 min-h-[2.5rem] flex leading-tight">
				{title}
			</h3>
			<p class="text-gray-600 text-sm line-clamp-2 leading-relaxed">
				{intro || summary}
			</p>
			<div class="flex items-center justify-between text-xs text-gray-500">
				<div class="flex items-center gap-4">
					{#if tags.length > 0}
						<Badge
							class="text-xs bg-secondary/50 hover:bg-secondary text-secondary-foreground"
							onclick={(e) => {
								e.preventDefault();
								e.stopPropagation();
								goto(`/tags/${tags[0]}`);
							}}>{tags[0]}</Badge
						>
					{/if}
					<div class="flex items-center gap-1">
						<Eye class="w-3 h-3" />
						<span>{viewCount}</span>
					</div>
					<div class="flex items-center gap-1">
						<MessageSquare class="w-3 h-3" />
						<span>{commentCount}</span>
					</div>
				</div>
			</div>
			<div class="text-xs text-gray-400 pl-1">{formatDate(publishedAt)}</div>
		</CardContent>
	</Card>
</a>
