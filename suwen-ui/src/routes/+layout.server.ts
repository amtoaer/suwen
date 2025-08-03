import { request } from '@/api';
import { type Site } from '@/type';

export const load = async () => {
	const data = request<Site>('/api/site', {
		method: 'GET'
	});
	return data;
};
