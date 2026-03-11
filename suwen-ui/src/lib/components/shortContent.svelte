<script lang="ts">
	import {
		Carousel,
		CarouselContent,
		CarouselItem,
		CarouselPrevious,
		CarouselNext
	} from '@/components/ui/carousel';

	let { coverImages, title, content, renderedHtml } = $props();
</script>

<article class="max-w-4xl mx-auto">
	<Carousel
		class="relative mb-5 group rounded-md overflow-hidden"
		opts={{
			align: 'start',
			loop: true
		}}
	>
		<CarouselPrevious class="top-1/2 left-4 z-10" />
		<CarouselContent>
			{#each coverImages as image (image)}
				<CarouselItem>
					<img fetchpriority="high" src={image} alt={title} class="object-cover size-full" />
				</CarouselItem>
			{/each}
		</CarouselContent>
		<CarouselNext class="top-1/2 right-4 z-10" />
	</Carousel>
	<span class="prose">
		<h3>{title}</h3>
		<!-- eslint-disable-next-line -->
		<p>{@html renderedHtml || content}</p>
	</span>
</article>
