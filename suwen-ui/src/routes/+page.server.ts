import { request } from '@/api';
import type { ArticleByList, Short } from '@/type';

export const load = async ({ fetch, url }) => {
	const sort = url.searchParams.get('sort');
	const [shorts, articles] = await Promise.all([
		request<Short[]>(fetch, '/api/shorts'),
		request<ArticleByList[]>(fetch, '/api/articles' + (sort ? `?sort=${sort}` : ''))
	]);
	return {
		shorts,
		articles
	};
};
