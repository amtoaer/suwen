export interface Site {
	siteName: string;
	intro: string;
	displayName: string;
	avatarUrl: string;
	keywords: string[];
	relatedLinks: RelatedLink[];
	tabs: Tab[];
}

export interface RelatedLink {
	name: string;
	url: string;
}

export interface Tab {
	name: string;
	path: string;
}

export interface ArticleByList {
	slug: string;
	title: string;
	coverImages: string[];
	tags: string[];
	viewCount: number;
	commentCount: number;
	publishedAt: string; // ISO date string
}

export interface ArticleBySlug {
	title: string;
	renderedHtml: string;
	tags: string[];
	viewCount: number;
	commentCount: number;
	publishedAt: string; // ISO date string
}

export interface Short {
	slug: string;
	title: string;
	coverImages: string[];
	content: string;
}

export interface TagWithCount {
	name: string;
	count: number;
}

export interface Archive {
	slug: string;
	title: string;
	publishedAt: string; // ISO date string
}

export interface ApiResponse<T> {
	statusCode: number;
	data?: T;
	message?: string;
}

export type ApiError = ApiResponse<never>;
