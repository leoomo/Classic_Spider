<script lang="ts">
	import Card from './Card.svelte';
	import type { Card as CardType } from '$lib/types/game';

	interface Props {
		cards: CardType[];
		columnIndex: number;
		selectedIndex: number | null;
		onCardClick: (cardIndex: number) => void;
	}

	let { cards = [], columnIndex, selectedIndex, onCardClick }: Props = $props();

	// 计算每张牌的偏移位置（面朝上的卡牌间距更大）
	function getCardOffset(index: number): number {
		let offset = 0;
		for (let i = 0; i < index; i++) {
			offset += cards[i].face_up ? 40 : 18;
		}
		return offset;
	}
</script>

<div class="column" role="list" aria-label="第 {columnIndex + 1} 列">
	{#each cards as card, idx (card.id)}
		<Card
			{card}
			offset={getCardOffset(idx)}
			selected={selectedIndex === idx}
			onclick={() => onCardClick?.(idx)}
		/>
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
		min-height: 600px;
		width: 140px;
		padding: 8px;
		flex-shrink: 0;
	}

	.empty-slot {
		width: 120px;
		height: 168px;
		border: 3px dashed rgba(255, 255, 255, 0.5);
		border-radius: 10px;
		display: flex;
		align-items: center;
		justify-content: center;
		margin: 8px;
		background: rgba(0, 0, 0, 0.15);
	}

	.placeholder {
		font-size: 56px;
		color: rgba(255, 255, 255, 0.4);
		font-weight: bold;
	}
</style>
