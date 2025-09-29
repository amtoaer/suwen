import { request } from '@/api';
import type { ArticleBySlug, Comment } from '@/type';

export const load = async ({ fetch, params }) => {
	const { slug } = params;
	const [article, comments, likes] = await Promise.all([
		request<ArticleBySlug>(fetch, `/api/articles/${slug}`),
		request<Comment[]>(fetch, `/api/articles/${slug}/comments`),
		request<boolean>(fetch, `/api/articles/${slug}/likes`)
	]);
	return {
		article,
		comments,
		slug,
		liked: likes
	};
};
