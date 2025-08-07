<script lang="ts">
	import ArticleContent from '@/components/articleContent.svelte';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();
	let { siteName, intro, article } = data;
</script>

<svelte:head>
	<title>{article.title} - {siteName}</title>
	<meta name="description" content={intro} />
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
