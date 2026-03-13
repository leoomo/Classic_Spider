<script lang="ts">
	import type { Card as CardType } from '$lib/types/game';

	interface Props {
		card: CardType;
		offset?: number;
		selected?: boolean;
		onclick?: () => void;
	}

	let { card, offset = 0, selected = false, onclick }: Props = $props();

	const suitSymbols: Record<string, string> = {
		spade: '♠',
		heart: '♥',
		diamond: '♦',
		club: '♣'
	};

	let suitSymbol = $derived(suitSymbols[card.suit] || '?');
	let isRed = $derived(card.suit === 'heart' || card.suit === 'diamond');

	let displayValue = $derived.by(() => {
		switch (card.value) {
			case 1: return 'A';
			case 11: return 'J';
			case 12: return 'Q';
			case 13: return 'K';
			default: return String(card.value);
		}
	});
</script>

{#if card.face_up}
	<div
		class="card"
		class:selected
		class:red={isRed}
		style="top: {offset}px;"
		role="listitem"
		tabindex="0"
		{onclick}
	>
		<div class="card-inner">
			<span class="value">{displayValue}</span>
			<span class="suit">{suitSymbol}</span>
		</div>
	</div>
{:else}
	<div
		class="card back"
		style="top: {offset}px;"
		role="listitem"
	>
		<div class="card-back-pattern"></div>
	</div>
{/if}

<style>
	.card {
		position: absolute;
		width: 85px;
		height: 120px;
		border-radius: 8px;
		cursor: pointer;
		transition: transform 0.25s ease-out, box-shadow 0.25s ease-out;
		user-select: none;
	}

	.card-inner {
		width: 100%;
		height: 100%;
		background: white;
		border-radius: 8px;
		display: flex;
		flex-direction: column;
		justify-content: space-between;
		padding: 6px;
		border: 2px solid #bbb;
		box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
	}

	.card.red .card-inner {
		color: #c41e3a;
	}

	.card:not(.red) .card-inner {
		color: #1a1a1a;
	}

	.card .value {
		font-size: 22px;
		font-weight: bold;
		line-height: 1;
	}

	.card .suit {
		font-size: 32px;
		line-height: 1;
		align-self: flex-end;
	}

	.card.selected {
		transform: translateY(-8px);
		box-shadow: 0 8px 16px rgba(0, 0, 0, 0.4), 0 0 0 3px rgba(255, 215, 0, 0.6);
	}

	.card:hover:not(.selected) {
		box-shadow: 0 4px 8px rgba(0, 0, 0, 0.3);
		transform: scale(1.02);
	}

	.card.back {
		position: absolute;
		width: 85px;
		height: 120px;
		border-radius: 8px;
		background: linear-gradient(135deg, #1565c0 0%, #1976d2 50%, #1565c0 100%);
		border: 3px solid #0d47a1;
		box-shadow: 2px 2px 6px rgba(0, 0, 0, 0.3);
		cursor: default;
	}

	.card-back-pattern {
		width: 100%;
		height: 100%;
		background-image: repeating-linear-gradient(
			45deg,
			#0d47a1 0px,
			#0d47a1 2px,
			transparent 2px,
			transparent 8px
		);
		opacity: 0.3;
		border-radius: 5px;
	}
</style>
