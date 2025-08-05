import { request } from '@/api';
import type { Short } from '@/type';

export const load = async ({ params }) => {
	const { slug } = params;
	const short = await request<Short>(`/api/shorts/${slug}`);
	return {
		short
	};
};
