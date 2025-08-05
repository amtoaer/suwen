import { request } from '@/api';
import type { ArticleByList, Short } from '@/type';

export const load = async ({ url }) => {
	const sort = url.searchParams.get('sort');
	const [shorts, articles] = await Promise.all([
		request<Short[]>('/api/shorts'),
		request<ArticleByList[]>('/api/articles' + (sort ? `?sort=${sort}` : ''))
	]);
	return {
		shorts,
		articles
	};
};
