<script lang="ts">
	import Card from './Card.svelte';
	import type { Card as CardType } from '$lib/types/game';

	interface Props {
		cards: CardType[];
		columnIndex: number;
		selectedIndex: number | null;
		onCardClick: (cardIndex: number) => void;
		onDragStart?: (colIndex: number, cardIndex: number, event: MouseEvent) => void;
		shake?: boolean;
		isDropTarget?: boolean;
		dropValid?: boolean;
	}

	let { cards = [], columnIndex, selectedIndex, onCardClick, onDragStart, shake = false, isDropTarget = false, dropValid = true }: Props = $props();

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
</script>

<div
	class="column"
	class:shake
	class:drop-target-valid={isDropTarget && dropValid}
	class:drop-target-invalid={isDropTarget && !dropValid}
	role="list"
	aria-label="第 {columnIndex + 1} 列"
>
	{#each cards as card, idx (card.id)}
		<div
			class="card-wrapper"
			style="top: {getCardOffset(idx)}px;"
			onmousedown={(e) => handleMouseDown(idx, e)}
			onclick={() => onCardClick?.(idx)}
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
		min-height: 400px;
		width: 95px;
		padding: 4px;
		flex-shrink: 0;
		transition: background 0.15s ease, box-shadow 0.15s ease;
		border-radius: 12px;
	}

	.card-wrapper {
		position: absolute;
		left: 4px;
		cursor: grab;
	}

	.card-wrapper:active {
		cursor: grabbing;
	}

	.empty-slot {
		width: 85px;
		height: 120px;
		border: 2px dashed rgba(255, 255, 255, 0.5);
		border-radius: 8px;
		display: flex;
		align-items: center;
		justify-content: center;
		margin: 4px;
		background: rgba(0, 0, 0, 0.15);
	}

	.placeholder {
		font-size: 40px;
		color: rgba(255, 255, 255, 0.4);
		font-weight: bold;
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

	/* 拖拽目标高亮 */
	.column.drop-target-valid {
		background: rgba(76, 175, 80, 0.25);
		box-shadow: inset 0 0 0 3px rgba(76, 175, 80, 0.6);
	}

	.column.drop-target-invalid {
		background: rgba(244, 67, 54, 0.15);
		box-shadow: inset 0 0 0 3px rgba(244, 67, 54, 0.4);
	}
</style>
