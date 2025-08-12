import { request } from '@/api.js';

export const load = async ({ params }) => {
	const tag = params.tag;
	const articles = await request(`/api/tags/${tag}/articles`);
	return {
		tag,
		articles
	};
};
