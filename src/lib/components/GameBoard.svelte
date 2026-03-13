<script lang="ts">
	import { onMount } from 'svelte';
	import Column from './Column.svelte';
	import type { GameState, Card, Suit } from '$lib/types/game';

	let gameState = $state<GameState | null>(null);
	let selectedCard = $state<{ colIndex: number; cardIndex: number } | null>(null);
	let isLoading = $state(true);
	let error = $state<string | null>(null);

	// Check if running in Tauri or browser
	const isTauri = typeof window !== 'undefined' && '__TAURI__' in window;

	async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
		if (isTauri) {
			const { invoke: tauriInvoke } = await import('@tauri-apps/api/core');
			return tauriInvoke<T>(cmd, args);
		} else {
			// Use mock for browser testing
			return invokeMock<T>(cmd, args);
		}
	}

	// Mock implementation for browser testing
	function invokeMock<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
		console.log(`[Mock] invoke('${cmd}')`, args);

		switch (cmd) {
			case 'new_game': {
				const difficulty = (args?.difficulty as number) || 1;
				const state = createMockGameState(difficulty);
				return Promise.resolve(state as T);
			}
			case 'move_cards': {
				const fromCol = args?.from_col as number;
				const startIdx = args?.start_idx as number;
				const toCol = args?.to_col as number;
				const result = mockMoveCards(fromCol, startIdx, toCol);
				return Promise.resolve(result as T);
			}
			case 'deal_cards': {
				const result = mockDealCards();
				return Promise.resolve(result as T);
			}
			default:
				return Promise.reject(`Unknown command: ${cmd}`);
		}
	}

	function createMockGameState(difficulty: number): GameState {
		const suits: Suit[] = difficulty === 1
			? ['spade']
			: difficulty === 2
				? ['spade', 'heart']
				: ['spade', 'heart', 'diamond', 'club'];

		// Create 104 cards (2 decks)
		const allCards: Card[] = [];
		let cardId = 0;

		for (const suit of suits) {
			for (let value = 1; value <= 13; value++) {
				const copies = 8 / suits.length;
				for (let i = 0; i < copies; i++) {
					allCards.push({ id: cardId++, suit, value, face_up: false });
				}
			}
		}

		// Shuffle
		for (let i = allCards.length - 1; i > 0; i--) {
			const j = Math.floor(Math.random() * (i + 1));
			[allCards[i], allCards[j]] = [allCards[j], allCards[i]];
		}

		// Deal to 10 columns (54 cards)
		const columns: Card[][] = [];
		for (let i = 0; i < 10; i++) {
			const count = i < 4 ? 6 : 5;
			const column: Card[] = [];
			for (let j = 0; j < count; j++) {
				const card = allCards.pop()!;
				column.push({
					...card,
					face_up: j === count - 1
				});
			}
			columns.push(column);
		}

		// Remaining 50 cards go to stock
		const stock = allCards.slice(0, 50);

		return {
			columns,
			stock,
			completed: 0,
			score: 500,
			moves: 0,
			difficulty,
			game_over: false,
			won: false
		};
	}

	function mockMoveCards(fromCol: number, startIdx: number, toCol: number): GameState | null {
		if (!gameState) return null;

		const newColumns = gameState.columns.map(col => [...col]);
		const movingCards = newColumns[fromCol].slice(startIdx);

		// Validate move
		const targetColumn = newColumns[toCol];
		if (targetColumn.length === 0) {
			// Can place any sequence on empty column
		} else {
			const topCard = targetColumn[targetColumn.length - 1];
			const firstMovingCard = movingCards[0];
			if (topCard.value !== firstMovingCard.value + 1) {
				return null; // Invalid move
			}
		}

		// Check if moving cards form a valid sequence (same suit, descending)
		for (let i = 0; i < movingCards.length - 1; i++) {
			if (movingCards[i].suit !== movingCards[i + 1].suit ||
				movingCards[i].value !== movingCards[i + 1].value + 1) {
				return null;
			}
		}

		// Execute move
		newColumns[fromCol] = newColumns[fromCol].slice(0, startIdx);
		newColumns[toCol] = [...newColumns[toCol], ...movingCards];

		// Flip top card of source column if needed
		if (newColumns[fromCol].length > 0) {
			const topCard = newColumns[fromCol][newColumns[fromCol].length - 1];
			if (!topCard.face_up) {
				newColumns[fromCol][newColumns[fromCol].length - 1] = { ...topCard, face_up: true };
			}
		}

		// Check for complete sequences
		let completed = gameState.completed;
		const newTargetColumn = newColumns[toCol];
		if (newTargetColumn.length >= 13) {
			// Check if last 13 cards form K-A sequence of same suit
			const last13 = newTargetColumn.slice(-13);
			let isComplete = true;
			const suit = last13[0].suit;
			for (let i = 0; i < 13; i++) {
				if (last13[i].suit !== suit || last13[i].value !== 13 - i || !last13[i].face_up) {
					isComplete = false;
					break;
				}
			}
			if (isComplete) {
				newColumns[toCol] = newTargetColumn.slice(0, -13);
				completed++;
			}
		}

		return {
			...gameState,
			columns: newColumns,
			completed,
			moves: gameState.moves + 1,
			score: Math.max(0, gameState.score - 1)
		};
	}

	function mockDealCards(): GameState | null {
		if (!gameState || gameState.stock.length < 10) return null;

		// Check if all columns have at least one card
		if (gameState.columns.some(col => col.length === 0)) {
			return null;
		}

		const newStock = [...gameState.stock];
		const newColumns = gameState.columns.map((col, i) => {
			const card = { ...newStock.shift()!, face_up: true };
			return [...col, card];
		});

		return {
			...gameState,
			columns: newColumns,
			stock: newStock,
			moves: gameState.moves + 1,
			score: Math.max(0, gameState.score - 1)
		};
	}

	async function initGame(difficulty: number = 1) {
		try {
			isLoading = true;
			error = null;
			gameState = await invoke<GameState>('new_game', { difficulty });
			selectedCard = null;
		} catch (e) {
			error = `Failed to load: ${e}`;
			console.error('Failed to create game:', e);
		} finally {
			isLoading = false;
		}
	}

	async function handleCardClick(colIndex: number, cardIndex: number) {
		if (!gameState || isLoading) return;

		const column = gameState.columns[colIndex];
		const card = column[cardIndex];

		// Only click face-up cards
		if (!card.face_up) return;

		if (selectedCard === null) {
			// Select card - check if it's a valid sequence from this card down
			const cardsFromIndex = column.slice(cardIndex);
			let isValidSequence = true;
			for (let i = 0; i < cardsFromIndex.length - 1; i++) {
				if (cardsFromIndex[i].suit !== cardsFromIndex[i + 1].suit ||
					cardsFromIndex[i].value !== cardsFromIndex[i + 1].value + 1) {
					isValidSequence = false;
					break;
				}
			}
			if (isValidSequence) {
				selectedCard = { colIndex, cardIndex };
			}
		} else if (selectedCard.colIndex === colIndex && selectedCard.cardIndex === cardIndex) {
			// Deselect
			selectedCard = null;
		} else {
			// Try to move
			const result = await invoke<GameState | null>('move_cards', {
				from_col: selectedCard.colIndex,
				start_idx: selectedCard.cardIndex,
				to_col: colIndex
			});
			if (result) {
				gameState = result;
			}
			selectedCard = null;
		}
	}

	async function handleDeal() {
		if (!gameState || gameState.stock.length < 10 || isLoading) return;

		// Check if all columns have cards
		if (gameState.columns.some(col => col.length === 0)) {
			error = 'All columns must have cards before dealing';
			setTimeout(() => { error = null; }, 2000);
			return;
		}

		try {
			const result = await invoke<GameState>('deal_cards');
			if (result) {
				gameState = result;
			}
		} catch (e) {
			error = `Deal failed: ${e}`;
			console.error('Failed to deal:', e);
		}
	}

	onMount(() => {
		initGame(1);
	});

	let remainingDeals = $derived(gameState ? Math.floor(gameState.stock.length / 10) : 0);
</script>

<div class="game-container">
	<!-- Top toolbar -->
	<header class="toolbar">
		<div class="game-info">
			<span class="info-item">
				<span class="label">Score</span>
				<span class="value">{gameState?.score ?? 500}</span>
			</span>
			<span class="info-item">
				<span class="label">Moves</span>
				<span class="value">{gameState?.moves ?? 0}</span>
			</span>
			<span class="info-item">
				<span class="label">Completed</span>
				<span class="value">{gameState?.completed ?? 0}/8</span>
			</span>
		</div>
		<div class="actions">
			<button class="btn" onclick={() => initGame(1)}>
				<span class="btn-icon">↻</span>
				New Game
			</button>
		</div>
	</header>

	<!-- Main game area -->
	<main class="game-board">
		{#if isLoading}
			<div class="loading">
				<div class="spinner"></div>
				<span>Loading...</span>
			</div>
		{:else if error}
			<div class="error">
				<span>{error}</span>
				<button class="btn" onclick={() => initGame(1)}>Retry</button>
			</div>
		{:else if gameState}
			<!-- 10 columns -->
			<div class="columns">
				{#each gameState.columns as column, index}
					<Column
						cards={column}
						columnIndex={index}
						selectedIndex={selectedCard?.colIndex === index ? selectedCard.cardIndex : null}
						onCardClick={(cardIndex) => handleCardClick(index, cardIndex)}
					/>
				{/each}
			</div>

			<!-- Bottom area -->
			<div class="bottom-area">
				<!-- Completed area -->
				<div class="foundation-area">
					{#each Array(8) as _, i}
						<div class="foundation" class:filled={i < gameState.completed}>
							{#if i < gameState.completed}
								<span class="check">✓</span>
							{/if}
						</div>
					{/each}
				</div>

				<!-- Stock pile -->
				<div class="stock-area">
					<button
						class="stock-pile"
						disabled={!gameState.stock.length}
						onclick={handleDeal}
					>
						{#each Array(remainingDeals) as _, i}
							<div class="stock-card" style="left: {i * 3}px; top: {i * 2}px;"></div>
						{/each}
					</button>
					<span class="stock-label">Remaining: {remainingDeals} deals</span>
				</div>
			</div>
		{/if}
	</main>
</div>

<style>
	.game-container {
		display: flex;
		flex-direction: column;
		height: 100vh;
		background: linear-gradient(135deg, #1a472a 0%, #2d5a3f 50%, #1a472a 100%);
		color: white;
		overflow: hidden;
	}

	.toolbar {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 12px 24px;
		background: rgba(0, 0, 0, 0.3);
		backdrop-filter: blur(10px);
		border-bottom: 1px solid rgba(255, 255, 255, 0.1);
	}

	.game-info {
		display: flex;
		gap: 32px;
	}

	.info-item {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 4px;
	}

	.label {
		font-size: 12px;
		color: rgba(255, 255, 255, 0.6);
		text-transform: uppercase;
		letter-spacing: 1px;
	}

	.value {
		font-size: 24px;
		font-weight: bold;
	}

	.btn {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 12px 24px;
		border: none;
		border-radius: 8px;
		background: rgba(255, 255, 255, 0.15);
		color: white;
		font-size: 16px;
		font-weight: 600;
		cursor: pointer;
		transition: all 0.2s;
	}

	.btn:hover {
		background: rgba(255, 255, 255, 0.25);
		transform: translateY(-2px);
	}

	.btn:active {
		transform: translateY(0);
	}

	.btn-icon {
		font-size: 18px;
	}

	.game-board {
		flex: 1;
		display: flex;
		flex-direction: column;
		padding: 20px;
		overflow: auto;
	}

	.columns {
		display: flex;
		justify-content: center;
		gap: 8px;
		flex-wrap: nowrap;
	}

	.bottom-area {
		display: flex;
		justify-content: space-between;
		align-items: flex-end;
		margin-top: auto;
		padding-top: 20px;
	}

	.foundation-area {
		display: flex;
		gap: 8px;
	}

	.foundation {
		width: 60px;
		height: 84px;
		border: 2px solid rgba(255, 255, 255, 0.3);
		border-radius: 6px;
		display: flex;
		align-items: center;
		justify-content: center;
		background: rgba(0, 0, 0, 0.2);
	}

	.foundation.filled {
		background: linear-gradient(135deg, #2e7d32 0%, #1b5e20 100%);
		border-color: #4caf50;
	}

	.check {
		font-size: 28px;
		color: white;
	}

	.stock-area {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 8px;
	}

	.stock-pile {
		position: relative;
		width: 90px;
		height: 126px;
		border: none;
		border-radius: 8px;
		background: transparent;
		cursor: pointer;
		padding: 0;
	}

	.stock-pile:disabled {
		cursor: not-allowed;
		opacity: 0.5;
	}

	.stock-card {
		position: absolute;
		width: 90px;
		height: 126px;
		border-radius: 8px;
		background: linear-gradient(135deg, #1565c0 0%, #1976d2 50%, #1565c0 100%);
		border: 3px solid #0d47a1;
		box-shadow: 2px 2px 5px rgba(0, 0, 0, 0.3);
	}

	.stock-label {
		font-size: 14px;
		color: rgba(255, 255, 255, 0.7);
	}

	.loading, .error {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		height: 100%;
		gap: 16px;
		font-size: 18px;
	}

	.spinner {
		width: 40px;
		height: 40px;
		border: 3px solid rgba(255, 255, 255, 0.3);
		border-top-color: white;
		border-radius: 50%;
		animation: spin 1s linear infinite;
	}

	@keyframes spin {
		to { transform: rotate(360deg); }
	}

	.error {
		color: #ef5350;
	}
</style>
