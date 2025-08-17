export interface Site {
	siteName: string;
	intro: string;
	displayName: string;
	avatarUrl: string;
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

export interface TocItem {
	id: string;
	text: string;
	level: number;
}

export interface ArticleByList {
	slug: string;
	title: string;
	intro: string | null;
	summary: string | null;
	coverImages: string[];
	tags: string[];
	viewCount: number;
	commentCount: number;
	publishedAt: string; // ISO date string
}

export interface ArticleBySlug {
	title: string;
	renderedHtml: string;
	summary: string | null;
	intro: string | null;
	toc: TocItem[];
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
	tagName: string;
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
