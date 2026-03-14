<script lang="ts">
	import Card from './Card.svelte';
	import type { Card as CardType } from '$lib/types/game';

	interface Props {
		cards: CardType[];
		columnIndex: number;
		selectedIndex: number | null;
		hintStartIndex?: number | null;
		isHintTarget?: boolean;
		onCardClick: (cardIndex: number) => void;
		onDragStart?: (colIndex: number, cardIndex: number, event: MouseEvent) => void;
		shake?: boolean;
		isDropTarget?: boolean;
		dropValid?: boolean;
	}

	let { cards = [], columnIndex, selectedIndex, hintStartIndex = null, isHintTarget = false, onCardClick, onDragStart, shake = false, isDropTarget = false, dropValid = true }: Props = $props();

	// 计算每张牌的偏移位置（面朝上的卡牌间距更大）
	function getCardOffset(index: number): number {
		let offset = 0;
		for (let i = 0; i < index; i++) {
			offset += cards[i].face_up ? 22 : 8;
		}
		return offset;
	}

	// 处理拖拽开始
	function handleMouseDown(cardIndex: number, event: MouseEvent) {
		// 只处理左键
		if (event.button !== 0) return;

		const card = cards[cardIndex];
		if (!card.face_up) return;

		// 检查是否可以拖拽（必须是有序序列）
		const cardsFromIndex = cards.slice(cardIndex);
		let canDrag = true;
		for (let i = 0; i < cardsFromIndex.length - 1; i++) {
			if (cardsFromIndex[i].suit !== cardsFromIndex[i + 1].suit ||
				cardsFromIndex[i].value !== cardsFromIndex[i + 1].value + 1) {
				canDrag = false;
				break;
			}
		}

		if (canDrag && onDragStart) {
			onDragStart(columnIndex, cardIndex, event);
		}
	}

	// 处理键盘事件（a11y支持）
	function handleKeyDown(cardIndex: number, event: KeyboardEvent) {
		if (event.key === 'Enter' || event.key === ' ') {
			event.preventDefault();
			onCardClick?.(cardIndex);
		}
	}
</script>

<div
	class="column"
	class:shake
	class:drop-target-valid={isDropTarget && dropValid}
	class:drop-target-invalid={isDropTarget && !dropValid}
	class:hint-target={isHintTarget}
	style={!isDropTarget ? 'background: transparent; box-shadow: none;' : ''}
	role="list"
	aria-label="第 {columnIndex + 1} 列"
>
	{#each cards as card, idx (card.id)}
		<div
			class="card-wrapper"
			class:hint-card={hintStartIndex !== null && idx === hintStartIndex}
			style="top: {getCardOffset(idx)}px;"
			onmousedown={(e) => handleMouseDown(idx, e)}
			onclick={() => onCardClick?.(idx)}
			onkeydown={(e) => handleKeyDown(idx, e)}
			role="button"
			tabindex={card.face_up ? 0 : -1}
			aria-label="{card.face_up ? `点击选择卡牌` : '背面朝上的卡牌'}"
		>
			<Card
				{card}
				selected={selectedIndex === idx}
			/>
		</div>
	{/each}

	{#if cards.length === 0}
		<div class="empty-slot">
			<span class="placeholder">K</span>
		</div>
	{/if}
</div>

<style>
	.column {
		position: relative;
		min-height: 420px;
		width: 105px;
		padding: 4px;
		flex-shrink: 0;
		transition: background 0.15s ease, box-shadow 0.15s ease;
		border-radius: 12px;
		user-select: none;
		-webkit-user-select: none;
	}

	.card-wrapper {
		position: absolute;
		left: 4px;
		cursor: grab;
		user-select: none;
		-webkit-user-select: none;
	}

	.card-wrapper:active {
		cursor: grabbing;
	}

	.empty-slot {
		width: 95px;
		height: 138px;
		border: 2px dashed rgba(255, 255, 255, 0.4);
		border-radius: 8px;
		display: flex;
		align-items: center;
		justify-content: center;
		margin: 4px;
		background: rgba(0, 0, 0, 0.12);
		transition: all 0.2s ease;
	}

	.empty-slot:hover {
		background: rgba(0, 0, 0, 0.2);
		border-color: rgba(255, 255, 255, 0.5);
	}

	.placeholder {
		font-size: 44px;
		color: rgba(255, 255, 255, 0.7);
		font-weight: 700;
		text-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
	}

	/* 抖动动画 - 错误反馈 */
	.shake {
		animation: shake 0.5s ease-in-out;
	}

	@keyframes shake {
		0%, 100% { transform: translateX(0); }
		10%, 30% { transform: translateX(-8px); }
		20%, 40% { transform: translateX(8px); }
		30%, 50% { transform: translateX(-4px); }
		40%, 60% { transform: translateX(4px); }
		70%, 80% { transform: translateX(2px); }
		80%, 90% { transform: translateX(-2px); }
		90%, 100% { transform: translateX(0); }
	}

	/* 拖拽目标高亮 - 更明显的反馈 */
	.column.drop-target-valid {
		background: rgba(76, 175, 80, 0.2);
		box-shadow:
			inset 0 0 0 4px rgba(76, 175, 80, 0.8),
			0 0 15px rgba(76, 175, 80, 0.3);
	}

	.column.drop-target-invalid {
		background: rgba(244, 67, 54, 0.12);
		box-shadow:
			inset 0 0 0 4px rgba(244, 67, 54, 0.6),
			0 0 15px rgba(244, 67, 54, 0.2);
	}

	/* 提示高亮样式 - 更明显但不刺眼 */
	.column.hint-target {
		background: rgba(255, 215, 0, 0.15);
		box-shadow:
			inset 0 0 0 4px rgba(255, 215, 0, 0.7),
			0 0 20px rgba(255, 215, 0, 0.3);
		animation: hint-pulse 1.2s ease-in-out infinite;
	}

	.card-wrapper.hint-card {
		animation: hint-glow 1.2s ease-in-out infinite;
		z-index: 50;
	}

	@keyframes hint-pulse {
		0%, 100% {
			box-shadow:
				inset 0 0 0 4px rgba(255, 215, 0, 0.6),
				0 0 15px rgba(255, 215, 0, 0.2);
		}
		50% {
			box-shadow:
				inset 0 0 0 5px rgba(255, 215, 0, 0.9),
				0 0 25px rgba(255, 215, 0, 0.4);
		}
	}

	@keyframes hint-glow {
		0%, 100% {
			filter: drop-shadow(0 0 10px rgba(255, 215, 0, 0.9));
			transform: scale(1);
		}
		50% {
			filter: drop-shadow(0 0 18px rgba(255, 215, 0, 1));
			transform: scale(1.02);
		}
	}

	/* 响应式优化 */
	@media (max-width: 1200px) {
		.column {
			width: 98px;
			min-height: 380px;
			padding: 3px;
		}

		.empty-slot {
			width: 88px;
			height: 128px;
		}

		.placeholder {
			font-size: 42px;
		}
	}

	@media (max-width: 1024px) {
		.column {
			width: 90px;
			min-height: 350px;
			padding: 2px;
		}

		.empty-slot {
			width: 80px;
			height: 116px;
		}

		.placeholder {
			font-size: 38px;
		}
	}

	@media (max-width: 768px) {
		.column {
			width: 82px;
			min-height: 320px;
		}

		.empty-slot {
			width: 72px;
			height: 105px;
		}

		.placeholder {
			font-size: 34px;
		}
	}
</style>
