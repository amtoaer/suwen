import { request } from '@/api';
import type { ArticleByList, Short } from '@/type';

export const load = async () => {
	const [shorts, articles] = await Promise.all([
		request<Short[]>('/api/shorts'),
		request<ArticleByList[]>('/api/articles')
	]);
	return {
		shorts,
		articles
	};
};
