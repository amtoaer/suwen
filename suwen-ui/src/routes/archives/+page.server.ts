import { request } from '@/api';
import { type Archive, type TagWithCount } from '@/type';

export const load = async ({ fetch }) => {
	const [tags, archives] = await Promise.all([
		request<TagWithCount[]>(fetch, '/api/tags'),
		request<Array<[number, Archive[]]>>(fetch, '/api/archives')
	]);
	return { tags, archives };
};
