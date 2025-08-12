<script lang="ts">
	import { Card, CardContent } from '@/components/ui/card';
	import { Badge } from '@/components/ui/badge';
	import { Eye, MessageSquare } from '@lucide/svelte';
	import { goto } from '$app/navigation';

	let { slug, coverImages, title, description, tags, viewCount, commentCount, publishedAt, lazy } =
		$props();

	let formatDate = (date: Date) => {
		const now = new Date();
		const diff = now.getTime() - date.getTime();
		const minutes = Math.floor(diff / (1000 * 60));
		if (minutes < 60) {
			return `${minutes} 分钟前`;
		}
		const hours = Math.floor(minutes / 60);
		if (hours < 24) {
			return `${hours} 小时前`;
		}
		const days = Math.floor(hours / 24);
		if (days < 30) {
			return `${days} 天前`;
		}
		const months = Math.floor(days / 30);
		if (months < 12) {
			return `${months} 个月前`;
		}
		const years = Math.floor(months / 12);
		return `${years} 年前`;
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
				draggable="false"
				{...lazy ? { loading: 'lazy' } : {}}
			/>
		</div>
		<CardContent class="px-4 pb-4 flex flex-col flex-1">
			<h3 class="font-semibold text-gray-900 mb-2">{title}</h3>
			<p class="text-gray-600 text-sm mb-3 line-clamp-2 flex-1">
				{description}
			</p>
			<div class="flex items-center justify-between text-xs text-gray-500 mt-auto">
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
			<div class="text-xs text-gray-400 mt-2 pl-1">{formatDate(new Date(publishedAt))}</div>
		</CardContent>
	</Card>
</a>
