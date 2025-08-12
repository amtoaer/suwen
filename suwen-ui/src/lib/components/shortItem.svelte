<script lang="ts">
	let { slug, title, image, width = 96, height = 96 } = $props();
</script>

<a class="w-24 h-24 rounded-lg overflow-hidden bg-gray-100" href={`/shorts/${slug}`}>
	<img
		fetchpriority="high"
		src={`https://amto.cc/cdn-cgi/image/width=100,height=100,fit=cover/${image}`}
		alt={title}
		{width}
		{height}
		class="w-full h-full object-cover sm:hover:scale-105 sm:transition-transform sm:duration-400 sm:ease-in-out"
		draggable="false"
	/>
</a>
