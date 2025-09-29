<script lang="ts">
	import type { Comment, IdentityInfo } from '@/type';
	import { MessageCircle, Trash } from '@lucide/svelte';
	import { request } from '@/api';
	import FallbackAvatar from './fallbackAvatar.svelte';

	let {
		comments = [],
		articleSlug,
		me,
		refreshComments
	}: {
		comments: Comment[];
		articleSlug: string;
		me: IdentityInfo;
		refreshComments: () => Promise<void>;
	} = $props();

	let mainComment = $state('');
	let isSubmitting = $state(false);

	let activeReplyId = $state<number | null>(null);
	let replyContents = $state<Record<number, string>>({});
	let replySubmitting = $state<Record<number, boolean>>({});

	const formatDate = (dateString: string) => {
		const now = new Date();
		const commentDate = new Date(dateString);
		const diffInSeconds = Math.floor((now.getTime() - commentDate.getTime()) / 1000);

		if (diffInSeconds < 60) return '刚刚';
		if (diffInSeconds < 3600) return `${Math.floor(diffInSeconds / 60)} 分钟前`;
		if (diffInSeconds < 86400) return `${Math.floor(diffInSeconds / 3600)} 小时前`;
		if (diffInSeconds < 2592000) return `${Math.floor(diffInSeconds / 86400)} 天前`;
		if (diffInSeconds < 31536000) return `${Math.floor(diffInSeconds / 2592000)} 个月前`;
		return `${Math.floor(diffInSeconds / 31536000)} 年前`;
	};

	const submitMainComment = async () => {
		if (!mainComment.trim() || isSubmitting) return;
		isSubmitting = true;
		try {
			await request(fetch, `/api/articles/${articleSlug}/comments`, {
				method: 'POST',
				json: {
					content: mainComment.trim()
				}
			});
			await refreshComments();
			mainComment = '';
		} catch (error) {
			console.error('Error submitting comment:', error);
		} finally {
			isSubmitting = false;
		}
	};

	const toggleReplyForm = (commentId: number) => {
		if (activeReplyId === commentId) {
			activeReplyId = null;
		} else {
			activeReplyId = commentId;
			// 初始化该评论的回复内容
			if (!replyContents[commentId]) {
				replyContents[commentId] = '';
			}
		}
	};

	// 新增：提交回复
	const submitReply = async (commentId: number) => {
		const content = replyContents[commentId]?.trim();
		if (!content || replySubmitting[commentId]) return;

		replySubmitting[commentId] = true;
		try {
			await request(fetch, `/api/articles/${articleSlug}/comments`, {
				method: 'POST',
				json: {
					content: content,
					parent_id: commentId
				}
			});
			await refreshComments();
			replyContents[commentId] = '';
			activeReplyId = null;
		} catch (error) {
			console.error('Error submitting reply:', error);
		} finally {
			replySubmitting[commentId] = false;
		}
	};
</script>

<div class="max-w-4xl mt-12 mx-auto" id="comments" style="scroll-margin-top: 2vh;">
	<h3 class="text-lg font-medium mb-8 border-b-1 text-gray-900 pb-2">
		{comments.length} 条评论
	</h3>
	<div class="mb-8">
		<div class="flex gap-3 items-start">
			<div class="flex-shrink-0">
				<FallbackAvatar
					avatar={me.avatarUrl}
					displayName={me.displayName}
					class="w-10 h-10 rounded-full text-gray-600"
				/>
			</div>
			<div class="flex-1 relative">
				<div class="border border-gray-200 rounded-lg bg-white">
					<textarea
						bind:value={mainComment}
						placeholder="写下你的评论"
						class="w-full p-4 border-0 resize-none focus:outline-none text-gray-700 placeholder-gray-400"
						rows="3"
					></textarea>
				</div>
				<div class="flex justify-end mt-3">
					<button
						onclick={submitMainComment}
						disabled={!mainComment.trim() || isSubmitting}
						class="px-4 py-2 bg-red-500 hover:bg-red-600 text-white rounded-full text-sm font-medium transition-colors disabled:cursor-not-allowed disabled:opacity-50"
					>
						{isSubmitting ? '提交中...' : '提交'}
					</button>
				</div>
			</div>
		</div>
	</div>

	<div class="space-y-4">
		{#each comments as comment, index (comment.id)}
			<div class="flex gap-3 {index > 0 ? 'pt-5 border-t border-dashed border-gray-200' : ''}">
				<FallbackAvatar
					avatar={comment.commenter.avatarUrl}
					displayName={comment.commenter.displayName}
					class="w-10 h-10 rounded-full text-gray-600"
				/>
				<div class="flex-1 min-w-0">
					<div class="flex items-center gap-2 mb-2">
						<span class="font-medium text-orange-500">
							{comment.commenter.displayName}
						</span>
						<span class="text-gray-400">•</span>
						<span class="text-sm text-gray-500">
							{formatDate(comment.createdAt)}
						</span>
					</div>
					<div class="text-gray-700 mb-3 leading-relaxed">
						{comment.isDeleted ? '该评论已删除' : comment.content}
					</div>
					<div class="flex items-center gap-5 text-sm">
						<button
							onclick={() => toggleReplyForm(comment.id)}
							class="flex items-center gap-1 text-gray-500 hover:text-orange-400 transition-colors"
						>
							<MessageCircle class="w-4 h-4" />
							<span>回复 {comment.replies?.length || 0}</span>
						</button>
						{#if !comment.isDeleted && (me.isAdmin || me.id == comment.commenter.id)}
							<button
								onclick={async () => {
									if (confirm('确定要删除这条评论吗？')) {
										try {
											await request(fetch, `/api/articles/${articleSlug}/comments`, {
												method: 'DELETE',
												json: {
													id: comment.id
												}
											});
											await refreshComments();
										} catch (error) {
											console.error('Error deleting comment:', error);
										}
									}
								}}
								class="flex items-center gap-1 text-gray-500 hover:text-red-400 transition-colors"
							>
								<Trash class="w-4 h-4" />
								<span>删除</span>
							</button>
						{/if}
					</div>

					{#if activeReplyId === comment.id}
						<div class="mt-4 ml-0">
							<div class="flex gap-3 items-start">
								<div class="flex-shrink-0">
									<FallbackAvatar
										avatar={me.avatarUrl}
										displayName={me.displayName}
										class="w-8 h-8 rounded-full text-gray-600"
									/>
								</div>
								<div class="flex-1 relative">
									<div class="border border-gray-200 rounded-lg bg-white">
										<textarea
											bind:value={replyContents[comment.id]}
											placeholder="回复 {comment.commenter.displayName}"
											class="w-full p-3 border-0 resize-none focus:outline-none text-gray-700 placeholder-gray-400 text-sm"
											rows="3"
										></textarea>
									</div>
									<div class="flex justify-end mt-2 gap-2">
										<button
											onclick={() => toggleReplyForm(comment.id)}
											class="px-3 py-1 bg-gray-100 hover:bg-gray-200 text-gray-600 rounded-full text-xs font-medium transition-colors"
										>
											取消
										</button>
										<button
											onclick={() => submitReply(comment.id)}
											disabled={!replyContents[comment.id]?.trim() || replySubmitting[comment.id]}
											class="px-3 py-1 bg-red-500 hover:bg-red-600 text-white rounded-full text-xs font-medium transition-colors disabled:cursor-not-allowed disabled:opacity-50"
										>
											{replySubmitting[comment.id] ? '提交中...' : '提交'}
										</button>
									</div>
								</div>
							</div>
						</div>
					{/if}

					{#if comment.replies && comment.replies.length > 0}
						<div class="mt-4 space-y-4">
							{#each comment.replies as reply (reply.id)}
								<div class="flex gap-3">
									<div class="flex-shrink-0">
										<FallbackAvatar
											avatar={reply.commenter.avatarUrl}
											displayName={reply.commenter.displayName}
											class="w-8 h-8 rounded-full text-gray-600"
										/>
									</div>
									<div class="flex-1 min-w-0">
										<div class="flex items-center gap-2 mb-1">
											<span class="font-medium text-orange-500 text-sm">
												{reply.commenter.displayName}
											</span>
											<span class="text-xs text-gray-500">
												{formatDate(reply.createdAt)}
											</span>
										</div>
										<div class="text-gray-700 text-sm mb-2 leading-relaxed">
											{reply.isDeleted ? '该回复已删除' : reply.content}
										</div>
										{#if !reply.isDeleted && (me.isAdmin || me.id == reply.commenter.id)}
											<div class="flex items-center gap-4 text-sm">
												<button
													onclick={async () => {
														if (confirm('确定要删除这条回复吗？')) {
															try {
																await request(fetch, `/api/articles/${articleSlug}/comments`, {
																	method: 'DELETE',
																	json: {
																		id: reply.id
																	}
																});
																await refreshComments();
															} catch (error) {
																console.error('Error deleting reply:', error);
															}
														}
													}}
													class="flex items-center gap-1 text-gray-500 hover:text-red-400 transition-colors"
												>
													<Trash class="w-4 h-4" />
													<span>删除</span>
												</button>
											</div>
										{/if}
									</div>
								</div>
							{/each}
						</div>
					{/if}
				</div>
			</div>
		{/each}
	</div>

	{#if comments.length === 0}
		<div class="text-center py-12 text-gray-400">
			<MessageCircle class="w-12 h-12 mx-auto mb-3 opacity-30" />
			<p>暂无评论，快来抢沙发吧！</p>
		</div>
	{/if}
</div>
