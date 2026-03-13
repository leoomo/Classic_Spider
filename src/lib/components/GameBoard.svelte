<script lang="ts">
	import { onMount } from 'svelte';
	import Column from './Column.svelte';
	import { soundManager } from '$lib/utils/sound';
	import type { GameState, Card, Suit } from '$lib/types/game';

	let gameState = $state<GameState | null>(null);
	let selectedCard = $state<{ colIndex: number; cardIndex: number } | null>(null);
	let isLoading = $state(true);
	let error = $state<string | null>(null);
	let isMuted = $state(false);

	// Check if running in Tauri or browser
	const isTauri = typeof window !== 'undefined' && '__TAURI__' in window;

	async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
		if (isTauri) {
			const { invoke: tauriInvoke } = await import('@tauri-apps/api/core');
			return tauriInvoke<T>(cmd, args);
		} else {
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

		for (let i = allCards.length - 1; i > 0; i--) {
			const j = Math.floor(Math.random() * (i + 1));
			[allCards[i], allCards[j]] = [allCards[j], allCards[i]];
		}

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

		const targetColumn = newColumns[toCol];
		if (targetColumn.length === 0) {
			// Can place any sequence on empty column
		} else {
			const topCard = targetColumn[targetColumn.length - 1];
			const firstMovingCard = movingCards[0];
			if (topCard.value !== firstMovingCard.value + 1) {
				return null;
			}
		}

		for (let i = 0; i < movingCards.length - 1; i++) {
			if (movingCards[i].suit !== movingCards[i + 1].suit ||
				movingCards[i].value !== movingCards[i + 1].value + 1) {
				return null;
			}
		}

		newColumns[fromCol] = newColumns[fromCol].slice(0, startIdx);
		newColumns[toCol] = [...newColumns[toCol], ...movingCards];

		if (newColumns[fromCol].length > 0) {
			const topCard = newColumns[fromCol][newColumns[fromCol].length - 1];
			if (!topCard.face_up) {
				newColumns[fromCol][newColumns[fromCol].length - 1] = { ...topCard, face_up: true };
			}
		}

		let completed = gameState.completed;
		const newTargetColumn = newColumns[toCol];
		if (newTargetColumn.length >= 13) {
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
				soundManager.play('complete');
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

		if (gameState.columns.some(col => col.length === 0)) {
			return null;
		}

		const newStock = [...gameState.stock];
		const newColumns = gameState.columns.map((col) => {
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
			soundManager.play('deal');
		} catch (e) {
			error = `加载失败: ${e}`;
			console.error('Failed to create game:', e);
		} finally {
			isLoading = false;
		}
	}

	async function handleCardClick(colIndex: number, cardIndex: number) {
		if (!gameState || isLoading) return;

		const column = gameState.columns[colIndex];
		const card = column[cardIndex];

		if (!card.face_up) return;

		if (selectedCard === null) {
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
				soundManager.play('click');
			} else {
				soundManager.play('error');
			}
		} else if (selectedCard.colIndex === colIndex && selectedCard.cardIndex === cardIndex) {
			selectedCard = null;
		} else {
			const result = await invoke<GameState | null>('move_cards', {
				from_col: selectedCard.colIndex,
				start_idx: selectedCard.cardIndex,
				to_col: colIndex
			});
			if (result) {
				gameState = result;
				soundManager.play('move');

				// 检查是否胜利
				if (result.completed === 8) {
					soundManager.play('win');
				}
			} else {
				soundManager.play('error');
			}
			selectedCard = null;
		}
	}

	async function handleDeal() {
		if (!gameState || gameState.stock.length < 10 || isLoading) return;

		if (gameState.columns.some(col => col.length === 0)) {
			error = '发牌前，所有列都必须有牌';
			soundManager.play('error');
			setTimeout(() => { error = null; }, 2000);
			return;
		}

		try {
			const result = await invoke<GameState>('deal_cards');
			if (result) {
				gameState = result;
				soundManager.play('deal');
			}
		} catch (e) {
			error = `发牌失败: ${e}`;
			console.error('Failed to deal:', e);
		}
	}

	function toggleMute() {
		isMuted = soundManager.toggleMute();
	}

	onMount(() => {
		soundManager.preload();
		initGame(1);
	});

	let remainingDeals = $derived(gameState ? Math.floor(gameState.stock.length / 10) : 0);
</script>

<div class="game-container">
	<!-- 顶部工具栏 -->
	<header class="toolbar">
		<div class="game-info">
			<span class="info-item">
				<span class="label">分数</span>
				<span class="value">{gameState?.score ?? 500}</span>
			</span>
			<span class="info-item">
				<span class="label">步数</span>
				<span class="value">{gameState?.moves ?? 0}</span>
			</span>
			<span class="info-item">
				<span class="label">完成</span>
				<span class="value">{gameState?.completed ?? 0}/8</span>
			</span>
		</div>
		<div class="actions">
			<button class="btn mute-btn" onclick={toggleMute} title={isMuted ? '开启音效' : '静音'}>
				{#if isMuted}
					<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5"/><line x1="23" y1="9" x2="17" y2="15"/><line x1="17" y1="9" x2="23" y2="15"/></svg>
				{:else}
					<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5"/><path d="M15.54 8.46a5 5 0 0 1 0 7.07"/><path d="M19.07 4.93a10 10 0 0 1 0 14.14"/></svg>
				{/if}
			</button>
			<button class="btn primary" onclick={() => initGame(1)}>
				<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 12a9 9 0 1 1-9-9c2.52 0 4.93 1 6.74 2.74L21 8"/><path d="M21 3v5h-5"/></svg>
				新游戏
			</button>
		</div>
	</header>

	<!-- 主游戏区域 -->
	<main class="game-board">
		{#if isLoading}
			<div class="loading">
				<div class="spinner"></div>
				<span>加载中...</span>
			</div>
		{:else if error}
			<div class="error">
				<span>{error}</span>
				<button class="btn" onclick={() => initGame(1)}>重试</button>
			</div>
		{:else if gameState}
			<!-- 10列卡牌 -->
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

			<!-- 底部区域 -->
			<div class="bottom-area">
				<!-- 完成区域 -->
				<div class="foundation-area">
					{#each Array(8) as _, i}
						<div class="foundation" class:filled={i < gameState.completed}>
							{#if i < gameState.completed}
								<span class="check">✓</span>
							{/if}
						</div>
					{/each}
				</div>

				<!-- 牌堆 -->
				<div class="stock-area">
					<button
						class="stock-pile"
						disabled={!gameState.stock.length}
						onclick={handleDeal}
					>
						{#each Array(remainingDeals) as _, i}
							<div class="stock-card" style="left: {i * 4}px; top: {i * 3}px;"></div>
						{/each}
					</button>
					<span class="stock-label">剩余发牌: {remainingDeals} 次</span>
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
		padding: 16px 32px;
		background: rgba(0, 0, 0, 0.35);
		backdrop-filter: blur(12px);
		border-bottom: 1px solid rgba(255, 255, 255, 0.1);
	}

	.game-info {
		display: flex;
		gap: 48px;
	}

	.info-item {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 6px;
	}

	.label {
		font-size: 18px;
		color: rgba(255, 255, 255, 0.7);
		text-transform: uppercase;
		letter-spacing: 2px;
		font-weight: 500;
	}

	.value {
		font-size: 40px;
		font-weight: bold;
	}

	.actions {
		display: flex;
		gap: 16px;
	}

	.btn {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 16px 32px;
		border: none;
		border-radius: 12px;
		background: rgba(255, 255, 255, 0.12);
		color: white;
		font-size: 22px;
		font-weight: 600;
		cursor: pointer;
		transition: all 0.25s ease-out;
	}

	.btn:hover {
		background: rgba(255, 255, 255, 0.22);
		transform: translateY(-3px);
		box-shadow: 0 6px 20px rgba(0, 0, 0, 0.2);
	}

	.btn:active {
		transform: translateY(0);
	}

	.btn.primary {
		background: linear-gradient(135deg, #4caf50 0%, #2e7d32 100%);
		box-shadow: 0 4px 15px rgba(76, 175, 80, 0.3);
	}

	.btn.primary:hover {
		background: linear-gradient(135deg, #66bb6a 0%, #388e3c 100%);
		box-shadow: 0 6px 20px rgba(76, 175, 80, 0.4);
	}

	.mute-btn {
		padding: 16px;
		min-width: 56px;
		justify-content: center;
	}

	.game-board {
		flex: 1;
		display: flex;
		flex-direction: column;
		padding: 24px;
		overflow: auto;
	}

	.columns {
		display: flex;
		justify-content: center;
		gap: 12px;
		flex-wrap: nowrap;
	}

	.bottom-area {
		display: flex;
		justify-content: space-between;
		align-items: flex-end;
		margin-top: auto;
		padding-top: 24px;
	}

	.foundation-area {
		display: flex;
		gap: 12px;
	}

	.foundation {
		width: 80px;
		height: 112px;
		border: 3px solid rgba(255, 255, 255, 0.35);
		border-radius: 10px;
		display: flex;
		align-items: center;
		justify-content: center;
		background: rgba(0, 0, 0, 0.25);
		transition: all 0.3s ease;
	}

	.foundation.filled {
		background: linear-gradient(135deg, #2e7d32 0%, #1b5e20 100%);
		border-color: #4caf50;
		box-shadow: 0 4px 15px rgba(76, 175, 80, 0.4);
	}

	.check {
		font-size: 48px;
		color: white;
	}

	.stock-area {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 12px;
	}

	.stock-pile {
		position: relative;
		width: 120px;
		height: 168px;
		border: none;
		border-radius: 10px;
		background: transparent;
		cursor: pointer;
		padding: 0;
		transition: transform 0.25s ease-out;
	}

	.stock-pile:hover:not(:disabled) {
		transform: scale(1.05);
	}

	.stock-pile:disabled {
		cursor: not-allowed;
		opacity: 0.5;
	}

	.stock-card {
		position: absolute;
		width: 120px;
		height: 168px;
		border-radius: 10px;
		background: linear-gradient(135deg, #1565c0 0%, #1976d2 50%, #1565c0 100%);
		border: 4px solid #0d47a1;
		box-shadow: 3px 3px 10px rgba(0, 0, 0, 0.4);
	}

	.stock-label {
		font-size: 20px;
		color: rgba(255, 255, 255, 0.8);
		font-weight: 500;
	}

	.loading, .error {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		height: 100%;
		gap: 20px;
		font-size: 24px;
	}

	.spinner {
		width: 56px;
		height: 56px;
		border: 4px solid rgba(255, 255, 255, 0.3);
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
