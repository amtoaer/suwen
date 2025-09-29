import { extractApiResponse, rawRequest, request } from '@/api';
import { type IdentityInfo, type Site } from '@/type';

export const load = async ({ fetch, locals }) => {
	const resp = await rawRequest(fetch, '/api/me');
	const setCookie = resp.headers.get('set-cookie');
	if (setCookie) {
		locals.setCookie = setCookie;
	}
	const me: IdentityInfo = await extractApiResponse<IdentityInfo>(resp);
	const site = await request<Site>(fetch, '/api/site');
	return { me, site };
};
