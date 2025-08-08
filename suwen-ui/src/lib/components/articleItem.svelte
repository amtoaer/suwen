<script lang="ts">
	import { Card, CardContent } from '@/components/ui/card';
	import { Badge } from '@/components/ui/badge';
	import { Eye, MessageSquare } from '@lucide/svelte';
	import { goto } from '$app/navigation';

	let { slug, coverImages, title, description, tags, viewCount, commentCount, publishedAt } =
		$props();

	let formatDate = (date: Date) => {
		const now = new Date();
		const diff = now.getTime() - date.getTime();
		const minutes = Math.floor(diff / (1000 * 60));
		if (minutes < 60) {
			return `${minutes}分钟前`;
		}
		const hours = Math.floor(minutes / 60);
		if (hours < 24) {
			return `${hours}小时前`;
		}
		const days = Math.floor(hours / 24);
		return `${days}天前`;
	};
</script>

<a href={`/articles/${slug}`} class="block">
	<Card class="overflow-hidden hover:shadow-lg transition-shadow !py-0 group size-full">
		<span class="h-48 overflow-hidden">
			<img
				fetchpriority="high"
				loading="lazy"
				src={coverImages[0]}
				alt={title}
				class="size-full object-cover sm:group-hover:scale-110 sm:transition-transform sm:duration-400 sm:ease-in-out"
				draggable="false"
			/>
		</span>
		<CardContent class="px-4 pb-4">
			<h3 class="font-semibold text-gray-900 mb-2">{title}</h3>
			<p class="text-gray-600 text-sm mb-3 line-clamp-2">
				{description}
			</p>
			<div class="flex items-center justify-between text-xs text-gray-500">
				<div class="flex items-center gap-4">
					<Badge
						class="text-xs bg-secondary/50 hover:bg-secondary text-secondary-foreground"
						onclick={(e) => {
							e.preventDefault();
							e.stopPropagation();
							goto(`/tags/${tags[0]}`);
						}}>{tags[0]}</Badge
					>
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
			<div class="text-xs text-gray-400 mt-2">{formatDate(new Date(publishedAt))}</div>
		</CardContent>
	</Card>
</a>
