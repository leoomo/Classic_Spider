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
		width: 120px;
		height: 168px;
		border-radius: 10px;
		cursor: pointer;
		transition: transform 0.25s ease-out, box-shadow 0.25s ease-out;
		user-select: none;
	}

	.card-inner {
		width: 100%;
		height: 100%;
		background: white;
		border-radius: 10px;
		display: flex;
		flex-direction: column;
		justify-content: space-between;
		padding: 10px;
		border: 2px solid #bbb;
		box-shadow: 0 4px 8px rgba(0, 0, 0, 0.2);
	}

	.card.red .card-inner {
		color: #c41e3a;
	}

	.card:not(.red) .card-inner {
		color: #1a1a1a;
	}

	.card .value {
		font-size: 32px;
		font-weight: bold;
		line-height: 1;
	}

	.card .suit {
		font-size: 48px;
		line-height: 1;
		align-self: flex-end;
	}

	.card.selected {
		transform: translateY(-12px);
		box-shadow: 0 12px 24px rgba(0, 0, 0, 0.4), 0 0 0 3px rgba(255, 215, 0, 0.6);
	}

	.card:hover:not(.selected) {
		box-shadow: 0 6px 12px rgba(0, 0, 0, 0.3);
		transform: scale(1.02);
	}

	.card.back {
		position: absolute;
		width: 120px;
		height: 168px;
		border-radius: 10px;
		background: linear-gradient(135deg, #1565c0 0%, #1976d2 50%, #1565c0 100%);
		border: 4px solid #0d47a1;
		box-shadow: 3px 3px 8px rgba(0, 0, 0, 0.3);
		cursor: default;
	}

	.card-back-pattern {
		width: 100%;
		height: 100%;
		background-image: repeating-linear-gradient(
			45deg,
			#0d47a1 0px,
			#0d47a1 3px,
			transparent 3px,
			transparent 12px
		);
		opacity: 0.3;
		border-radius: 6px;
	}
</style>
