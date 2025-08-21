import type { Handle } from '@sveltejs/kit';

export const handle: Handle = async ({ event, resolve }) => {
	const response = await resolve(event);
	const setCookie = event.locals.setCookie;
	if (setCookie) {
		response.headers.set('set-cookie', setCookie);
	}
	return response;
};
