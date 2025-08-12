import { request } from '@/api';
import { type Archive, type TagWithCount } from '@/type';

export const load = async () => {
	const [tags, archives] = await Promise.all([
		request<TagWithCount[]>('/api/tags'),
		request<Array<[number, Archive[]]>>('/api/archives')
	]);
	return { tags, archives };
};
