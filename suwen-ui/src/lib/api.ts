import type { ApiResponse } from './type';

export async function rawRequest(
	fetch: typeof window.fetch,
	url: string,
	options?: {
		method?: string;
		json?: unknown;
		query?: Record<string, string>;
		headers?: Record<string, string>;
		[key: string]: unknown;
	}
): Promise<Response> {
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
	return response;
}

export async function request<T>(
	fetch: typeof window.fetch,
	requestUrl: string,
	options?: {
		method?: string;
		json?: unknown;
		query?: Record<string, string>;
		headers?: Record<string, string>;
		[key: string]: unknown;
	}
): Promise<T> {
	const response = await rawRequest(fetch, requestUrl, options);
	return await extractApiResponse<T>(response);
}

export async function extractApiResponse<T>(response: Response): Promise<T> {
	const apiResponse: ApiResponse<T> = await response.json();
	if (apiResponse.statusCode >= 400 || apiResponse.data === undefined) {
		throw new Error(apiResponse.message || `API Error: ${apiResponse.statusCode}`);
	}
	return apiResponse.data;
}
