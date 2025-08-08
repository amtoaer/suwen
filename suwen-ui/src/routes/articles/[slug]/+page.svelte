<script lang="ts">
	import ArticleContent from '@/components/articleContent.svelte';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();
	let { siteName, article } = data;
</script>

<svelte:head>
	<title>{article.title} - {siteName}</title>
</svelte:head>

<ArticleContent
	title={article.title}
	content={article.renderedHtml}
	toc={article.toc}
	summary={null}
	publishedDate={article.publishedAt}
	tags={article.tags}
	views={article.viewCount}
/>
