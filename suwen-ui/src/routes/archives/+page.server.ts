export const load = async () => {
	return {
		tags: [
			{
				name: 'Rust',
				count: 3
			},
			{
				name: 'Svelte',
				count: 2
			},
			{
				name: 'JavaScript',
				count: 5
			},
			{
				name: 'Web Development',
				count: 4
			}
		],
		groupedArchives: {
			'2023': [
				{
					key: 'how-to-use-async-await-in-rust',
					title: '如何在 Rust 中使用 async/await',
					publishedDate: new Date('2023-10-01')
				},
				{
					key: 'understanding-svelte-reactiveness',
					title: '深入理解 Svelte 的反应性',
					publishedDate: new Date('2023-09-15')
				}
			]
		}
	};
};
