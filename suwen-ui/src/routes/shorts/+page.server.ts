import { request } from '@/api';
import type { Short } from '@/type';

export const load = async ({ url }) => {
	const sort = url.searchParams.get('sort');
	const shorts = await request<Short[]>('/api/shorts' + (sort ? `?sort=${sort}` : ''));
	return {
		shorts
	};
};
