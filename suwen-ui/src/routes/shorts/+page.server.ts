import { request } from '@/api';
import type { Short } from '@/type';

export const load = async ({ fetch, url }) => {
	const sort = url.searchParams.get('sort');
	const shorts = await request<Short[]>(fetch, '/api/shorts' + (sort ? `?sort=${sort}` : ''));
	return {
		shorts
	};
};
