import type { ApiResponse } from './type';
import { browser } from '$app/environment';

export async function request<T>(
	url: string,
	options?: {
		method?: string;
		json?: unknown;
		query?: Record<string, string>;
		headers?: Record<string, string>;
		[key: string]: unknown;
	}
): Promise<T> {
	if (!browser && !url.startsWith('http')) {
		url = `http://localhost:3000${url.startsWith('/') ? '' : '/'}${url}`;
	}
	let requestUrl = url;
	const requestOptions: RequestInit = {
		method: options?.method || 'GET',
		headers: {
			'Content-Type': 'application/json',
			...options?.headers
		}
	};
	if (options?.query) {
		const searchParams = new URLSearchParams(options.query);
		requestUrl += `?${searchParams.toString()}`;
	}
	if (options?.json) {
		requestOptions.body = JSON.stringify(options.json);
	}
	Object.assign(requestOptions, options, {
		method: requestOptions.method,
		headers: requestOptions.headers
	});
	const response = await fetch(requestUrl, requestOptions);
	const apiResponse: ApiResponse<T> = await response.json();
	if (apiResponse.statusCode >= 400 || !apiResponse.data) {
		throw new Error(apiResponse.message || `API Error: ${apiResponse.statusCode}`);
	}
	return apiResponse.data;
}
