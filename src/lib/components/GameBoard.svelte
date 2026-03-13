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
	let shakeColumn = $state<number | null>(null);
	let showRestorePrompt = $state(false);
	let canUndoState = $state(false);
	let canRedoState = $state(false);
	let showDifficultyModal = $state(false);
	let showVictoryModal = $state(false);

	const isTauri = typeof window !== 'undefined' && '__TAURI__' in window;

	const difficultyOptions = [
		{ level: 1, name: '简单', suits: '1种花色', description: '适合新手' },
		{ level: 2, name: '中等', suits: '2种花色', description: '有一定挑战' },
		{ level: 3, name: '困难', suits: '4种花色', description: '高手挑战' }
	];

	async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
		if (isTauri) {
			const { invoke: tauriInvoke } = await import('@tauri-apps/api/core');
			return tauriInvoke<T>(cmd, args);
		} else {
			return invokeMock<T>(cmd, args);
		}
	}

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
			case 'can_undo':
				return Promise.resolve(false as T);
			case 'can_redo':
				return Promise.resolve(false as T);
			case 'has_saved_game':
				return Promise.resolve(false as T);
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
			for (let i = 0; i < 13; i++) {
				const card = last13[i];
				if (card.suit !== 'spade' || card.value !== 13 - i) {
					isComplete = false;
					break;
				}
			}
			if (isComplete) {
				newColumns[toCol] = newColumns[toCol].slice(0, -13);
				completed += 1;
			}
		}

		return {
			...gameState,
			columns: newColumns,
			stock: gameState.stock,
			completed,
			score: Math.max(0, gameState.score - 1),
			moves: gameState.moves + 1,
			difficulty: gameState.difficulty,
			game_over: false,
			won: completed === 8
		};
	}

	function mockDealCards(): GameState | null {
		if (!gameState) return null;

		if (gameState.columns.some(col => col.length === 0)) {
			return null;
		}

		const newStock = [...gameState.stock];
		const newColumns = gameState.columns.map(col => [...col]);

		for (let col of newColumns) {
			if (newStock.length === 0) break;
			const card = newStock.pop()!;
			card.face_up = true;
			col.push(card);
		}

		return {
			...gameState,
			columns: newColumns,
			stock: newStock,
			moves: gameState.moves + 1
		};
	}

	async function initGame(difficulty: number = 1) {
		try {
			isLoading = true;
			error = null;
			showDifficultyModal = false;
			showVictoryModal = false;
			soundManager.play('shuffle');
			gameState = await invoke<GameState>('new_game', { difficulty });
			selectedCard = null;
			showRestorePrompt = false;
		} catch (e) {
			error = `加载失败: ${e}`;
			console.error('Failed to create game:', e);
		} finally {
			isLoading = false;
		}
	}

	function openNewGameModal() {
		showDifficultyModal = true;
	}

	function selectDifficulty(level: number) {
		initGame(level);
	}

	async function restoreGame() {
		try {
			isLoading = true;
			error = null;
			gameState = await invoke<GameState>('load_game');
			selectedCard = null;
			showRestorePrompt = false;
		} catch (e) {
			error = `恢复失败: ${e}`;
			console.error('Failed to load game:', e);
			initGame(1);
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
				shakeColumn = colIndex;
				setTimeout(() => { shakeColumn = null; }, 500);
			}
		} else if (selectedCard.colIndex === colIndex && selectedCard.cardIndex === cardIndex) {
			selectedCard = null;
		} else {
			const movingCardsCount = gameState.columns[selectedCard.colIndex].length - selectedCard.cardIndex;
			const previousCompleted = gameState.completed;
			const result = await invoke<GameState | null>('move_cards', {
				from_col: selectedCard.colIndex,
				start_idx: selectedCard.cardIndex,
				to_col: colIndex
			});
			if (result) {
				gameState = result;

				if (movingCardsCount === 1) {
					soundManager.play('move');
				} else {
					soundManager.play('slide');
				}

				if (result.completed > previousCompleted) {
					soundManager.play('complete');
				}

				if (result.completed === 8) {
					soundManager.play('win');
				}
			} else {
				soundManager.play('error');
				shakeColumn = colIndex;
				setTimeout(() => { shakeColumn = null; }, 500);
			}
			selectedCard = null;
		}
	}

	async function handleDeal() {
		if (!gameState || gameState.stock.length < 10 || isLoading) return;

		if (gameState.columns.some(col => col.length === 0)) {
			error = '发牌前，所有列都必须有牌';
			soundManager.play('error');
			shakeColumn = -1;
			setTimeout(() => { shakeColumn = null; }, 500);
			setTimeout(() => { error = null; }, 2000);
			return;
		}

		try {
			const result = await invoke<GameState>('deal_cards');
			if (result) {
				gameState = result;
				soundManager.play('flip');
			}
		} catch (e) {
			error = `发牌失败: ${e}`;
			console.error('Failed to deal:', e);
		}
	}

	function toggleMute() {
		isMuted = soundManager.toggleMute();
	}

	async function handleUndo() {
		if (!canUndoState || isLoading) return;
		try {
			gameState = await invoke<GameState>('undo');
			selectedCard = null;
			soundManager.play('flip');
		} catch (e) {
			console.error('Undo failed:', e);
		}
	}

	async function handleRedo() {
		if (!canRedoState || isLoading) return;
		try {
			gameState = await invoke<GameState>('redo');
			selectedCard = null;
			soundManager.play('flip');
		} catch (e) {
			console.error('Redo failed:', e);
		}
	}

	onMount(async () => {
		soundManager.preload();
		try {
			const hasSave = await invoke<boolean>('has_saved_game');
			if (hasSave) {
				showRestorePrompt = true;
			} else {
				initGame(1);
			}
		} catch (e) {
			console.error('Failed to check saved game:', e);
			initGame(1);
		}
	});

	let remainingDeals = $derived(gameState ? Math.floor(gameState.stock.length / 10) : 0);

	$effect(() => {
		if (gameState && !isLoading) {
			invoke<boolean>('can_undo').then((v) => (canUndoState = v)).catch(() => {});
			invoke<boolean>('can_redo').then((v) => (canRedoState = v)).catch(() => {});
		}
	});

	// Victory detection
	$effect(() => {
		if (gameState?.completed === 8 && !showVictoryModal) {
			showVictoryModal = true;
			soundManager.play('win');
		}
	});
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
			<!-- 撤销按钮 -->
			<button class="btn undo-btn" onclick={handleUndo} disabled={!canUndoState || isLoading} title="撤销">
				<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
					<path d="M3 7v6h6"/><path d="M21 17a9 9 0 0 0-9-9 9 9 0 0 0-6 2.3L3 13"/>
				</svg>
			</button>
			<!-- 重做按钮 -->
			<button class="btn redo-btn" onclick={handleRedo} disabled={!canRedoState || isLoading} title="重做">
				<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
					<path d="M21 7v6h-6"/><path d="M3 17a9 9 0 0 1 9-9 9 9 0 0 1 6 2.3l3 2.7"/>
				</svg>
			</button>
			<!-- 新游戏按钮 -->
			<button class="btn primary" onclick={openNewGameModal}>
				<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/><path d="M3 3v5h5"/></svg>
				新游戏
			</button>
		</div>
	</header>

	<!-- 主游戏区域 -->
	<main class="game-board">
		{#if showRestorePrompt}
			<!-- 📋 存档恢复提示 -->
			<div class="restore-overlay">
				<div class="restore-modal">
					<h2 class="restore-title">发现存档</h2>
					<p class="restore-text">检测到上次未完成的游戏，是否继续？</p>
					<div class="restore-buttons">
						<button class="btn primary" onclick={restoreGame}>
							继续游戏
						</button>
						<button class="btn" onclick={() => showDifficultyModal = true}>
							新游戏
						</button>
					</div>
				</div>
			</div>
		{/if}

		<!-- 难度选择模态框 -->
		{#if showDifficultyModal}
			<div class="modal-overlay" onclick={() => showDifficultyModal = false}>
				<div class="modal difficulty-modal" onclick={(e) => e.stopPropagation()}>
					<h2 class="modal-title">选择难度</h2>
					<div class="difficulty-options">
						{#each difficultyOptions as option}
							<button
								class="difficulty-option"
								class:selected={gameState?.difficulty === option.level}
								onclick={() => selectDifficulty(option.level)}
							>
								<span class="difficulty-name">{option.name}</span>
								<span class="difficulty-suits">{option.suits}</span>
								<span class="difficulty-desc">{option.description}</span>
							</button>
						{/each}
					</div>
					<button class="btn cancel-btn" onclick={() => showDifficultyModal = false}>
						取消
					</button>
				</div>
			</div>
		{/if}

		<!-- 胜利庆祝模态框 -->
		{#if showVictoryModal}
			<div class="victory-overlay">
				<div class="confetti-container">
					{#each Array(50) as _, i}
						<div class="confetti" style="--delay: {Math.random() * 3}s; --x: {Math.random() * 100}vw; --rotate: {Math.random() * 720 - 360}deg;"></div>
					{/each}
				</div>
				<div class="victory-modal">
					<h2 class="victory-title">🎉 恭喜通关！</h2>
					<div class="victory-stats">
						<div class="stat">
							<span class="stat-label">最终得分</span>
							<span class="stat-value">{gameState?.score}</span>
						</div>
						<div class="stat">
							<span class="stat-label">总步数</span>
							<span class="stat-value">{gameState?.moves}</span>
						</div>
					</div>
					<div class="victory-buttons">
						<button class="btn primary" onclick={openNewGameModal}>
							再来一局
						</button>
					</div>
				</div>
			</div>
		{:else if isLoading}
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
						shake={shakeColumn === index}
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
		padding: 12px 24px;
		background: rgba(0, 0, 0, 0.35);
		backdrop-filter: blur(12px);
		border-bottom: 1px solid rgba(255, 255, 255, 0.1);
		flex-shrink: 0;
	}

	.game-info {
		display: flex;
		gap: 40px;
	}

	.info-item {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 4px;
	}

	.label {
		font-size: 18px;
		color: rgba(255, 255, 255, 0.7);
		text-transform: uppercase;
		letter-spacing: 2px;
		font-weight: 500;
	}

	.value {
		font-size: 36px;
		font-weight: bold;
	}

	.actions {
		display: flex;
		gap: 12px;
	}

	.btn {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 14px 24px;
		border: none;
		border-radius: 10px;
		background: rgba(255, 255, 255, 0.12);
		color: white;
		font-size: 20px;
		font-weight: 600;
		cursor: pointer;
		transition: all 0.25s ease-out;
	}

	.btn:hover:not(:disabled) {
		background: rgba(255, 255, 255, 0.22);
		transform: translateY(-2px);
		box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
	}

	.btn:active:not(:disabled) {
		transform: translateY(0);
	}

	.btn.primary {
		background: linear-gradient(135deg, #4caf50 0%, #2e7d32 100%);
		box-shadow: 0 2px 8px rgba(76, 175, 80, 0.3);
	}

	.btn.primary:hover:not(:disabled) {
		background: linear-gradient(135deg, #66bb6a 0%, #388e3c 100%);
		box-shadow: 0 4px 12px rgba(76, 175, 80, 0.4);
	}

	.btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
		transform: none;
	}

	.mute-btn, .undo-btn, .redo-btn {
		padding: 14px;
		min-width: 52px;
		justify-content: center;
	}

	.mute-btn svg, .undo-btn svg, .redo-btn svg {
		width: 24px;
		height: 24px;
	}

	.game-board {
		flex: 1;
		display: flex;
		flex-direction: column;
		padding: 8px;
		overflow: hidden;
		min-height: 0;
	}

	.columns {
		display: flex;
		justify-content: center;
		gap: 4px;
		flex-wrap: nowrap;
		flex: 1;
		min-height: 0;
	}

	.bottom-area {
		display: flex;
		justify-content: space-between;
		align-items: flex-end;
		padding-top: 8px;
		flex-shrink: 0;
	}

	.foundation-area {
		display: flex;
		gap: 4px;
	}

	.foundation {
		width: 55px;
		height: 78px;
		border: 2px solid rgba(255, 255, 255, 0.35);
		border-radius: 6px;
		display: flex;
		align-items: center;
		justify-content: center;
		background: rgba(0, 0, 0, 0.25);
		transition: all 0.3s ease;
	}

	.foundation.filled {
		background: linear-gradient(135deg, #2e7d32 0%, #1b5e20 100%);
		border-color: #4caf50;
		box-shadow: 0 2px 8px rgba(76, 175, 80, 0.4);
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
		padding-bottom: 8px;
	}

	.stock-pile {
		position: relative;
		width: 85px;
		height: 120px;
		border: none;
		border-radius: 8px;
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
		width: 85px;
		height: 120px;
		border-radius: 8px;
		background: linear-gradient(135deg, #1565c0 0%, #1976d2 50%, #1565c0 100%);
		border: 3px solid #0d47a1;
		box-shadow: 2px 2px 6px rgba(0, 0, 0, 0.4);
	}

	.stock-label {
		font-size: 18px;
		color: rgba(255, 255, 255, 0.9);
		font-weight: 500;
		white-space: nowrap;
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
		to {
			transform: rotate(360deg);
		}
	}

	.error {
		color: #ef5350;
	}

	/* 存档恢复提示框 */
	.restore-overlay {
		position: fixed;
		top: 0;
		left: 0;
		right: 0;
		bottom: 0;
		background: rgba(0, 0, 0, 0.75);
		display: flex;
		justify-content: center;
		align-items: center;
		z-index: 1000;
		backdrop-filter: blur(8px);
	}

	.restore-modal {
		background: linear-gradient(135deg, #2d5a3f 0%, #1a472a 100%);
		border-radius: 20px;
		padding: 40px 50px;
		text-align: center;
		box-shadow: 0 20px 60px rgba(0, 0, 0, 0.4);
		max-width: 420px;
		animation: modal-appear 0.3s ease-out;
	}

	@keyframes modal-appear {
		from {
			opacity: 0;
			transform: scale(0.9);
		}
		to {
			opacity: 1;
			transform: scale(1);
		}
	}

	.restore-title {
		font-size: 36px;
		font-weight: bold;
		color: white;
		margin-bottom: 16px;
	}

	.restore-text {
		font-size: 22px;
		color: rgba(255, 255, 255, 0.9);
		margin-bottom: 32px;
	}

	.restore-buttons {
		display: flex;
		gap: 16px;
		justify-content: center;
	}

	/* 难度选择模态框 */
	.modal-overlay {
		position: fixed;
		top: 0;
		left: 0;
		right: 0;
		bottom: 0;
		background: rgba(0, 0, 0, 0.75);
		display: flex;
		justify-content: center;
		align-items: center;
		z-index: 1000;
		backdrop-filter: blur(8px);
	}

	.modal {
		background: linear-gradient(135deg, #2d5a3f 0%, #1a472a 100%);
		border-radius: 20px;
		padding: 40px 50px;
		text-align: center;
		box-shadow: 0 20px 60px rgba(0, 0, 0, 0.4);
		animation: modal-appear 0.3s ease-out;
	}

	.modal-title {
		font-size: 36px;
		font-weight: bold;
		color: white;
		margin-bottom: 32px;
	}

	.difficulty-options {
		display: flex;
		flex-direction: column;
		gap: 16px;
		margin-bottom: 24px;
	}

	.difficulty-option {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 8px;
		padding: 24px 48px;
		border: 3px solid rgba(255, 255, 255, 0.3);
		border-radius: 16px;
		background: rgba(255, 255, 255, 0.1);
		color: white;
		cursor: pointer;
		transition: all 0.25s ease;
	}

	.difficulty-option:hover {
		background: rgba(255, 255, 255, 0.2);
		border-color: rgba(255, 255, 255, 0.5);
		transform: translateY(-4px);
		box-shadow: 0 8px 25px rgba(0, 0, 0, 0.3);
	}

	.difficulty-option.selected {
		border-color: #4caf50;
		background: rgba(76, 175, 80, 0.2);
	}

	.difficulty-name {
		font-size: 28px;
		font-weight: bold;
	}

	.difficulty-suits {
		font-size: 20px;
		color: rgba(255, 255, 255, 0.8);
	}

	.difficulty-desc {
		font-size: 16px;
		color: rgba(255, 255, 255, 0.6);
	}

	.cancel-btn {
		background: rgba(255, 255, 255, 0.1);
		padding: 12px 32px;
		font-size: 18px;
	}

	/* 胜利庆祝 */
	.victory-overlay {
		position: fixed;
		top: 0;
		left: 0;
		right: 0;
		bottom: 0;
		background: rgba(0, 0, 0, 0.8);
		display: flex;
		justify-content: center;
		align-items: center;
		z-index: 2000;
	}

	.confetti-container {
		position: absolute;
		top: 0;
		left: 0;
		width: 100%;
		height: 100%;
		pointer-events: none;
		overflow: hidden;
	}

	.confetti {
		position: absolute;
		width: 12px;
		height: 12px;
		background: var(--color, #f0f);
		left: var(--x);
		top: -20px;
		animation: confetti-fall 3s ease-in-out var(--delay) infinite;
		transform: rotate(var(--rotate));
	}

	.confetti:nth-child(4n+1) { --color: #ff6b6b; }
	.confetti:nth-child(4n+2) { --color: #4ecdc4; }
	.confetti:nth-child(4n+3) { --color: #ffe66d; }
	.confetti:nth-child(4n+4) { --color: #95e1d3; }

	@keyframes confetti-fall {
		0% {
			transform: translateY(0) rotate(0deg);
			opacity: 1;
		}
		100% {
			transform: translateY(100vh) rotate(720deg);
			opacity: 0;
		}
	}

	.victory-modal {
		background: linear-gradient(135deg, #2d5a3f 0%, #1a472a 100%);
		border-radius: 24px;
		padding: 48px 64px;
		text-align: center;
		box-shadow: 0 30px 80px rgba(0, 0, 0, 0.5);
		animation: victory-appear 0.5s ease-out;
		z-index: 2001;
	}

	@keyframes victory-appear {
		0% {
			opacity: 0;
			transform: scale(0.5) rotate(-10deg);
		}
		50% {
			transform: scale(1.1) rotate(2deg);
		}
		100% {
			opacity: 1;
			transform: scale(1) rotate(0deg);
		}
	}

	.victory-title {
		font-size: 48px;
		font-weight: bold;
		color: #ffe66d;
		margin-bottom: 32px;
		text-shadow: 0 4px 20px rgba(255, 230, 109, 0.5);
		animation: pulse 1s ease-in-out infinite;
	}

	@keyframes pulse {
		0%, 100% { transform: scale(1); }
		50% { transform: scale(1.05); }
	}

	.victory-stats {
		display: flex;
		justify-content: center;
		gap: 48px;
		margin-bottom: 40px;
	}

	.stat {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 8px;
	}

	.stat-label {
		font-size: 20px;
		color: rgba(255, 255, 255, 0.7);
	}

	.stat-value {
		font-size: 48px;
		font-weight: bold;
		color: #4ecdc4;
	}

	.victory-buttons {
		display: flex;
		justify-content: center;
		gap: 16px;
	}
</style>
