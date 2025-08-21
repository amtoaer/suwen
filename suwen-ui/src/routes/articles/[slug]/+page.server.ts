import { request } from '@/api';
import type { ArticleBySlug } from '@/type';

export const load = async ({ fetch, params }) => {
	const { slug } = params;
	const article = await request<ArticleBySlug>(fetch, `/api/articles/${slug}`);
	return {
		article
	};
};
