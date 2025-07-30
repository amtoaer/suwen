<script lang="ts">
	import ArticleContent from '@/components/articleContent.svelte';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();
	let { title, description, articleData } = data;
</script>

<svelte:head>
	<title>{articleData.title} - {title}</title>
	<meta name="description" content={description} />
</svelte:head>

<ArticleContent
	title={articleData.title}
	content={articleData.content}
	summary={articleData.summary}
	publishedDate={articleData.publishedDate}
	tags={articleData.tags}
	views={articleData.views}
/>
