<script lang="ts">
	import { type CarouselAPI } from '$lib/components/ui/carousel/context.js';
	import {
		Carousel,
		CarouselContent,
		CarouselItem,
		CarouselPrevious,
		CarouselNext
	} from '@/components/ui/carousel';

	let { key, images, title, content } = $props();
	let api = $state<CarouselAPI>();
</script>

<div>
	<a href={`/shorts/${key}`} draggable="false">
		<Carousel
			setApi={(emblaApi) => (api = emblaApi)}
			class="relative mb-5 group rounded-md overflow-hidden"
			opts={{
				align: 'start',
				loop: true
			}}
		>
			<CarouselPrevious
				class="top-1/2 left-4 z-10 cursor-pointer"
				onclick={(e) => {
					e.preventDefault();
					e.stopPropagation();
					if (api) {
						api.scrollPrev();
					}
				}}
			/>
			<CarouselContent class="h-48">
				{#each images as image}
					<CarouselItem class="h-full">
						<img src={image} alt={title} loading="lazy" class="object-cover size-full" />
					</CarouselItem>
				{/each}
			</CarouselContent>
			<CarouselNext
				class="top-1/2 right-4 z-10 cursor-pointer"
				onclick={(e) => {
					e.preventDefault();
					e.stopPropagation();
					if (api) {
						api.scrollNext();
					}
				}}
			/>
		</Carousel>

		<span class="line-clamp-2">
			<h2 class="font-bold">{title}</h2>
			<p class="text-gray-500 text-sm">{content}</p>
		</span>
	</a>
</div>
