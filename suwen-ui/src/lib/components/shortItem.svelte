<script lang="ts">
	let { title, image, width = 96, height = 96 } = $props();
</script>

<div class="w-24 h-24 rounded-lg overflow-hidden bg-gray-100">
	<img
		src={image}
		alt={title}
		{width}
		{height}
		class="w-full h-full object-cover"
		draggable="false"
	/>
</div>
