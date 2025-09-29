<script lang="ts">
	import mediumZoom from 'medium-zoom';
	import ArticleContent from '@/components/articleContent.svelte';
	import type { PageProps } from './$types';
	import Comments from '@/components/comments.svelte';
	import { request } from '@/api';
	import { type Comment } from '@/type';

	let { data }: PageProps = $props();

	let siteName = $derived(data.site.siteName);
	let article = $derived(data.article);
	let slug = $derived(data.slug);

	let comments = $state(data.comments || []);
	let liked = $state(data.liked || false);
	let viewCount = $state(data.article.viewCount || 0);
	let likeCount = $state(data.article.likeCount || 0);

	$effect(() => {
		comments = data.comments || [];
		liked = data.liked || false;
		viewCount = article.viewCount || 0;
		likeCount = article.likeCount || 0;
	});

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
	{slug}
	title={article.title}
	content={article.renderedHtml}
	toc={article.toc}
	summary={article.summary}
	publishedDate={article.publishedAt}
	tags={article.tags}
	views={viewCount}
	likes={likeCount}
	{comments}
	{liked}
	setViews={(v: number) => (viewCount = v)}
	setLikes={(l: number) => (likeCount = l)}
	setLiked={(b: boolean) => (liked = b)}
/>
<Comments
	{comments}
	articleSlug={data.slug}
	me={data.me}
	refreshComments={async () => {
		comments = await request<Comment[]>(fetch, `/api/articles/${data.slug}/comments`);
	}}
/>
