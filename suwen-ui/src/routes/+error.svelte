<svelte:options runes={true} />

<script lang="ts">
	import { page } from '$app/state';
</script>

<h1>{page.status}</h1>
<h2>{page.error?.message}</h2>
<a href="/">返回首页</a>
