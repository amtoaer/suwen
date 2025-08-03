import { request } from '@/api';
import type { ArticleBySlug } from '@/type';

export const load = async ({ params }) => {
	const { slug } = params;
	const article = await request<ArticleBySlug>(`/api/articles/${slug}`);
	return {
		article
	};
};
