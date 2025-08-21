<script lang="ts">
	import mediumZoom from 'medium-zoom';
	import ArticleContent from '@/components/articleContent.svelte';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();
	let { siteName } = data.site;

	let article = $derived(data.article);

	$effect(() => {
		if (article.renderedHtml) {
			mediumZoom('article img');
		}
	});
</script>

<svelte:head>
	<title>{article.title} - {siteName}</title>
	<meta name="description" content={article.intro || article.summary} />
</svelte:head>

<ArticleContent
	title={article.title}
	content={article.renderedHtml}
	toc={article.toc}
	summary={article.summary}
	publishedDate={article.publishedAt}
	tags={article.tags}
	views={article.viewCount}
/>
