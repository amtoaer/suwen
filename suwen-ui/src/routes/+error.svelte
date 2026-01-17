<svelte:options runes={true} />

<script lang="ts">
	import { page } from '$app/state';
	import { TriangleAlert } from '@lucide/svelte';

	const status = $derived(page.status);
	const message = $derived(page.error?.message || '');

	const getErrorTitle = (code: number) => {
		switch (code) {
			case 404:
				return '页面未找到';
			case 403:
				return '禁止访问';
			case 500:
				return '服务器错误';
			case 503:
				return '服务不可用';
			default:
				return '出错了';
		}
	};

	const getErrorDescription = (code: number) => {
		switch (code) {
			case 404:
				return '抱歉，您访问的页面不存在或已被移除';
			case 403:
				return '抱歉，您没有权限访问此页面';
			case 500:
				return '服务器遇到了一些问题，请稍后再试';
			case 503:
				return '服务暂时不可用，请稍后再试';
			default:
				return '发生了一些意外错误';
		}
	};
</script>

<svelte:head>
	<title>{status} - {getErrorTitle(status)}</title>
</svelte:head>

<div class="flex flex-col items-center px-4 py-8">
	<TriangleAlert class="w-12 h-12 text-red-500 mb-3" strokeWidth={1.5} />

	<h1 class="text-4xl font-bold text-gray-900 mb-2">
		{status}
	</h1>

	<h2 class="text-lg font-medium text-gray-700 mb-1">
		{getErrorTitle(status)}
	</h2>

	<p class="text-sm text-gray-500 text-center mb-6">
		{getErrorDescription(status)}
	</p>

	{#if message}
		<div class="mb-6 px-3 py-2 bg-red-50 rounded-md border border-red-100 max-w-xl">
			<p class="text-xs text-red-600 font-mono break-words">
				{message}
			</p>
		</div>
	{/if}
</div>
