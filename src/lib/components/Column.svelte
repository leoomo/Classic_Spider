<script lang="ts">
	import Card from './Card.svelte';
	import type { Card as CardType } from '$lib/types/game';

	interface Props {
		cards: CardType[];
		columnIndex: number;
		onCardClick?: (cardIndex: number) => void;
		selectedIndex?: number | null;
	}

	let { cards, columnIndex, onCardClick, selectedIndex = null }: Props = $props();

	// 计算每张牌的偏移位置
	function getCardOffset(index: number): number {
		let offset = 0;
		for (let i = 0; i < index; i++) {
			offset += cards[i].face_up ? 25 : 12;
		}
		return offset;
	}
</script>

<div class="column" role="list" aria-label="第 {columnIndex + 1} 列">
	{#each cards as card, idx (card.id)}
		<Card
			{card}
			index={idx}
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
		min-height: 450px;
		width: 100px;
		padding: 5px;
	}

	.empty-slot {
		width: 90px;
		height: 126px;
		border: 2px dashed rgba(255, 255, 255, 0.4);
		border-radius: 8px;
		display: flex;
		align-items: center;
		justify-content: center;
		margin: 5px;
	}

	.placeholder {
		font-size: 32px;
		color: rgba(255, 255, 255, 0.3);
		font-weight: bold;
	}
</style>
