import { request } from '@/api';
import type { Short } from '@/type';

export const load = async ({ fetch, params }) => {
	const { slug } = params;
	const short = await request<Short>(fetch, `/api/shorts/${slug}`);
	return {
		short
	};
};
