import { request } from '@/api.js';

export const load = async ({ fetch, params }) => {
	const tag = params.tag;
	const articles = await request(fetch, `/api/tags/${tag}/articles`);
	return {
		tag,
		articles
	};
};
